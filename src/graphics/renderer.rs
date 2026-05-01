use std::time::Duration;

use ash::{Device, vk};

use crate::{CommandBuffer, Context, Error, RenderTarget, Result, Scene, Swapchain, vk_query};

/// A builder for a renderer.
#[repr(C)]
pub struct RendererBuilder<'a> {
    /// The number of frames in flight to use.
    num_frames: usize,

    /// The ideal number of swapchain images.
    num_images: usize,

    /// The maximum number of descriptor sets.
    max_descriptor_sets: u32,

    /// The pool sizes for the descriptor pool.
    pool_sizes: &'a [vk::DescriptorPoolSize],
}

/// Per-frame synchronization objects and command buffer.
#[derive(Clone, Default)]
struct Frame {
    /// The "image available" semaphore.
    image_available: vk::Semaphore,

    /// The "render finished" semaphore.
    render_finished: vk::Semaphore,

    /// The "in flight" fence.
    in_flight: vk::Fence,

    /// The frame command buffer.
    command_buffer: vk::CommandBuffer,
}

/// A Vulkan renderer.
pub struct Renderer {
    /// The source context.
    context: Context,

    /// The swapchain.
    swapchain: Swapchain,

    /// The descriptor pool.
    descriptor_pool: vk::DescriptorPool,

    /// The command pool.
    command_pool: vk::CommandPool,

    /// The frame objects.
    frames: Box<[Frame]>,

    /// The index of the next frame to be rendered.
    frame_index: usize,

    /// The index of the next target swapchain image.
    image_index: usize,

    /// Whether the swapchain is outdated and needs to be recreated.
    is_swapchain_outdated: bool,
}

impl<'a> RendererBuilder<'a> {
    /// Builds the renderer.
    #[inline]
    #[must_use]
    pub fn build(self, context: &Context) -> Result<Renderer> {
        Renderer::new(self, context)
    }

    /// Sets the number of frames in flight.
    #[inline]
    #[must_use]
    pub const fn num_frames(mut self, n: usize) -> Self {
        self.num_frames = n;
        self
    }

    /// Sets the ideal number of swapchain images.
    #[inline]
    #[must_use]
    pub const fn num_images(mut self, n: usize) -> Self {
        self.num_images = n;
        self
    }

    /// Sets the maximum number of descriptor sets.
    #[inline]
    #[must_use]
    pub const fn max_descriptor_sets(mut self, n: usize) -> Self {
        self.max_descriptor_sets = n as u32;
        self
    }

    /// Sets the pool sizes for the descriptor pool.
    #[inline]
    #[must_use]
    pub fn pool_sizes(mut self, sizes: &'a [vk::DescriptorPoolSize]) -> Self {
        self.pool_sizes = sizes;
        self
    }
}

impl Renderer {
    /// The timeout time in nanoseconds.
    const TIMEOUT: u64 = Duration::from_secs(1).as_nanos() as u64;

