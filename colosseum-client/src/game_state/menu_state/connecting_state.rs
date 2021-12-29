// Copyright 2021 Chay Nabors.

use std::convert::TryFrom;
use std::rc::Rc;
use std::time::Duration;

use colosseum::message::Message;
use colosseum::message::ProtocolVersion;
use gear::event::Event;
use gear::event::InputEvent;
use gear::event::NetworkEvent;
use gear::Engine;
use gear::KeyCode;
use gear::NetworkConfig;
use gear::Packet;
use gear::Socket;
use log::info;

use super::splash_state::SplashState;
use crate::config::Config;
use crate::game_state::menu_state::navigation_state::NavigationState;
use crate::game_state::menu_state::MenuSubState;
use crate::game_state::state_transition::StateTransition;

pub struct ConnectingState {
    pub config: Rc<Config>,
    pub socket: Rc<Socket>,
}

impl ConnectingState {
    pub fn from_splash_state(splash_state: &SplashState, engine: &mut Engine) -> Self {
        let config = splash_state.config.clone();

        let socket = engine
            .network
            .bind_with_config(
                "0.0.0.0:0",
                NetworkConfig {
                    idle_connection_timeout: Duration::from_secs(60),
                    heartbeat_interval: Some(Duration::from_secs(25)),
                    ..Default::default()
                },
            )
            .unwrap();

        let protover = Message::try_from(&ProtocolVersion(0)).unwrap();
        socket.send(Packet::reliable_ordered(config.server_address, bincode::serialize(&protover).unwrap(), None));

        Self { config, socket: Rc::new(socket) }
    }

    pub fn handle_event(&mut self, event: &Event) -> StateTransition<MenuSubState> {
        match event {
            Event::InputEvent(event) => match event {
                InputEvent::KeyboardEvent(event) => match event.virtual_keycode {
                    Some(keycode) => match keycode {
                        KeyCode::Escape => StateTransition::Old,
                        _ => StateTransition::None,
                    },
                    None => StateTransition::None,
                },
                _ => StateTransition::None,
            },
            Event::NetworkEvent(event) => match event {
                NetworkEvent::Message(packet) => {
                    if packet.addr() != self.config.server_address {
                        return StateTransition::None;
                    }

                    let message = bincode::deserialize::<Message>(packet.payload()).unwrap();
                    info!("Received message from server: {}", message.type_);

                    match message.type_ {
                        colosseum::message::MessageType::ProtocolVersion => {
                            let protover = ProtocolVersion::try_from(&message).unwrap();
                            info!("Server running protocol version: {:?}", protover);
                            StateTransition::New(MenuSubState::NavigationState(NavigationState::from_connecting_state(self)))
                        },
                        _ => StateTransition::None,
                    }
                },
                NetworkEvent::Connect(_) => {
                    info!("Connected to server");
                    StateTransition::None
                },
                _ => StateTransition::None,
            },
            _ => StateTransition::None,
        }
    }
}
