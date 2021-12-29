// Copyright 2021 Chay Nabors.

use std::convert::TryFrom;
use std::rc::Rc;

use colosseum::combat_state::CombatState as InternalCombatState;
use colosseum::message::Message;
use gear::event::Event;
use gear::event::InputEvent;
use gear::event::NetworkEvent;
use gear::Engine;
use gear::KeyCode;
use gear::KeyState;
use gear::Socket;
use log::info;

use super::connecting_state::ConnectingState;
use super::MenuSubState;
use crate::config::Config;
use crate::game_state::combat_state::CombatState;
use crate::game_state::state_transition::StateTransition;
use crate::game_state::GameState;
use crate::socket::ClientSocket;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuOption {
    Play = 0,
    Quit,
}

impl MenuOption {
    fn next(self) -> MenuOption {
        match self {
            MenuOption::Play => MenuOption::Quit,
            MenuOption::Quit => MenuOption::Play,
        }
    }

    fn prev(self) -> MenuOption {
        match self {
            MenuOption::Play => MenuOption::Quit,
            MenuOption::Quit => MenuOption::Play,
        }
    }
}

pub struct NavigationState {
    pub config: Rc<Config>,
    pub socket: Rc<Socket>,
    pub option: MenuOption,
    pub waiting_for_state: bool,
}

impl NavigationState {
    pub fn from_connecting_state(connecting_state: &ConnectingState) -> Self {
        Self {
            config: connecting_state.config.clone(),
            socket: connecting_state.socket.clone(),
            option: MenuOption::Play,
            waiting_for_state: false,
        }
    }

    pub fn handle_event(&mut self, event: &Event, engine: &mut Engine) -> StateTransition<MenuSubState> {
        match event {
            Event::InputEvent(event) => {
                if self.waiting_for_state {
                    return StateTransition::None;
                }

                match event {
                    InputEvent::KeyboardEvent(event) => {
                        if event.state == KeyState::Released {
                            return StateTransition::None;
                        }

                        if let Some(keycode) = event.virtual_keycode {
                            match keycode {
                                KeyCode::S => self.option = self.option.next(),
                                KeyCode::W => self.option = self.option.prev(),
                                KeyCode::Return => match self.option {
                                    MenuOption::Play => {
                                        let server_address = self.config.server_address;
                                        let party = &self.config.test_party;
                                        self.socket.send_message(server_address, party);
                                        self.waiting_for_state = true;
                                    },
                                    MenuOption::Quit => engine.terminate(),
                                },
                                KeyCode::Escape => return StateTransition::Old,
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                }
            },
            Event::NetworkEvent(event) => match event {
                NetworkEvent::Message(packet) => {
                    if packet.addr() != self.config.server_address {
                        return StateTransition::None;
                    }

                    let message = bincode::deserialize::<Message>(packet.payload()).unwrap();
                    info!("Received message from server: {}", message.type_);

                    match message.type_ {
                        colosseum::message::MessageType::CombatState => {
                            if self.waiting_for_state {
                                let shared_state = InternalCombatState::try_from(&message).unwrap();
                                info!("Received shared state from server");
                                return StateTransition::Change(GameState::CombatState(CombatState::new(
                                    self.config.clone(),
                                    self.socket.clone(),
                                    shared_state,
                                )));
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
