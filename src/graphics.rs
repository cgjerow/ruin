use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window; // bring Rng trait into scope

use crate::camera::{
    Camera, CameraAction, CameraController, CameraInputMap, ThreeDimensionalCameraController,
    TwoDimensionalCameraController, UniversalCameraController,
};
use crate::engine::ElementsToRender;
use crate::game_element::StatefulElement;
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
    #[allow(dead_code)]
    camera: Camera,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    // maybe move to engine?
    pub camera_controller: Box<dyn CameraController>,
}

impl Graphics {
    pub fn set_background(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub async fn new(
        window: Arc<Window>,
        camera: Camera,
        camera_controller: Box<dyn CameraController>,
    ) -> anyhow::Result<Graphics> {
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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            camera,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_uniform,
            texture_bind_group_layout,
            camera_bind_group_layout,
            camera_controller,
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
            self.camera.update_aspect_ratio(width, height);
        }
    }

    pub fn update_camera(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn load_image(&mut self) {}

    pub fn render(&mut self, to_render: ElementsToRender) -> Result<(), wgpu::SurfaceError> {
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
            let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });
            render_pass.set_bind_group(1, &camera_bind_group, &[]);

            println!("RENDERING {} ELEMENTS", to_render.elements.len());
            // Group elements by texture id
            let mut groups: HashMap<String, Vec<StatefulElement>> = HashMap::new();

            for element in to_render.elements {
                let tex_id = element.get_texture_id();
                groups.entry(tex_id).or_default().push(element);
            }

            // Cache bind groups to avoid recreating them repeatedly
            let mut bind_group_cache: HashMap<String, wgpu::BindGroup> = HashMap::new();

            for (tex_id, group_elements) in groups {
                // fprintln!("IN GROUP {:?}", group_elements);
                let mut all_vertices = Vec::new();
                let mut all_indices = Vec::new();
                const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

                for element in group_elements.iter() {
                    if let Some(vertices) = element.build_vertices() {
                        // println!("HMMM {:?} {}", vertices, group_elements.iter().len());
                        let base_index = all_vertices.len() as u16;
                        all_vertices.extend_from_slice(&vertices);
                        // Offset indices by base_index
                        let offset_indices: Vec<u16> =
                            QUAD_INDICES.iter().map(|i| i + base_index).collect();
                        all_indices.extend_from_slice(&offset_indices);
                    }
                }

                // Create or reuse diffuse bind group for this texture
                let diffuse_bind_group =
                    bind_group_cache.entry(tex_id.clone()).or_insert_with(|| {
                        let tex = group_elements[0].get_texture(); // all share the same texture

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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    // color: [f32; 3], this was previously used for just rendering objects with filled colors
    pub tex_coords: [f32; 2],
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &crate::camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
