// Copyright 2021 Chay Nabors.

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;

use colosseum_core::combat_event::CombatEvent;
use colosseum_core::combat_state::CombatState;
use colosseum_core::effect::TargetingScheme;
use colosseum_core::skill::SkillIdentifier;
use colosseum_core::target::Target;
use colosseum_core::weapon::Weapon;
use log::warn;

use super::confirmation_state::ConfirmationState;
use super::skill_state::SkillState;
use super::targeting_state::TargetingState;
use super::waiting_state::WaitingState;
use super::StateTransition;
use super::TurnState;
use crate::config::Config;
use crate::socket::ClientSocket;

#[derive(Debug)]
pub enum Action {
    Attack,
    Skill,
    Skip,
}

impl Action {
    fn first() -> Self {
        Action::Attack
    }

    fn next(&self) -> Self {
        match self {
            Action::Attack => Action::Skill,
            Action::Skill => Action::Skip,
            Action::Skip => Action::Attack,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Action::Attack => Action::Skip,
            Action::Skill => Action::Attack,
            Action::Skip => Action::Skill,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ActionIdentifier {
    Attack,
    Skill(SkillIdentifier),
}

#[derive(Debug)]
pub struct ActionState {
    pub config: Rc<Config>,
    pub shared_state: Rc<RefCell<CombatState>>,
    pub active: Target,
    pub action: Action,
}

impl ActionState {
    pub fn from_waiting_state(waiting_state: &WaitingState, active: Target) -> Self {
        Self {
            config: waiting_state.config.clone(),
            shared_state: waiting_state.shared_state.clone(),
            active,
            action: Action::first(),
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
                gear::event::InputEvent::KeyboardEvent(event) => {
                    if event.state == KeyState::Released {
                        return StateTransition::None;
                    }

                    if let Some(keycode) = event.virtual_keycode {
                        match keycode {
                            KeyCode::S => self.action = self.action.next(),
                            KeyCode::W => self.action = self.action.prev(),
                            KeyCode::Return => match self.action {
                                Action::Attack => {
                                    let active = self.active;
                                    let active =
                                        &self.shared_state.borrow().parties[active.party_index].members[active.member_index];
                                    let targeting_scheme = match active.weapon {
                                        Some(weapon) => <&Weapon>::from(weapon).effect.targeting_scheme,
                                        None => TargetingScheme::SingleTarget,
                                    };

                                    let viable_targets = self.shared_state.borrow().get_target_list();

                                    if viable_targets.len() == 0 {
                                        match active.weapon {
                                            Some(weapon) => warn!("No valid targets for weapon: {}", weapon),
                                            None => warn!("No valid targets for an unarmed attack"),
                                        }
                                    } else {
                                        match targeting_scheme {
                                            TargetingScheme::All => {
                                                return StateTransition::New(TurnState::ConfirmationState(
                                                    ConfirmationState::from_action_state(
                                                        self,
                                                        CombatEvent::AttackEvent {
                                                            source: self.active,
                                                            targets: viable_targets,
                                                        },
                                                    ),
                                                ))
                                            },
                                            TargetingScheme::MultiTarget(_) | TargetingScheme::SingleTarget => {
                                                return StateTransition::New(TurnState::TargetingState(
                                                    TargetingState::from_action_state(
                                                        self,
                                                        targeting_scheme,
                                                        viable_targets,
                                                    ),
                                                ))
                                            },
                                        }
                                    }
                                },
                                Action::Skill => {
                                    return StateTransition::New(TurnState::SkillState(SkillState::from_action_state(self)))
                                },
                                Action::Skip => {
                                    socket.send_message(server_address, &CombatEvent::SkipEvent);
                                    return StateTransition::New(TurnState::WaitingState(WaitingState::new(
                                        self.config.clone(),
                                        self.shared_state.clone(),
                                    )));
                                },
                            },
                            KeyCode::Escape => self.action = Action::first(),
                            _ => (),
                        }
                    }
                },
                _ => (),
            },
            _ => (),
        }

        StateTransition::None
    }
}
