use ash::vk;
use ibis::{
    Arena, ColorFormat, CommandBuffer, ComputePipeline, Context, DescriptorSet,
    DescriptorSetLayout, Image, Pipeline, PipelineLayout, PrimaryCommandBuffer, RGBA, Renderer,
    Result, Sampler, StagedBuffer, Vec2, Vec4, Version, Window,
};
use rand::{RngExt, rng};
use std::time::{Duration, Instant};

#[repr(C)]
struct Vertex {
    c: Vec4,
    p: Vec2,
    v: Vec2,
    m: f32,
    _pad: [f32; 3],
}

const N_VERTICES: usize = 5000;

fn main() -> Result {
    let mut window = Window::builder()
        .size(1600, 1600)
        .title("Gravity Simulation")
        .build()?;
    let context = Context::builder()
        .app_name(c"Gravity Simulation")
        .app_version(Version::new(1, 0, 0))
        .use_validation(true)
        .build(&window)?;
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(4),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(2),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(2),
    ];
    let mut renderer = Renderer::builder()
        .max_descriptor_sets(8)
        .pool_sizes(&pool_sizes)
        .num_frames(2)
        .build(&context)?;
    let descriptor_bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .stage_flags(vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let descriptor_set_layout = DescriptorSetLayout::new(&context, &descriptor_bindings)?;
    let pipeline_layout = PipelineLayout::builder()
        .set_layouts(&[descriptor_set_layout.layout()])
        .push_constant_ranges(&[vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(4)])
        .build(&context)?;
    let gravity_compute = ComputePipeline::new(&context, &pipeline_layout, "shaders/gravity.spv")?;
    let fade_compute = ComputePipeline::new(&context, &pipeline_layout, "shaders/fade.spv")?;
    let trail_pipeline = Pipeline::builder()
        .color_attachment_format(vk::Format::R8G8B8A8_UNORM)
        .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
        .build(&pipeline_layout, "shaders/trail.spv", "shaders/frag.spv")?;
    let body_pipeline = Pipeline::builder()
        .color_attachment_format(renderer.format())
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .build(&pipeline_layout, "shaders/body.spv", "shaders/frag.spv")?;
    let tex_pipeline = Pipeline::builder()
        .color_attachment_format(renderer.format())
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .build(&pipeline_layout, "shaders/text.spv", "shaders/texf.spv")?;

    let vertex_size = size_of::<Vertex>() * N_VERTICES;
    let image_size = 16000000;
    let allocator = Arena::new(
        &context,
        &[
            (
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                (vertex_size * 2) + image_size,
            ),
            (
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_size * 2,
            ),
        ],
    )?;

    // Create image and sampler.
    let image = Image::new(
        &context,
        &allocator,
        window.width() as usize,
        window.height() as usize,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::STORAGE
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::COLOR_ATTACHMENT,
    )?;
    let sampler = Sampler::new(&context, vk::Filter::NEAREST)?;

    // Stage the SSBOs.
    let vertices = rand_vertices();
    let buffer1 = StagedBuffer::new(
        &context,
        &allocator,
        &vertices,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )?;
    let buffer2 = StagedBuffer::new(
        &context,
        &allocator,
        &vertices,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )?;

    // Bind to descriptors.
    let set_ab = DescriptorSet::new(&renderer, descriptor_set_layout.layout())?;
    set_ab.bind_buffer(
        0,
        buffer1.buffer(),
        vertex_size,
        vk::DescriptorType::STORAGE_BUFFER,
    );
    set_ab.bind_buffer(
        1,
        buffer2.buffer(),
        vertex_size,
        vk::DescriptorType::STORAGE_BUFFER,
    );
    set_ab.bind_storage_image(2, image.view(), vk::ImageLayout::GENERAL);
    set_ab.bind_combined_image_sampler(
        3,
        image.view(),
        sampler.sampler(),
        vk::ImageLayout::READ_ONLY_OPTIMAL,
    );

    let set_ba = DescriptorSet::new(&renderer, descriptor_set_layout.layout())?;
    set_ba.bind_buffer(
        0,
        buffer2.buffer(),
        vertex_size,
        vk::DescriptorType::STORAGE_BUFFER,
    );
    set_ba.bind_buffer(
        1,
        buffer1.buffer(),
        vertex_size,
        vk::DescriptorType::STORAGE_BUFFER,
    );
    set_ba.bind_storage_image(2, image.view(), vk::ImageLayout::GENERAL);
    set_ba.bind_combined_image_sampler(
        3,
        image.view(),
        sampler.sampler(),
        vk::ImageLayout::READ_ONLY_OPTIMAL,
    );

    const TARGET_FPS: u32 = 100;
    const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

    // Hot swap boolean.
    let mut flipped = false;
    let mut n_frames = 0;

    while !window.should_close() {
        let frame_start = Instant::now();
        window.update();

        // Bind the read and write buffers.
        let (compute_set, read_buffer, write_buffer) = if flipped {
            (&set_ba, &buffer2, &buffer1)
        } else {
            (&set_ab, &buffer1, &buffer2)
        };

        let scene = renderer.begin()?;

        scene.bind_descriptor_sets(
            &pipeline_layout,
            vk::PipelineBindPoint::COMPUTE,
            0,
            &[compute_set.set()],
        );
        scene.bind_descriptor_sets(
            &pipeline_layout,
            vk::PipelineBindPoint::GRAPHICS,
            0,
            &[compute_set.set()],
        );
        scene.push_constants(
            &pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            &(N_VERTICES as u32),
        );

        // Fade the image.
        let old_layout = if n_frames == 0 {
            vk::ImageLayout::UNDEFINED
        } else {
            vk::ImageLayout::READ_ONLY_OPTIMAL
        };
        scene.image_barrier(
            image.image(),
            vk::PipelineStageFlags2::FRAGMENT_SHADER,
            vk::AccessFlags2::SHADER_SAMPLED_READ,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            old_layout,
            vk::ImageLayout::GENERAL,
        );
        scene.bind_compute_pipeline(&fade_compute);
        let width = window.width();
        let height = window.height();
        scene.dispatch((width + 15) / 16, (height + 15) / 16, 1);

        // Compute gravity on particles.
        scene.buffer_barrier(
            read_buffer.buffer().handle(),
            vk::PipelineStageFlags2::VERTEX_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ,
        );
        scene.buffer_barrier(
            write_buffer.buffer().handle(),
            vk::PipelineStageFlags2::VERTEX_SHADER,
            vk::AccessFlags2::NONE,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_WRITE,
        );

        scene.bind_compute_pipeline(&gravity_compute);
        scene.dispatch((N_VERTICES as u32 + 15) / 16, 1, 1);

        scene.buffer_barrier(
            write_buffer.buffer().handle(),
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_WRITE,
            vk::PipelineStageFlags2::VERTEX_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ,
        );

        // Render to image.
        let render = scene.begin_render(&image, vk::ImageLayout::GENERAL, None);
        scene.bind_pipeline(&trail_pipeline);
        scene.draw(4, N_VERTICES, 0, 0);
        scene.end_render(render, vk::ImageLayout::READ_ONLY_OPTIMAL);

        scene.image_barrier(
            image.image(),
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags2::FRAGMENT_SHADER,
            vk::AccessFlags2::SHADER_SAMPLED_READ,
            vk::ImageLayout::READ_ONLY_OPTIMAL,
            vk::ImageLayout::READ_ONLY_OPTIMAL,
        );

        let render = scene.begin_render(&renderer, vk::ImageLayout::UNDEFINED, Some(RGBA::BLACK));

        // Render trails texture.
        scene.bind_pipeline(&tex_pipeline);
        scene.draw(3, 1, 0, 0);

        // Render bodies.
        scene.bind_pipeline(&body_pipeline);
        scene.draw(3, N_VERTICES, 0, 0);

        scene.end_render(render, vk::ImageLayout::PRESENT_SRC_KHR);

        renderer.submit(scene)?;

        flipped = !flipped;
        n_frames += 1;

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - elapsed);
        }
    }

    Ok(())
}

fn rand_vertices() -> Vec<Vertex> {
    (0..N_VERTICES)
        .map(|_| Vertex {
            c: Vec4::new(
                rng().random_range(0.2..1.0),
                rng().random_range(0.2..1.0),
                rng().random_range(0.2..1.0),
                rng().random_range(0.5..1.0),
            ),
            p: Vec2::new(
                rng().random_range(-1_000.0..1_000.0),
                rng().random_range(-1_000.0..1_000.0),
            ),
            v: Vec2::ZERO,
            m: rng().random_range(1.0..100.0),
            _pad: std::array::from_fn(|_| 0.0),
        })
        .collect()
}
