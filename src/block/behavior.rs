use crate::block_state::BlockState;

pub fn requires_flat_surface(state: &BlockState) -> bool {
    match state.block_type.as_ref() {
        "redstone_torch" => true,
        "redstone_wire" => true,
        "repeater" => true,
        _ => false,
    }
}

/// Returns true if the given block type with the given block state provides a surface on top is
/// legal for placement of blocks such as redstone wire. For example a full block, or upside-down
/// stairs.
pub fn is_flat_surface(state: &BlockState) -> bool {
    match state.block_type.as_ref() {
        "iron_block" => true,
        "sandstone" => true,
        _ => false,
    }
}
