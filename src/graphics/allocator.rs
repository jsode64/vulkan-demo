use ash::vk;
use std::sync::Arc;

use crate::Result;

/// A trait for a GPU memory allocator.
pub trait Allocator: Sized {
    /// Returns a memory allocation for the given memory requirements and properties.
    #[must_use]
    fn allocate(
        &self,
        requirements: &vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Allocation<Self>>;

    /// Frees the memory.
    fn free(&self, allocation: &Allocation<Self>);
}

/// A GPU memory allocation.
pub struct Allocation<A: Allocator> {
    /// The source allocator.
    allocator: Arc<A>,

    /// The allocated memory handle.
    memory: vk::DeviceMemory,

    /// The allocation's offset.
    offset: usize,

    /// The allocation's size.
    size: usize,
}

impl<A: Allocator> Allocation<A> {
    /// Creates an allocation with the given allocator and memory information.
    #[inline]
    #[must_use]
    pub fn new(allocator: &Arc<A>, memory: vk::DeviceMemory, offset: usize, size: usize) -> Self {
        let allocator = allocator.clone();
        Self {
            allocator,
            memory,
            offset,
            size,
        }
    }

    /// Returns the allocation's source allocator.
    #[inline]
    #[must_use]
    pub fn allocator(&self) -> &A {
        &self.allocator
    }

    /// Returns the allocation's memory handle.
    #[inline]
    #[must_use]
    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    /// Returns the allocation's memory offset.
    #[inline]
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the allocation's size.
    #[inline]
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl<A: Allocator> Drop for Allocation<A> {
    fn drop(&mut self) {
        A::free(&self.allocator, self);
    }
}

/// Returns the index of the first matching memory type, or `None` if none match.
#[must_use]
pub fn find_memory_type(
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let iter = device_memory_properties
        .memory_types
        .into_iter()
        .take(device_memory_properties.memory_type_count as usize)
        .enumerate();
    for (i, memory_type) in iter {
        let index_matches = (filter & (1 << i)) != 0;
        let flags_match = memory_type.property_flags.contains(properties);
        if index_matches && flags_match {
            return Some(i as u32);
        }
    }

    None
}
