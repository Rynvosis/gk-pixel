use bevy::{
    core_pipeline::prepass::{NormalPrepass, ViewPrepassTextures},
    material::descriptor::BindGroupLayoutDescriptor,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            binding_types::{texture_2d, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayoutEntries, PipelineCache, ShaderStages,
            ShaderType, TextureSampleType,
        },
        renderer::RenderDevice,
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        Render, RenderApp, RenderStartup,
        RenderSystems::PrepareBindGroups,
    },
};

use crate::edges::{init_pixel_edges_pipeline, PixelEdgesThreshold};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<PixelCreases>::default(),
        UniformComponentPlugin::<PixelCreases>::default(),
    ));

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(
            RenderStartup,
            init_pixel_creases_layout.before(init_pixel_edges_pipeline),
        );
        render_app.add_systems(
            Render,
            prepare_pixel_creases_bind_groups.in_set(PrepareBindGroups),
        );
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[require(NormalPrepass, PixelEdgesThreshold)]
#[extract_app(RenderApp)]
#[extract_component_sync_target((Self, PixelCreasesBindGroup))]
pub struct PixelCreases {
    pub lighten: f32,
    pub crease_threshold: f32,
}

impl Default for PixelCreases {
    fn default() -> Self {
        Self {
            lighten: 0.15,
            crease_threshold: 0.5,
        }
    }
}

#[derive(Resource)]
pub(super) struct PixelCreasesBindGroupLayout(pub BindGroupLayoutDescriptor);

fn init_pixel_creases_layout(mut commands: Commands) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "pixel_creases_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                uniform_buffer::<PixelCreases>(true),
                texture_2d(TextureSampleType::Float { filterable: false }),
            ),
        ),
    );

    commands.insert_resource(PixelCreasesBindGroupLayout(bind_group_layout));
}

// Lives on the pixel camera's render-world entity.
#[derive(Component)]
pub struct PixelCreasesBindGroup {
    pub bind_group: BindGroup,
    pub uniform_index: u32,
}

fn prepare_pixel_creases_bind_groups(
    mut commands: Commands,
    pixel_creases_uniforms: Res<ComponentUniforms<PixelCreases>>,
    render_device: Res<RenderDevice>,
    pixel_creases_layout: Res<PixelCreasesBindGroupLayout>,
    pipeline_cache: Res<PipelineCache>,
    pixel_creases_cameras: Query<
        (
            Entity,
            &DynamicUniformIndex<PixelCreases>,
            &ViewPrepassTextures,
        ),
        With<PixelCreases>,
    >,
) {
    let Some(pixel_creases_uniforms_binding) = pixel_creases_uniforms.uniforms().binding() else {
        return;
    };
    for (entity, uniform_index, view_prepass) in pixel_creases_cameras.iter() {
        let Some(normal_view) = view_prepass.normal_view() else {
            continue;
        };
        let bind_group = render_device.create_bind_group(
            "pixel_creases_bind_group",
            &pipeline_cache.get_bind_group_layout(&pixel_creases_layout.0),
            &BindGroupEntries::sequential((pixel_creases_uniforms_binding.clone(), normal_view)),
        );
        commands.entity(entity).insert(PixelCreasesBindGroup {
            bind_group,
            uniform_index: uniform_index.index(),
        });
    }
}
