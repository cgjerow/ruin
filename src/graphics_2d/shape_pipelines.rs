use wgpu::{FragmentState, RenderPipelineDescriptor, VertexState};

use crate::graphics_2d::vertex::ColorVertex;

pub fn create_color_shapes_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    shader_module: &wgpu::ShaderModule,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Debug Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Debug Quad Outlines"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: shader_module,
            entry_point: Some("vs_main"),
            buffers: &[ColorVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
