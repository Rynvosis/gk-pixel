use bevy::{
    material::descriptor::BindGroupLayoutDescriptor,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayoutEntries,
            PipelineCache, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        Render, RenderApp, RenderStartup,
        RenderSystems::PrepareBindGroups,
    },
};

use crate::edges::init_pixel_edges_pipeline;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<PixelOutline>::default(),
        UniformComponentPlugin::<PixelOutline>::default(),
    ));

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(
            RenderStartup,
            init_pixel_outline_layout.before(init_pixel_edges_pipeline),
        );
        render_app.add_systems(
            Render,
            prepare_pixel_outline_bind_groups.in_set(PrepareBindGroups),
        );
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
#[extract_component_sync_target((Self, PixelOutlineBindGroup))]
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

#[derive(Resource)]
pub(super) struct PixelOutlineBindGroupLayout(pub BindGroupLayoutDescriptor);

fn init_pixel_outline_layout(mut commands: Commands) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "pixel_outline_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::FRAGMENT,
            uniform_buffer::<PixelOutline>(true),
        ),
    );

    commands.insert_resource(PixelOutlineBindGroupLayout(bind_group_layout));
}

// Lives on the pixel camera's render-world entity.
// Holds the finished bind group and the offset into the uniform buffer.
#[derive(Component)]
pub struct PixelOutlineBindGroup {
    pub bind_group: BindGroup,
    pub uniform_index: u32,
}

fn prepare_pixel_outline_bind_groups(
    mut commands: Commands,
    pixel_outline_uniforms: Res<ComponentUniforms<PixelOutline>>,
    render_device: Res<RenderDevice>,
    pixel_outline_layout: Res<PixelOutlineBindGroupLayout>,
    pipeline_cache: Res<PipelineCache>,
    pixel_outline_cameras: Query<(Entity, &DynamicUniformIndex<PixelOutline>), With<PixelOutline>>,
) {
    let Some(pixel_outline_uniforms_binding) = pixel_outline_uniforms.uniforms().binding() else {
        return;
    };
    for (entity, uniform_index) in pixel_outline_cameras.iter() {
        let bind_group = render_device.create_bind_group(
            "pixel_outline_bind_group",
            &pipeline_cache.get_bind_group_layout(&pixel_outline_layout.0),
            &BindGroupEntries::single(pixel_outline_uniforms_binding.clone()),
        );
        commands.entity(entity).insert(PixelOutlineBindGroup {
            bind_group,
            uniform_index: uniform_index.index(),
        });
    }
}
