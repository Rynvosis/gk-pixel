use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    core_pipeline::{
        fullscreen_material::fullscreen_material_system, tonemapping::tonemapping, Core3d,
        Core3dSystems, FullscreenShader,
    },
    material::descriptor::{BindGroupLayoutDescriptor, FragmentState, RenderPipelineDescriptor},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayoutEntries, Canonical, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, Specializer,
            SpecializerKey, TextureSampleType, TextureUsages, Variants,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::{ViewDepthStencilTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        RenderApp, RenderStartup,
    },
};

mod creases;
mod outlines;

pub use crate::edges::creases::PixelCreases;
pub use crate::edges::outlines::PixelOutlines;
use crate::{
    edges::{
        creases::{PixelCreasesBindGroup, PixelCreasesBindGroupLayout},
        outlines::{PixelOutlinesBindGroup, PixelOutlinesBindGroupLayout},
    },
    PaletteSnap, PIXEL_TEXTURE_FORMAT,
};

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "edges.wesl");

    app.add_plugins((
        ExtractComponentPlugin::<PixelEdgesThreshold>::default(),
        UniformComponentPlugin::<PixelEdgesThreshold>::default(),
        creases::plugin,
        outlines::plugin,
    ));

    app.add_systems(PostUpdate, let_pixel_edges_read_camera_depth);

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .add_systems(RenderStartup, init_pixel_edges_pipeline)
            .add_systems(
                Core3d,
                pixel_edges_pass
                    .in_set(Core3dSystems::PostProcess)
                    .after(tonemapping)
                    .after(fullscreen_material_system::<PaletteSnap>),
            );
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct PixelEdgesThreshold {
    pub threshold: f32,
}
impl Default for PixelEdgesThreshold {
    fn default() -> Self {
        Self { threshold: 2.0 }
    }
}

#[derive(Resource)]
struct PixelEdgesPipeline {
    bind_group_layout: BindGroupLayoutDescriptor,
    depth_sampler: Sampler,
    pipeline_variants: Variants<RenderPipeline, PixelEdgesPipelineSpecializer>,
}

struct PixelEdgesPipelineSpecializer {
    pixel_creases_layout: BindGroupLayoutDescriptor,
    pixel_outlines_layout: BindGroupLayoutDescriptor,
}
impl Specializer<RenderPipeline> for PixelEdgesPipelineSpecializer {
    type Key = PixelEdgesPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        descriptor: &mut RenderPipelineDescriptor,
    ) -> Result<Canonical<Self::Key>, BevyError> {
        if key.has_creases {
            descriptor
                .fragment_mut()?
                .shader_defs
                .push("CREASES".into());
            descriptor.set_layout(1, self.pixel_creases_layout.clone());
        }
        if key.has_outlines {
            descriptor
                .fragment_mut()?
                .shader_defs
                .push("OUTLINES".into());
            descriptor.set_layout(2, self.pixel_outlines_layout.clone());
        }
        Ok(key)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, SpecializerKey)]
struct PixelEdgesPipelineKey {
    has_creases: bool,
    has_outlines: bool,
}

fn init_pixel_edges_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    render_device: Res<RenderDevice>,
    pixel_creases_layout: Res<PixelCreasesBindGroupLayout>,
    pixel_outlines_layout: Res<PixelOutlinesBindGroupLayout>,
) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "pixel_edges_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<ViewUniform>(true),
                sampler(SamplerBindingType::NonFiltering),
                uniform_buffer::<PixelEdgesThreshold>(true),
            ),
        ),
    );

    let depth_sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("pixel_edges_depth_sampler"),
        ..default()
    });
    let pipeline_variants = Variants::new(
        PixelEdgesPipelineSpecializer {
            pixel_creases_layout: pixel_creases_layout.0.clone(),
            pixel_outlines_layout: pixel_outlines_layout.0.clone(),
        },
        RenderPipelineDescriptor {
            label: Some("pixel_edges_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            vertex: fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: load_embedded_asset!(asset_server.as_ref(), "edges.wesl"),
                targets: vec![Some(PIXEL_TEXTURE_FORMAT.into())],
                ..default()
            }),
            ..default()
        },
    );

    commands.insert_resource(PixelEdgesPipeline {
        bind_group_layout,
        depth_sampler,
        pipeline_variants,
    });
}

fn pixel_edges_pass(
    view: ViewQuery<(
        &ViewTarget,
        &DynamicUniformIndex<PixelEdgesThreshold>,
        AnyOf<(&PixelCreasesBindGroup, &PixelOutlinesBindGroup)>,
        &ViewDepthStencilTexture,
        &ViewUniformOffset,
    )>,
    view_uniforms: Res<ViewUniforms>,
    edges_threshold_uniforms: Res<ComponentUniforms<PixelEdgesThreshold>>,
    mut pixel_edges_pipeline: ResMut<PixelEdgesPipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    mut ctx: RenderContext,
) -> Result<(), BevyError> {
    let (
        view_target,
        edges_threshold_uniform_index,
        (maybe_pixel_creases_bind_group, maybe_pixel_outlines_bind_group),
        depth_texture,
        view_uniform_offset,
    ) = view.into_inner();
    let Some(view_uniforms_binding) = view_uniforms.uniforms.binding() else {
        return Ok(());
    };
    let Some(edges_threshold_uniforms_binding) = edges_threshold_uniforms.uniforms().binding()
    else {
        return Ok(());
    };

    let pipeline_id = pixel_edges_pipeline.pipeline_variants.specialize(
        &pipeline_cache,
        PixelEdgesPipelineKey {
            has_creases: maybe_pixel_creases_bind_group.is_some(),
            has_outlines: maybe_pixel_outlines_bind_group.is_some(),
        },
    )?;
    let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
        return Ok(());
    };
    let post_process = view_target.post_process_write();

    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("pixel_edges_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: default(),
            })],
            ..default()
        });

    pass.set_pipeline(pipeline);

    let depth_texture_view = &depth_texture.attachment.texture.default_view;

    // Basic bind group things used by outlines/creases and/or just passthrough
    let pixel_edges_bind_group = render_device.create_bind_group(
        "pixel_edges_bind_group",
        &pipeline_cache.get_bind_group_layout(&pixel_edges_pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            depth_texture_view,
            view_uniforms_binding,
            &pixel_edges_pipeline.depth_sampler,
            edges_threshold_uniforms_binding,
        )),
    );
    pass.set_bind_group(
        0,
        &pixel_edges_bind_group,
        &[
            view_uniform_offset.offset,
            edges_threshold_uniform_index.index(),
        ],
    );

    if let Some(pixel_creases_bind_group) = maybe_pixel_creases_bind_group {
        pass.set_bind_group(
            1,
            &pixel_creases_bind_group.bind_group,
            &[pixel_creases_bind_group.uniform_index],
        );
    }
    if let Some(pixel_outlines_bind_group) = maybe_pixel_outlines_bind_group {
        pass.set_bind_group(
            2,
            &pixel_outlines_bind_group.bind_group,
            &[pixel_outlines_bind_group.uniform_index],
        );
    }
    pass.draw(0..3, 0..1);
    Ok(())
}

fn let_pixel_edges_read_camera_depth(
    mut cameras: Query<
        &mut Camera3d,
        (
            Or<(With<PixelCreases>, With<PixelOutlines>)>,
            Or<(Added<PixelCreases>, Added<PixelOutlines>, Changed<Camera3d>)>,
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
