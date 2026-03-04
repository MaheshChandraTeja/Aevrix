struct Viewport { w: f32, h: f32 };
@group(0) @binding(0) var<uniform> vp : Viewport;

@group(1) @binding(0) var samp : sampler;
@group(1) @binding(1) var tex : texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> (
    @builtin(position) vec4<f32>,
    @location(0) vec2<f32>) {
  
  var pos = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
  );

  let P = pos[vid];
  
  
  
  let ndc = vec2<f32>( P.x * 2.0 - 1.0, 1.0 - P.y * 2.0 );
  return (vec4<f32>(ndc, 0.0, 1.0), P);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
  
  let c = textureSampleLevel(tex, samp, uv, 0.0);
  return c;
}
