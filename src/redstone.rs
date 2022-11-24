use crate::{
    block::{spawn_block, BlockState},
    constants::BLOCKS,
    lines::LineMaterial,
};
use bevy::prelude::*;

pub struct RedstonePlugin;

impl Plugin for RedstonePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_floor)
            .add_startup_system(setup_lights);
    }
}

fn setup_floor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    let block_type = "sandstone";
    let block = (block_type, BlockState::initial_state_for(block_type));
    for x in 0..16 {
        for z in 0..16 {
            let transform =
                Transform::from_xyz(x as f32 * BLOCKS, -1.0 * BLOCKS, z as f32 * BLOCKS);
            spawn_block(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut line_materials,
                block.clone(),
                transform,
            );
        }
    }
}

fn setup_lights(mut commands: Commands) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}
