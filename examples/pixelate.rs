#![expect(missing_docs, reason = "WIP")]

use bevy::prelude::*;
use gk_pixel::{PaletteSnap, PixelCamera, PixelPlugin};

mod common;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, common::plugin, PixelPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        PixelCamera { texels_tall: 120 },
        PaletteSnap::new([
            //resurrect-32 palette. Todo: add more palettes and option to change between in the demo
            Srgba::hex("ffffff").unwrap(),
            Srgba::hex("fb6b1d").unwrap(),
            Srgba::hex("e83b3b").unwrap(),
            Srgba::hex("831c5d").unwrap(),
            Srgba::hex("c32454").unwrap(),
            Srgba::hex("f04f78").unwrap(),
            Srgba::hex("f68181").unwrap(),
            Srgba::hex("fca790").unwrap(),
            Srgba::hex("e3c896").unwrap(),
            Srgba::hex("ab947a").unwrap(),
            Srgba::hex("966c6c").unwrap(),
            Srgba::hex("625565").unwrap(),
            Srgba::hex("3e3546").unwrap(),
            Srgba::hex("0b5e65").unwrap(),
            Srgba::hex("0b8a8f").unwrap(),
            Srgba::hex("1ebc73").unwrap(),
            Srgba::hex("91db69").unwrap(),
            Srgba::hex("fbff86").unwrap(),
            Srgba::hex("fbb954").unwrap(),
            Srgba::hex("cd683d").unwrap(),
            Srgba::hex("9e4539").unwrap(),
            Srgba::hex("7a3045").unwrap(),
            Srgba::hex("6b3e75").unwrap(),
            Srgba::hex("905ea9").unwrap(),
            Srgba::hex("a884f3").unwrap(),
            Srgba::hex("eaaded").unwrap(),
            Srgba::hex("8fd3ff").unwrap(),
            Srgba::hex("4d9be6").unwrap(),
            Srgba::hex("4d65b4").unwrap(),
            Srgba::hex("484a77").unwrap(),
            Srgba::hex("30e1b9").unwrap(),
            Srgba::hex("8ff8e2").unwrap(),
        ]),
    ));
}
