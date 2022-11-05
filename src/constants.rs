use bevy::render::view::RenderLayers;
use minecraft_assets::schemas::models::BlockFace;

use crate::block::block_state::{BlockState, Repeater};

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

pub const BLOCK_PALETTE: [(&'static str, BlockState); 3] = [
    ("iron_block", BlockState::Stateless),
    (
        "repeater",
        BlockState::Repeater(Repeater {
            facing: BlockFace::South,
            powered: true,
            delay: 1,
            locked: false,
        }),
    ),
    ("sandstone", BlockState::Stateless),
];

/// Blocks previews for the block picker are rendered to textures via a camera in this layer.
/// Cameras and other entities may be associated with one or more layers; a camera will only render
/// entities in a matching layer. Using UI_LAYER lets us render block previews that don't show up
/// in the game world because they are not seen by the main camera.
pub const BLOCK_PREVIEW_LAYER: RenderLayers = RenderLayers::layer(1);
