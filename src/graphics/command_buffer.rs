use ash::{Device, vk};
use std::slice;

use crate::{Allocator, Buffer, ColorFormat, ComputePipeline, Pipeline, PipelineLayout, RGBA};

/// A loadable command buffer.
pub trait CommandBuffer {
    /// Returns the device handle.
    #[must_use]
    fn device(&self) -> &Device;

    /// Returns the command buffer handle.
    #[must_use]
    fn command_buffer(&self) -> vk::CommandBuffer;

    /// Binds the graphics pipeline.
    fn bind_pipeline(&self, pipeline: &Pipeline) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline(),
            );
        }
    }

    /// Binds vertex buffers.
    fn bind_vertex_buffers(
        &self,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, first_binding, buffers, offsets);
        }
    }

    /// Binds one vertex buffer.
    fn bind_vertex_buffer<A: Allocator>(&self, binding: u32, buffer: &Buffer<A>) {
        let buffers = [buffer.handle()];
        let offsets = [0];
        self.bind_vertex_buffers(binding, &buffers, &offsets);
    }

    /// Pushes typed constants to the current pipeline.
    fn push_constants<T: Sized>(
        &self,
        pipeline_layout: &PipelineLayout,
        stage_flags: vk::ShaderStageFlags,
        offset: u32,
        constants: &T,
    ) {
        let data = unsafe {
            slice::from_raw_parts(
                (constants as *const T).cast::<u8>(),
                std::mem::size_of::<T>(),
            )
        };
        unsafe {
            self.device().cmd_push_constants(
                self.command_buffer(),
                pipeline_layout.pipeline_layout(),
                stage_flags,
                offset,
                data,
            );
        }
    }

    /// Draws.
    fn draw(
        &self,
        num_vertices: usize,
        num_instances: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_draw(
                command_buffer,
                num_vertices as u32,
                num_instances as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }

    /// Binds a compute pipeline.
    fn bind_compute_pipeline(&self, pipeline: &ComputePipeline) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline(),
            );
        }
    }

    /// Binds descriptor sets to the current pipeline.
    fn bind_descriptor_sets(
        &self,
        pipeline_layout: &PipelineLayout,
        bind_point: vk::PipelineBindPoint,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                bind_point,
                pipeline_layout.pipeline_layout(),
                first_set,
                descriptor_sets,
                &[],
            );
        }
    }

    /// Dispatches a compute shader.
    fn dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_dispatch(command_buffer, group_count_x, group_count_y, group_count_z);
        }
    }

    /// Inserts a buffer barrier.
    fn buffer_barrier(
        &self,
        buffer: vk::Buffer,
        src_stage: vk::PipelineStageFlags2,
        src_access: vk::AccessFlags2,
        dst_stage: vk::PipelineStageFlags2,
        dst_access: vk::AccessFlags2,
    ) {
        let barriers = [vk::BufferMemoryBarrier2::default()
            .src_stage_mask(src_stage)
            .src_access_mask(src_access)
            .dst_stage_mask(dst_stage)
            .dst_access_mask(dst_access)
            .buffer(buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE)];
        let dependency_info = vk::DependencyInfo::default()
            .dependency_flags(vk::DependencyFlags::empty())
            .memory_barriers(&[])
            .buffer_memory_barriers(&barriers)
            .image_memory_barriers(&[]);
        unsafe {
            self.device()
                .cmd_pipeline_barrier2(self.command_buffer(), &dependency_info);
        }
    }

    /// Inserts an image barrier.
    fn image_barrier(
        &self,
        image: vk::Image,
        src_stage: vk::PipelineStageFlags2,
        src_access: vk::AccessFlags2,
        dst_stage: vk::PipelineStageFlags2,
        dst_access: vk::AccessFlags2,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let barriers = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src_stage)
            .src_access_mask(src_access)
            .dst_stage_mask(dst_stage)
            .dst_access_mask(dst_access)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(image)
            .subresource_range(subresource_range)];
        let dependency_info = vk::DependencyInfo::default()
            .dependency_flags(vk::DependencyFlags::empty())
            .memory_barriers(&[])
            .buffer_memory_barriers(&[])
            .image_memory_barriers(&barriers);
        unsafe {
            self.device()
                .cmd_pipeline_barrier2(self.command_buffer(), &dependency_info);
        }
    }
}

