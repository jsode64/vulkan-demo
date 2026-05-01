use ash::{Device, vk};
use std::fs::File;

use crate::{Context, Error, Result, graphics::vertex_input::VertexInput, vk_query};

/// A shader module.
struct ShaderModule<'a> {
    /// The source device for cleanup.
    device: &'a Device,

    /// The shader module handle.
    shader_module: vk::ShaderModule,
}

/// A builder for a pipeline layout.
pub struct PipelineLayoutBuilder<'a> {
    /// The descriptor set layouts.
    set_layouts: &'a [vk::DescriptorSetLayout],

    /// The push constant ranges.
    push_constant_ranges: &'a [vk::PushConstantRange],
}

/// A pipeline layout.
pub struct PipelineLayout {
    /// The source context.
    context: Context,

    /// The pipeline layout handle.
    pipeline_layout: vk::PipelineLayout,
}

/// A builder for a pipeline.
pub struct PipelineBuilder<'a> {
    /// The tesselation shader paths.
    tesselation_paths: Option<(&'a str, &'a str)>,

    /// The geometry shader path.
    geometry_path: Option<&'a str>,

    /// The binding descriptions.
    input_binding_descriptions: &'static [vk::VertexInputBindingDescription],

    /// The input attribute description.
    input_attribute_descriptions: &'static [vk::VertexInputAttributeDescription],

    /// The primitive topology used by the input assembly.
    topology: vk::PrimitiveTopology,

    /// The number of patch control points when tessellation is enabled.
    patch_control_points: u32,

    /// The color attachment format used by dynamic rendering.
    color_attachment_format: vk::Format,

    /// The optional depth attachment format used by dynamic rendering.
    depth_attachment_format: vk::Format,

    /// The optional stencil attachment format used by dynamic rendering.
    stencil_attachment_format: vk::Format,
}

/// A pipeline.
pub struct Pipeline {
    /// The source context.
    context: Context,

    /// The pipeline handle.
    pipeline: vk::Pipeline,
}

impl<'a> ShaderModule<'a> {
    /// Creates a shader module.
    #[must_use]
    pub fn new(device: &'a Device, path: &str) -> Result<Self> {
        let code = read_spirv(path)?;
        let create_info = vk::ShaderModuleCreateInfo::default()
            .flags(vk::ShaderModuleCreateFlags::empty())
            .code(&code);
        let shader_module = unsafe { vk_query!(device.create_shader_module(&create_info, None))? };

        Ok(Self {
            device,
            shader_module,
        })
    }

    /// Returns the shader module handle.
    #[inline]
    #[must_use]
    pub fn shader_module(&self) -> vk::ShaderModule {
        self.shader_module
    }
}

impl<'a> Drop for ShaderModule<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader_module, None);
        }
    }
}

impl<'a> PipelineLayoutBuilder<'a> {
    /// Builds the pipeline layout.
    #[inline]
    #[must_use]
    pub fn build(self, context: &Context) -> Result<PipelineLayout> {
        PipelineLayout::new(&self, context)
    }

    /// Sets the descriptor set layouts.
    #[inline]
    #[must_use]
    pub fn set_layouts(mut self, set_layouts: &'a [vk::DescriptorSetLayout]) -> Self {
        self.set_layouts = set_layouts;
        self
    }

    /// Sets the push constant ranges.
    #[inline]
    #[must_use]
    pub fn push_constant_ranges(
        mut self,
        push_constant_ranges: &'a [vk::PushConstantRange],
    ) -> Self {
        self.push_constant_ranges = push_constant_ranges;
        self
    }
}

/// A compute pipeline.
pub struct ComputePipeline {
    /// The source context.
    context: Context,

    /// The compute pipeline handle.
    pipeline: vk::Pipeline,
}

impl PipelineLayout {
    /// Returns a default pipeline layout builder.
    pub const fn builder<'a>() -> PipelineLayoutBuilder<'a> {
        PipelineLayoutBuilder {
            set_layouts: &[],
            push_constant_ranges: &[],
        }
    }

    /// Creates a pipeline layout from the builder.
    pub fn new(builder: &PipelineLayoutBuilder, context: &Context) -> Result<Self> {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .flags(vk::PipelineLayoutCreateFlags::empty())
            .set_layouts(builder.set_layouts)
            .push_constant_ranges(builder.push_constant_ranges);
        let pipeline_layout =
            unsafe { vk_query!(context.device().create_pipeline_layout(&create_info, None))? };

        let context = context.clone();
        Ok(PipelineLayout {
            context,
            pipeline_layout,
        })
    }

