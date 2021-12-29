// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Target {
    pub party_index: usize,
    pub member_index: usize,
}
