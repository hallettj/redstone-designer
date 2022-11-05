use crate::block_picker::SelectedBlockType;
use crate::cursor::Cursor;
use crate::lines::LineMaterial;
use crate::user_input::{sent_command, UiCommand};
use bevy::prelude::*;

use super::spawn_block::spawn_block;

pub fn place_block(
    selected: Res<SelectedBlockType>,
    user_input: EventReader<UiCommand>,
    cursor: Res<Cursor>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    if sent_command(user_input, UiCommand::PlaceBlock) {
        if let Some(transform) = cursor.place_block_transform {
            spawn_block(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut line_materials,
                selected.block.clone(),
                transform,
            )
        }
    }
}

pub fn destroy_block(
    mut commands: Commands,
    user_input: EventReader<UiCommand>,
    cursor: Res<Cursor>,
) {
    if sent_command(user_input, UiCommand::DestroyBlock) {
        if let Some(block) = cursor.current_block {
            commands.entity(block).despawn_recursive();
        }
    }
}
