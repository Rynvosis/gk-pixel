import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::view::{
    View,
    depth_ndc_to_view_z,
    frag_coord_to_ndc,
    frag_coord_to_uv,
    position_ndc_to_view
};

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var depth_texture: texture_2d<f32>;
@group(0) @binding(2)
var<uniform> view: View;
@group(0) @binding(3)
var depth_sampler: sampler;

struct PixelOutline {
    color: vec4<f32>,
    threshold: f32,
}
@group(1) @binding(0)
var<uniform> outline_settings: PixelOutline;

const EMPTY_RAW_DEPTH: f32 = 1e-6;

fn frag_coord_to_raw_depth(frag_coord_xy: vec2<f32>) -> f32 {
    let raw_depth = textureSampleLevel(
            depth_texture,
            depth_sampler,
            frag_coord_to_uv(frag_coord_xy, view.viewport),
            0.0,
        ).x;
    return max(raw_depth, EMPTY_RAW_DEPTH);
}

fn frag_coord_to_camera_distance(frag_coord: vec4<f32>) -> f32 {
    return raw_depth_to_camera_distance(frag_coord_to_raw_depth(frag_coord.xy));
}

fn raw_depth_to_camera_distance(raw_depth: f32) -> f32 {
    return -depth_ndc_to_view_z(raw_depth, view.clip_from_view, view.view_from_clip);
}

fn frag_coord_to_view_position(frag_coord: vec4<f32>) -> vec3<f32> {
    return position_ndc_to_view(frag_coord_to_ndc(frag_coord, view.viewport), view.view_from_clip);
}

fn frag_coord_is_outline(frag_coord: vec4<f32>) -> bool {
    let raw_depth = frag_coord_to_raw_depth(frag_coord.xy);
    let camera_distance = raw_depth_to_camera_distance(raw_depth);
    let frag_coord_with_raw_depth = vec4<f32>(frag_coord.xy, raw_depth, frag_coord.w);

    let axes = array(vec2(1.0, 0.0), vec2(0.0, 1.0));
    for(var i = 0; i < 2; i++) {
        let offset = vec4<f32>(axes[i], 0.0, 0.0);

        let texel_size = distance(
                frag_coord_to_view_position(frag_coord_with_raw_depth + offset),
                frag_coord_to_view_position(frag_coord_with_raw_depth - offset),
            ) / 2.0;

        let distance_behind_neighbors =
            2.0 * camera_distance - frag_coord_to_camera_distance(frag_coord + offset)
            - frag_coord_to_camera_distance(frag_coord - offset);
        if distance_behind_neighbors > outline_settings.threshold * texel_size {
            return true;
        }
    }
    return false;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureLoad(source_texture, vec2<i32>(in.position.xy), 0);
    if frag_coord_is_outline(in.position) {
        return vec4(
                mix(scene_color.rgb, outline_settings.color.rgb, outline_settings.color.a),
                1.0,
            );
    }
    return scene_color;
}
