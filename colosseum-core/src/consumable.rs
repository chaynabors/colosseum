// Copyright 2021 Chay Nabors.

#[path = "generated/consumable.rs"]
mod consumable;
pub use consumable::ConsumableIdentifier;
use serde::Deserialize;
use serde::Serialize;

use crate::effect::Effect;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Consumable {
    pub display_name: String,
    pub description: String,
    pub max_count: u32,
    pub effect: Effect,
}
