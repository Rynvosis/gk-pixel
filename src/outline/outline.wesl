import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::view::{View, depth_ndc_to_view_z};

struct PixelOutline {
    color: vec4<f32>,
    depth_threshold: f32,
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var<uniform> settings: PixelOutline;
@group(0) @binding(2)
var depth_texture: texture_2d<f32>;
@group(0) @binding(3)
var<uniform> view: View;
@group(0) @binding(4)
var depth_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    //debug: return greyscale depth texture
    let depth = textureSampleLevel(depth_texture, depth_sampler, in.uv, 0.0);
    let linear_depth = -depth_ndc_to_view_z(depth.x, view.clip_from_view, view.view_from_clip);
    let grey = linear_depth / 40.0;
    return vec4(grey, grey, grey, 1.0);
}
