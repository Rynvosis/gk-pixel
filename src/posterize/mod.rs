use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    core_pipeline::{
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
        tonemapping::tonemapping,
        Core3dSystems,
    },
    ecs::{schedule::ScheduleConfigs, system::BoxedSystem},
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType, RenderApp},
    shader::ShaderRef,
};

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "posterize.wesl");
    app.add_plugins(FullscreenMaterialPlugin::<Posterize>::default());
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
#[non_exhaustive]
pub struct Posterize {
    pub levels: u32,
    #[cfg(target_arch = "wasm32")]
    _webgl2_padding_0: u32,
    #[cfg(target_arch = "wasm32")]
    _webgl2_padding_1: u32,
    #[cfg(target_arch = "wasm32")]
    _webgl2_padding_2: u32,
}
impl Posterize {
    pub fn new(levels: u32) -> Self {
        Self {
            levels,
            #[cfg(target_arch = "wasm32")]
            _webgl2_padding_0: 0,
            #[cfg(target_arch = "wasm32")]
            _webgl2_padding_1: 0,
            #[cfg(target_arch = "wasm32")]
            _webgl2_padding_2: 0,
        }
    }
}
impl Default for Posterize {
    fn default() -> Self {
        Self::new(4)
    }
}
impl FullscreenMaterial for Posterize {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("posterize.wesl")).with_source("embedded"),
        )
    }

    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        system.in_set(Core3dSystems::PostProcess).after(tonemapping)
    }
}
