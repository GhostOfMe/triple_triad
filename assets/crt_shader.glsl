// https://yutannihilation.github.io/wgpugd-presentation-202205/en.html#/wgsl-code-for-retro-crt-monitor-effect

let CURVATURE: vec2<f32> = vec2<f32>(10.0, 10.0);
let RESOLUTION: vec2<f32> = vec2<f32>(320.0, 240.0);
let BRIGHTNESS: f32 = 4.0;
let PI: f32 = 3.14159;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}
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
 
@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  var remapped_tex_coords = curveRemapUV(in.uv);
    
  return textureSample(t, s, in.uv) * in.color 
      * scanLineIntensity(remapped_tex_coords.x, RESOLUTION.x, 0.1) 
      * scanLineIntensity(remapped_tex_coords.y, RESOLUTION.y, 1.0);
}