    /// Returns the pipeline layout handle.
    #[inline]
    #[must_use]
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

impl<'a> PipelineBuilder<'a> {
    /// Builds the pipeline.
    #[inline]
    #[must_use]
    pub fn build(
        self,
        layout: &PipelineLayout,
        vertex_path: &str,
        fragment_path: &str,
    ) -> Result<Pipeline> {
        Pipeline::new(self, &layout.context, layout, vertex_path, fragment_path)
    }

    /// Sets the tesselation shader paths.
    #[inline]
    #[must_use]
    pub const fn tesselation_paths(mut self, control: &'a str, evaluation: &'a str) -> Self {
        self.tesselation_paths = Some((control, evaluation));
        self
    }

    /// Sets the geometry shader path.
    #[inline]
    #[must_use]
    pub const fn geometry_path(mut self, path: &'a str) -> Self {
        self.geometry_path = Some(path);
        self
    }

    /// Uses vertex input data for the given type.
    #[inline]
    #[must_use]
    pub const fn vertex_input_type<T: VertexInput>(mut self) -> Self {
        self.input_binding_descriptions = T::BINDING_DESCRIPTIONS;
        self.input_attribute_descriptions = T::ATTRIBUTE_DESCRIPTIONS;
        self
    }

    /// Sets the primitive topology.
    #[inline]
    #[must_use]
    pub const fn topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Sets the number of patch control points for tessellation.
    #[inline]
    #[must_use]
    pub const fn patch_control_points(mut self, patch_control_points: u32) -> Self {
        self.patch_control_points = patch_control_points;
        self
    }

    /// Uses the dynamic rendering color attachment format.
    #[inline]
    #[must_use]
    pub const fn color_attachment_format(mut self, format: vk::Format) -> Self {
        self.color_attachment_format = format;
        self
    }

    /// Uses the dynamic rendering depth attachment format.
    #[inline]
    #[must_use]
    pub const fn depth_attachment_format(mut self, format: vk::Format) -> Self {
        self.depth_attachment_format = format;
        self
    }

    /// Uses the dynamic rendering stencil attachment format.
    #[inline]
    #[must_use]
    pub const fn stencil_attachment_format(mut self, format: vk::Format) -> Self {
        self.stencil_attachment_format = format;
        self
    }
}

impl Pipeline {
    /// Returns a default pipeline builder.
    pub const fn builder<'a>() -> PipelineBuilder<'a> {
        PipelineBuilder {
            tesselation_paths: None,
            geometry_path: None,
            input_binding_descriptions: &[],
            input_attribute_descriptions: &[],
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            patch_control_points: 0,
            color_attachment_format: vk::Format::UNDEFINED,
            depth_attachment_format: vk::Format::UNDEFINED,
            stencil_attachment_format: vk::Format::UNDEFINED,
        }
    }

