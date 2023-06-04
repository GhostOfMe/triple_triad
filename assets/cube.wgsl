let CURVATURE: vec2<f32> = vec2<f32>(10.0, 10.0);
let RESOLUTION: vec2<f32> = vec2<f32>(320.0, 240.0);
let BRIGHTNESS: f32 = 4.0;
let PI: f32 = 3.14159;

fn curveRemapUV(uv_in: vec2<f32>) -> vec2<f32> {
    var uv_out: vec2<f32>;

    // as we near the edge of our screen apply greater distortion using a cubic function
    uv_out = uv_in * 2.0 - 1.0;
    var offset: vec2<f32> = abs(uv_out.yx) / CURVATURE;

    uv_out = uv_out + uv_out * offset * offset;
    return uv_out * 0.5 + 0.5;
}

fn scanLineIntensity(uv_in: f32, resolution: f32, opacity: f32) -> vec4<f32> {
     var intensity: f32 = sin(uv_in * resolution * PI * 2.0);
     intensity = ((0.5 * intensity) + 0.5) * 0.9 + 0.1;
     return vec4<f32>(vec3<f32>(pow(intensity, opacity)), 1.0);
 }

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct Locals {
    transform: mat4x4<f32>,
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> locals: Locals;

@group(0) @binding(1)
var t_color: texture_2d<f32>;

@group(0) @binding(2)
var s_sampler: sampler;


@vertex
fn vs_main(
    @location(0) pos: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = locals.transform * locals.rotation * pos;
    out.position = out.position / out.position.w;
    out.tex_coord = tex_coord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var remapped_tex_coords = curveRemapUV(in.position.xy / vec2<f32>(800.0, 600.0));
    var tex = textureSample(t_color, s_sampler, in.tex_coord);
    tex *= scanLineIntensity(remapped_tex_coords.y, RESOLUTION.y, 1.0);
    tex *= scanLineIntensity(remapped_tex_coords.x, RESOLUTION.x, 0.1); 
    var blend = dot(in.tex_coord - vec2<f32>(0.5, 0.5), in.tex_coord - vec2<f32>(0.5, 0.5));
    return mix(tex, vec4<f32>(0.0, 0.0, 0.0, 0.0), blend);
}
