mod block_state;
mod bounding_box;
mod placing_block;
mod spawn_block;

use bevy::prelude::*;

use crate::{cursor::Cursor, lines::LineMaterial};

pub use self::{
    block_state::BlockState,
    spawn_block::{spawn_block, spawn_block_preview_for_block_picker},
};
use self::{
    placing_block::{destroy_block, place_block},
    spawn_block::spawn_test_block,
};

#[derive(Component, Clone, Default)]
pub struct BlockOutline;

#[derive(Bundle, Clone, Default)]
pub struct BlockBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default())
            .add_startup_system(spawn_test_block)
            .add_system(highlight_block_on_hover)
            .add_system(place_block)
            .add_system(destroy_block);
    }
}

fn highlight_block_on_hover(
    cursor: Res<Cursor>,
    mut query_block_outlines: Query<(&mut Visibility, &Parent), With<BlockOutline>>,
) {
    if !cursor.is_changed() {
        return;
    }
    for (mut visibility, parent) in query_block_outlines.iter_mut() {
        let is_visible = cursor.current_block.contains(&parent.get());
        if visibility.is_visible != is_visible {
            visibility.is_visible = is_visible;
        }
    }
}
