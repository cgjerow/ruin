use std::collections::HashMap;

use wgpu::include_wgsl;

use crate::graphics_2d::{
    shape_pipelines::create_2d_pipeline,
    shape_tesselation::TessellatedShape2D,
    vertex::{DebugInstanceVertex, Vertex},
};

pub type ThicknessKey = u32;
pub type Index = u16;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ShapeType {
    Rectangle,
    RectangleOutline(ThicknessKey),
    Circle,
    CircleOutline(ThicknessKey),
}

pub struct DebugRenderBatch {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    shape_meshes: HashMap<ShapeType, TessellatedShape2D>,
    instance_batches: HashMap<ShapeType, Vec<DebugInstanceVertex>>,
}

impl DebugRenderBatch {
    pub fn new(
        device: &wgpu::Device,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> Self {
        let initial_vertex_index_capacity: u64 = 256 * 1024;
        let initial_capacity: u64 = 1024 * 1024 * 100;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: initial_vertex_index_capacity as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Don't map immediately
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Index Buffer"),
            size: initial_vertex_index_capacity as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Don't map immediately
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: initial_capacity as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // Don't map immediately
        });

        let mut shape_meshes = HashMap::new();
        let world_thickness = 1;
        let thickness = 0.1;
        let segments = 200;
        let unit = 1.0;

        shape_meshes.insert(ShapeType::Rectangle, TessellatedShape2D::rect(unit, unit));
        shape_meshes.insert(
            ShapeType::RectangleOutline(world_thickness),
            TessellatedShape2D::rect_outline(unit, unit, thickness),
        );
        shape_meshes.insert(
            ShapeType::Circle,
            TessellatedShape2D::circle(unit, segments),
        );
        shape_meshes.insert(
            ShapeType::CircleOutline(world_thickness),
            TessellatedShape2D::circle_outline(unit, thickness, segments),
        );

        let mut instance_batches = HashMap::new();
        instance_batches.insert(ShapeType::Rectangle, Vec::new());
        instance_batches.insert(ShapeType::RectangleOutline(world_thickness), Vec::new());
        instance_batches.insert(ShapeType::Circle, Vec::new());
        instance_batches.insert(ShapeType::CircleOutline(world_thickness), Vec::new());

        let pipeline = create_2d_pipeline(
            "Debug Render Batch Pipeline",
            device,
            format,
            &device
                .create_shader_module(include_wgsl!("shaders/2d_camera_and_color_instanced.wgsl")),
            &[Vertex::desc(), DebugInstanceVertex::desc()],
            &Vec::from([camera_bind_group_layout]),
            None,
        );

        DebugRenderBatch {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            shape_meshes,
            instance_batches,
        }
    }

    pub fn add_instance(&mut self, shape_type: &ShapeType, instance: DebugInstanceVertex) {
        self.instance_batches
            .get_mut(shape_type)
            .unwrap()
            .push(instance);
    }

    pub fn flush_batch(
        &mut self,
        queue: &mut wgpu::Queue,
        pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        for (shape_type, instances) in &mut self.instance_batches {
            if instances.is_empty() {
                continue;
            }
            let mesh = self.shape_meshes.get(&shape_type).unwrap();
            let vertices: Vec<Vertex> = mesh
                .vertices
                .iter()
                .map(|v| Vertex {
                    position: [v.x, v.y],
                })
                .collect();

            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..instances.len() as u32);

            instances.clear();
        }
    }
}
