struct VertexInput {
    @location(0) position: vec2<f32>,    // Vertex position (unit quad coords)
    @location(1) instance_pos: vec2<f32>, // Instance position
    @location(2) instance_scale: vec2<f32>, // Instance scale
    @location(3) instance_color: vec4<f32>, // Instance color
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera_matrix: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = input.position * input.instance_scale + input.instance_pos;

    output.clip_pos = camera_matrix * vec4<f32>(world_pos, 0.0, 1.0);
    output.color = input.instance_color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}

