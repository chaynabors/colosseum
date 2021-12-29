// Copyright 2021 Chay Nabors.

use std::rc::Rc;

use gear::event::Event;
use gear::event::InputEvent;
use gear::Engine;
use log::info;

use crate::config::Config;
use crate::game_state::menu_state::connecting_state::ConnectingState;
use crate::game_state::menu_state::MenuSubState;
use crate::game_state::state_transition::StateTransition;

pub struct SplashState {
    pub config: Rc<Config>,
}

impl SplashState {
    pub fn new(config: Rc<Config>) -> Self {
        Self { config }
    }

    pub fn handle_event(&mut self, event: &Event, engine: &mut Engine) -> StateTransition<MenuSubState> {
        match event {
            Event::InputEvent(event) => match event {
                InputEvent::KeyboardEvent(_) => {
                    info!("Connecting to server");
                    StateTransition::New(MenuSubState::ConnectingState(ConnectingState::from_splash_state(&self, engine)))
                },
                _ => StateTransition::None,
            },
            _ => StateTransition::None,
        }
    }
}
