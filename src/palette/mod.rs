use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    color::palettes::css::MAGENTA,
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
    embedded_asset!(app, "palette.wesl");
    app.add_plugins(FullscreenMaterialPlugin::<PaletteSnap>::default());
}

// const also replicated in palette.wesl, keep the 2 inline with each other!
const MAX_PALETTE_COLORS: usize = 256;

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct PaletteSnap {
    // colors are stored as Oklab, not RGB. W value stored but discarded.
    colors: [Vec4; MAX_PALETTE_COLORS],
    color_count: u32,
}
impl PaletteSnap {
    pub fn new(palette: impl IntoIterator<Item = impl Into<Color>>) -> Self {
        let mut colors = [Oklaba::from(MAGENTA).to_vec4(); MAX_PALETTE_COLORS];
        let mut color_count = 0;

        let mut palette = palette.into_iter();
        for (slot, color) in colors.iter_mut().zip(palette.by_ref()) {
            let color: Color = color.into();
            *slot = Oklaba::from(color).to_vec4();
            color_count += 1;
        }
        if color_count == 0 {
            color_count = 1;
            warn!("`PaletteSnap` was created with no colors, using default magenta. Pass at least one color to `PaletteSnap::new`.");
        }
        if palette.next().is_some() {
            warn!("`PaletteSnap` was created with colors exceeding the supported limit of {}, remaining colors were not used.",MAX_PALETTE_COLORS);
        }
        Self {
            colors,
            color_count,
        }
    }
}
impl Default for PaletteSnap {
    fn default() -> Self {
        Self::new(std::iter::empty::<Color>())
    }
}
impl FullscreenMaterial for PaletteSnap {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("palette.wesl")).with_source("embedded"),
        )
    }

    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        system.in_set(Core3dSystems::PostProcess).after(tonemapping)
    }
}
