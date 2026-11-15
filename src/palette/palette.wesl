import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::color_operations::{linear_rgb_to_oklab, oklab_to_linear_rgb};

const MAX_PALETTE_COLORS: u32 = 256;
struct PaletteSnap {
    colors: array<vec4<f32>, MAX_PALETTE_COLORS>,
    color_count: u32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: PaletteSnap;

fn nearest_palette_color(linear_rgb: vec3<f32>) -> vec3<f32> {
    let oklab = linear_rgb_to_oklab(linear_rgb);
    var nearest = settings.colors[0].xyz;
    var nearest_distance_squared = dot(oklab - nearest, oklab - nearest);
    for (var i = 1u; i < settings.color_count; i++) {
        let candidate = settings.colors[i].xyz;
        let distance_squared = dot(oklab - candidate, oklab - candidate);
        if distance_squared < nearest_distance_squared {
            nearest = candidate;
            nearest_distance_squared = distance_squared;
        }
    }
    return oklab_to_linear_rgb(nearest);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let source_color = textureSample(source_texture, source_sampler, in.uv);
    let snapped_linear = nearest_palette_color(source_color.rgb);
    return vec4<f32>(snapped_linear, source_color.a);
}