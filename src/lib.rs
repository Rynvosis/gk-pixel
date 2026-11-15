#![expect(missing_docs, reason = "WIP")]

mod posterize;
mod upscale;

pub use crate::posterize::*;
use crate::upscale::{UpscaleSettings, UpscaleSource};
use bevy::{
    asset::AssetEventSystems,
    camera::{visibility::RenderLayers, CameraUpdateSystems, RenderTarget},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::{DebandDither, Tonemapping},
    },
};

const PIXEL_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub struct PixelPlugin;
impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            fit_pixel_camera
                //after post camera update, before asset event picks up the new image
                .before(AssetEventSystems)
                .after(CameraUpdateSystems),
        );
        app.add_observer(pixel_camera_added);

        app.add_plugins((upscale::plugin, plugin));
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
pub(crate) struct UpscaledBy(Entity);

fn disable_smoothing(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .insert((Msaa::Off, DebandDither::Disabled));
}

fn pixel_camera_added(
    add: On<Add<PixelCamera>>,
    mut commands: Commands,
    cameras: Query<(&RenderTarget, &Camera)>,
) {
    let Ok((render_target, camera)) = cameras.get(add.entity) else {
        return;
    };
    commands.spawn((
        Name::new("Upscale Camera"),
        Camera2d,
        Camera {
            order: camera.order + 1,
            ..default()
        },
        render_target.clone(),
        RenderLayers::none(),
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        Upscales(add.entity),
    ));
}

struct PixelLayout {
    pixels_per_texel: u32,
    texels: UVec2,
    origin: IVec2,
}
impl PixelLayout {
    fn fit(target_size: UVec2, texels_tall: u32) -> Self {
        let pixels_per_texel = (target_size.y as f32 / texels_tall as f32).round().max(1.0) as u32;
        let texels = UVec2::new(
            target_size.x.div_ceil(pixels_per_texel),
            target_size.y.div_ceil(pixels_per_texel),
        );
        let origin = (target_size.as_ivec2() - (texels * pixels_per_texel).as_ivec2()) / 2;
        Self {
            pixels_per_texel,
            texels,
            origin,
        }
    }

    fn upscale_settings(&self) -> UpscaleSettings {
        UpscaleSettings {
            origin: self.origin,
            pixels_per_texel: self.pixels_per_texel as i32,
        }
    }
}

fn fit_pixel_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cameras: Query<(Entity, &Camera, &PixelCamera, &RenderTarget, &UpscaledBy)>,
    upscale_cameras: Query<(&Camera, Option<&UpscaleSource>)>,
) {
    for (camera_entity, camera, pixel_camera, render_target, upscaled_by) in &cameras {
        let upscale_entity = upscaled_by.0;
        if !camera.is_active {
            commands.entity(upscale_entity).remove::<UpscaleSource>();
            continue;
        }

        let Ok((upscale_camera, previous_source)) = upscale_cameras.get(upscale_entity) else {
            continue;
        };
        let Some(target_size) = upscale_camera
            .physical_target_size()
            .filter(|size| size.min_element() > 0)
        else {
            continue;
        };

        let layout = PixelLayout::fit(target_size, pixel_camera.texels_tall);

        let handle = match reusable_image(render_target, &images, previous_source, layout.texels) {
            Some(handle) => handle,
            None => replace_render_image(
                &mut commands,
                &mut images,
                camera_entity,
                previous_source,
                layout.texels,
            ),
        };

        commands
            .entity(upscale_entity)
            .insert((layout.upscale_settings(), UpscaleSource(handle)));
    }
}

fn reusable_image(
    render_target: &RenderTarget,
    images: &Assets<Image>,
    previous_source: Option<&UpscaleSource>,
    size: UVec2,
) -> Option<Handle<Image>> {
    let target = render_target.as_image()?;
    let source = previous_source?;
    let image = images.get(target)?;
    (image.size() == size && source.0 == *target).then(|| target.clone())
}

fn replace_render_image(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    camera_entity: Entity,
    previous_source: Option<&UpscaleSource>,
    size: UVec2,
) -> Handle<Image> {
    if let Some(source) = previous_source {
        images.remove(&source.0);
    }
    let handle = images.add(Image::new_target_texture(
        size.x,
        size.y,
        PIXEL_TEXTURE_FORMAT,
        None,
    ));
    commands
        .entity(camera_entity)
        .insert(RenderTarget::from(handle.clone()));
    handle
}
