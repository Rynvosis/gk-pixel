//! todo docs
use std::f32::consts::PI;

use bevy::{
    camera::ScalingMode, color::palettes::basic::SILVER, ecs::entity::EntityHashMap,
    input::common_conditions::input_just_pressed, prelude::*,
};
use gk_pixel::{PixelCamera, PixelCreases, PixelOutlines};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
        Update,
        (
            rotate,
            toggle_projection.run_if(input_just_pressed(KeyCode::KeyP)),
            toggle_creases.run_if(input_just_pressed(KeyCode::KeyC)),
            toggle_outlines.run_if(input_just_pressed(KeyCode::KeyO)),
        ),
    );
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 14.0;
const Z_EXTENT: f32 = 8.0;
const THICKNESS: f32 = 0.1;
/// World units visible vertically in orthographic mode
const ORTHO_VIEWPORT_HEIGHT: f32 = 10.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut hue = 0.0_f32;
    let mut next_material = || {
        hue = (hue + 222.5) % 360.0;
        materials.add(Color::oklch(0.75, 0.12, hue))
    };

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Tetrahedron::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cone::default()),
        meshes.add(ConicalFrustum::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
        meshes.add(Segment3d::default()),
        meshes.add(Polyline3d::new(vec![
            Vec3::new(-0.5, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
        ])),
    ];

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
        meshes.add(Extrusion::new(
            ConvexPolygon::new(vec![
                Vec2::new(0.0, 0.8),
                Vec2::new(-0.47, 0.25),
                Vec2::new(-0.47, -0.65),
                Vec2::new(0.47, -0.65),
                Vec2::new(0.47, 0.25),
            ])
            .unwrap(),
            1.0,
        )),
    ];

    let ring_extrusions = [
        meshes.add(Extrusion::new(Rectangle::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(
            Ring::new(Circle::new(1.0), Circle::new(0.5)),
            1.,
        )),
        meshes.add(Extrusion::new(Circle::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(
            {
                // This is an approximation; Ellipse does not implement Inset as concentric ellipses do not have parallel curves
                let outer = Ellipse::default();
                let mut inner = outer;
                inner.half_size -= Vec2::splat(THICKNESS);
                Ring::new(outer, inner)
            },
            1.,
        )),
        meshes.add(Extrusion::new(
            RegularPolygon::default().to_ring(THICKNESS),
            1.,
        )),
        meshes.add(Extrusion::new(Triangle2d::default().to_ring(THICKNESS), 1.)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(next_material()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Row::Front.z(),
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(next_material()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                Row::Middle.z(),
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_ring_extrusions = ring_extrusions.len();

    for (i, shape) in ring_extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(next_material()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_ring_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                Row::Rear.z(),
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Text::new(
            "Press 'P' to toggle orthographic/perspective camera\n\
            Press 'O' to toggle outlines\n\
            Press 'C' to toggle creases",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

fn toggle_projection(mut cameras: Query<&mut Projection, With<Camera3d>>) {
    for mut projection in &mut cameras {
        let is_perspective = matches!(*projection, Projection::Perspective(_));
        *projection = if is_perspective {
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: ORTHO_VIEWPORT_HEIGHT,
                },
                ..OrthographicProjection::default_3d()
            })
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        };
    }
}

fn toggle_outlines(
    mut commands: Commands,
    pixel_cameras: Query<(Entity, Option<&PixelOutlines>), With<PixelCamera>>,
    mut outlines_stash: Local<EntityHashMap<PixelOutlines>>,
) {
    for (entity, maybe_outlines) in &pixel_cameras {
        if let Some(outlines) = maybe_outlines {
            outlines_stash.insert(entity, *outlines);
            commands.entity(entity).remove::<PixelOutlines>();
        } else {
            commands
                .entity(entity)
                .insert(outlines_stash.remove(&entity).unwrap_or_default());
        }
    }
}

fn toggle_creases(
    mut commands: Commands,
    pixel_cameras: Query<(Entity, Option<&PixelCreases>), With<PixelCamera>>,
    mut creases_stash: Local<EntityHashMap<PixelCreases>>,
) {
    for (entity, maybe_creases) in &pixel_cameras {
        if let Some(creases) = maybe_creases {
            creases_stash.insert(entity, *creases);
            commands.entity(entity).remove::<PixelCreases>();
        } else {
            commands
                .entity(entity)
                .insert(creases_stash.remove(&entity).unwrap_or_default());
        }
    }
}

#[derive(Clone, Copy)]
enum Row {
    Front,
    Middle,
    Rear,
}

impl Row {
    fn z(self) -> f32 {
        match self {
            Row::Front => Z_EXTENT / 2.,
            Row::Middle => 0.,
            Row::Rear => -Z_EXTENT / 2.,
        }
    }
}
