mod error;
mod graphics;
mod math;
mod version;

pub use error::{Error, Result};
pub use graphics::{
    Allocation, Allocator, Arena, Buffer, ColorFormat, CommandBuffer, ComputePipeline, Context,
    DescriptorSet, DescriptorSetLayout, DynBuffer, Image, Pipeline, PipelineLayout,
    PrimaryCommandBuffer, RGBA, Renderer, Sampler, Scene, StagedBuffer, VertexInput, Window,
    find_memory_type, pixel_f_to_u, pixel_u_to_f,
};
pub use math::{Rect, Vec2, Vec3, Vec4};
pub use version::Version;

pub(crate) use graphics::{RenderTarget, Swapchain, vk_query};
pub(crate) use math::{Float, Num, Signed};

pub const VERSION: Version = Version::new(1, 0, 0);
