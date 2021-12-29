// Copyright 2021 Chay Nabors.

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;

use colosseum::combat_event::CombatEvent;
use colosseum::combat_state::CombatState;
use colosseum::target::Target;
use gear::event::Event;
use gear::event::InputEvent;
use gear::KeyCode;
use gear::KeyState;
use gear::Socket;

use super::action_state::ActionState;
use super::skill_state::SkillState;
use super::targeting_state::TargetingState;
use super::waiting_state::WaitingState;
use super::StateTransition;
use super::TurnState;
use crate::config::Config;
use crate::socket::ClientSocket;

#[derive(Debug)]
pub struct ConfirmationState {
    config: Rc<Config>,
    shared_state: Rc<RefCell<CombatState>>,
    active: Target,
    event: CombatEvent,
}

impl ConfirmationState {
    pub fn from_action_state(action_state: &ActionState, event: CombatEvent) -> Self {
        Self {
            config: action_state.config.clone(),
            shared_state: action_state.shared_state.clone(),
            active: action_state.active,
            event,
        }
    }

    pub fn from_skill_state(skill_state: &SkillState, event: CombatEvent) -> Self {
        Self {
            config: skill_state.config.clone(),
            shared_state: skill_state.shared_state.clone(),
            active: skill_state.active,
            event,
        }
    }

    pub fn from_targeting_state(targeting_state: &TargetingState, event: CombatEvent) -> Self {
        Self {
            config: targeting_state.config.clone(),
            shared_state: targeting_state.shared_state.clone(),
            active: targeting_state.active,
            event,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        server_address: SocketAddr,
        socket: &Socket,
    ) -> StateTransition<TurnState> {
        match event {
            Event::InputEvent(event) => match event {
                InputEvent::KeyboardEvent(event) => {
                    if event.state == KeyState::Released || event.virtual_keycode.is_none() {
                        return StateTransition::None;
                    }

                    let keycode = event.virtual_keycode.unwrap();
                    match keycode {
                        KeyCode::Return => {
                            socket.send_message(server_address, &self.event);
                            return StateTransition::New(TurnState::WaitingState(WaitingState::new(
                                self.config.clone(),
                                self.shared_state.clone(),
                            )));
                        },
                        KeyCode::Escape => return StateTransition::Old,
                        _ => (),
                    }
                },
                _ => (),
            },
            _ => (),
        }

        StateTransition::None
    }
}
