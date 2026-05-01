use ash::vk;

use crate::{CommandBuffer, Context, PrimaryCommandBuffer};

pub struct Scene {
    /// The source context.
    context: Context,

    /// The command buffer.
    command_buffer: vk::CommandBuffer,
}

impl Scene {
    /// Returns a scene.
    pub(crate) fn new(context: &Context, command_buffer: vk::CommandBuffer) -> Self {
        let context = context.clone();
        Self {
            context,
            command_buffer,
        }
    }
}

impl CommandBuffer for Scene {
    fn device(&self) -> &ash::Device {
        self.context.device()
    }

    fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}

impl PrimaryCommandBuffer for Scene {}
