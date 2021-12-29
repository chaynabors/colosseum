// Copyright 2021 Chay Nabors.

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use colosseum::combat_event::CombatEvent;
use colosseum::combat_state::CombatState;
use colosseum::effect::TargetingScheme;
use colosseum::skill::SkillIdentifier;
use colosseum::target::Target;
use gear::event::Event;
use gear::KeyCode;
use gear::KeyState;

use super::action_state::ActionIdentifier;
use super::action_state::ActionState;
use super::confirmation_state::ConfirmationState;
use super::skill_state::SkillState;
use super::StateTransition;
use super::TurnState;
use crate::config::Config;

#[derive(Debug)]
pub struct TargetingState {
    pub config: Rc<Config>,
    pub shared_state: Rc<RefCell<CombatState>>,
    pub active: Target,
    pub action_identifier: ActionIdentifier,
    pub targeting_scheme: TargetingScheme,
    pub viable_targets: Vec<Target>,
    pub targets: Vec<Target>,
    pub target_index: usize,
}

impl TargetingState {
    pub fn from_action_state(
        action_state: &ActionState,
        targeting_scheme: TargetingScheme,
        viable_targets: Vec<Target>,
    ) -> Self {
        Self {
            config: action_state.config.clone(),
            shared_state: action_state.shared_state.clone(),
            active: action_state.active,
            action_identifier: ActionIdentifier::Attack,
            targeting_scheme,
            viable_targets,
            targets: vec![],
            target_index: 0,
        }
    }

    pub fn from_skill_state(
        skill_state: &SkillState,
        skill_identifier: SkillIdentifier,
        targeting_scheme: TargetingScheme,
        viable_targets: Vec<Target>,
    ) -> Self {
        Self {
            config: skill_state.config.clone(),
            shared_state: skill_state.shared_state.clone(),
            active: skill_state.active,
            action_identifier: ActionIdentifier::Skill(skill_identifier),
            targeting_scheme,
            viable_targets,
            targets: vec![],
            target_index: 0,
        }
    }

    pub fn from_self(targeting_state: &Self, viable_targets: Vec<Target>, targets: Vec<Target>) -> Self {
        Self {
            config: targeting_state.config.clone(),
            shared_state: targeting_state.shared_state.clone(),
            active: targeting_state.active,
            action_identifier: targeting_state.action_identifier,
            targeting_scheme: targeting_state.targeting_scheme,
            viable_targets,
            targets,
            target_index: 0,
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

                    match keycode {
                        KeyCode::S => self.target_index = (self.target_index + 1) % self.viable_targets.len(),
                        KeyCode::W => {
                            if self.target_index == 0 {
                                self.target_index = self.viable_targets.len() - 1
                            } else {
                                self.target_index -= 1
                            }
                        },
                        KeyCode::Return => {
                            let mut viable_targets = self.viable_targets.clone();
                            let mut targets = self.targets.clone();
                            targets.push(viable_targets.remove(self.target_index));

                            match self.targeting_scheme {
                                TargetingScheme::All => {
                                    unimplemented!("Targeting scheme was 'all', but entered target selection state")
                                },
                                TargetingScheme::MultiTarget(target_count) => {
                                    if targets.len() == target_count || viable_targets.len() == 0 {
                                        let event = match self.action_identifier {
                                            ActionIdentifier::Attack => {
                                                CombatEvent::AttackEvent { source: self.active, targets }
                                            },
                                            ActionIdentifier::Skill(skill) => {
                                                CombatEvent::SkillEvent { source: self.active, targets, skill }
                                            },
                                        };

                                        return StateTransition::New(TurnState::ConfirmationState(
                                            ConfirmationState::from_targeting_state(self, event),
                                        ));
                                    }

                                    return StateTransition::New(TurnState::TargetingState(TargetingState::from_self(
                                        self,
                                        viable_targets,
                                        targets,
                                    )));
                                },
                                TargetingScheme::SingleTarget => {
                                    let event = match self.action_identifier {
                                        ActionIdentifier::Attack => {
                                            CombatEvent::AttackEvent { source: self.active, targets }
                                        },
                                        ActionIdentifier::Skill(skill) => {
                                            CombatEvent::SkillEvent { source: self.active, targets, skill }
                                        },
                                    };

                                    return StateTransition::New(TurnState::ConfirmationState(
                                        ConfirmationState::from_targeting_state(self, event),
                                    ));
                                },
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

impl Display for TargetingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Targeting: {}", self.target_index)
    }
}
