#![expect(missing_docs, reason = "WIP")]

mod upscale;

use crate::upscale::{UpscaleSettings, UpscaleSource};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraUpdateSystems, ImageRenderTarget, RenderTarget};
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::{PrimaryWindow, WindowResized};

const PIXEL_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub struct PixelPlugin;
impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (fit_pixel_camera.run_if(
                on_message::<WindowResized>.or_else(any_match_filter::<Changed<PixelCamera>>),
            ),)
                .before(CameraUpdateSystems),
        );
        app.add_observer(pixel_camera_added);

        app.add_plugins(upscale::plugin);
    }
}

#[derive(Component)]
#[component(on_add = disable_smoothing)]
pub struct PixelCamera {
    pub texels_tall: u32,
}

#[derive(Component)]
#[relationship(relationship_target = UpscaledBy)]
pub(crate) struct Upscales(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = Upscales, linked_spawn)]
pub(crate) struct UpscaledBy(Vec<Entity>);

fn disable_smoothing(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .insert((Msaa::Off, DebandDither::Disabled));
}

fn pixel_camera_added(
    add: On<Add<PixelCamera>>,
    mut commands: Commands,
    targets: Query<(&RenderTarget, &Camera)>,
) {
    let Ok((pixel_target, pixel_camera)) = targets.get(add.entity) else {
        return;
    };
    commands.spawn((
        Name::new("Upscale Camera"),
        Camera2d,
        Camera {
            order: pixel_camera.order + 1,
            ..default()
        },
        pixel_target.clone(),
        RenderLayers::none(),
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        Upscales(add.entity),
    ));
}

// for now assuming rendering straight to window not a custom other target
fn fit_pixel_camera(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(
        Entity,
        &PixelCamera,
        &RenderTarget,
        &UpscaledBy,
        &mut Projection,
    )>,
) {
    let window_size = window.physical_size();
    if window_size.min_element() == 0 {
        return;
    }

    for (camera_entity, pixel_camera, render_target, upscaled_by, mut projection) in &mut cameras {
        let Some(upscale_entity) = upscaled_by.iter().next() else {
            continue;
        };

        let pixels_per_texel = (window_size.y as f32 / pixel_camera.texels_tall as f32)
            .round()
            .max(1.0) as u32;
        let texels = UVec2::new(
            window_size.x.div_ceil(pixels_per_texel),
            window_size.y.div_ceil(pixels_per_texel),
        );
        let origin = (window_size.as_ivec2() - (texels * pixels_per_texel).as_ivec2()) / 2;

        let current = match render_target {
            RenderTarget::Image(image_target)
                if images
                    .get(&image_target.handle)
                    .is_some_and(|image| image.size() == texels) =>
            {
                Some(image_target.handle.clone())
            }
            _ => None,
        };

        let handle = match current {
            Some(handle) => handle,
            None => {
                projection.set_changed();
                let image =
                    Image::new_target_texture(texels.x, texels.y, PIXEL_TEXTURE_FORMAT, None);
                let handle = images.add(image);
                commands
                    .entity(camera_entity)
                    .insert(RenderTarget::Image(ImageRenderTarget::from(handle.clone())));
                handle
            }
        };

        commands.entity(upscale_entity).insert((
            UpscaleSettings {
                origin,
                pixels_per_texel: pixels_per_texel as i32,
            },
            UpscaleSource(handle),
        ));
    }
}
