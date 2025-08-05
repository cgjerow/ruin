use wgpu::{FragmentState, RenderPipelineDescriptor, VertexState};

pub fn create_2d_pipeline(
    label: &str,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    vertex_desc: &[wgpu::VertexBufferLayout],
    bind_group_layouts: &Vec<&wgpu::BindGroupLayout>,
    depth_stencil: Option<wgpu::DepthStencilState>,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_desc,
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        multisample: wgpu::MultisampleState::default(),
        depth_stencil,
        multiview: None,
        cache: None,
    })
}
