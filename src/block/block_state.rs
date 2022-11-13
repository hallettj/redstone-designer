use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bevy::prelude::*;
use minecraft_assets::{
    api::AssetPack,
    schemas::{
        blockstates::{multipart::StateValue, Variant},
        models::BlockFace,
        BlockStates,
    },
};

/// The current state of a specific block. Matches against block states defined in
/// minecraft/assets/minecraft/blockstates/ to determine which block model to render, and how to
/// render it.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct BlockState(HashMap<String, StateValue>);

impl BlockState {
    /// To calculate the initial state for a block type we load an arbitrary state from the list of
    /// block state variants for that type, and then set the values for each state property to
    /// predefined defaults. This works because block state files list every allowed state property
    /// for each variant.
    pub fn initial_state_for(asset_pack: AssetPack, block_type: &str) -> Result<Self> {
        let mut state = match get_block_states(asset_pack, block_type)? {
            BlockStates::Variants { variants } => {
                let state_string = variants.keys().cloned().next().ok_or(anyhow!(
                    "No block states found for block type, {}",
                    block_type
                ))?;
                Ok(Self::new(&state_string))
            }
            BlockStates::Multipart { cases: _ } => Err(anyhow!(
                "Initial block state for multipart blocks is not implemented yet. Block type: {}",
                block_type
            )),
        }?;
        for (prop, value) in state.0.iter_mut() {
            *value = default_state_value(prop)?;
        }
        Ok(state)
    }

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

    /// Set facing for a block that supports it. Fails silently otherwise.
    pub fn set_facing(&mut self, face: BlockFace) {
        let facing = match face {
            BlockFace::North => Some("north"),
            BlockFace::South => Some("south"),
            BlockFace::East => Some("east"),
            BlockFace::West => Some("west"),
            BlockFace::Up => Some("up"),
            BlockFace::Down => Some("down"),
        };
        if let Some(facing) = facing {
            self.update("facing".to_owned(), StateValue::String(facing.to_owned()));
        }
    }
}

fn default_state_value(prop: &str) -> Result<StateValue> {
    match prop {
        "delay" => Ok(StateValue::String("1".to_owned())),
        "facing" => Ok(StateValue::String("south".to_owned())),
        "locked" => Ok(StateValue::Bool(false)),
        "powered" => Ok(StateValue::Bool(false)),
        _ => Err(anyhow!("No default state value for property, {}", prop)),
    }
}

fn get_block_states(asset_pack: AssetPack, block_type: &str) -> Result<BlockStates> {
    let block_states = asset_pack
        .load_blockstates(block_type)
        .with_context(|| format!("no block states found for \"{}\"", block_type))?;
    Ok(block_states)
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use maplit::hashmap;
    use minecraft_assets::{
        api::AssetPack,
        schemas::{
            blockstates::{multipart::StateValue, ModelProperties, Variant},
            models::BlockFace,
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

    #[test]
    fn initial_state_for_repeater() -> Result<()> {
        let state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        let expected = BlockState::new("delay=1,facing=south,locked=false,powered=false");
        assert_eq!(state, expected, "initial state for repeater");
        Ok(())
    }

    #[test]
    fn sets_allowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        state.set_facing(BlockFace::West);
        assert_eq!(
            state.0.get("facing"),
            Some(&StateValue::String("west".to_owned())),
            "facing should be set to 'west'"
        );
        Ok(())
    }

    #[test]
    fn does_not_set_disallowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        let initial_face = state.0.get("facing").cloned();
        state.set_facing(BlockFace::Up);
        assert_eq!(
            state.0.get("facing").cloned(),
            initial_face,
            "facing hasn't changed"
        );
        Ok(())
    }

    #[test]
    fn does_note_set_disallowed_state_property() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        state.update("foo".to_owned(), StateValue::Bool(true));
        assert_eq!(state.0.get("foo"), None, "facing hasn't changed");
        Ok(())
    }

    fn test_asset_pack() -> AssetPack {
        AssetPack::at_path("assets/minecraft/")
    }

    fn get_blockstates(block_type: &str) -> Result<BlockStates> {
        let block_states = test_asset_pack()
            .load_blockstates(block_type)
            .with_context(|| format!("no block states found for \"{}\"", block_type))?;
        Ok(block_states)
    }
}
