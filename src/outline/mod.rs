use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    core_pipeline::{
        fullscreen_material::fullscreen_material_system, tonemapping::tonemapping, Core3d,
        Core3dSystems, FullscreenShader,
    },
    material::descriptor::{
        BindGroupLayoutDescriptor, CachedRenderPipelineId, FragmentState, RenderPipelineDescriptor,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayoutEntries, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureSampleType, TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::{ViewDepthStencilTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        RenderApp, RenderStartup,
    },
};

use crate::{PaletteSnap, PIXEL_TEXTURE_FORMAT};

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "outline.wesl");

    app.add_plugins((
        ExtractComponentPlugin::<PixelOutline>::default(),
        UniformComponentPlugin::<PixelOutline>::default(),
    ));

    app.add_systems(PostUpdate, allow_depth_sampling);

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .add_systems(RenderStartup, init_pixel_outline_pipeline)
            .add_systems(
                Core3d,
                pixel_outline_pass
                    .in_set(Core3dSystems::PostProcess)
                    .after(tonemapping)
                    .after(fullscreen_material_system::<PaletteSnap>),
            );
    }
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct PixelOutline {
    pub color: Vec4,
    pub depth_threshold: f32,
}

#[derive(Resource)]
struct PixelOutlinePipeline {
    bind_group_layout: BindGroupLayoutDescriptor,
    pipeline_id: CachedRenderPipelineId,
    depth_sampler: Sampler,
}

fn allow_depth_sampling(
    mut cameras: Query<
        &mut Camera3d,
        (
            With<PixelOutline>,
            Or<(Added<PixelOutline>, Changed<Camera3d>)>,
        ),
    >,
) {
    for mut camera_3d in &mut cameras {
        let usages = TextureUsages::from(camera_3d.depth_texture_usages);
        if !usages.contains(TextureUsages::TEXTURE_BINDING) {
            camera_3d.depth_texture_usages = (usages | TextureUsages::TEXTURE_BINDING).into();
        }
    }
}

fn init_pixel_outline_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "pixel_outline_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<PixelOutline>(true),
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<ViewUniform>(true),
                sampler(SamplerBindingType::NonFiltering),
            ),
        ),
    );

    let depth_sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("pixel_outline_depth_sampler"),
        ..default()
    });

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("pixel_outline_pipeline".into()),
        layout: vec![bind_group_layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: load_embedded_asset!(asset_server.as_ref(), "outline.wesl"),
            targets: vec![Some(PIXEL_TEXTURE_FORMAT.into())],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(PixelOutlinePipeline {
        bind_group_layout,
        pipeline_id,
        depth_sampler,
    });
}

fn pixel_outline_pass(
    view: ViewQuery<(
        &ViewTarget,
        &DynamicUniformIndex<PixelOutline>,
        &ViewDepthStencilTexture,
        &ViewUniformOffset,
    )>,
    view_uniforms: Res<ViewUniforms>,
    pixel_outline_pipeline: Res<PixelOutlinePipeline>,
    pipeline_cache: Res<PipelineCache>,
    component_uniforms: Res<ComponentUniforms<PixelOutline>>,
    mut ctx: RenderContext,
) {
    let (view_target, uniform_index, depth_texture, view_uniform_offset) = view.into_inner();
    let Some(pipeline) = pipeline_cache.get_render_pipeline(pixel_outline_pipeline.pipeline_id)
    else {
        return;
    };
    let Some(pixel_outline_uniforms_binding) = component_uniforms.uniforms().binding() else {
        return;
    };
    let Some(view_uniforms_binding) = view_uniforms.uniforms.binding() else {
        return;
    };
    let depth_view = &depth_texture.attachment.texture.default_view;
    let post_process = view_target.post_process_write();
    let bind_group = ctx.render_device().create_bind_group(
        "pixel_outline_bind_group",
        &pipeline_cache.get_bind_group_layout(&pixel_outline_pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            pixel_outline_uniforms_binding,
            depth_view,
            view_uniforms_binding,
            // normal_view,
            &pixel_outline_pipeline.depth_sampler,
        )),
    );

    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("pixel_outline_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: default(),
            })],
            ..default()
        });

    pass.set_pipeline(pipeline);
    pass.set_bind_group(
        0,
        &bind_group,
        &[uniform_index.index(), view_uniform_offset.offset],
    );
    pass.draw(0..3, 0..1);
}
