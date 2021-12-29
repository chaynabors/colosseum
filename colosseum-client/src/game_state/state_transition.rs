// Copyright 2021 Chay Nabors.

use crate::game_state::GameState;

pub enum StateTransition<T> {
    None,
    Old,
    New(T),
    Change(GameState),
}
