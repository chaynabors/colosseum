// Copyright 2021 Chay Nabors.

use crate::effect::Effect;

#[path = "generated/weapon.rs"]
mod weapon;
use serde::Deserialize;
use serde::Serialize;
pub use weapon::WeaponIdentifier;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Weapon {
    pub display_name: String,
    pub description: String,
    pub effect: Effect,
}
