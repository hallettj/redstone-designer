#![feature(option_result_contains)]

mod block;
mod block_picker;
mod camera;
mod constants;
mod cursor;
mod lines;
mod redstone;
mod user_input;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_rapier3d::prelude::*;
use block::BlockPlugin;
use block_picker::BlockPickerPlugin;
use camera::CameraPlugin;
use cursor::CursorPlugin;
use redstone::RedstonePlugin;
use user_input::UserInputPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.43, 0.69, 1.0))) // sky color
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(CursorPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(BlockPlugin)
        .add_plugin(BlockPickerPlugin)
        .add_plugin(RedstonePlugin)
        .add_plugin(UserInputPlugin)
        .run();
}
