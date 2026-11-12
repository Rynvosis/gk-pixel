#![expect(missing_docs, reason = "WIP")]

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy::window::{PrimaryWindow, WindowResized};

pub struct PixelPlugin;
impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (fit_pixel_camera.run_if(
                on_message::<WindowResized>.or_else(any_match_filter::<Changed<PixelCamera>>),
            ),),
        );
        app.add_observer(pixel_camera_added);
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
    mut cameras: Query<(Entity, &PixelCamera, &RenderTarget)>,
) {
    let window_size = window.physical_size();
    if window_size.min_element() == 0 {
        return;
    }

    for (camera_entity, pixel_camera, render_target) in &mut cameras {
        let pixels_per_texel = (window_size.y as f32 / pixel_camera.texels_tall as f32)
            .round()
            .max(1.0) as u32;
        let texels = UVec2::new(
            window_size.x.div_ceil(pixels_per_texel),
            window_size.y.div_ceil(pixels_per_texel),
        );

        let fits = matches!(
            render_target,
            RenderTarget::Image(image_target)
                if images.get(&image_target.handle).is_some_and(|image| image.size() == texels)
        );
        if !fits {
            let mut image =
                Image::new_target_texture(texels.x, texels.y, TextureFormat::Rgba8UnormSrgb, None);
            image.sampler = ImageSampler::nearest();
            let image = images.add(image);
            commands
                .entity(camera_entity)
                .insert(RenderTarget::Image(ImageRenderTarget::from(image.clone())));
        }
    }
}
