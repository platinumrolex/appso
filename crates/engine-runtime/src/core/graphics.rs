use std::sync::Arc;
use wgpu::{Adapter, DeviceType, Instance, Surface};
use wgpu_text::glyph_brush::ab_glyph::FontArc;
use winit::window::Window;
use crate::core::Camera;
use diagram_app::DiagramApp;


pub struct WgpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub brush: wgpu_text::TextBrush<FontArc>,
    pub quad_pipeline: wgpu::RenderPipeline,
}

const GPU_USAGE_RECOMMENDED: bool = true;

// -----------------------------------------------------------------------------
// Graphics Initialization
// -----------------------------------------------------------------------------
pub fn init_graphics(
    instance: &Instance,
    window: Arc<Window>,
    camera: &mut Camera,
    diagram: &DiagramApp,
) -> WgpuState {
    println!("[Graphics] Initializing WGPU System...");
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = pollster::block_on(select_adapter(instance, &surface, false));
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            ..Default::default()
        },
    ))
    .expect("Failed to create device");

    let size = window.inner_size();
    camera.screen_width = size.width as f32;
    camera.screen_height = size.height as f32;

    let ez = camera.effective_zoom();
    let (wcx, wcy) = diagram.get_world_center_of_nodes();
    camera.pan_x = (size.width as f32 / 2.0) - (wcx * ez);
    camera.pan_y = (size.height as f32 / 2.0) - (wcy * ez);

    let caps = surface.get_capabilities(&adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: caps.formats[0],
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let font = FontArc::try_from_slice(include_bytes!("../../../../assets/VERDANA.TTF")).unwrap();
    let brush = wgpu_text::BrushBuilder::using_font(font).build(
        &device,
        config.width,
        config.height,
        config.format,
    );

    // --- NEW QUAD PIPELINE CODE ---
    let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Quad Shader"),
    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
        struct VertexOutput {
            @builtin(position) clip_pos: vec4<f32>,
            @location(0) color: vec4<f32>,
            @location(1) @interpolate(flat) size: vec2<f32>,
            @location(2) @interpolate(flat) radius: f32,
            @location(3) local_pos: vec2<f32>,
        };

        @vertex
        fn vs_main(
            @builtin(vertex_index) v_idx: u32,
            @location(0) rect: vec4<f32>,
            @location(1) color: vec4<f32>,
            @location(2) radius: f32,
            @location(3) screen: vec2<f32>,
        ) -> VertexOutput {
            var pos = array<vec2<f32>, 6>(
                vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
                vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0)
            );
            let p = pos[v_idx];
            let x = rect.x + p.x * rect.z;
            let y = rect.y + p.y * rect.w;
            let nx = (x / screen.x) * 2.0 - 1.0;
            let ny = 1.0 - (y / screen.y) * 2.0;

            var out: VertexOutput;
            out.clip_pos = vec4<f32>(nx, ny, 0.0, 1.0);
            out.color = color;
            out.size = rect.zw;
            out.radius = radius;
            out.local_pos = p * rect.zw;
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            let half_size = in.size * 0.5;
            let r = min(in.radius, min(half_size.x, half_size.y));
            let p = in.local_pos - half_size;
            let q = abs(p) - (half_size - vec2(r, r));
            let dist = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
            let aa = 1.0;
            let alpha = 1.0 - smoothstep(-aa, aa, dist);
            return vec4(in.color.rgb, in.color.a * alpha);
        }
    "#)),
});

let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Quad Layout"),
    bind_group_layouts: &[],
    immediate_size: 0, // Required for v23+
});

let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Quad Pipeline"),
    layout: Some(&quad_pipeline_layout),
    vertex: wgpu::VertexState {
        module: &quad_shader,
        entry_point: Some("vs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(), // Required for v23+
        buffers: &[wgpu::VertexBufferLayout {
        //    array_stride: 40, // 4 (rect) + 4 (color) + 2 (screen) floats = 10 * 4 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            array_stride: 44, // 11 * 4 bytes
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32, offset: 32, shader_location: 2 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 36, shader_location: 3 },
            ],
         //   attributes: &[
         //       wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
         //       wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
         //       wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 32, shader_location: 2 },
         //   ],
            
        }],
    },
    fragment: Some(wgpu::FragmentState {
        module: &quad_shader,
        entry_point: Some("fs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(), // Required for v23+
        targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    }),
    primitive: wgpu::PrimitiveState::default(),
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    multiview_mask: None, // Changed from None to None (v23 uses Option<NonZeroU32>)
    cache: None,
});
    // --- END NEW QUAD PIPELINE CODE ---

    println!("[Graphics] GPU Ready. Resources Allocated.");

    WgpuState {
        surface,
        device,
        queue,
        config,
        brush,
        quad_pipeline, // Add the pipeline to the returned struct
    }
}

pub async fn select_adapter(
    instance: &Instance,
    surface: &Surface<'_>,
    user_wants_high_perf: bool,
) -> Adapter {
    let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
    let target_type = if GPU_USAGE_RECOMMENDED || user_wants_high_perf {
        DeviceType::DiscreteGpu
    } else {
        DeviceType::IntegratedGpu
    };

    println!("[GPU] Searching for: {:?}", target_type);

    let mut selected = adapters
        .iter()
        .find(|a| {
            let info = a.get_info();
            info.device_type == target_type && a.is_surface_supported(surface)
        })
        .cloned();

    if selected.is_none() {
        println!("[GPU] Preferred type {:?} not found. Falling back...", target_type);
        selected = adapters.into_iter().find(|a| a.is_surface_supported(surface));
    }

    let adapter = selected.expect("CRITICAL: No compatible GPU adapter found!");
    let info = adapter.get_info();
    println!("[GPU] Final Selection: {} ({:?})", info.name, info.device_type);
    adapter
}

pub fn get_window_refresh_rate(window: &winit::window::Window) -> u32 {
    window
        .current_monitor()
        .and_then(|m| m.video_modes().next())
        .map(|mode| (mode.refresh_rate_millihertz() / 1000).max(10))
        .unwrap_or(60)
}