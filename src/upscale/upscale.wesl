import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct UpscaleSettings {
    pixels_per_texel: i32,
    origin: vec2<i32>,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var<uniform> settings: UpscaleSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    let offset = pixel - settings.origin;

    if any(offset < vec2<i32>(0)) {
        discard;
    }

    let texel = offset / settings.pixels_per_texel;
    let source_size = vec2<i32>(textureDimensions(source_texture));
    if any(texel >= source_size) {
        discard;
    }

    return textureLoad(source_texture, texel, 0);
}