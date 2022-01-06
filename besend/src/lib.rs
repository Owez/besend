mod errors;
mod message;

use std::net::{SocketAddr, UdpSocket};

pub use errors::{Error, Result};
pub use message::MessageContent;

use uuid::Uuid;

use crate::message::MessageSender;

/// General implementor for a `.to_bytes()` method which could error
pub trait ToBytes {
    /// Converts self into a vector of bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

/// General implementor for a `.from_bytes()` method which could error
pub trait FromBytes: Sized {
    /// Converts any potential iterator of bytes into new self
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self>;
}

pub struct State {
    pub id: Uuid,
    pub socket: UdpSocket,
}

impl State {
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            socket: UdpSocket::bind(addr)?,
        })
    }

    pub fn advertise_file(&self, _name: impl Into<String>) -> Result<()> {
        todo!();
    }

    pub fn advertise_sound(&self, _name: impl Into<String>) -> Result<()> {
        todo!();
    }

    pub fn listen(&self, file: bool, sound: bool) -> Result<Peer> {
        if !file && !sound {
            return Err(Error::NotListening);
        }

        todo!()
    }

    pub fn connect(&self, peer: Peer) -> Result<()> {
        // send an interested message
        let msg_sender = MessageSender::new(&peer, MessageContent::Interested(peer.id));
        msg_sender.send(self)?;

        // wait for reply
        todo!()
    }
}

pub struct Peer {
    pub id: Uuid,
    pub addr: SocketAddr,
}
