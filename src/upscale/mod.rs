use bevy::{
    asset::embedded_asset,
    core_pipeline::{Core2d, Core2dSystems, FullscreenShader},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, FragmentState, PipelineCache, RenderPassDescriptor,
            RenderPipelineDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, ViewQuery},
        texture::GpuImage,
        view::ViewTarget,
        RenderApp, RenderStartup,
    },
};

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "upscale.wesl");
    app.add_plugins((
        ExtractComponentPlugin::<UpscaleSettings>::default(),
        ExtractComponentPlugin::<UpscaleSource>::default(),
        UniformComponentPlugin::<UpscaleSettings>::default(),
    ));

    app.sub_app_mut(RenderApp)
        .add_systems(RenderStartup, init_upscale_pipeline)
        .add_systems(Core2d, upscale_pass.in_set(Core2dSystems::PostProcess));
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub(crate) struct UpscaleSettings {
    pub(crate) pixels_per_texel: i32,
    pub(crate) origin: IVec2,
}

#[derive(Component, Default, Clone, ExtractComponent)]
#[extract_app(RenderApp)]
pub(crate) struct UpscaleSource(pub Handle<Image>);

#[derive(Resource)]
struct UpscalePipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedRenderPipelineId,
}
fn init_upscale_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "upscale_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<UpscaleSettings>(true),
            ),
        ),
    );

    let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("upscale_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: asset_server.load("embedded://gk_pixel/upscale/upscale.wesl"),
            targets: vec![Some(TextureFormat::Rgba8UnormSrgb.into())],
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(UpscalePipeline { layout, pipeline });
}

fn upscale_pass(
    view: ViewQuery<(
        &ViewTarget,
        &UpscaleSource,
        &DynamicUniformIndex<UpscaleSettings>,
    )>,
    upscale_pipeline: Res<UpscalePipeline>,
    pipeline_cache: Res<PipelineCache>,
    component_uniforms: Res<ComponentUniforms<UpscaleSettings>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut ctx: RenderContext,
) {
    let (view_target, upscale_source, uniform_index) = view.into_inner();
    let Some(pipeline) = pipeline_cache.get_render_pipeline(upscale_pipeline.pipeline) else {
        return;
    };
    let Some(uniforms_binding) = component_uniforms.uniforms().binding() else {
        return;
    };
    let Some(gpu_image) = gpu_images.get(&upscale_source.0) else {
        return;
    };
    let texture_view = &gpu_image.texture_view;

    let bind_group = ctx.render_device().create_bind_group(
        "upscale_bind_group",
        &pipeline_cache.get_bind_group_layout(&upscale_pipeline.layout),
        &BindGroupEntries::sequential((texture_view, uniforms_binding)),
    );

    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("upscale_pass"),
            color_attachments: &[Some(view_target.get_color_attachment())],
            ..default()
        });

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[uniform_index.index()]);
    pass.draw(0..3, 0..1);
}
