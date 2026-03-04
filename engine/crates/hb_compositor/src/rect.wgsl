struct Viewport { w: f32, h: f32 };
@group(0) @binding(0) var<uniform> vp : Viewport;

struct Instance {
  xywh: vec4<f32>,   
  rgba: vec4<f32>,   
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32,
           @location(0) xywh: vec4<f32>,
           @location(1) rgba: vec4<f32>) -> (@builtin(position) vec4<f32>, @location(0) vec4<f32>) {
  
  var quad = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0)
  );

  let uv = quad[vid];
  let px = xywh.xy + uv * xywh.zw; 
  
  let ndc = vec2<f32>( (px.x / vp.w) * 2.0 - 1.0, 1.0 - (px.y / vp.h) * 2.0 );
  return (vec4<f32>(ndc, 0.0, 1.0), rgba);
}

@fragment
fn fs_main(@location(0) rgba: vec4<f32>) -> @location(0) vec4<f32> {
  return rgba;
}
