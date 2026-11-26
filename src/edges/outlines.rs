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

use crate::edges::{init_pixel_edges_pipeline, PixelEdgesThreshold};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<PixelOutlines>::default(),
        UniformComponentPlugin::<PixelOutlines>::default(),
    ));

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(
            RenderStartup,
            init_pixel_outlines_layout.before(init_pixel_edges_pipeline),
        );
        render_app.add_systems(
            Render,
            prepare_pixel_outlines_bind_groups.in_set(PrepareBindGroups),
        );
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[require(PixelEdgesThreshold)]
#[extract_app(RenderApp)]
#[extract_component_sync_target((Self, PixelOutlinesBindGroup))]
pub struct PixelOutlines {
    pub color: Vec4,
}

impl Default for PixelOutlines {
    fn default() -> Self {
        Self { color: Vec4::W }
    }
}

#[derive(Resource)]
pub(super) struct PixelOutlinesBindGroupLayout(pub BindGroupLayoutDescriptor);

fn init_pixel_outlines_layout(mut commands: Commands) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "pixel_outlines_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::FRAGMENT,
            uniform_buffer::<PixelOutlines>(true),
        ),
    );

    commands.insert_resource(PixelOutlinesBindGroupLayout(bind_group_layout));
}

// Lives on the pixel camera's render-world entity.
#[derive(Component)]
pub struct PixelOutlinesBindGroup {
    pub bind_group: BindGroup,
    pub uniform_index: u32,
}

fn prepare_pixel_outlines_bind_groups(
    mut commands: Commands,
    pixel_outlines_uniforms: Res<ComponentUniforms<PixelOutlines>>,
    render_device: Res<RenderDevice>,
    pixel_outlines_layout: Res<PixelOutlinesBindGroupLayout>,
    pipeline_cache: Res<PipelineCache>,
    pixel_outlines_cameras: Query<
        (Entity, &DynamicUniformIndex<PixelOutlines>),
        With<PixelOutlines>,
    >,
) {
    let Some(pixel_outlines_uniforms_binding) = pixel_outlines_uniforms.uniforms().binding() else {
        return;
    };
    for (entity, uniform_index) in pixel_outlines_cameras.iter() {
        let bind_group = render_device.create_bind_group(
            "pixel_outlines_bind_group",
            &pipeline_cache.get_bind_group_layout(&pixel_outlines_layout.0),
            &BindGroupEntries::single(pixel_outlines_uniforms_binding.clone()),
        );
        commands.entity(entity).insert(PixelOutlinesBindGroup {
            bind_group,
            uniform_index: uniform_index.index(),
        });
    }
}
