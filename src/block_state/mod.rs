mod states_by_block_type;

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bevy::prelude::*;
use minecraft_assets::schemas::{
    blockstates::{multipart::StateValue, Variant},
    models::BlockFace,
    BlockStates,
};

use states_by_block_type::state_values_for;

/// The current state of a specific block. Matches against block states defined in
/// minecraft/assets/minecraft/blockstates/ to determine which block model to render, and how to
/// render it.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct BlockState {
    pub block_type: String,
    values: HashMap<String, StateValue>,
}

impl BlockState {
    /// To calculate the initial state for a block type we load an arbitrary state from the list of
    /// block state variants for that type, and then set the values for each state property to
    /// predefined defaults. This works because block state files list every allowed state property
    /// for each variant.
    pub fn initial_state_for(block_type: &str) -> Self {
        let allowed_values = state_values_for(block_type);
        let mut values = HashMap::new();
        for (key, mut value) in allowed_values {
            values.insert(key.to_owned(), value.swap_remove(0).into());
        }
        Self {
            block_type: block_type.to_owned(),
            values,
        }
    }

    #[allow(dead_code)]
    pub fn new(block_type: &str, state_values: &str) -> Self {
        BlockState {
            block_type: block_type.to_owned(),
            values: Self::parse(state_values),
        }
    }

    #[allow(dead_code)]
    fn parse(input: &str) -> HashMap<String, StateValue> {
        input
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
            .collect()
    }

    pub fn active_variant(&self, block_states: BlockStates) -> Variant {
        let cases = block_states.into_multipart();
        let mut variants: Vec<_> = cases
            .into_iter()
            .filter_map(|case| {
                if case.applies(
                    self.values
                        .iter()
                        .map(|(state, value)| (state.as_str(), value)),
                ) {
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

    /// Sets a property value if that would result in a valid block state.
    pub fn update(&mut self, prop: &str, value: StateValue) -> Result<()> {
        let allowed_states = state_values_for(&self.block_type);
        let allowed_values = allowed_states.get(prop).ok_or(anyhow!(
            "no values for prop, {}, for block type, {}",
            prop,
            self.block_type
        ))?;
        if allowed_values
            .iter()
            .any(|allowed_value| *allowed_value == value)
        {
            self.values
                .entry(prop.to_owned())
                .and_modify(|v| *v = value);
            Ok(())
        } else {
            Err(anyhow!(
                "setting {}={:?} is not allowed for block type, {}",
                prop,
                value,
                self.block_type
            ))
        }
    }

    /// Set facing for a block that supports it.
    pub fn set_facing(&mut self, face: BlockFace) -> Result<()> {
        let facing = match face {
            BlockFace::North => "north",
            BlockFace::South => "south",
            BlockFace::East => "east",
            BlockFace::West => "west",
            BlockFace::Up => "up",
            BlockFace::Down => "down",
        };
        self.update("facing", StateValue::String(facing.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
        let state = BlockState::new("sandstone", "");
        assert_eq!(state.values, HashMap::default());
    }

    #[test]
    fn constructs_non_empty_state() {
        let actual = BlockState::new("repeater", "delay=2,facing=north,locked=false,powered=true");
        let expected = BlockState {
            block_type: "repeater".to_owned(),
            values: hashmap! {
                "delay".to_string() => StateValue::String("2".to_string()),
                "facing".to_string() => StateValue::String("north".to_string()),
                "locked".to_string() => StateValue::Bool(false),
                "powered".to_string() => StateValue::Bool(true),
            },
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn selects_the_correct_variant() -> Result<()> {
        let block_states = get_blockstates("repeater")?;
        let state = BlockState::new("repeater", "delay=2,facing=north,locked=false,powered=true");
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
        let state = BlockState::initial_state_for("repeater");
        let expected = BlockState::new(
            "repeater",
            "delay=1,facing=south,locked=false,powered=false",
        );
        assert_eq!(state, expected, "initial state for repeater");
        Ok(())
    }

    #[test]
    fn sets_allowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for("repeater");
        state.set_facing(BlockFace::West)?;
        assert_eq!(
            state.values.get("facing"),
            Some(&StateValue::String("west".to_owned())),
            "facing should be set to 'west'"
        );
        Ok(())
    }

    #[test]
    fn does_not_set_disallowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for("repeater");
        let initial_face = state.values.get("facing").cloned();
        state.set_facing(BlockFace::Up)?;
        assert_eq!(
            state.values.get("facing").cloned(),
            initial_face,
            "facing hasn't changed"
        );
        Ok(())
    }

    #[test]
    fn does_note_set_disallowed_state_property() -> Result<()> {
        let mut state = BlockState::initial_state_for("repeater");
        state.update("foo", StateValue::Bool(true))?;
        assert_eq!(state.values.get("foo"), None, "facing hasn't changed");
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
