// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::lifetime::Lifetime;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ModifierExpression {
    Add(f64),
    Multiply(f64),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Modifier {
    pub expression: ModifierExpression,
    pub lifetime: Lifetime,
}
