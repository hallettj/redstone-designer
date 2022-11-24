use std::collections::HashMap;

use maplit::hashmap;
use minecraft_assets::schemas::blockstates::multipart::StateValue;

use StaticStateValue::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticStateValue {
    B(bool),
    S(&'static str),
}

impl From<StaticStateValue> for StateValue {
    fn from(value: StaticStateValue) -> Self {
        match value {
            B(b) => StateValue::Bool(b),
            S(s) => StateValue::String(s.to_owned()),
        }
    }
}

impl PartialEq<StateValue> for StaticStateValue {
    fn eq(&self, rhs: &StateValue) -> bool {
        match (self, rhs) {
            (B(a), StateValue::Bool(b)) => a == b,
            (S(a), StateValue::String(b)) => a == b,
            _ => false,
        }
    }
}

/// Return valid block state keys and values for a given block type. The first value in each list
/// will be used for the default state.
pub fn state_values_for(block_type: &str) -> HashMap<&'static str, Vec<StaticStateValue>> {
    match block_type {
        "redstone_wire" => hashmap! {
            "east"  => vec![S("none"), S("side|up")],
            "north" => vec![S("none"), S("side|up")],
            "south" => vec![S("none"), S("side|up")],
            "west"  => vec![S("none"), S("side|up")],
        },
        "repeater" => hashmap! {
            "delay" => vec![S("1"), S("2"), S("3"), S("4")],
            "facing" => vec![S("south"), S("north"), S("east"), S("west")],
            "locked" => vec![B(false), B(true)],
            "powered" => vec![B(false), B(true)],
        },
        "redstone_torch" => hashmap! {
            "lit" => vec![B(false), B(true)],
        },
        _ => HashMap::new(),
    }
}
