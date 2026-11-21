use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{ShaderType, TextureUsages},
        RenderApp,
    },
};
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct PixelOutline {
    pub color: Vec4,
    pub threshold: f32,
}

impl Default for PixelOutline {
    fn default() -> Self {
        Self {
            color: Vec4::W,
            threshold: 2.0,
        }
    }
}

pub(crate) fn let_pixel_outline_read_camera_depth(
    mut cameras: Query<
        &mut Camera3d,
        (
            With<PixelOutline>,
            Or<(Added<PixelOutline>, Changed<Camera3d>)>,
        ),
    >,
) {
    for mut camera_3d in &mut cameras {
        let depth_texture_usages = TextureUsages::from(camera_3d.depth_texture_usages);
        if !depth_texture_usages.contains(TextureUsages::TEXTURE_BINDING) {
            camera_3d.depth_texture_usages =
                (depth_texture_usages | TextureUsages::TEXTURE_BINDING).into();
        }
    }
}
