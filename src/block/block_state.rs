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
#[derive(Component, Clone, Debug, PartialEq)]
pub struct BlockState {
    block_type: String,
    values: HashMap<String, StateValue>,
}

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
                Ok(Self::new(block_type, &state_string))
            }
            BlockStates::Multipart { cases: _ } => Err(anyhow!(
                "Initial block state for multipart blocks is not implemented yet. Block type: {}",
                block_type
            )),
        }?;
        for (prop, value) in state.values.iter_mut() {
            *value = default_state_value(prop)?;
        }
        Ok(state)
    }

    pub fn new(block_type: &str, state_values: &str) -> Self {
        BlockState {
            block_type: block_type.to_owned(),
            values: Self::parse(state_values),
        }
    }

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

    /// Sets a property value if that would result in a valid block state. We check validity by
    /// checking against all block state variants for the given block type. Fails silently if the
    /// update is not valid.
    /// TODO: store reference to asset pack in struct instead of requiring argument
    pub fn update(&mut self, asset_pack: AssetPack, prop: String, value: StateValue) {
        let mut updated = self.values.clone();
        updated.entry(prop).and_modify(|v| *v = value);
        let matches_valid_state = match get_block_states(asset_pack, &self.block_type).unwrap() {
            BlockStates::Variants { variants } => variants
                .keys()
                .any(|state_string| Self::parse(state_string) == updated),
            BlockStates::Multipart { cases: _ } => Err(anyhow!(
                "Initial block state for multipart blocks is not implemented yet. Block type: {}",
                self.block_type
            ))
            .unwrap(),
        };
        if matches_valid_state {
            self.values = updated;
        }
    }

    /// Set facing for a block that supports it. Fails silently otherwise.
    pub fn set_facing(&mut self, asset_pack: AssetPack, face: BlockFace) {
        let facing = match face {
            BlockFace::North => Some("north"),
            BlockFace::South => Some("south"),
            BlockFace::East => Some("east"),
            BlockFace::West => Some("west"),
            BlockFace::Up => Some("up"),
            BlockFace::Down => Some("down"),
        };
        if let Some(facing) = facing {
            self.update(
                asset_pack,
                "facing".to_owned(),
                StateValue::String(facing.to_owned()),
            );
        }
    }
}

fn default_state_value(prop: &str) -> Result<StateValue> {
    match prop {
        "delay" => Ok(StateValue::String("1".to_owned())),
        "facing" => Ok(StateValue::String("south".to_owned())),
        "lit" => Ok(StateValue::Bool(false)),
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
        let state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        let expected = BlockState::new(
            "repeater",
            "delay=1,facing=south,locked=false,powered=false",
        );
        assert_eq!(state, expected, "initial state for repeater");
        Ok(())
    }

    #[test]
    fn sets_allowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        state.set_facing(test_asset_pack(), BlockFace::West);
        assert_eq!(
            state.values.get("facing"),
            Some(&StateValue::String("west".to_owned())),
            "facing should be set to 'west'"
        );
        Ok(())
    }

    #[test]
    fn does_not_set_disallowed_facing_value() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        let initial_face = state.values.get("facing").cloned();
        state.set_facing(test_asset_pack(), BlockFace::Up);
        assert_eq!(
            state.values.get("facing").cloned(),
            initial_face,
            "facing hasn't changed"
        );
        Ok(())
    }

    #[test]
    fn does_note_set_disallowed_state_property() -> Result<()> {
        let mut state = BlockState::initial_state_for(test_asset_pack(), "repeater")?;
        state.update(test_asset_pack(), "foo".to_owned(), StateValue::Bool(true));
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
