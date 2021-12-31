struct Locals {
    mvp: mat4x4<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> locals: Locals;
[[group(0), binding(1)]]
var albedo: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = locals.mvp * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    out.tex_coord = in.tex_coord;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let albedo_dimensions = vec2<f32>(textureDimensions(albedo));
    return textureLoad(albedo, vec2<i32>(in.tex_coord * albedo_dimensions), 0) * vec4<f32>(in.normal, 1.0);
    //return vec4<f32>(in.normal, 1.0);
}
