mod allocator;
mod arena;
mod buffer;
mod color;
mod command_buffer;
mod context;
mod descriptor_set;
mod image;
mod pipeline;
mod renderer;
mod scene;
mod swapchain;
mod vertex_input;
mod window;

pub use allocator::{Allocation, Allocator, find_memory_type};
pub use arena::Arena;
pub use buffer::{Buffer, DynBuffer, StagedBuffer};
pub use color::{ColorFormat, RGBA, pixel_f_to_u, pixel_u_to_f};
pub use command_buffer::{CommandBuffer, PrimaryCommandBuffer, RenderTarget};
pub use context::Context;
pub use descriptor_set::{DescriptorSet, DescriptorSetLayout};
pub use image::{Image, Sampler};
pub use pipeline::{ComputePipeline, Pipeline, PipelineLayout};
pub use renderer::Renderer;
pub use scene::Scene;
pub use swapchain::Swapchain;
pub use vertex_input::VertexInput;
pub use window::Window;

macro_rules! vk_query {
    ($result:expr) => {
        $result.map_err(|e: ash::vk::Result| crate::Error::VkError(e))
    };
}

pub(crate) use vk_query;
