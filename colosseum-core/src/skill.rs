// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::effect::Effect;

#[path = "generated/skill.rs"]
mod skill;
pub use skill::SkillIdentifier;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Skill {
    pub display_name: String,
    pub description: String,
    pub effect: Effect,
}
