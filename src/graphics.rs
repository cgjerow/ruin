use rand::Rng;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct Graphics {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    background_color: wgpu::Color,
    #[allow(dead_code)]
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
}

impl Graphics {
    pub fn set_background(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub async fn new(window: Arc<Window>) -> anyhow::Result<Graphics> {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // This is the part of the window that we draw to
        let surface = instance.create_surface(window.clone()).unwrap();

        // handle for actual graphics card
        // use this to get info about the graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader-1.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Primary Render Pipeline Layout"),
                bind_group_layouts: &[],   // TODO
                push_constant_ranges: &[], // TODO
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Primary Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            window,
            render_pipeline,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let game_state = generate_fake_game_state(); // <-- call it here
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            for enemy in game_state.enemies.iter() {
                println!(
                    "drawing enemy {} at ({}, {})",
                    enemy.id, enemy.location.x, enemy.location.y
                );
                //
                // 1. Create rectangle (1x1 unit size) at enemy location
                let rect = world_to_clip(enemy.location.x, enemy.location.y, 1.0, 1.0);
                let vertex_data = rect.vertices();

                // 2. Create vertex buffer
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Enemy Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertex_data),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                // 3. Set vertex buffer
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                // 4. Draw 6 vertices (2 triangles)
                render_pass.draw(0..6, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0, // matches @location(0) in shader for position
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the 6 vertices forming 2 triangles for this rectangle
    pub fn vertices(&self) -> [Vertex; 6] {
        let x = self.x;
        let y = self.y;
        let w = self.width;
        let h = self.height;

        [
            // Triangle 1
            Vertex { position: [x, y] }, // bottom-left
            Vertex {
                position: [x + w, y],
            }, // bottom-right
            Vertex {
                position: [x + w, y + h],
            }, // top-right
            // Triangle 2
            Vertex { position: [x, y] }, // bottom-left
            Vertex {
                position: [x + w, y + h],
            }, // top-right
            Vertex {
                position: [x, y + h],
            }, // top-left
        ]
    }
}

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Player {
    pub location: Position,
}

#[derive(Debug)]
pub struct Enemy {
    pub id: String,
    pub location: Position,
}

#[derive(Debug)]
pub struct GameState {
    pub player: Player,
    pub enemies: Vec<Enemy>,
}

pub fn generate_fake_game_state() -> GameState {
    let mut rng = rand::rng();

    // Player always starts at (1, 1)
    let player = Player {
        location: Position { x: 1.0, y: 1.0 },
    };

    // Generate 5 enemies with random locations
    let enemies = (0..5)
        .map(|i| Enemy {
            id: format!("enemy_{}", i),
            location: Position {
                x: rng.random_range(0.0..10.0),
                y: rng.random_range(0.0..10.0),
            },
        })
        .collect();

    GameState { player, enemies }
}

fn world_to_clip(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    // assuming world coordinates in 0..10 range for both axes,
    // map to -1..1 clip space for rendering
    let clip_x = (x / 10.0) * 2.0 - 1.0;
    let clip_y = (y / 10.0) * 2.0 - 1.0;
    let clip_w = (width / 10.0) * 2.0;
    let clip_h = (height / 10.0) * 2.0;
    Rectangle::new(clip_x, clip_y, clip_w, clip_h)
}
