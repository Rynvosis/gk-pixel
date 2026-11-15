import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::color_operations::{linear_rgb_to_srgb, srgb_to_linear_rgb};

struct Posterize {
    levels: u32,
    @if(SIXTEEN_BYTE_ALIGNMENT)
    _wasm_padding_0: u32,
    @if(SIXTEEN_BYTE_ALIGNMENT)
    _wasm_padding_1: u32,
    @if(SIXTEEN_BYTE_ALIGNMENT)
    _wasm_padding_2: u32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: Posterize;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let source_colour = textureSample(source_texture, source_sampler, in.uv);
    let source_srgb = linear_rgb_to_srgb(source_colour.rgb);

    let intervals = f32(max(settings.levels, 2u) - 1u);
    let snapped_srgb = round(source_srgb * intervals) / intervals;
    let snapped_linear = srgb_to_linear_rgb(snapped_srgb);

    return vec4<f32>(snapped_linear, source_colour.a);
}