// Copyright 2021 Chay Nabors.

mod config;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::sleep;
use std::thread::{self,};
use std::time::Duration;
use std::time::Instant;

use colosseum::combat_event::CombatEvent;
use colosseum::combat_state::CombatState;
use colosseum::message::Message;
use colosseum::message::MessageType;
use colosseum::message::ProtocolVersion;
use colosseum::message::TakeTurn;
use colosseum::party::Party;
use colosseum::target::Target;
use config::Config;
use crossbeam::channel::Sender;
use crossbeam::channel::TryRecvError;
use laminar::Config as NetworkConfig;
use laminar::Packet;
use laminar::SocketEvent;
use log::error;
use log::info;

pub trait Client {
    fn send_message<T: TryInto<Message, Error = bincode::Error>>(
        &self,
        sender: &Sender<Packet>,
        payload: T,
    ) -> anyhow::Result<()>;
}

impl Client for SocketAddr {
    fn send_message<T: TryInto<Message, Error = bincode::Error>>(
        &self,
        sender: &Sender<Packet>,
        payload: T,
    ) -> anyhow::Result<()> {
        let message: Message = payload.try_into()?;
        let payload = bincode::serialize(&message)?;
        sender.send(Packet::reliable_ordered(*self, payload, None))?;
        Ok(())
    }
}

struct Participant {
    pub address: SocketAddr,
    pub ownership: Vec<Target>,
}

struct Match {
    pub participants: Vec<Participant>,
    pub spectators: Vec<SocketAddr>,
    pub combat_state: CombatState,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter(None, log::LevelFilter::Info)
        .init();

    let config = load_config();
    let address = config.address;

    let mut socket = laminar::Socket::bind_with_config(
        address,
        NetworkConfig {
            idle_connection_timeout: Duration::from_secs(120),
            heartbeat_interval: Some(Duration::from_secs(50)),
            ..Default::default()
        },
    )?;
    let sender = socket.get_packet_sender();
    let mut receiver = Some(socket.get_event_receiver());

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop = stop_signal.clone();

    let mut socket_thread = Some(thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            socket.manual_poll(Instant::now());
            sleep(Duration::from_millis(1));
        }
    }));

    let mut clients: Vec<SocketAddr> = vec![];
    let mut ready_clients: Vec<(SocketAddr, Party)> = vec![];
    let mut matches_by_client: HashMap<SocketAddr, Rc<RefCell<Match>>> = HashMap::default();

    loop {
        match &receiver {
            Some(recv) => match recv.try_recv() {
                Ok(message) => match message {
                    SocketEvent::Packet(packet) => {
                        let message = bincode::deserialize::<Message>(packet.payload()).unwrap();
                        info!("Received message from {}: {:?}", packet.addr(), message.type_);
                        match matches_by_client.get(&packet.addr()) {
                            Some(match_) => {
                                let mut match_ = match_.borrow_mut();
                                match message.type_ {
                                    MessageType::CombatEvent => {
                                        let event = CombatEvent::try_from(&message).unwrap();

                                        // propogate message to participants
                                        for participant in &match_.participants {
                                            participant.address.send_message(&sender, &event).unwrap();
                                        }

                                        // propogate message to spectators
                                        for spectator in &match_.spectators {
                                            spectator.send_message(&sender, &event).unwrap();
                                        }

                                        match_.combat_state.process_event(&event);

                                        // tell next client to take a turn
                                        let ready = match_.combat_state.next_combatant();
                                        let mut owner = None;
                                        for i in 0..match_.participants.len() {
                                            if match_.participants[i].ownership.contains(&ready) {
                                                owner = Some(i);
                                                break;
                                            }
                                        }

                                        match owner {
                                            Some(owner) => {
                                                info!(
                                                    "Requested that {} takes a turn for {}",
                                                    match_.participants[owner].address,
                                                    match_.combat_state.parties[ready.party_index].members
                                                        [ready.member_index]
                                                        .name
                                                );
                                                match_.participants[owner]
                                                    .address
                                                    .send_message(&sender, &TakeTurn { target: ready })
                                                    .unwrap();
                                            },
                                            None => error!("Match participant has no owner but needs to take a turn"),
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            None => {
                                if clients.contains(&packet.addr()) {
                                    if message.type_ == MessageType::Party {
                                        let party = Party::try_from(&message).unwrap();
                                        ready_clients.push((packet.addr().clone(), party));

                                        if ready_clients.len() > 1 {
                                            let (addr2, party2) = ready_clients.pop().unwrap();
                                            let (addr1, party1) = ready_clients.pop().unwrap();

                                            let combat_state = CombatState { parties: vec![party1, party2] };

                                            addr1.send_message(&sender, &combat_state).unwrap();
                                            addr2.send_message(&sender, &combat_state).unwrap();

                                            let target_list = combat_state.get_target_list();
                                            let ownership1 = target_list
                                                .iter()
                                                .filter(|target| target.party_index == 0)
                                                .copied()
                                                .collect();
                                            let ownership2 = target_list
                                                .iter()
                                                .filter(|target| target.party_index == 1)
                                                .copied()
                                                .collect();

                                            let match_ = Rc::new(RefCell::new(Match {
                                                participants: vec![
                                                    Participant { address: addr1, ownership: ownership1 },
                                                    Participant { address: addr2, ownership: ownership2 },
                                                ],
                                                spectators: vec![],
                                                combat_state,
                                            }));

                                            {
                                                let mut match_ = match_.borrow_mut();

                                                let ready = match_.combat_state.next_combatant();
                                                let mut owner = None;
                                                for i in 0..match_.participants.len() {
                                                    if match_.participants[i].ownership.contains(&ready) {
                                                        owner = Some(i);
                                                        break;
                                                    }
                                                }

                                                match owner {
                                                    Some(owner) => {
                                                        info!(
                                                            "Requested that {} takes a turn for {}",
                                                            match_.participants[owner].address,
                                                            match_.combat_state.parties[ready.party_index].members
                                                                [ready.member_index]
                                                                .name
                                                        );
                                                        match_.participants[owner]
                                                            .address
                                                            .send_message(&sender, &TakeTurn { target: ready })
                                                            .unwrap();
                                                    },
                                                    None => {
                                                        error!("Match participant has no owner but needs to take a turn")
                                                    },
                                                }
                                            }

                                            matches_by_client.insert(addr1, match_.clone());
                                            matches_by_client.insert(addr2, match_);
                                        }
                                    }
                                } else if message.type_ == MessageType::ProtocolVersion {
                                    let protover = Message::try_from(&ProtocolVersion(0)).unwrap();
                                    sender
                                        .send(Packet::reliable_ordered(
                                            packet.addr(),
                                            bincode::serialize(&protover).unwrap(),
                                            None,
                                        ))
                                        .unwrap();
                                }
                            },
                        }
                    },
                    SocketEvent::Connect(address) => clients.push(address),
                    SocketEvent::Timeout(address) => info!("{} timed out", address),
                    SocketEvent::Disconnect(address) => info!("{} disconnected", address),
                },
                Err(e) => match e {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => {
                        socket_thread.take().unwrap().join().unwrap();
                        receiver.take();
                        break;
                    },
                },
            },
            None => break,
        }
    }

    stop_signal.swap(true, Ordering::Relaxed);

    Ok(())
}

fn load_config() -> Config {
    let path = Path::new("config.json");

    match path.exists() {
        true => {
            let config = fs::read(path).unwrap();
            serde_json::from_slice::<Config>(&config).unwrap()
        },
        false => {
            let config = Config::default();
            fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            config
        },
    }
}
