use ash::{Device, vk};
use std::{marker::PhantomData, ptr};

use crate::{Allocation, Allocator, Context, Renderer, Result, vk_query};

/// A buffer and allocation.
pub struct Buffer<A: Allocator> {
    /// The source context.
    context: Context,

    /// The memory allocation.
    allocation: Allocation<A>,

    /// The buffer handle.
    buffer: vk::Buffer,
}

/// A dynamic buffer whose data changes often.
///
/// # Note
/// This is made to work with an `ibis::Renderer` and will use one "frame" of
/// memory per renderer frame in flight.
pub struct DynBuffer<T: Sized, A: Allocator> {
    /// The buffer.
    buffer: Buffer<A>,

    /// The data capacity.
    capacity: usize,

    _t: PhantomData<T>,
}

/// A staged buffer whose data never changes.
pub struct StagedBuffer<A: Allocator>(Buffer<A>);

impl<A: Allocator> Buffer<A> {
    /// Creates a buffer with the given size and usage.
    #[must_use]
    pub fn new(
        context: &Context,
        allocator: &A,
        size: usize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let device = context.device();

        // Create the buffer.
        let create_info = vk::BufferCreateInfo::default()
            .flags(vk::BufferCreateFlags::empty())
            .size(size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[]);
        let buffer = unsafe { vk_query!(device.create_buffer(&create_info, None))? };

        // Allocate memory.
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation = allocator.allocate(&memory_requirements, properties)?;

        // Bind memory.
        unsafe {
            vk_query!(device.bind_buffer_memory(
                buffer,
                allocation.memory(),
                allocation.offset() as u64
            ))?
        };

        let context = context.clone();
        Ok(Self {
            context,
            allocation,
            buffer,
        })
    }

    /// Returns the buffer's allocation.
    #[inline]
    #[must_use]
    pub fn allocation(&self) -> &Allocation<A> {
        &self.allocation
    }

    /// Returns the buffer handle.
    #[inline]
    #[must_use]
    pub fn handle(&self) -> vk::Buffer {
        self.buffer
    }
}

impl<A: Allocator> Drop for Buffer<A> {
    fn drop(&mut self) {
        unsafe {
            self.context.device().destroy_buffer(self.buffer, None);
        }
    }
}

impl<T: Sized, A: Allocator> DynBuffer<T, A> {
    /// Creates a new dynamic buffer.
    #[must_use]
    pub fn new(
        renderer: &Renderer,
        alloctor: &A,
        capacity: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        let frame_size = capacity * size_of::<T>();
        let num_frames = renderer.num_frames();
        let buffer = Buffer::new(
            renderer.context(),
            alloctor,
            frame_size * num_frames,
            usage,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        Ok(Self {
            buffer,
            capacity,
            _t: PhantomData,
        })
    }

    /// Writes the data to the buffer at the given memory frame.
    ///
    /// # Note
    /// If the data has more than N elements where N is the capacity, only the
    /// first N will be used.
    #[must_use]
    pub fn write(&self, frame_index: usize, data: &[T]) -> Result {
        let frame_size = self.capacity * size_of::<T>();
        let offset = (frame_size * frame_index) as u64;
        unsafe {
            write_to_memory(
                self.buffer.context.device(),
                self.buffer.allocation.memory(),
                data.as_ptr(),
                offset,
                self.capacity.min(data.len()),
            )
        }
    }

    /// Returns the buffer.
    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &Buffer<A> {
        &self.buffer
    }

    /// Returns the memory allocation.
    #[inline]
    #[must_use]
    pub fn memory(&self) -> &Allocation<A> {
        self.buffer.allocation()
    }

    /// Returns the buffer data's capacity.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<A: Allocator> StagedBuffer<A> {
    /// Creates a staged buffer with the given data and usage.
    ///
    /// # Note
    /// The usage flag `TRANSFER_DST` will be used automatically.
    #[must_use]
    pub fn new<T: Sized>(
        context: &Context,
        allocator: &A,
        data: &[T],
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        let device = context.device();
        let size = data.len() * size_of::<T>();

        // Create and allocate the source and destination buffers and memory.
        let src = Buffer::new(
            context,
            allocator,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let dst = Buffer::new(
            context,
            allocator,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Write the data to the source buffer.
        unsafe {
            write_to_memory(
                device,
                src.allocation.memory(),
                data.as_ptr(),
                0,
                data.len(),
            )?
        };

        // Submit the transfer.
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let copy_regions = [vk::BufferCopy::default()
            .src_offset(0)
            .dst_offset(0)
            .size(size as u64)];
        let command_buffer = context.temporary_command_buffer()?;
        let command_buffers = [command_buffer];
        let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        unsafe {
            vk_query!(device.begin_command_buffer(command_buffer, &begin_info))?;
            device.cmd_copy_buffer(command_buffer, src.buffer, dst.buffer, &copy_regions);
            vk_query!(device.end_command_buffer(command_buffer))?;

            vk_query!(device.queue_submit(context.main_queue(), &submit_infos, vk::Fence::null()))?;
            vk_query!(device.queue_wait_idle(context.main_queue()))?;
        };

        Ok(Self(dst))
    }

    /// Returns the buffer.
    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &Buffer<A> {
        &self.0
    }
}

/// Maps the memory and copies the data to it.
#[must_use]
unsafe fn write_to_memory<T: Sized>(
    device: &Device,
    memory: vk::DeviceMemory,
    src: *const T,
    offset: u64,
    n: usize,
) -> Result {
    // Get the mapped memory.
    let size = (n * size_of::<T>()) as u64;
    let dst = unsafe {
        vk_query!(device.map_memory(memory, offset, size, vk::MemoryMapFlags::empty()))? as *mut T
    };

    // Write and unmap.
    unsafe {
        ptr::copy_nonoverlapping(src, dst, n);
        device.unmap_memory(memory);
    }

    Ok(())
}
