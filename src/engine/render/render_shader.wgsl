// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) global_pos: vec3<f32>,
}

;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput,) -> VertexOutput {
    let model_matrix = mat4x4<f32>(instance.model_matrix_0, instance.model_matrix_1, instance.model_matrix_2, instance.model_matrix_3,);
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.pos, 1.0);
    out.global_pos = model.pos;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let brightness = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
    var levels: f32 = 4;

    var pixel_pos_x = u32(in.clip_position.x);
    var pixel_pos_y = u32(in.clip_position.y);

    if ((pixel_pos_y + pixel_pos_x) % 2 == 0) {
        levels *= 2;
    }

    var out_color = round(color * levels) / levels;
    return out_color;
}
