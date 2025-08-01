use std::collections::HashMap;

use cgmath::Vector2;

use crate::{
    graphics_2d::{shape_tesselation::TessellatedShape2D, vertex::TextureVertex, RenderElement2D},
    texture::Texture,
};

#[derive(Debug, Clone)]
pub struct WorldRenderBatch {
    current_vertex_buffer_offset: u64,
    current_index_buffer_offset: u64,
    vertex_count_offset: u16,
    batched_texture_vertices: Vec<TextureVertex>,
    batched_indices: Vec<u16>,
    previous_texture: String,
    bind_group_cache: HashMap<String, wgpu::BindGroup>,
}

impl WorldRenderBatch {
    pub fn new() -> Self {
        Self {
            current_vertex_buffer_offset: 0,
            current_index_buffer_offset: 0,
            vertex_count_offset: 0,
            batched_texture_vertices: Vec::new(),
            batched_indices: Vec::new(),
            previous_texture: "".to_string(),
            bind_group_cache: HashMap::new(),
        }
    }

    pub fn add_texture(
        &mut self,
        id: String,
        texture: Texture,
        device: &mut wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.bind_group_cache.insert(
            id,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: texture_bind_group_layout,
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
    }

    pub fn enqueue_next_texture(
        &mut self,
        element: &RenderElement2D,
        queue: &mut wgpu::Queue,
        pass: &mut wgpu::RenderPass,
        vertex_buffer: &mut wgpu::Buffer,
        index_buffer: &mut wgpu::Buffer,
    ) {
        let should_flush_batch =
            self.previous_texture != "" && self.previous_texture != element.texture_id;
        if should_flush_batch {
            self.flush_batch(queue, pass, vertex_buffer, index_buffer);
        }

        let mut shape = TessellatedShape2D::from(
            &element.shape.scale(Vector2 {
                x: element.size[0],
                y: element.size[1],
            }),
            100,
        );
        shape.recenter(Vector2 {
            x: element.position[0],
            y: element.position[1],
        });

        let vertices = shape.into_tex(element.uv_coords);
        let indices: Vec<u16> = shape
            .indices
            .iter()
            .map(|i| i + self.vertex_count_offset)
            .collect();
        self.batched_texture_vertices.extend(&vertices);
        self.batched_indices.extend(indices);
        self.vertex_count_offset += vertices.len() as u16;
        self.previous_texture = element.texture_id.clone();
    }

    pub fn flush_batch(
        &mut self,
        queue: &mut wgpu::Queue,
        pass: &mut wgpu::RenderPass,
        vertex_buffer: &mut wgpu::Buffer,
        index_buffer: &mut wgpu::Buffer,
    ) {
        if self.previous_texture == "" {
            return;
        }
        pass.set_bind_group(
            0,
            &*self.bind_group_cache.get(&self.previous_texture).unwrap(),
            &[],
        );
        queue.write_buffer(
            &vertex_buffer,
            self.current_vertex_buffer_offset as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.batched_texture_vertices),
        );

        queue.write_buffer(
            &index_buffer,
            self.current_index_buffer_offset as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.batched_indices),
        );

        let vertex_slice_size = (self.batched_texture_vertices.len()
            * std::mem::size_of::<TextureVertex>())
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
        pass.draw_indexed(0..self.batched_indices.len() as u32, 0, 0..1);

        // Increment offsets for the next sprite
        self.vertex_count_offset = 0;
        self.current_vertex_buffer_offset += vertex_slice_size;
        self.current_index_buffer_offset += index_slice_size;
        self.batched_indices.clear();
        self.batched_texture_vertices.clear();
        self.previous_texture = "".to_string();
    }

    pub fn reset_context(&mut self) {
        self.current_index_buffer_offset = 0;
        self.current_vertex_buffer_offset = 0;
        self.vertex_count_offset = 0;
        self.batched_texture_vertices.clear();
        self.batched_indices.clear();
        self.previous_texture = "".to_string();
    }
}
