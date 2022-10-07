mod block;
mod redstone;

use bevy::{prelude::*, render::texture::ImageSettings};
use redstone::RedstonePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.43, 0.69, 1.0))) // sky color
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(RedstonePlugin)
        .run();
}
