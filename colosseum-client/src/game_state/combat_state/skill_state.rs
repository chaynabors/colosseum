// Copyright 2021 Chay Nabors.

use std::cell::RefCell;
use std::rc::Rc;

use colosseum::combat_event::CombatEvent;
use colosseum::combat_state::CombatState;
use colosseum::effect::TargetingScheme;
use colosseum::skill::Skill;
use colosseum::target::Target;
use gear::event::Event;
use gear::KeyCode;
use gear::KeyState;
use log::warn;

use super::action_state::ActionState;
use super::confirmation_state::ConfirmationState;
use super::targeting_state::TargetingState;
use super::StateTransition;
use super::TurnState;
use crate::config::Config;

#[derive(Debug)]
pub struct SkillState {
    pub config: Rc<Config>,
    pub shared_state: Rc<RefCell<CombatState>>,
    pub active: Target,
    pub skill_index: usize,
}

impl SkillState {
    pub fn from_action_state(action_state: &ActionState) -> Self {
        Self {
            config: action_state.config.clone(),
            shared_state: action_state.shared_state.clone(),
            active: action_state.active,
            skill_index: 0,
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> StateTransition<TurnState> {
        match event {
            Event::InputEvent(event) => match event {
                gear::event::InputEvent::KeyboardEvent(event) => {
                    if event.state == KeyState::Released || event.virtual_keycode.is_none() {
                        return StateTransition::None;
                    }

                    let keycode = event.virtual_keycode.unwrap();
                    let active = self.active;
                    let active = &self.shared_state.borrow().parties[active.party_index].members[active.member_index];

                    match keycode {
                        KeyCode::S => self.skill_index = (self.skill_index + 1) % active.skills.len(),
                        KeyCode::W => {
                            if self.skill_index == 0 {
                                self.skill_index = active.skills.len() - 1
                            } else {
                                self.skill_index -= 1
                            }
                        },
                        KeyCode::Return => {
                            let skill_identifier = active.skills[self.skill_index];
                            let skill = <&Skill>::from(skill_identifier);
                            let viable_targets = self.shared_state.borrow().get_target_list();

                            if viable_targets.len() == 0 {
                                warn!("No valid targets for skill: {}", skill_identifier);
                            } else {
                                let targeting_scheme = skill.effect.targeting_scheme;

                                match targeting_scheme {
                                    TargetingScheme::All => {
                                        let event = CombatEvent::SkillEvent {
                                            source: self.active,
                                            skill: skill_identifier,
                                            targets: viable_targets,
                                        };

                                        return StateTransition::New(TurnState::ConfirmationState(
                                            ConfirmationState::from_skill_state(self, event),
                                        ));
                                    },
                                    TargetingScheme::MultiTarget(_) | TargetingScheme::SingleTarget => {
                                        return StateTransition::New(TurnState::TargetingState(
                                            TargetingState::from_skill_state(
                                                self,
                                                skill_identifier,
                                                targeting_scheme,
                                                viable_targets,
                                            ),
                                        ))
                                    },
                                }
                            }
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
