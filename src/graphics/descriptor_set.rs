use ash::vk;

use crate::{Allocator, Buffer, Context, Renderer, Result, vk_query};

/// A descriptor set layout.
pub struct DescriptorSetLayout {
    /// The source context.
    context: Context,

    /// The descriptor set layout.
    layout: vk::DescriptorSetLayout,
}

/// A descriptor set.
pub struct DescriptorSet {
    /// The source context.
    context: Context,

    /// The descriptor set.
    set: vk::DescriptorSet,
}

impl DescriptorSetLayout {
    /// Creates a descriptor set layout with the given bindings.
    #[must_use]
    pub fn new(context: &Context, bindings: &[vk::DescriptorSetLayoutBinding]) -> Result<Self> {
        let create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::empty())
            .bindings(bindings);
        let layout = unsafe {
            vk_query!(
                context
                    .device()
                    .create_descriptor_set_layout(&create_info, None)
            )?
        };

        let context = context.clone();
        Ok(Self { context, layout })
    }

    /// Returns the descriptor set layout handle.
    #[inline]
    #[must_use]
    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            let _ = self.context.device().device_wait_idle();

            self.context
                .device()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl DescriptorSet {
    /// Creates a descriptor set from the layout.
    #[must_use]
    pub fn new(renderer: &Renderer, layout: vk::DescriptorSetLayout) -> Result<Self> {
        let layouts = [layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(renderer.descriptor_pool())
            .set_layouts(&layouts);
        let set = unsafe {
            vk_query!(
                renderer
                    .context()
                    .device()
                    .allocate_descriptor_sets(&allocate_info)
            )?[0]
        };

        let context = renderer.context().clone();
        Ok(Self { context, set })
    }

    /// Binds the buffer to the binding at the given index.
    pub fn bind_buffer<A: Allocator>(
        &self,
        i: usize,
        buffer: &Buffer<A>,
        size: usize,
        descriptor_type: vk::DescriptorType,
    ) {
        let buffer_infos = [vk::DescriptorBufferInfo::default()
            .buffer(buffer.handle())
            .offset(0)
            .range(size as u64)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(i as u32)
            .descriptor_type(descriptor_type)
            .descriptor_count(1)
            .buffer_info(&buffer_infos)];
        unsafe { self.context.device().update_descriptor_sets(&writes, &[]) };
    }

    /// Binds an image and sampler.
    pub fn bind_combined_image_sampler(
        &self,
        binding: usize,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        layout: vk::ImageLayout,
    ) {
        let device = self.context.device();
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(layout)
            .image_view(image_view)
            .sampler(sampler)];
        let write = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(binding as u32)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info)];

        unsafe {
            device.update_descriptor_sets(&write, &[]);
        }
    }

    /// Binds a storage image.
    pub fn bind_storage_image(
        &self,
        binding: usize,
        image_view: vk::ImageView,
        layout: vk::ImageLayout,
    ) {
        let device = self.context.device();
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(layout)
            .image_view(image_view)];
        let write = [vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(binding as u32)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&image_info)];

        unsafe {
            device.update_descriptor_sets(&write, &[]);
        }
    }

    /// Returns the descriptor set handle.
    #[inline]
    #[must_use]
    pub fn set(&self) -> vk::DescriptorSet {
        self.set
    }
}

impl Drop for DescriptorSet {
    fn drop(&mut self) {
        unsafe {
            let _ = self.context.device().device_wait_idle();
        }
    }
}
