// Copyright 2021 Chay Nabors.

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

use colosseum::combat_event::CombatEvent;
use colosseum::combat_state::CombatState;
use colosseum::message::Message;
use colosseum::message::MessageType;
use colosseum::message::TakeTurn;
use gear::event::Event;
use log::info;

use super::action_state::ActionState;
use super::StateTransition;
use super::TurnState;
use crate::config::Config;

#[derive(Debug)]
pub struct WaitingState {
    pub config: Rc<Config>,
    pub shared_state: Rc<RefCell<CombatState>>,
}

impl WaitingState {
    pub fn new(config: Rc<Config>, shared_state: Rc<RefCell<CombatState>>) -> Self {
        Self { config, shared_state }
    }

    pub fn handle_event(&mut self, event: &Event) -> StateTransition<TurnState> {
        match event {
            Event::NetworkEvent(event) => match event {
                gear::event::NetworkEvent::Message(packet) => {
                    if packet.addr() != self.config.server_address {
                        return StateTransition::None;
                    }

                    let message = bincode::deserialize::<Message>(packet.payload()).unwrap();
                    info!("Received message from server: {}", message.type_);

                    match message.type_ {
                        MessageType::TakeTurn => {
                            let take_turn = TakeTurn::try_from(&message).unwrap();
                            return StateTransition::New(TurnState::ActionState(ActionState::from_waiting_state(
                                self,
                                take_turn.target,
                            )));
                        },
                        MessageType::CombatEvent => {
                            let event = CombatEvent::try_from(&message).unwrap();
                            self.shared_state.borrow_mut().process_event(&event);

                            info!("Combat event result:");

                            for party in &self.shared_state.borrow().parties {
                                for member in &party.members {
                                    info!("    {} hp: {}", member.name, member.hp);
                                }
                            }
                        },
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
