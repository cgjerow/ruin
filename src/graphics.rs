use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::engine::GameState;
use crate::texture::Texture;

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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader-1.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Primary Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Primary Vertex Buffer"),
            mapped_at_creation: false,
            size: (std::mem::size_of::<Vertex>() * 6 * 1000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Primary Index Buffer"),
            mapped_at_creation: false,
            // this doens't need to be this big but I didn't want to do math rn
            size: (std::mem::size_of::<Vertex>() * 6 * 1000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
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
            vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
        })
    }

    pub fn load_texture_from_path(&self, path: &str) -> Texture {
        let image = image::open(path).expect(&format!("Unable to open asset: {}", path));
        self.create_gpu_texture(&image, path)
    }

    pub fn create_gpu_texture(&self, image: &DynamicImage, label: &str) -> Texture {
        Texture::from_image(&self.device, &self.queue, image, Some(label)).unwrap()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn load_image(&mut self) {}

    pub fn render(&mut self, game_state: GameState) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

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

            let tex = if game_state.show_mittens {
                &game_state.mittens
            } else {
                &game_state.tree
            };
            let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&tex.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

            render_pass.set_bind_group(0, &diffuse_bind_group, &[]);

            let mut all_vertices = Vec::new();
            let mut all_indices = Vec::new();
            /*
            for enemy in game_state.enemies.iter() {
                println!(
                    "drawing enemy {} at ({}, {})",
                    enemy.id, enemy.location.x, enemy.location.y
                );
                //
                // 1. Create rectangle (1x1 unit size) at enemy location
                let rect = world_to_clip(enemy.location.x, enemy.location.y, 1.0, 1.0);
                all_vertices.extend_from_slice(&rect.vertices());
            }
            */
            /*
            const VERTICES: &[Vertex] = &[
                Vertex {
                    position: [-0.0868241, 0.49240386, 0.0],
                    color: [0.5, 0.0, 0.5],
                }, // A
                Vertex {
                    position: [-0.49513406, 0.06958647, 0.0],
                    color: [0.5, 0.0, 0.5],
                }, // B
                Vertex {
                    position: [-0.21918549, -0.44939706, 0.0],
                    color: [0.5, 0.0, 0.5],
                }, // C
                Vertex {
                    position: [0.35966998, -0.3473291, 0.0],
                    color: [0.5, 0.0, 0.5],
                }, // D
                Vertex {
                    position: [0.44147372, 0.2347359, 0.0],
                    color: [0.5, 0.0, 0.5],
                }, // E
            ];
            */
            const VERTICES: &[Vertex] = &[
                // Changed
                Vertex {
                    position: [-0.0868241, 0.49240386, 0.0],
                    tex_coords: [0.4131759, 0.00759614],
                }, // A
                Vertex {
                    position: [-0.49513406, 0.06958647, 0.0],
                    tex_coords: [0.0048659444, 0.43041354],
                }, // B
                Vertex {
                    position: [-0.21918549, -0.44939706, 0.0],
                    tex_coords: [0.28081453, 0.949397],
                }, // C
                Vertex {
                    position: [0.35966998, -0.3473291, 0.0],
                    tex_coords: [0.85967, 0.84732914],
                }, // D
                Vertex {
                    position: [0.44147372, 0.2347359, 0.0],
                    tex_coords: [0.9414737, 0.2652641],
                }, // E
            ];

            const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];

            all_vertices.extend_from_slice(VERTICES);
            all_indices.extend_from_slice(INDICES);

            let padded_indices = if all_indices.len() % 2 != 0 {
                let mut padded = all_indices.to_vec();
                padded.push(0); // pad with one extra u16
                padded
            } else {
                all_indices.clone()
            };

            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&padded_indices));
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..padded_indices.len() as u32, 0, 0..1);
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    // color: [f32; 3], this was previously used for just rendering objects with filled colors
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/*
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
            Vertex {
                position: [x, y, 0.0],
                color: [x, y, 0.0],
            }, // bottom-left
            Vertex {
                position: [x + w, y, 0.0],
                color: [x, y, 0.0],
            }, // bottom-right
            Vertex {
                position: [x + w, y + h, 0.0],
                color: [x, y, 0.0],
            }, // top-right
            // Triangle 2
            Vertex {
                position: [x, y, 0.0],
                color: [x, y, 0.0],
            }, // bottom-left
            Vertex {
                position: [x + w, y + h, 0.0],
                color: [x, y, 0.0],
            }, // top-right
            Vertex {
                position: [x, y + h, 0.0],
                color: [x, y, 0.0],
            }, // top-left
        ]
    }
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
*/