/// A render target.
pub trait RenderTarget {
    /// Returns the target's width.
    #[must_use]
    fn width(&self) -> usize;

    /// Returns the target's height.
    #[must_use]
    fn height(&self) -> usize;

    /// Returns the target's image handle.
    #[must_use]
    fn image(&self) -> vk::Image;

    /// Returns the target's image view.
    #[must_use]
    fn view(&self) -> vk::ImageView;
}

/// A primary command buffer.
///
/// Exposes the functions only primary command buffers can use:
/// - `vkCmdBeginRendering` in `begin_render`
/// - `vkCmdEndRendering` in `end_render`
/// - `vkCmdExecuteCommands` in `execute`
pub trait PrimaryCommandBuffer: CommandBuffer {
    /// Begins rendering.
    ///
    /// Performst these commands in order:
    /// - Sets the viewport and scissor to match the target
    /// - Sets an image barrier to convert the image to `COLOR_ATTACHMENT_OPTIMAL`
    /// - Begins rendering
    fn begin_render<'a, T: RenderTarget>(
        &self,
        target: &'a T,
        old_layout: vk::ImageLayout,
        clear_color: Option<RGBA>,
    ) -> Render<'a, T> {
        // Set viewport and scissor.
        let width = target.width();
        let height = target.height();
        let extent = vk::Extent2D::default()
            .width(width as u32)
            .height(height as u32);
        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(width as f32)
            .height(height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];
        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(extent)];

        // Convert image to a renderable state.
        self.image_barrier(
            target.image(),
            vk::PipelineStageFlags2::empty(),
            vk::AccessFlags2::empty(),
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            old_layout,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        // Begin rendering.
        let clear_value = clear_color
            .map(RGBA::as_clear_color_value)
            .unwrap_or_default();
        let load_op = if clear_color.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        };
        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(extent);
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(target.view())
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_value)];
        let rendering_info = vk::RenderingInfo::default()
            .flags(vk::RenderingFlags::empty())
            .render_area(render_area)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&color_attachments);

        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            device.cmd_set_scissor(command_buffer, 0, &scissors);
            device.cmd_begin_rendering(command_buffer, &rendering_info);
        }

        Render { target }
    }

    /// Ends rendering and converts the image to the given format.
    fn end_render<T: RenderTarget>(&self, render: Render<T>, layout: vk::ImageLayout) {
        // Convert image to the given format.
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let image_barriers = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::empty())
            .dst_access_mask(vk::AccessFlags2::empty())
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(layout)
            .image(render.target.image())
            .subresource_range(subresource_range)];
        let dependency_info = vk::DependencyInfo::default()
            .dependency_flags(vk::DependencyFlags::empty())
            .memory_barriers(&[])
            .buffer_memory_barriers(&[])
            .image_memory_barriers(&image_barriers);

        let device = self.device();
        let command_buffer = self.command_buffer();
        unsafe {
            device.cmd_end_rendering(command_buffer);
            device.cmd_pipeline_barrier2(command_buffer, &dependency_info);
        }
    }

    /// Executes the secondary command buffer.
    fn execute<T: CommandBuffer>(&self, secondary: &T) {
        let secondaries = [secondary.command_buffer()];
        unsafe {
            self.device()
                .cmd_execute_commands(self.command_buffer(), &secondaries);
        }
    }
}

/// Information for a render.
pub struct Render<'a, T: RenderTarget> {
    /// The target being rendered to, or `None` if the render ended.
    target: &'a T,
}
