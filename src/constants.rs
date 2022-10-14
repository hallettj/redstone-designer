use minecraft_assets::schemas::models::BlockFace;

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
