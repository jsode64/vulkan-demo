use ash::vk;

use crate::{Allocation, Allocator, Context, RGBA, RenderTarget, Result, vk_query};

/// An image.
pub struct Image<A: Allocator> {
    /// The source context.
    context: Context,

    /// The memory allocation.
    allocation: Allocation<A>,

    /// The image handle.
    image: vk::Image,

    /// The image view handle.
    view: vk::ImageView,

    /// The image's width.
    width: usize,

    /// The image's height.
    height: usize,
}

/// An image sampler.
pub struct Sampler {
    /// The source context.
    context: Context,

    /// The sampler.
    sampler: vk::Sampler,
}

impl<A: Allocator> Image<A> {
    /// Creates an image with the given size format and usage.
    #[must_use]
    pub fn new(
        context: &Context,
        allocator: &A,
        width: usize,
        height: usize,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self> {
        let device = context.device();

        // Create the image.
        let extent = vk::Extent3D::default()
            .width(width as u32)
            .height(height as u32)
            .depth(1);
        let image_create_info = vk::ImageCreateInfo::default()
            .flags(vk::ImageCreateFlags::empty())
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = unsafe { vk_query!(device.create_image(&image_create_info, None))? };

        // Allocate and bind memory for the image before creating an image view.
        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        let allocation =
            allocator.allocate(&memory_requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
        unsafe { vk_query!(device.bind_image_memory(image, allocation.memory(), 0))? };

        // Create the image view after memory is bound.
        let component_mapping = vk::ComponentMapping::default()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY);
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let view_create_info = vk::ImageViewCreateInfo::default()
            .flags(vk::ImageViewCreateFlags::empty())
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(component_mapping)
            .subresource_range(subresource_range);
        let view = unsafe { vk_query!(device.create_image_view(&view_create_info, None))? };

        let context = context.clone();
        Ok(Self {
            context,
            allocation,
            image,
            view,
            width,
            height,
        })
    }

    /// Fills the image with the given color.
    #[must_use]
    pub fn fill(&self, color: RGBA) -> Result {
        let device = self.context.device();

        // Submit an image fill.
        let clear_color = vk::ClearColorValue {
            float32: [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ],
        };
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let command_buffer = self.context.temporary_command_buffer()?;
        let command_buffers = [command_buffer];
        let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        unsafe {
            vk_query!(device.begin_command_buffer(command_buffer, &begin_info))?;
            // Transition from UNDEFINED -> TRANSFER_DST_OPTIMAL before clearing with clear_color_image.
            let subresource_range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let to_transfer_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .image(self.image)
                .subresource_range(subresource_range);
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_transfer_barrier],
            );

            let subresource_ranges = [subresource_range];
            device.cmd_clear_color_image(
                command_buffer,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color,
                &subresource_ranges,
            );

            // Transition image to SHADER_READ_ONLY_OPTIMAL so it can be sampled by shaders.
            let barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(self.image)
                .subresource_range(subresource_range);
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            vk_query!(device.end_command_buffer(command_buffer))?;
            vk_query!(device.queue_submit(
                self.context.main_queue(),
                &submit_infos,
                vk::Fence::null()
            ))?;
            vk_query!(device.queue_wait_idle(self.context.main_queue()))?;
            device.free_command_buffers(self.context.transfer_pool(), &command_buffers);
        }

        Ok(())
    }

    /// Returns the image handle.
    #[inline]
    #[must_use]
    pub fn image(&self) -> vk::Image {
        self.image
    }

    /// Returns the image view handle.
    #[inline]
    #[must_use]
    pub fn view(&self) -> vk::ImageView {
        self.view
    }
}

impl<A: Allocator> Drop for Image<A> {
    fn drop(&mut self) {
        let device = self.context.device();

        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }
    }
}

impl Sampler {
    /// Creates a sampler with the given parameters.
    #[must_use]
    pub fn new(context: &Context, filter: vk::Filter) -> Result<Self> {
        let device = context.device();
        let create_info = vk::SamplerCreateInfo::default()
            .flags(vk::SamplerCreateFlags::empty())
            .mag_filter(filter)
            .min_filter(filter)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(1.0)
            .anisotropy_enable(false)
            .max_anisotropy(0.0)
            .compare_enable(false)
            .compare_op(vk::CompareOp::NEVER)
            .min_lod(0.0)
            .max_lod(1.0)
            .unnormalized_coordinates(false);
        let sampler = unsafe { vk_query!(device.create_sampler(&create_info, None))? };

        let context = context.clone();
        Ok(Self { context, sampler })
    }

    /// Returns the sampler handle.
    #[inline]
    #[must_use]
    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        let device = self.context.device();

        unsafe {
            device.destroy_sampler(self.sampler, None);
        }
    }
}

impl<A: Allocator> RenderTarget for Image<A> {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn view(&self) -> vk::ImageView {
        self.view
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}
