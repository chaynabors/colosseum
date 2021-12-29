// Copyright 2021 Chay Nabors.

use std::convert::TryFrom;
use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::combat_event::CombatEvent;
use crate::combat_state::CombatState;
use crate::party::Party;
use crate::target::Target;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolVersion(pub u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakeTurn {
    pub target: Target,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Victory;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageType {
    ProtocolVersion = 1, // this number must never change
    CombatState = 20,
    TakeTurn = 22,
    CombatEvent = 23,
    Victory = 30,
    Party = 255,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub type_: MessageType,
    payload: Vec<u8>,
}

macro_rules! try_into_message {
    ($type:ident) => {
        impl TryFrom<&$type> for Message {
            type Error = bincode::Error;

            fn try_from(value: &$type) -> Result<Self, Self::Error> {
                Ok(Message { type_: MessageType::$type, payload: bincode::serialize(value)? })
            }
        }
    };
}

macro_rules! try_from_message {
    ($type:ident) => {
        impl TryFrom<&Message> for $type {
            type Error = bincode::Error;

            fn try_from(value: &Message) -> Result<Self, Self::Error> {
                bincode::deserialize(&value.payload)
            }
        }
    };
}

macro_rules! message_conversions {
    ($type:ident) => {
        try_into_message!($type);
        try_from_message!($type);
    };
}

message_conversions!(ProtocolVersion);
message_conversions!(CombatState);
message_conversions!(TakeTurn);
message_conversions!(CombatEvent);
message_conversions!(Victory);
message_conversions!(Party);
