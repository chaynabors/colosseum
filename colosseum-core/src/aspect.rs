// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Aspect {
    Fire,
    Frost,
    Lightning,
    Physical,
}
