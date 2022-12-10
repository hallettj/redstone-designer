use crate::{
    block::{is_flat_surface, requires_flat_surface},
    block_state::BlockState,
    int_vec3::IntVec3,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorldState {
    bounds: (IntVec3, IntVec3),
    positions: Vec<WorldPosition>,
}

impl WorldState {
    pub fn new(bounds: (IntVec3, IntVec3)) -> Self {
        WorldState {
            bounds,
            positions: vec![],
        }
    }

    pub fn insert_block(
        &mut self,
        pos: IntVec3,
        state: BlockState,
    ) -> Result<(), InvalidPlacement> {
        self.assert_valid_placement(pos, &state)?;
        self.positions.push(WorldPosition { pos, state });
        Ok(())
    }

    fn assert_valid_placement(
        &self,
        pos: IntVec3,
        state: &BlockState,
    ) -> Result<(), InvalidPlacement> {
        if !self.is_in_bounds(pos) {
            return Err(InvalidPlacement::OutOfBounds);
        }
        if self.is_position_occupied(pos) {
            return Err(InvalidPlacement::PositionOccupied);
        }
        if requires_flat_surface(&state) && !self.is_flat_surface(pos + IntVec3::NEG_Y) {
            return Err(InvalidPlacement::NotAFlatSurface);
        }
        Ok(())
    }

    fn is_in_bounds(&self, pos: IntVec3) -> bool {
        let (low, high) = self.bounds;
        low.x <= pos.x
            && pos.x <= high.x
            && low.y <= pos.y
            && pos.y <= high.y
            && low.z <= pos.z
            && pos.z <= high.z
    }

    fn is_position_occupied(&self, pos: IntVec3) -> bool {
        self.positions.iter().any(|p| p.pos == pos)
    }

    fn is_flat_surface(&self, pos: IntVec3) -> bool {
        match self.positions.iter().find(|p| p.pos == pos) {
            Some(block) => is_flat_surface(&block.state),
            None => false,
        }
    }
}

pub enum InvalidPlacement {
    OutOfBounds,
    PositionOccupied,
    NotAFlatSurface,
}

#[derive(Clone, Debug, PartialEq)]
struct WorldPosition {
    pos: IntVec3,
    state: BlockState,
}