    /// Returns a default renderer builder.
    #[inline]
    #[must_use]
    pub fn builder() -> RendererBuilder<'static> {
        RendererBuilder {
            num_frames: 2,
            num_images: 3,
            max_descriptor_sets: 0,
            pool_sizes: &[],
        }
    }

    /// Creates a renderer for the given context.
    #[must_use]
    pub(crate) fn new<'a>(builder: RendererBuilder<'a>, context: &Context) -> Result<Self> {
        let swapchain =
            Swapchain::new(context, vk::SwapchainKHR::null(), builder.num_images as u32)?;
        let descriptor_pool = create_descriptor_pool(context.device(), &builder)?;
        let command_pool = create_command_pool(context.device(), context.main_queue_index())?;
        let frames = create_frames(context.device(), command_pool, builder.num_frames)?;
        let context = context.clone();

        Ok(Self {
            context,
            swapchain,
            descriptor_pool,
            command_pool,
            frames,
            frame_index: 0,
            image_index: 0,
            is_swapchain_outdated: false,
        })
    }

    /// Begins rendering.
    #[must_use]
    pub fn begin(&mut self) -> Result<Scene> {
        let device = self.context.device();

        // Get the next frame index and wait for it.
        let frame_index = self.frame_index;
        let frame = &self.frames[frame_index];
        let fences = [frame.in_flight];
        unsafe {
            device
                .wait_for_fences(&fences, true, Self::TIMEOUT)
                .map_err(|e| Error::VkError(e))?
        };

        // Get next swapchain image.
        let acquire_image_info = vk::AcquireNextImageInfoKHR::default()
            .swapchain(self.swapchain.swapchain())
            .timeout(Self::TIMEOUT)
            .semaphore(frame.image_available)
            .fence(vk::Fence::null())
            .device_mask(1);
        let swapchain_device = self.swapchain.device();
        let (image_index, is_swapchain_outdated) =
            unsafe { vk_query!(swapchain_device.acquire_next_image2(&acquire_image_info))? };
        self.image_index = image_index as usize;
        self.is_swapchain_outdated = is_swapchain_outdated;

        // Begin the command buffer.
        let command_buffer = frame.command_buffer;
        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::empty());
        unsafe { vk_query!(device.begin_command_buffer(command_buffer, &begin_info))? };

        Ok(Scene::new(&self.context, command_buffer))
    }

    /// Submits the given scene.
    #[must_use]
    pub fn submit(&mut self, scene: Scene) -> Result {
        let device = self.context.device();
        let command_buffer = scene.command_buffer();
        let frame = &self.frames[self.frame_index];
        self.frame_index = (self.frame_index + 1) % self.num_frames();

        // End the command buffer.
        unsafe {
            vk_query!(device.end_command_buffer(command_buffer))?;
        }

        let fences = [frame.in_flight];
        unsafe { vk_query!(device.reset_fences(&fences))? };

        // Submit commands.
        let wait_semaphores = [frame.image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [command_buffer];
        let signal_semaphores = [frame.render_finished];
        let submit_infos = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];
        let graphics_queue = self.context.main_queue();
        unsafe {
            device
                .queue_submit(graphics_queue, &submit_infos, frame.in_flight)
                .map_err(|e| Error::VkError(e))?
        };

        // Present frame.
        let wait_semaphores = [frame.render_finished];
        let swapchains = [self.swapchain.swapchain()];
        let image_indices = [self.image_index as u32];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        let present_queue = self.context.present_queue();
        let is_swapchain_outdated = unsafe {
            self.swapchain
                .device()
                .queue_present(present_queue, &present_info)
                .map_err(|e| Error::VkError(e))?
        };

        if is_swapchain_outdated {
            return self.recreate_swapchain();
        }

        Ok(())
    }

    /// Recreates the swapchain.
    #[must_use]
    fn recreate_swapchain(&mut self) -> Result {
        let new_swapchain = Swapchain::new(
            self.context(),
            self.swapchain.swapchain(),
            self.swapchain.images().len() as u32,
        )?;
        self.swapchain.destroy(self.context.device());
        self.swapchain = new_swapchain;

        Ok(())
    }

    /// Returns the renderer's source context.
    #[inline]
    #[must_use]
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the renderer's swapchain.
    #[inline]
    #[must_use]
    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    /// Returns the renderer's descriptor pool.
    #[inline]
    #[must_use]
    pub fn descriptor_pool(&self) -> vk::DescriptorPool {
        self.descriptor_pool
    }

    /// Returns the renderer's command pool.
    #[inline]
    #[must_use]
    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    /// Returns the number of frames in flight.
    #[inline]
    #[must_use]
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    /// Returns the current frame index.
    #[inline]
    #[must_use]
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    /// Returns the renderer's images' format.
    #[inline]
    #[must_use]
    pub fn format(&self) -> vk::Format {
        self.swapchain.surface_format().format
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let device = self.context.device();

        let _ = unsafe { device.device_wait_idle() };

        for frame in &self.frames {
            unsafe {
                device.destroy_fence(frame.in_flight, None);
                device.destroy_semaphore(frame.render_finished, None);
                device.destroy_semaphore(frame.image_available, None);
            }
        }

        self.swapchain.destroy(device);

        unsafe {
            device.destroy_command_pool(self.command_pool, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        };
    }
}

impl RenderTarget for Renderer {
    #[inline]
    fn image(&self) -> vk::Image {
        self.swapchain.images()[self.image_index].image()
    }

    #[inline]
    fn view(&self) -> vk::ImageView {
        self.swapchain.images()[self.image_index].image_view()
    }

    #[inline]
    fn width(&self) -> usize {
        self.swapchain.extent().width as usize
    }

    #[inline]
    fn height(&self) -> usize {
        self.swapchain.extent().height as usize
    }
}

/// Creates a descriptor pool.
#[must_use]
fn create_descriptor_pool<'a>(
    device: &Device,
    builder: &RendererBuilder<'a>,
) -> Result<vk::DescriptorPool> {
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .flags(vk::DescriptorPoolCreateFlags::empty())
        .max_sets(builder.max_descriptor_sets as u32)
        .pool_sizes(&builder.pool_sizes);
    let descriptor_pool = unsafe { vk_query!(device.create_descriptor_pool(&create_info, None))? };

    Ok(descriptor_pool)
}

/// Creates a command pool.
#[must_use]
fn create_command_pool(device: &Device, queue_family_index: u32) -> Result<vk::CommandPool> {
    let create_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);
    let command_pool = unsafe {
        device
            .create_command_pool(&create_info, None)
            .map_err(|e| Error::VkError(e))?
    };

    Ok(command_pool)
}

/// Creates N frame objects.
#[must_use]
fn create_frames(
    device: &Device,
    command_pool: vk::CommandPool,
    num_frames: usize,
) -> Result<Box<[Frame]>> {
    // Allocate command buffers.
    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(num_frames as u32);
    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .map_err(|e| Error::VkError(e))?
    };

    let mut frames = vec![Frame::default(); num_frames].into_boxed_slice();
    for (i, frame) in frames.iter_mut().enumerate() {
        // Create semaphores.
        let semaphore_create_info =
            vk::SemaphoreCreateInfo::default().flags(vk::SemaphoreCreateFlags::empty());
        let image_available = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| Error::VkError(e))?
        };
        let render_finished = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| Error::VkError(e))?
        };

        // Create in flight fence.
        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .map_err(|e| Error::VkError(e))?
        };

        frame.image_available = image_available;
        frame.render_finished = render_finished;
        frame.in_flight = in_flight;
        frame.command_buffer = command_buffers[i];
    }

    Ok(frames)
}
