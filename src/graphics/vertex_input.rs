use ash::vk;

use crate::Rect;

/// A vertex buffer input type.
pub trait VertexInput {
    /// The input binding descriptions.
    const BINDING_DESCRIPTIONS: &[vk::VertexInputBindingDescription];

    /// The input attribute descriptions.
    const ATTRIBUTE_DESCRIPTIONS: &[vk::VertexInputAttributeDescription];
}

impl VertexInput for Rect<f32> {
    const BINDING_DESCRIPTIONS: &[vk::VertexInputBindingDescription] =
        &[vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Rect<f32>>() as u32,
            input_rate: vk::VertexInputRate::INSTANCE,
        }];

    const ATTRIBUTE_DESCRIPTIONS: &[vk::VertexInputAttributeDescription] =
        &[vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 0,
        }];
}
