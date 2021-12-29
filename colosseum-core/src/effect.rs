// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::aspect::Aspect;
use crate::attribute::Attribute;
use crate::combatant::Combatant;
use crate::gender::Gender;
use crate::lifetime::Lifetime;
use crate::modifier::Modifier;

#[derive(Clone, Copy, Debug)]
pub enum EffectSource<'a> {
    None,
    Origin,
    Other(&'a Combatant),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SubEffect {
    Damage { aspect: Aspect, multiplier: f64 },
    DOT { aspect: Aspect, multiplier: f64, lifetime: Lifetime },
    Modifier { modifier: Modifier, attribute: Attribute },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TargetFlag {
    Any,
    Gender(Gender),
    Origin,
}

impl TargetFlag {
    pub fn satisfied(&self, target: &Combatant, source: EffectSource) -> bool {
        match *self {
            TargetFlag::Any => true,
            TargetFlag::Gender(gender) => target.gender == gender,
            TargetFlag::Origin => match source {
                EffectSource::Origin => true,
                _ => false,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TargetingScheme {
    All,
    MultiTarget(usize),
    SingleTarget,
}

impl Default for TargetingScheme {
    fn default() -> TargetingScheme {
        TargetingScheme::All
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Effect {
    pub sub_effects: Vec<SubEffect>,
    pub target_flags: Vec<Vec<TargetFlag>>,
    pub targeting_scheme: TargetingScheme,
}
