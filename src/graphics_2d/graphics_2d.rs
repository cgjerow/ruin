// Core 2D rendering engine
use std::collections::HashMap;
use std::sync::Arc;

use image::DynamicImage;
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;

use crate::camera_2d::Camera2D;
use crate::graphics::Graphics;
use crate::graphics_2d::{CameraUniform2D, Vertex};

use crate::texture::Texture;
use crate::world::World;

#[derive(Debug, Clone)]
pub struct RenderElement2D {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub z_order: f32, // for Y-based sorting (e.g., lower y = drawn on top)
    pub texture: Texture,
    pub texture_id: String,
    pub uv_coords: [[f32; 2]; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone)]
pub struct RenderQueue2D {
    pub elements: Vec<RenderElement2D>,
}

pub struct Graphics2D {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,
    background_color: Color,
    window: Arc<Window>,
    pub camera: Camera2D,
    camera_buffer: Buffer,
    camera_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    depth_texture: Texture,
}

impl Graphics2D {
    pub async fn new(window: Arc<Window>, camera: Camera2D) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let shader =
            device.create_shader_module(include_wgsl!("shaders/2d_camera_and_sprite.wgsl"));

        let depth_texture = Texture::create_depth_texture(&device, &config, "2d_depth_texture");

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("2D Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual, // only draw if closer
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),

            multiview: None,
            cache: None,
        });

        let mut camera_uniform = CameraUniform2D::new();
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Camera 2D Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            background_color: wgpu::Color {
                r: 116.0 / 255.0,
                b: 63.0 / 255.0,
                g: 57.0 / 255.0,
                a: 255.0 / 255.0,
            },
            window,
            camera,
            camera_buffer,
            camera_bind_group_layout,
            texture_bind_group_layout,
            depth_texture,
            render_pipeline,
        })
    }

    pub fn create_gpu_texture(&self, id: String, image: &DynamicImage, label: &str) -> Texture {
        Texture::from_image(id, &self.device, &self.queue, image, Some(label)).unwrap()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.camera.update_aspect_ratio(width, height);
    }

    pub fn update_camera(&mut self, target: [f32; 2]) {
        self.camera
            .update_follow(cgmath::vec2(target[0], target[1]));
        let mut uniform = CameraUniform2D::new();
        uniform.update(&self.camera);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn render(&mut self, render_queue: RenderQueue2D) -> Result<(), SurfaceError> {
        if !self.is_surface_configured {
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("2D Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("2D Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.background_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // clear to farthest
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let camera_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                layout: &self.camera_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                }],
                label: Some("2D Camera Bind Group"),
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(1, &camera_bind_group, &[]);

            let mut elements = render_queue.clone().elements.clone();
            elements.sort_by(|a, b| {
                a.z_order
                    .partial_cmp(&b.z_order)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Cache bind groups
            let mut bind_group_cache: HashMap<String, wgpu::BindGroup> = HashMap::new();
            let mut current_texture_id: Option<String> = None;

            for element in elements.iter() {
                // Only switch texture bind group if texture_id changes
                if current_texture_id.as_ref() != Some(&element.texture_id) {
                    let texture = &element.texture;

                    let bind_group = bind_group_cache
                        .entry(element.texture_id.clone())
                        .or_insert_with(|| {
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &self.texture_bind_group_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(&texture.view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                                    },
                                ],
                                label: Some("Texture Bind Group"),
                            })
                        });

                    pass.set_bind_group(0, &*bind_group, &[]);
                    current_texture_id = Some(element.texture_id.clone());
                }

                let vertices = Self::build_quad_vertices_2d(element);
                let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("2D Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("2D Index Buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    fn build_quad_vertices_2d(element: &RenderElement2D) -> [Vertex; 4] {
        let [w, h] = element.size;
        let [x, y] = element.position;
        let flip_x = if element.flip_x { -1.0 } else { 1.0 };
        let flip_y = if element.flip_y { -1.0 } else { 1.0 };

        let hw = w * 0.5 * flip_x;
        let hh = h * 0.5 * flip_y;

        let positions = [
            [x - hw, y + hh],
            [x + hw, y + hh],
            [x + hw, y - hh],
            [x - hw, y - hh],
        ];

        let tex = element.uv_coords;

        [
            Vertex {
                position: positions[0],
                tex_coords: tex[0],
            },
            Vertex {
                position: positions[1],
                tex_coords: tex[1],
            },
            Vertex {
                position: positions[2],
                tex_coords: tex[2],
            },
            Vertex {
                position: positions[3],
                tex_coords: tex[3],
            },
        ]
    }
}

impl Graphics for Graphics2D {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }

    fn update_camera(&mut self) {
        // Handled externally for 2D for now
    }

    fn set_background(&mut self, color: Color) {
        self.background_color = color;
    }

    fn render(&mut self, world: &World) {
        let queue = world.extract_render_queue_2d();
        let _ = self.render(queue);
    }

    fn load_texture_from_path(&self, path: &str) -> Texture {
        let image = image::open(path).unwrap().flipv();
        self.create_gpu_texture(path.to_string(), &image, path)
    }

    fn process_camera_event(&mut self, _event: &winit::event::WindowEvent) {}

    fn move_camera_for_follow(
        &mut self,
        position: [f32; 3],
        _v: [f32; 3],
        _a: [f32; 3],
        _o: [f32; 3],
    ) {
        self.update_camera([position[0], position[1]]);
    }
}