    /// Creates a pipeline from the builder.
    pub fn new(
        builder: PipelineBuilder,
        context: &Context,
        layout: &PipelineLayout,
        vertex_path: &str,
        fragment_path: &str,
    ) -> Result<Self> {
        let device = context.device();

        // Create the shader stages.
        let vertex_module = ShaderModule::new(device, vertex_path)?;
        let fragment_module = ShaderModule::new(device, fragment_path)?;
        let tesselation_control_module;
        let tesselation_evaluation_module;
        let geometry_module;
        let main = c"main";
        let mut shader_stages = Vec::with_capacity(5);
        shader_stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .flags(vk::PipelineShaderStageCreateFlags::empty())
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module.shader_module())
                .name(main),
        );
        shader_stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .flags(vk::PipelineShaderStageCreateFlags::empty())
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module.shader_module())
                .name(main),
        );
        if let Some((control_path, evaluation_path)) = builder.tesselation_paths {
            tesselation_control_module = ShaderModule::new(device, control_path)?;
            tesselation_evaluation_module = ShaderModule::new(device, evaluation_path)?;
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .flags(vk::PipelineShaderStageCreateFlags::empty())
                    .stage(vk::ShaderStageFlags::TESSELLATION_CONTROL)
                    .module(tesselation_control_module.shader_module())
                    .name(main),
            );
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .flags(vk::PipelineShaderStageCreateFlags::empty())
                    .stage(vk::ShaderStageFlags::TESSELLATION_EVALUATION)
                    .module(tesselation_evaluation_module.shader_module())
                    .name(main),
            );
        }
        if let Some(geometry_path) = builder.geometry_path {
            geometry_module = ShaderModule::new(device, geometry_path)?;
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .flags(vk::PipelineShaderStageCreateFlags::empty())
                    .stage(vk::ShaderStageFlags::GEOMETRY)
                    .module(geometry_module.shader_module())
                    .name(main),
            );
        }

        // Create the pipeline.
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .flags(vk::PipelineVertexInputStateCreateFlags::empty())
            .vertex_binding_descriptions(builder.input_binding_descriptions)
            .vertex_attribute_descriptions(builder.input_attribute_descriptions);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .flags(vk::PipelineInputAssemblyStateCreateFlags::empty())
            .topology(builder.topology)
            .primitive_restart_enable(false);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .flags(vk::PipelineViewportStateCreateFlags::empty())
            .viewport_count(1)
            .scissor_count(1);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .flags(vk::PipelineDynamicStateCreateFlags::empty())
            .dynamic_states(&dynamic_states);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .flags(vk::PipelineRasterizationStateCreateFlags::empty())
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(1.0);
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .flags(vk::PipelineMultisampleStateCreateFlags::empty())
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA);
        let color_blend_attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .flags(vk::PipelineColorBlendStateCreateFlags::empty())
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let tessellation_state = builder.tesselation_paths.map(|_| {
            vk::PipelineTessellationStateCreateInfo::default()
                .flags(vk::PipelineTessellationStateCreateFlags::empty())
                .patch_control_points(builder.patch_control_points)
        });
        let color_attachment_formats = [builder.color_attachment_format];
        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats)
            .depth_attachment_format(builder.depth_attachment_format)
            .stencil_attachment_format(builder.stencil_attachment_format);
        let mut create_info = vk::GraphicsPipelineCreateInfo::default()
            .flags(vk::PipelineCreateFlags::empty())
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(layout.pipeline_layout)
            .render_pass(vk::RenderPass::null())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .push_next(&mut rendering_info);
        if let Some(tessellation_state) = tessellation_state.as_ref() {
            create_info = create_info.tessellation_state(tessellation_state);
        }
        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .map_err(|(_, e)| Error::VkError(e))?[0]
        };

        let context = context.clone();
        Ok(Self { context, pipeline })
    }

    /// Returns the pipeline handle.
    #[inline]
    #[must_use]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            let _ = self.context.device().device_wait_idle();
            self.context.device().destroy_pipeline(self.pipeline, None);
        }
    }
}

impl ComputePipeline {
    /// Creates a compute pipeline from the given path and layout.
    #[must_use]
    pub fn new(context: &Context, layout: &PipelineLayout, path: &str) -> Result<Self> {
        let device = context.device();
        let shader_module = ShaderModule::new(device, path)?;
        let shader_stage = vk::PipelineShaderStageCreateInfo::default()
            .flags(vk::PipelineShaderStageCreateFlags::empty())
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module.shader_module())
            .name(c"main");
        let create_infos = [vk::ComputePipelineCreateInfo::default()
            .flags(vk::PipelineCreateFlags::empty())
            .stage(shader_stage)
            .layout(layout.pipeline_layout())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)];
        let pipeline = unsafe {
            device
                .create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .map_err(|(_, e)| Error::VkError(e))?[0]
        };

        let context = context.clone();
        Ok(Self { context, pipeline })
    }

    /// Returns the compute pipeline handle.
    #[inline]
    #[must_use]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        let device = self.context.device();

        unsafe {
            let _ = device.device_wait_idle();

            device.destroy_pipeline(self.pipeline, None);
        }
    }
}

/// Returns the contents of the given SPIRV file.
#[must_use]
fn read_spirv(path: &str) -> Result<Box<[u32]>> {
    let mut f = File::open(path).map_err(|e| Error::IoError(e))?;
    let spirv = ash::util::read_spv(&mut f).map_err(Error::IoError)?;

    Ok(spirv.into_boxed_slice())
}
