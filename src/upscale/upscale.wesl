import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct UpscaleSettings {
    pixels_per_texel: i32,
    origin: vec2<i32>,
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> settings: UpscaleSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    let relative = pixel - settings.origin;

    if any(relative < vec2<i32>(0)) {
        discard;
    }

    let texel = relative / settings.pixels_per_texel;
    let dims = vec2<i32>(textureDimensions(tex));
    if any(texel >= dims) {
        discard;
    }

    return textureLoad(tex, texel, 0);
}