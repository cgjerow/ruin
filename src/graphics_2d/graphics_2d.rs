// Core 2D rendering engine
use std::collections::HashMap;
use std::sync::Arc;

use cgmath::Vector2;
use image::DynamicImage;
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;

use crate::camera_2d::Camera2D;
use crate::graphics::Graphics;
use crate::graphics_2d::shape_pipelines::create_color_shapes_pipeline;
use crate::graphics_2d::shape_tesselation::TessellatedShape2D;
use crate::graphics_2d::space::Space;
use crate::graphics_2d::{CameraUniform2D, ColorVertex, TextureVertex};

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
}

#[derive(Debug, Clone)]
pub struct RenderQueue2D {
    pub transparent: Vec<RenderElement2D>,
    pub opaque: Vec<RenderElement2D>,
}

pub struct Graphics2D {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,
    background_color: Color,
    pub camera: Camera2D,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    static_camera_buffer: Buffer,
    static_camera_bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    camera_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    depth_texture: Texture,
    texture_batch_context: TextureBatchContext,
    color_shapes_pipeline: RenderPipeline,
    debug_shapes_batch: DebugShapesBatch,
}

#[derive(Debug, Clone)]
struct TextureBatchContext {
    current_vertex_buffer_offset: u64,
    current_index_buffer_offset: u64,
    vertex_count_offset: u16,
    batched_vertices: Vec<TextureVertex>,
    batched_indices: Vec<u16>,
    previous_texture: String,
    bind_group_cache: HashMap<String, BindGroup>,
}

impl TextureBatchContext {
    fn new() -> Self {
        Self {
            current_vertex_buffer_offset: 0,
            current_index_buffer_offset: 0,
            vertex_count_offset: 0,
            batched_vertices: Vec::new(),
            batched_indices: Vec::new(),
            previous_texture: "".to_string(),
            bind_group_cache: HashMap::new(),
        }
    }

    fn enqueue_next_texture(
        &mut self,
        element: &RenderElement2D,
        queue: &mut Queue,
        pass: &mut RenderPass,
        vertex_buffer: &mut Buffer,
        index_buffer: &mut Buffer,
    ) {
        let should_flush_batch =
            self.previous_texture != "" && self.previous_texture != element.texture_id;
        if should_flush_batch {
            self.flush_batch(queue, pass, vertex_buffer, index_buffer);
        }
        let vertices = Self::build_quad_vertices_2d(&element);
        self.batched_vertices.extend_from_slice(&vertices);
        self.batched_indices.extend_from_slice(&[
            self.vertex_count_offset,
            self.vertex_count_offset + 1,
            self.vertex_count_offset + 2,
            self.vertex_count_offset + 2,
            self.vertex_count_offset + 3,
            self.vertex_count_offset,
        ]);
        self.vertex_count_offset += vertices.len() as u16;
        self.previous_texture = element.texture_id.clone();
    }

    fn flush_batch(
        &mut self,
        queue: &mut Queue,
        pass: &mut RenderPass,
        vertex_buffer: &mut Buffer,
        index_buffer: &mut Buffer,
    ) {
        if self.previous_texture == "" {
            return;
        }
        /*
        println!("Prev {:?}", self.previous_texture);
        println!(
            "Flush batch: {} vertices, offset: {}",
            self.batched_vertices.len(),
            self.current_vertex_buffer_offset
        );
        */
        pass.set_bind_group(
            0,
            &*self.bind_group_cache.get(&self.previous_texture).unwrap(),
            &[],
        );
        // Write vertex data to the pre-allocated buffer
        queue.write_buffer(
            &vertex_buffer,
            self.current_vertex_buffer_offset as wgpu::BufferAddress, // Offset: Start writing from the beginning of the buffer
            bytemuck::cast_slice(&self.batched_vertices),
        );

        // Write index data to the pre-allocated buffer
        queue.write_buffer(
            &index_buffer,
            self.current_index_buffer_offset as wgpu::BufferAddress, // Offset: Start writing from the beginning of the buffer
            bytemuck::cast_slice(&self.batched_indices),
        );

        // Now, set the buffers and draw, but specify the slice relevant to THIS sprite's data
        let vertex_slice_size = (self.batched_vertices.len() * std::mem::size_of::<TextureVertex>())
            as wgpu::BufferAddress;
        let index_slice_size =
            (self.batched_indices.len() * std::mem::size_of::<u16>()) as wgpu::BufferAddress;

        pass.set_vertex_buffer(
            0,
            vertex_buffer.slice(
                self.current_vertex_buffer_offset as wgpu::BufferAddress
                    ..(self.current_vertex_buffer_offset + vertex_slice_size)
                        as wgpu::BufferAddress,
            ),
        );
        pass.set_index_buffer(
            index_buffer.slice(
                self.current_index_buffer_offset as wgpu::BufferAddress
                    ..(self.current_index_buffer_offset + index_slice_size) as wgpu::BufferAddress,
            ),
            wgpu::IndexFormat::Uint16,
        );
        pass.draw_indexed(0..self.batched_indices.len() as u32, 0, 0..1); // The indices here are relative to the start of the SLICE

        // Increment offsets for the next sprite
        self.vertex_count_offset = 0;
        self.current_vertex_buffer_offset += vertex_slice_size;
        self.current_index_buffer_offset += index_slice_size;
        self.batched_indices.clear();
        self.batched_vertices.clear();
        self.previous_texture = "".to_string();
    }

