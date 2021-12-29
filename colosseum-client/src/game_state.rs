// Copyright 2021 Chay Nabors.

mod combat_state;
mod menu_state;
mod state_transition;

use std::rc::Rc;

use combat_state::CombatState;
use gear::Engine;
use menu_state::MenuState;

use crate::config::Config;

pub enum GameState {
    MenuState(MenuState),
    CombatState(CombatState),
}

impl GameState {
    pub fn new(config: Rc<Config>, engine: &Engine) -> Self {
        Self::MenuState(MenuState::new(config, engine))
    }
}
