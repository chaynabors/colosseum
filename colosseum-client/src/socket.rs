// Copyright 2021 Chay Nabors.

use std::convert::TryInto;
use std::fmt::Debug;
use std::net::SocketAddr;

use colosseum_core::message::Message;

pub trait ClientSocket {
    fn send_message<T>(&self, address: SocketAddr, message: T)
    where
        T: TryInto<Message>,
        T::Error: Debug;
}

impl ClientSocket for Socket {
    fn send_message<T>(&self, address: SocketAddr, message: T)
    where
        T: TryInto<Message>,
        T::Error: Debug,
    {
        let message: Message = message.try_into().unwrap();
        let payload = bincode::serialize(&message).unwrap();
        self.send(Packet::reliable_ordered(address, payload, None));
    }
}