    fn reset_context(&mut self) {
        self.current_index_buffer_offset = 0;
        self.current_vertex_buffer_offset = 0;
        self.vertex_count_offset = 0;
        self.batched_vertices.clear();
        self.batched_indices.clear();
        self.previous_texture = "".to_string();
    }

    fn build_quad_vertices_2d(element: &RenderElement2D) -> [TextureVertex; 4] {
        let [w, h] = element.size;
        let [x, y] = element.position;

        let hw = w * 0.5;
        let hh = h * 0.5;

        let positions = [
            [x - hw, y + hh],
            [x + hw, y + hh],
            [x + hw, y - hh],
            [x - hw, y - hh],
        ];

        let tex = element.uv_coords;

        [
            TextureVertex {
                position: positions[0],
                tex_coords: tex[0],
            },
            TextureVertex {
                position: positions[1],
                tex_coords: tex[1],
            },
            TextureVertex {
                position: positions[2],
                tex_coords: tex[2],
            },
            TextureVertex {
                position: positions[3],
                tex_coords: tex[3],
            },
        ]
    }
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
        let debug_shader =
            device.create_shader_module(include_wgsl!("shaders/2d_camera_and_color.wgsl"));

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
                buffers: &[TextureVertex::desc()],
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

        let color_shapes_pipeline = create_color_shapes_pipeline(
            &device,
            config.format,
            &debug_shader,
            &camera_bind_group_layout,
        );

