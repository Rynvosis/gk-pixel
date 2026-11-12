//! todo docs

use bevy::app::{App, Startup};
use bevy::camera::Camera3d;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Transform};

mod common;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, common::plugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}
