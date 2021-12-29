// Copyright 2021 Chay Nabors.

mod action_state;
mod confirmation_state;
mod skill_state;
mod targeting_state;
mod waiting_state;

use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

use action_state::ActionState;
use colosseum::combat_state::CombatState as InternalCombatState;
use confirmation_state::ConfirmationState;
use gear::event::Event;
use gear::event::InputEvent;
use gear::Engine;
use gear::KeyCode;
use gear::KeyState;
use gear::Socket;
use log::info;
use skill_state::SkillState;
use targeting_state::TargetingState;
use waiting_state::WaitingState;

use super::state_transition::StateTransition;
use super::GameState;
use crate::config::Config;

#[derive(Debug)]
pub enum TurnState {
    WaitingState(WaitingState),
    ActionState(ActionState),
    SkillState(SkillState),
    TargetingState(TargetingState),
    ConfirmationState(ConfirmationState),
}

impl TurnState {
    pub fn new(config: Rc<Config>, shared_state: Rc<RefCell<InternalCombatState>>) -> Self {
        Self::WaitingState(WaitingState::new(config, shared_state))
    }
}

impl Display for TurnState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TurnState::WaitingState(_) => "Waiting State",
                TurnState::ActionState(_) => "Action State",
                TurnState::SkillState(_) => "Skill State",
                TurnState::TargetingState(_) => "Targeting State",
                TurnState::ConfirmationState(_) => "Confirmation State",
            }
        )
    }
}

#[derive(Debug)]
pub struct CombatState {
    pub config: Rc<Config>,
    pub socket: Rc<Socket>,
    pub shared_state: Rc<RefCell<InternalCombatState>>,
    pub turn_states: Vec<TurnState>,
}

impl CombatState {
    pub fn new(config: Rc<Config>, socket: Rc<Socket>, shared_state: InternalCombatState) -> Self {
        let shared_state = Rc::new(RefCell::new(shared_state));

        Self {
            config: config.clone(),
            socket,
            shared_state: shared_state.clone(),
            turn_states: vec![TurnState::new(config, shared_state)],
        }
    }

    pub fn handle_event(&mut self, event: &Event, engine: &mut Engine) -> Option<GameState> {
        let server_address = self.config.server_address;
        let socket = self.socket.as_ref();

        if let Event::UpdateEvent { delta_time: _ } = event {
            engine.renderer.set_clear_color([0.1, 0.1, 0.1, 1.0]).submit();

            return None;
        }

        let mut current_state = self.turn_states.pop().expect("Combat state has no turn states to mutate");

        let state_transition = match &mut current_state {
            TurnState::WaitingState(state) => state.handle_event(event),
            TurnState::ActionState(state) => state.handle_event(event, server_address, socket),
            TurnState::SkillState(state) => state.handle_event(event),
            TurnState::TargetingState(state) => state.handle_event(event),
            TurnState::ConfirmationState(state) => state.handle_event(event, server_address, socket),
        };

        match state_transition {
            StateTransition::None => self.turn_states.push(current_state),
            StateTransition::Old => {
                let state = self.turn_states.last().expect("Attempted to return to a nonexistant turn state");
                info!("Returning to state: {}", state);
            },
            StateTransition::New(new_state) => {
                info!("Transitioning turn state: {} -> {}", current_state, new_state);
                self.turn_states.push(current_state);
                self.turn_states.push(new_state);
            },
            StateTransition::Change(state) => return Some(state),
        }

        if let Event::InputEvent(event) = event {
            if let InputEvent::KeyboardEvent(event) = event {
                if let Some(keycode) = event.virtual_keycode {
                    if keycode == KeyCode::Space && event.state == KeyState::Pressed {
                        info!("{}", serde_json::to_string_pretty(self.shared_state.as_ref()).unwrap());
                    }
                }
            }
        }

        None
    }
}