        let mut camera_uniform = CameraUniform2D::new();
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Camera 2D Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("2D Camera Bind Group"),
        });

        let static_camera_uniform = CameraUniform2D::new();
        let static_camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Staic Camera 2D Buffer"),
            contents: bytemuck::cast_slice(&[static_camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let static_camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: static_camera_buffer.as_entire_binding(),
            }],
            label: Some("Static 2D Camera Bind Group"),
        });

        // Choose a generous initial capacity. You might need to resize if you hit limits.
        let initial_vertex_capacity = 1024 * 1024; // e.g., 1MB for vertices
        let initial_index_capacity = 256 * 1024; // e.g., 256KB for indices

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: initial_vertex_capacity as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Don't map immediately
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Index Buffer"),
            size: initial_index_capacity as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Don't map immediately
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
            camera,
            camera_buffer,
            camera_bind_group,
            static_camera_buffer,
            static_camera_bind_group,
            vertex_buffer,
            index_buffer,
            camera_bind_group_layout,
            texture_bind_group_layout,
            depth_texture,
            render_pipeline,
            texture_batch_context: TextureBatchContext::new(),
            color_shapes_pipeline,
            debug_shapes_batch: DebugShapesBatch::new(),
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

    pub fn update_camera(&mut self, target: [f32; 2], velocity: [f32; 2]) {
        self.camera.update_follow(
            cgmath::vec2(target[0], target[1]),
            cgmath::vec2(velocity[0], velocity[1]),
        );
        let mut uniform = CameraUniform2D::new();
        let mut static_uniform = CameraUniform2D::new();
        uniform.update(&self.camera);
        static_uniform.static_update(&self.camera);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
        self.queue.write_buffer(
            &self.static_camera_buffer,
            0,
            bytemuck::cast_slice(&[static_uniform]),
        );
    }

    pub fn render(&mut self, world: &World) -> Result<(), SurfaceError> {
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

        self.draw_game(world, &mut encoder, &view);
        self.draw_debug_batch(world, &mut encoder, &view);
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    fn draw_game(
        &mut self,
        world: &World,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view,
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

        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(1, &self.camera_bind_group, &[]);

        let render_queue = world.extract_render_queue_2d();
        let mut opaque = render_queue.clone().opaque.clone();
        opaque.sort_by(|a, b| {
            b.z_order
                .partial_cmp(&a.z_order)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for element in opaque.iter() {
            {
                self.texture_batch_context.enqueue_next_texture(
                    element,
                    &mut self.queue,
                    &mut pass,
                    &mut self.vertex_buffer,
                    &mut self.index_buffer,
                );
            }
        }
        // flush opaque
        self.texture_batch_context.flush_batch(
            &mut self.queue,
            &mut pass,
            &mut self.vertex_buffer,
            &mut self.index_buffer,
        );

        let mut transparent = render_queue.transparent.clone();
        transparent.sort_by(|a, b| {
            a.z_order
                .partial_cmp(&b.z_order)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for element in transparent.iter() {
            {
                self.texture_batch_context.enqueue_next_texture(
                    element,
                    &mut self.queue,
                    &mut pass,
                    &mut self.vertex_buffer,
                    &mut self.index_buffer,
                );
            }
        }
        // do final flush
        self.texture_batch_context.flush_batch(
            &mut self.queue,
            &mut pass,
            &mut self.vertex_buffer,
            &mut self.index_buffer,
        );

        self.texture_batch_context.reset_context();
    }

    pub fn build_debug_assets(&mut self, world: &World) {
        if !world.debug.enabled {
            return;
        }

        for (entity, animation) in world.animations.iter() {
            if let Some(t) = world.transforms_2d.get(&entity) {
                let current_frame = &animation.current_frame;

                // hitboxes
                for area in &current_frame.hitboxes {
                    if world.debug.show_hitboxes {
                        if area.active {
                            let pixels_per_unit = Vector2::from(current_frame.frame_pixel_dims);
                            let world_offset = Vector2::new(
                                (area.offset.x) * t.scale.x / pixels_per_unit.x,
                                (area.offset.y) * t.scale.y / pixels_per_unit.y,
                            );
                            let world_half_extents = Vector2::new(
                                area.shape.half_extents()[0] * t.scale.x.abs() / pixels_per_unit.x,
                                area.shape.half_extents()[1] * t.scale.y.abs() / pixels_per_unit.y,
                            );
                            let world_position = t.position + world_offset;

                            self.draw_debug_rect(
                                world_position,
                                world_half_extents,
                                [1.0, 0.0, 0.0, 1.0], // red with transparency
                                Space::World,
                            );
                        }
                    }
                }

                // hurtboxes
                for area in &current_frame.hurtboxes {
                    if world.debug.show_hurtboxes {
                        if area.active {
                            let pixels_per_unit = Vector2::from(current_frame.frame_pixel_dims);
                            let world_offset = Vector2::new(
                                (area.offset.x) * t.scale.x / pixels_per_unit.x,
                                (area.offset.y) * t.scale.y / pixels_per_unit.y,
                            );
                            let world_half_extents = Vector2::new(
                                area.shape.half_extents()[0] * t.scale.x.abs() / pixels_per_unit.x,
                                area.shape.half_extents()[1] * t.scale.y.abs() / pixels_per_unit.y,
                            );
                            let world_position = t.position + world_offset;

                            self.draw_debug_rect(
                                world_position,
                                world_half_extents,
                                [0.0, 0.0, 1.0, 1.0], // blue with transparency
                                Space::World,
                            );
                        }
                    }
                }

                for (_, area) in world.physical_colliders_2d.get(&entity).unwrap().iter() {
                    if world.debug.show_hurtboxes {
                        if area.active {
                            let pixels_per_unit = Vector2::new(1.0, 1.0); //Vector2::from(current_frame.frame_pixel_dims);
                            let world_offset = Vector2::new(
                                (area.offset.x) * t.scale.x / pixels_per_unit.x,
                                (area.offset.y) * t.scale.y / pixels_per_unit.y,
                            );
                            let world_half_extents = Vector2::new(
                                area.shape.half_extents()[0] * t.scale.x.abs() / pixels_per_unit.x,
                                area.shape.half_extents()[1] * t.scale.y.abs() / pixels_per_unit.y,
                            );
                            let world_position = t.position + world_offset;

                            self.draw_debug_rect(
                                world_position,
                                world_half_extents,
                                [0.0, 1.0, 1.0, 1.0], // blue with transparency
                                Space::World,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn draw_debug_rect(
        &mut self,
        center: Vector2<f32>,
        half_extents: Vector2<f32>,
        color: [f32; 4],
        space: Space,
    ) {
        let thickness = match space {
            Space::World => 0.03,
            Space::Canvas => 1.0,
        };

        let mut quad = TessellatedShape2D::rect_outline(half_extents.x, half_extents.y, thickness);
        quad.recenter(center);
        let mut circle = TessellatedShape2D::circle_outline(half_extents.x, thickness, 200);
        circle.recenter(center);

        // Insert quad with solid color, e.g., no texture bound
        self.debug_shapes_batch.push_tesselated_shape(quad, color);
        self.debug_shapes_batch.push_tesselated_shape(circle, color);
    }

    pub fn draw_debug_batch(
        &mut self,
        world: &World,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.build_debug_assets(world);

        if self.debug_shapes_batch.vertices.is_empty() {
            return;
        }

        {
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.debug_shapes_batch.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Index Buffer"),
                    contents: bytemuck::cast_slice(&self.debug_shapes_batch.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Debug Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Keep main pass contents
                        store: wgpu::StoreOp::Store,
                    },
                })],
                occlusion_query_set: None,
                timestamp_writes: None,
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.color_shapes_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.debug_shapes_batch.indices.len() as u32, 0, 0..1);

            self.debug_shapes_batch.clear();
        }

        {
            self.draw_debug_rect(
                Vector2 { x: 2.0, y: 2.0 },
                Vector2 { x: 0.5, y: 0.5 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 2.0, y: 2.0 },
                Vector2 { x: 3.0, y: 3.0 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 5.0, y: -5.0 },
                Vector2 { x: 300.0, y: 300.5 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 5.0, y: 5.0 },
                Vector2 { x: 300.0, y: 300.5 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 0.0, y: 0.0 },
                Vector2 { x: 3.0, y: 3.0 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 0.0, y: 0.0 },
                Vector2 { x: 0.5, y: 0.5 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 0.0, y: 0.0 },
                Vector2 { x: 3.0, y: 3.0 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 0.0, y: 0.0 },
                Vector2 { x: 300.0, y: 300.5 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );
            self.draw_debug_rect(
                Vector2 { x: 0.0, y: 0.0 },
                Vector2 { x: 3.0, y: 3.0 },
                [1.0, 0.0, 0.0, 1.0],
                Space::Canvas,
            );

            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.debug_shapes_batch.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Index Buffer"),
                    contents: bytemuck::cast_slice(&self.debug_shapes_batch.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            let mut static_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Debug Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Keep main pass contents
                        store: wgpu::StoreOp::Store,
                    },
                })],
                occlusion_query_set: None,
                timestamp_writes: None,
                depth_stencil_attachment: None,
            });

            static_pass.set_pipeline(&self.color_shapes_pipeline);
            static_pass.set_bind_group(0, &self.static_camera_bind_group, &[]);
            static_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            static_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            static_pass.draw_indexed(0..self.debug_shapes_batch.indices.len() as u32, 0, 0..1);
            self.debug_shapes_batch.clear();
        }
    }
}

impl Graphics for Graphics2D {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }

    fn update_camera(&mut self) {
        // Handled externally for 2D for now
    }

    fn get_camera_info(&self) -> crate::graphics::CameraInfo {
        return crate::graphics::CameraInfo {
            zoom: self.camera.zoom.clone(),
            position: [self.camera.position[0], self.camera.position[1], 0.0],
        };
    }

    fn set_background(&mut self, color: Color) {
        self.background_color = color;
    }

    fn render(&mut self, world: &World) {
        let _ = self.render(world);
    }

    fn load_texture_from_path(&mut self, id: &str, path: &str) -> Texture {
        let image = image::open(path).unwrap().flipv();
        let texture = self.create_gpu_texture(id.to_string(), &image, path);
        self.texture_batch_context.bind_group_cache.insert(
            id.to_string(),
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
            }),
        );
        return texture.clone();
    }

    fn process_camera_event(&mut self, _event: &winit::event::WindowEvent) {}

    fn move_camera_for_follow(
        &mut self,
        p: [f32; 3],
        v: [f32; 3],
        _acceleration: [f32; 3],
        _offset: [f32; 3],
    ) {
        self.update_camera(p[0..2].try_into().unwrap(), v[0..2].try_into().unwrap());
    }
}

struct DebugShapesBatch {
    pub vertices: Vec<ColorVertex>, // Includes position + color
    pub indices: Vec<u16>,          // For indexed quads
}

impl DebugShapesBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn push_tesselated_shape(&mut self, shape: TessellatedShape2D, color: [f32; 4]) {
        let base_index = self.vertices.len() as u16;

        for &pos in shape.vertices.iter() {
            self.vertices.push(ColorVertex {
                position: [pos.x, pos.y],
                color,
            })
        }
        self.indices.extend_from_slice(
            shape
                .indices
                .iter()
                .map(|f| f + base_index)
                .collect::<Vec<u16>>()
                .as_slice(),
        );
    }

    pub fn push_quad(&mut self, vertices: [Vector2<f32>; 4], color: [f32; 4]) {
        let base_index = self.vertices.len() as u16;

        for &pos in &vertices {
            self.vertices.push(ColorVertex {
                position: [pos.x, pos.y],
                color,
            });
        }

        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 1,
            base_index + 2,
            base_index + 2,
            base_index + 3,
            base_index + 3,
            base_index,
        ]);
    }
}
