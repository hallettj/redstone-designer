use std::collections::HashMap;

use bevy::prelude::*;
use minecraft_assets::schemas::{
    blockstates::{multipart::StateValue, Variant},
    models::BlockFace,
    BlockStates,
};

/// The current state of a specific block. Matches against block states defined in
/// minecraft/assets/minecraft/blockstates/ to determine which block model to render, and how to
/// render it.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct BlockState(HashMap<String, StateValue>);

impl BlockState {
    pub fn new(state_values: &str) -> Self {
        BlockState(
            state_values
                .split(',')
                .filter_map(|state_value| {
                    if state_value == "" {
                        None
                    } else {
                        let split: Vec<&str> = state_value.split('=').collect();
                        Some((split[0], split[1]))
                    }
                })
                .map(|(state, value)| (String::from(state), StateValue::from(value)))
                .collect(),
        )
    }

    pub fn active_variant(&self, block_states: BlockStates) -> Variant {
        let cases = block_states.into_multipart();
        let mut variants: Vec<_> = cases
            .into_iter()
            .filter_map(|case| {
                if case.applies(self.0.iter().map(|(state, value)| (state.as_str(), value))) {
                    Some(case.apply)
                } else {
                    None
                }
            })
            .collect();
        if variants.len() > 1 {
            panic!(
                "only ony variant is supported at this time; block state: {:?}",
                self
            );
        } else if variants.is_empty() {
            panic!("no variant found for block state: {:?}", self);
        } else {
            variants.remove(0)
        }
    }

    /// Set a state property if the named property is already present. (Each block type has a fixed
    /// set of allowed state properties that are included with its initial state.)
    pub fn update(&mut self, prop: String, value: StateValue) {
        self.0.entry(prop).and_modify(|v| *v = value);
    }

    /// Set facing for a block if it already has a facing state.
    pub fn set_facing(&mut self, face: BlockFace) {
        let facing = match face {
            BlockFace::North => Some("north"),
            BlockFace::South => Some("south"),
            BlockFace::East => Some("east"),
            BlockFace::West => Some("west"),
            BlockFace::Up => None, // repeaters only accept horizontal facing values
            BlockFace::Down => None,
        };
        if let Some(facing) = facing {
            self.update("facing".to_owned(), StateValue::String(facing.to_owned()));
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use maplit::hashmap;
    use minecraft_assets::{
        api::AssetPack,
        schemas::{
            blockstates::{multipart::StateValue, ModelProperties, Variant},
            BlockStates,
        },
    };

    use super::BlockState;

    #[test]
    fn constructs_empty_state() {
        let state = BlockState::new("");
        assert_eq!(state, BlockState::default());
    }

    #[test]
    fn constructs_non_empty_state() {
        let actual = BlockState::new("delay=2,facing=north,locked=false,powered=true");
        let expected = BlockState(hashmap! {
            "delay".to_string() => StateValue::String("2".to_string()),
            "facing".to_string() => StateValue::String("north".to_string()),
            "locked".to_string() => StateValue::Bool(false),
            "powered".to_string() => StateValue::Bool(true),
        });
        assert_eq!(actual, expected)
    }

    #[test]
    fn selects_the_correct_variant() -> Result<()> {
        let block_states = get_blockstates("repeater")?;
        let state = BlockState::new("delay=2,facing=north,locked=false,powered=true");
        let variant = state.active_variant(block_states);
        assert_eq!(
            variant,
            Variant::Single(ModelProperties {
                model: "minecraft:block/repeater_2tick_on".to_string(),
                x: 0,
                y: 180,
                uv_lock: false,
                weight: 1,
            })
        );
        Ok(())
    }

    fn get_blockstates(block_type: &str) -> Result<BlockStates> {
        let assets = AssetPack::at_path("assets/minecraft/");
        let block_states = assets
            .load_blockstates(block_type)
            .with_context(|| format!("no block states found for \"{}\"", block_type))?;
        Ok(block_states)
    }
}
