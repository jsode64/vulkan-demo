use ash::vk;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{Allocation, Allocator, Context, Error, Result, find_memory_type, vk_query};

/// An arena allocation for a memory type.
#[derive(Default)]
struct MemoryTypeArena {
    /// The memory allocation handle.
    memory: vk::DeviceMemory,

    /// The memory type index.
    memory_type_index: u32,

    /// The capacity/size of the allocation.
    size: usize,

    /// The bytes of memory already used.
    used: AtomicUsize,
}

/// An arena allocator for GPU memory.
pub struct ArenaInner {
    /// The source context.
    context: Context,

    /// The memory allocations.
    arenas: Box<[MemoryTypeArena]>,

    /// The memory type index mask.
    memory_indices: u32,
}

/// An arena allocator for GPU memory.
#[derive(Clone)]
pub struct Arena(pub Arc<ArenaInner>);

impl Arena {
    /// Creates an arena allocator with arenas for each arena information given.
    ///
    /// Each element in `infos` tells the properties to allocate an arena for
    /// and its size.
    #[must_use]
    pub fn new(context: &Context, infos: &[(vk::MemoryPropertyFlags, usize)]) -> Result<Self> {
        Ok(Self(Arc::new(ArenaInner::new(context, infos)?)))
    }
}

impl Allocator for Arena {
    fn allocate(
        &self,
        requirements: &vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Allocation<Self>> {
        let arena = &self.0;

        // Find a suitable arena.
        let filter = requirements.memory_type_bits & arena.memory_indices;
        let Some(memory_type_index) =
            find_memory_type(arena.context.memory_properties(), filter, properties)
        else {
            return Err(Error::AllocatorIncompatible);
        };
        let arena = arena
            .arenas
            .iter()
            .find(|arena| arena.memory_type_index == memory_type_index)
            .ok_or(Error::AllocatorIncompatible)?;

        loop {
            // Make sure there's room for the data.
            let used = arena.used.load(Ordering::Relaxed);
            let offset = align_up(used, requirements.alignment as usize);
            let needed = offset + requirements.size as usize;
            if needed > arena.size {
                return Err(Error::AllocatorFull);
            }

            // Grab the room if the size is up to date.
            let had_room = arena
                .used
                .compare_exchange(used, needed, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok();
            if had_room {
                let arc_ref = Arc::new(self.clone());
                return Ok(Allocation::new(
                    &arc_ref,
                    arena.memory,
                    offset,
                    requirements.size as usize,
                ));
            }
        }
    }

    fn free(&self, _allocation: &Allocation<Self>) {}
}

impl Drop for ArenaInner {
    fn drop(&mut self) {
        for arena in &self.arenas {
            unsafe {
                self.context.device().free_memory(arena.memory, None);
            }
        }
    }
}

impl ArenaInner {
    /// Creates an arena allocator with arenas for each arena information given.
    ///
    /// Each element in `infos` tells the properties to allocate an arena for
    /// and its size.
    #[must_use]
    pub fn new(context: &Context, infos: &[(vk::MemoryPropertyFlags, usize)]) -> Result<Self> {
        // Allocate arenas.
        let mut arenas = Vec::with_capacity(infos.len());
        let mut memory_indices = 0;
        for &(properties, size) in infos {
            let memory_type_index =
                find_memory_type(context.memory_properties(), u32::MAX, properties)
                    .ok_or(Error::AllocatorIncompatible)?;
            let allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(size as u64)
                .memory_type_index(memory_type_index);
            let memory =
                unsafe { vk_query!(context.device().allocate_memory(&allocate_info, None))? };

            arenas.push(MemoryTypeArena {
                memory,
                memory_type_index,
                size,
                used: AtomicUsize::new(0),
            });

            memory_indices |= 1 << memory_type_index;
        }

        let context = context.clone();
        let arenas = arenas.into_boxed_slice();
        Ok(Self {
            context,
            arenas,
            memory_indices,
        })
    }
}

/// Aligns the size upward.
#[inline]
#[must_use]
fn align_up(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}
