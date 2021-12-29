// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::combatant::Combatant;
use crate::item::Item;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Party {
    pub members: Vec<Combatant>,
    pub inventory: Vec<Item>,
}
