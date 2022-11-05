use anyhow::{anyhow, Context};
use bevy::{prelude::*, render::view::RenderLayers};
use minecraft_assets::schemas::models::BlockFace;

use crate::block::BlockState;

/// Unit of distance. The model scale in use sets 1.0 unit of distance in the render space to be
/// one Minecraft "pixel". A Minecraft block is 16 pixels.
pub const PIXELS: f32 = 1.0;

/// Unit of distance. The model scale in use sets 1.0 unit of distance in the render space to be
/// one Minecraft "pixel". A Minecraft block is 16 pixels.
pub const BLOCKS: f32 = 16.0 * PIXELS;

/// List of all six block face directions: Down, Up, North, South, West, East
pub const BLOCK_FACES: [BlockFace; 6] = [
    BlockFace::Down,
    BlockFace::Up,
    BlockFace::North,
    BlockFace::South,
    BlockFace::West,
    BlockFace::East,
];

/// Block face enum members paired with unit vectors indicating which side of a block the face
/// appears on relative to the center of the block.
pub const BLOCK_FACE_DIRECTIONS: [(BlockFace, Vec3); 6] = [
    (BlockFace::Down, Vec3::NEG_Y),
    (BlockFace::Up, Vec3::Y),
    (BlockFace::North, Vec3::NEG_Z),
    (BlockFace::South, Vec3::Z),
    (BlockFace::West, Vec3::NEG_X),
    (BlockFace::East, Vec3::X),
];

/// Available block types paired with initial state.
pub const BLOCK_PALETTE: [(&'static str, &'static str); 3] = [
    ("iron_block", ""),
    (
        "repeater",
        "delay=1,facing=south,locked=false,powered=false",
    ),
    ("sandstone", ""),
];

pub fn block_from_palette(block_type: &str) -> (&'static str, BlockState) {
    let (block_type, initial_state) = BLOCK_PALETTE
        .iter()
        .find(|(bt, _)| bt == &block_type)
        .context(anyhow!("block not found in palette, {:?}", block_type))
        .unwrap();
    (block_type, BlockState::new(initial_state))
}

/// Blocks previews for the block picker are rendered to textures via a camera in this layer.
/// Cameras and other entities may be associated with one or more layers; a camera will only render
/// entities in a matching layer. Using UI_LAYER lets us render block previews that don't show up
/// in the game world because they are not seen by the main camera.
pub const BLOCK_PREVIEW_LAYER: RenderLayers = RenderLayers::layer(1);
