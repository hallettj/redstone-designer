mod block;
mod camera;
mod constants;
mod cursor;
mod redstone;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use cursor::CursorPlugin;
use redstone::RedstonePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.43, 0.69, 1.0))) // sky color
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(CursorPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(RedstonePlugin)
        .run();
}
