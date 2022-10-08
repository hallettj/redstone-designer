mod block;
mod camera;
mod redstone;

use bevy::{prelude::*, render::texture::ImageSettings};
use camera::CameraPlugin;
use redstone::RedstonePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.43, 0.69, 1.0))) // sky color
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(RedstonePlugin)
        .run();
}
