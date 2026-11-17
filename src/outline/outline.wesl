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

fn distance_from_camera_at(texel: vec2<i32>) -> f32 {
    let uv = (vec2<f32>(texel) + 0.5) / vec2<f32>(textureDimensions(depth_texture));
    // depth read with sampler for WebGL2 compat
    let raw_depth = textureSampleLevel(depth_texture, depth_sampler, uv, 0.0).x;
    return -depth_ndc_to_view_z(raw_depth, view.clip_from_view, view.view_from_clip);
}

fn is_outline(texel: vec2<i32>) -> bool {
    let distance_from_camera = distance_from_camera_at(texel);
    let axes = array(vec2(1, 0), vec2(0, 1));
    for(var i = 0; i < 2; i++) {
        let axis = axes[i];
        // Edge check with slope, so steep faces dont create false positives.
        let distance_behind_neighbors =
            2 * distance_from_camera - distance_from_camera_at(texel + axis)
            - distance_from_camera_at(texel - axis);
        if distance_behind_neighbors > settings.depth_threshold {
            return true;
        }
    }
    return false;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let texel = vec2<i32>(in.position.xy);
    let scene = textureLoad(source_texture, texel, 0);
    if is_outline(texel) {
        return vec4(mix(scene.rgb, settings.color.rgb, settings.color.a), 1.0);
    }
    return scene;
}
