use cgmath::{Point3, Vector3};
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window; // bring Rng trait into scope

use crate::camera_3d::{Camera3D, CameraController};
use crate::graphics::Graphics;
use crate::graphics_3d::CameraUniform;
use crate::graphics_3d::Vertex;
use crate::texture::Texture;
use crate::world::World;

pub struct Graphics3D {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    background_color: wgpu::Color,
    #[allow(dead_code)]
    window: Arc<Window>,
    #[allow(dead_code)]
    camera: Camera3D,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: Texture,
    // maybe move to engine?
    pub camera_controller: Option<Box<dyn CameraController>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompareFunction {
    Undefined = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}

#[derive(Debug, Clone)]
pub struct RenderElement {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub texture: Texture,
    pub texture_id: String,
    pub uv_coords: [[f32; 2]; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

pub struct RenderQueue {
    pub elements: Vec<RenderElement>,
}

impl Graphics3D {
    pub fn set_background(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub async fn new(window: Arc<Window>, camera: Camera3D) -> anyhow::Result<Graphics3D> {
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

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/shader-1.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Primary Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,                  // Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
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

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            background_color: wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            window,
            camera,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_uniform,
            depth_texture,
            texture_bind_group_layout,
            camera_bind_group_layout,
            camera_controller: None,
        })
    }

    pub fn load_texture_from_path(&self, path: &str) -> Texture {
        let image = image::open(path)
            .expect(&format!("Unable to open asset: {}", path))
            .flipv();
        self.create_gpu_texture(path.to_string(), &image, path)
    }

    pub fn create_gpu_texture(&self, id: String, image: &DynamicImage, label: &str) -> Texture {
        Texture::from_image(id, &self.device, &self.queue, image, Some(label)).unwrap()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.camera.update_aspect_ratio(width, height);
        }
    }

    pub fn update_camera_config(
        &mut self,
        camera: Camera3D,
        camera_controller: Box<dyn CameraController>,
    ) {
        self.camera = camera;
        self.camera_controller = Some(camera_controller);
    }

    // Right now this runs every cycle, maybe need to optimize later but will also need to then
    // make sure this gets called whenever an update that affects camera occurs
    pub fn update_camera(&mut self) {
        if let Some(controller) = &self.camera_controller {
            controller.update_camera(&mut self.camera);
        }
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn move_camera_for_follow(
        &mut self,
        target: [f32; 3],
        velocity: [f32; 3],
        acceleration: [f32; 3],
        offset: [f32; 3],
    ) {
        let smooth_factor = 0.03;
        let velocity_scale = 0.4; // how far ahead camera looks based on speed
        let acceleration_scale = 0.15; // smaller scale for acceleration influence

        let target = Point3::new(target[0], target[1], target[2]);
        let offset_vec = Vector3::new(offset[0], offset[1], offset[2]);
        let velocity_vec = Vector3::new(velocity[0], velocity[1], velocity[2]);
        let acceleration_vec = Vector3::new(acceleration[0], acceleration[1], acceleration[2]);

        // Combine velocity and acceleration for predictive offset
        let predictive_offset =
            velocity_vec * velocity_scale + acceleration_vec * acceleration_scale;

        // Eye is offset from target in the direction of movement & acceleration
        let desired_eye = target + offset_vec + predictive_offset;

        // Smoothly interpolate camera position towards desired position
        self.camera.eye += (desired_eye - self.camera.eye) * smooth_factor;

        // Optionally look slightly ahead of the target
        self.camera.target = target + predictive_offset * 0.5;

        self.update_camera();
    }

    pub fn process_camera_events(&mut self, event: &winit::event::WindowEvent) {
        if let Some(controller) = &mut self.camera_controller {
            controller.process_events(event);
        }
    }

    fn build_vertices(element: &RenderElement) -> [Vertex; 4] {
        let [w, h, _z] = element.size;
        let [x, y, _z] = element.position;
        // Apply flipping scale
        let flip_x = if element.flip_x { -1.0 } else { 1.0 };
        let flip_y = if element.flip_y { -1.0 } else { 1.0 };

        let hw = w / 2.0 * flip_x;
        let hh = h / 2.0 * flip_y;

        // OVERRIDE Z FOR NOW
        let z = 0.0; // y / -1.0;

        let positions = [
            [x - hw, y + hh, z], // top-left
            [x + hw, y + hh, z], // top-right
            [x + hw, y - hh, z], // bottom-right
            [x - hw, y - hh, z], // bottom-left
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
    pub fn render(&mut self, to_render: RenderQueue) -> Result<(), wgpu::SurfaceError> {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });
            render_pass.set_bind_group(1, &camera_bind_group, &[]);

            // Group elements by texture id
            let mut groups: HashMap<String, Vec<RenderElement>> = HashMap::new();

            for element in to_render.elements {
                let tex_id = element.texture_id.clone();
                groups.entry(tex_id).or_default().push(element);
            }

            // Cache bind groups to avoid recreating them repeatedly
            let mut bind_group_cache: HashMap<String, wgpu::BindGroup> = HashMap::new();

            for (tex_id, group_elements) in groups {
                let mut all_vertices = Vec::new();
                let mut all_indices = Vec::new();
                const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
                for (i, e) in group_elements.iter().enumerate() {}
                for element in group_elements.iter() {
                    let vertices = Self::build_vertices(element);
                    let base_index = all_vertices.len() as u16;
                    all_vertices.extend_from_slice(&vertices);
                    // Offset indices by base_index
                    let offset_indices: Vec<u16> =
                        QUAD_INDICES.iter().map(|i| i + base_index).collect();
                    all_indices.extend_from_slice(&offset_indices);
                }

                // Create or reuse diffuse bind group for this texture
                let diffuse_bind_group =
                    bind_group_cache.entry(tex_id.clone()).or_insert_with(|| {
                        let tex = group_elements[0].texture.clone(); // all share the same texture

                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                            label: Some("cached_diffuse_bind_group"),
                        })
                    });

                render_pass.set_bind_group(0, &*diffuse_bind_group, &[]);
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Per-Group Vertex Buffer"),
                            contents: bytemuck::cast_slice(&all_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Per-Group Index Buffer"),
                            contents: bytemuck::cast_slice(&all_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                /*
                // Upload buffers for all vertices and indices in this group
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&all_vertices),
                );
                self.queue
                    .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&all_indices));

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                */
                render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
            }
        }

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

impl Graphics for Graphics3D {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }

    fn update_camera(&mut self) {
        self.update_camera();
    }

    fn set_background(&mut self, color: wgpu::Color) {
        self.set_background(color);
    }

    fn render(&mut self, world: &World) {
        self.render(world.extract_render_queue());
    }

    fn load_texture_from_path(&self, path: &str) -> Texture {
        self.load_texture_from_path(path)
    }

    fn process_camera_event(&mut self, event: &WindowEvent) {
        self.process_camera_events(event);
    }

    fn move_camera_for_follow(
        &mut self,
        position: [f32; 3],
        velocity: [f32; 3],
        acceleration: [f32; 3],
        offset: [f32; 3],
    ) {
        self.move_camera_for_follow(position, velocity, acceleration, offset);
    }
}
