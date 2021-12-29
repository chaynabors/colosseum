// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::consumable::ConsumableIdentifier;
use crate::skill::SkillIdentifier;
use crate::target::Target;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CombatEvent {
    AttackEvent { source: Target, targets: Vec<Target> },
    ConsumableEvent { source: Target, consumable: ConsumableIdentifier, targets: Vec<Target> },
    SkillEvent { source: Target, skill: SkillIdentifier, targets: Vec<Target> },
    SkipEvent,
}
