// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::aspect::Aspect;
use crate::lifetime::Lifetime;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct DOT {
    pub aspect: Aspect,
    pub damage_value: f64,
    pub lifetime: Lifetime,
}
