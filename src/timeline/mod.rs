mod world_state;

use bevy::prelude::*;

use crate::{block_state::BlockState, int_vec3::IntVec3, constants::WORLD_SIZE};

use self::world_state::{InvalidPlacement, WorldState};

type TimeIndex = i32;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct Timeline {
    bounds: (IntVec3, IntVec3),
    world_states: Vec<(TimeIndex, WorldState)>,
    random_seed: i32,
}

impl Timeline {
    /// Insert a block into the world at the start of the timeline if it is legal to do so. Will
    /// fail if there is already a block at the given position, or if the given block type is not
    /// allowed at the given position. (For example, placing redstone dust on top of a torch.)
    pub fn insert_block(
        &mut self,
        pos: IntVec3,
        state: BlockState,
    ) -> Result<(), InvalidPlacement> {
        let (_, world_state) = self
            .world_states
            .iter_mut()
            .find(|(time_index, _)| *time_index == 0)
            .unwrap();
        world_state.insert_block(pos, state)?;
        Ok(())
    }

    
}

impl Default for Timeline {
    fn default() -> Self {
        let bounds = (IntVec3::ZERO, IntVec3::ONE * WORLD_SIZE);
        Timeline {
            bounds,
            world_states: vec![(0, WorldState::new(bounds))],
            random_seed: 0,
        }
    }
}
