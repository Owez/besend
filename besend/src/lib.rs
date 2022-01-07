mod errors;
mod message;
pub(crate) mod serialize;

pub use errors::{Error, Result};
pub use message::MessageContent;
use serialize::{decode_uuid, Address};

use crate::message::MessageSender;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use uuid::Uuid;

const PORT: u16 = 7767;

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

    pub fn advertise_sound(&self, name: impl Into<String>) -> Result<()> {
        fn runner(state: &State, addr: SocketAddr, custom: &String) -> Result<()> {
            MessageSender::to_addr(addr, MessageContent::AdvertiseSound(custom.clone())).send(state)
        }
        self.ip_looper(runner, name.into())
    }

    pub fn advertise_availability(&self) -> Result<()> {
        fn runner(state: &State, addr: SocketAddr, _custom: &()) -> Result<()> {
            MessageSender::to_addr(addr, MessageContent::AdvertiseAvailability).send(state)
        }
        self.ip_looper(runner, ())
    }

    pub fn listen(&self, active: bool) -> Result<Peer> {
        if active {
            self.advertise_availability()?;
        }

        todo!()
    }

    pub fn connect(&self, peer: Peer, pin: impl Into<u16>) -> Result<()> {
        // send an interested message
        let msg_sender =
            MessageSender::to_peer(&peer, MessageContent::Interested((peer.id, pin.into())));
        msg_sender.send(self)?;

        // wait for reply
        todo!()
    }

    /// Loops over local ip addresses (192.168.x.x) and triggers `run` on each
    fn ip_looper<T>(
        &self,
        runner: impl Fn(&State, SocketAddr, &T) -> Result<()>,
        custom: T,
    ) -> Result<()> {
        for outer in 0..u8::MAX {
            for inner in 0..u8::MAX {
                let addr = SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(192, 168, outer, inner),
                    PORT,
                ));
                runner(self, addr, &custom)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Peer {
    pub id: Uuid,
    pub addr: SocketAddr,
}

impl ToBytes for Peer {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = self.id.as_bytes().to_vec();
        bytes.extend(Address(self.addr).to_bytes()?);
        Ok(bytes)
    }
}

impl FromBytes for Peer {
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self> {
        let mut bytes = bytes.into_iter();
        Ok(Self {
            id: decode_uuid(&mut bytes)?,
            addr: Address::from_bytes(bytes)?.0,
        })
    }
}
