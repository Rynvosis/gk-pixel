import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::view::{
    View,
    depth_ndc_to_view_z,
    direction_world_to_view,
    frag_coord_to_ndc,
    frag_coord_to_uv,
    position_ndc_to_view
};

struct PixelEdgesThreshold {
    threshold: f32,
}
@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var depth_texture: texture_2d<f32>;
@group(0) @binding(2)
var<uniform> view: View;
@group(0) @binding(3)
var depth_sampler: sampler;
@group(0) @binding(4)
var<uniform> edges_threshold: PixelEdgesThreshold;

struct PixelCreases {
    lighten: f32,
    crease_threshold: f32,
}
@if(CREASES)
@group(1) @binding(0)
var<uniform> creases_settings: PixelCreases;
@if(CREASES)
@group(1) @binding(1)
var normal_texture: texture_2d<f32>;

struct PixelOutlines {
    color: vec4<f32>,
}
@if(OUTLINES)
@group(2) @binding(0)
var<uniform> outlines_settings: PixelOutlines;

// depth logic bugs out at infinite distance.
const EMPTY_RAW_DEPTH: f32 = 1e-6;
const SMALLEST_THRESHOLD: f32 = 1e-6;

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

fn frag_coords_distance_at_depth(
    frag_coord_a: vec4<f32>,
    frag_coord_b: vec4<f32>,
    raw_depth: f32,
) -> f32 {
    return distance(
            frag_coord_to_view_position(vec4(frag_coord_a.xy, raw_depth, 0.0)),
            frag_coord_to_view_position(vec4(frag_coord_b.xy, raw_depth, 0.0)),
        );
}

fn frag_coord_to_view_normal(frag_coord: vec4<f32>) -> vec3<f32> {
    let world_normal = textureLoad(normal_texture, vec2<i32>(frag_coord.xy), 0).xyz * 2.0 - 1.0;
    return direction_world_to_view(world_normal, view.view_from_world);
}

fn frag_coord_clamped_to_image(frag_coord: vec4<f32>) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(depth_texture));
    return vec4(clamp(frag_coord.xy, vec2(0.5), size - 0.5), frag_coord.zw);
}

@if(CREASES)
fn color_to_crease_color(color: vec4<f32>) -> vec4<f32> {
    return vec4(mix(color.rgb, vec3(1.0), creases_settings.lighten), color.a);
}

@if(OUTLINES)
fn color_to_outline_color(color: vec4<f32>) -> vec4<f32> {
    return mix(color, outlines_settings.color, outlines_settings.color.a);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.position;
    let scene_color = textureLoad(source_texture, vec2<i32>(frag_coord.xy), 0);

    let raw_depth = frag_coord_to_raw_depth(frag_coord.xy);
    let camera_distance = raw_depth_to_camera_distance(raw_depth);
    @if(CREASES)
    let normal = frag_coord_to_view_normal(frag_coord);

    // Strength is value / threshold, so 1.0 is the edge cutoff.
    var crease_strength = 0.0;
    var outline_strength = 0.0;

    let depth_threshold = max(edges_threshold.threshold, SMALLEST_THRESHOLD);

    @if(CREASES)
    let crease_threshold = max(creases_settings.crease_threshold, SMALLEST_THRESHOLD);

    let axes = array(vec2(1.0, 0.0), vec2(0.0, 1.0));
    for(var i = 0; i < 2; i++) {
        let offset = vec4<f32>(axes[i], 0.0, 0.0);
        let neighbor_after = frag_coord_clamped_to_image(frag_coord + offset);
        let neighbor_before = frag_coord_clamped_to_image(frag_coord - offset);

        let texel_size = frag_coords_distance_at_depth(neighbor_after, neighbor_before, raw_depth)
            / 2.0;
        let distance_behind_neighbors =
            2.0 * camera_distance - frag_coord_to_camera_distance(neighbor_after)
            - frag_coord_to_camera_distance(neighbor_before);
        let depth_strength = distance_behind_neighbors / (depth_threshold * texel_size);

        @if(OUTLINES) {
            outline_strength = max(outline_strength, depth_strength);
        }

        @if(CREASES) {
            let neighbor_after_normal = frag_coord_to_view_normal(neighbor_after);
            let neighbor_before_normal = frag_coord_to_view_normal(neighbor_before);

            // Mults by 0 if not on the same surface.
            let same_surface = step(abs(depth_strength), 1.0);
            // Mults by 0 if neighbour is a better crease.
            let faces_camera_most = step(
                    max(neighbor_after_normal.z, neighbor_before_normal.z),
                    normal.z,
                );
            let bend = 1.0
                - min(dot(normal, neighbor_after_normal), dot(normal, neighbor_before_normal));

            crease_strength = max(
                    crease_strength,
                    same_surface * faces_camera_most * bend / crease_threshold,
                );
        }
    }

    var color = scene_color;
    @if(CREASES) {
        color = select(color, color_to_crease_color(scene_color), crease_strength >= 1.0);
    }
    @if(OUTLINES) {
        // Outlines take precedence over creases.
        color = select(color, color_to_outline_color(scene_color), outline_strength >= 1.0);
    }
    return color;
}
