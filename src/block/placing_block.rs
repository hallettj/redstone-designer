use crate::block_picker::SelectedBlockType;
use crate::camera::MainCamera;
use crate::cursor::Cursor;
use crate::lines::LineMaterial;
use crate::user_input::{sent_command, UiCommand};
use crate::util::{vec_to_block_face, HasRelativeDirection};
use bevy::prelude::*;
use minecraft_assets::schemas::models::BlockFace;

use super::spawn_block::spawn_block;

pub struct PlacingBlockPlugin;

impl Plugin for PlacingBlockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockRotation::default())
            .add_system(place_block)
            .add_system(rotate_block)
            .add_system(destroy_block);
    }
}

#[derive(Debug, Default, Resource)]
struct BlockRotation {
    direction: Option<BlockFace>,
}

fn place_block(
    selected: Res<SelectedBlockType>,
    block_rotation: Res<BlockRotation>,
    user_input: EventReader<UiCommand>,
    cursor: Res<Cursor>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    if sent_command(user_input, UiCommand::PlaceBlock) {
        let (block_type, mut block_state) = selected.block.clone();
        if let Some(direction) = block_rotation.direction {
            let _ = block_state.set_facing(direction);
        }
        if let Some(transform) = cursor.place_block_transform {
            spawn_block(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut line_materials,
                (block_type, block_state),
                transform,
            )
        }
    }
}

fn rotate_block(
    mut user_input: EventReader<UiCommand>,
    mut block_rotation: ResMut<BlockRotation>,
    query_camera: Query<&Transform, With<MainCamera>>,
) {
    for command in user_input.iter() {
        match command {
            UiCommand::RotateBlock(Some(dir)) => {
                let camera_transform = query_camera.get_single().unwrap();
                let face = vec_to_block_face(camera_transform.to_my(*dir));
                block_rotation.direction = Some(face);
            }
            UiCommand::RotateBlock(None) => {
                block_rotation.direction = None;
            }
            _ => {}
        }
    }
}

fn destroy_block(mut commands: Commands, user_input: EventReader<UiCommand>, cursor: Res<Cursor>) {
    if sent_command(user_input, UiCommand::DestroyBlock) {
        if let Some(block) = cursor.current_block {
            commands.entity(block).despawn_recursive();
        }
    }
}
