// --- Input struct for the vertex shader ---
struct VertexInput {
    @location(0) position: vec2<f32>,
}

// --- Output from vertex shader, input to fragment shader ---
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>, // MUST match fragment input
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0); // red
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
