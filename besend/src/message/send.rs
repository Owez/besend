use crate::{MessageContent, Peer, Result, State, ToBytes};
use std::net::SocketAddr;

pub struct MessageSender {
    pub addr: SocketAddr,
    pub content: MessageContent,
}

impl MessageSender {
    pub fn new(peer: &Peer, content: MessageContent) -> Self {
        Self {
            addr: peer.addr,
            content,
        }
    }

    pub fn send(&self, state: &State) -> Result<()> {
        let mut bytes = state.id.as_bytes().to_vec();
        bytes.extend(self.content.to_bytes()?);

        state.socket.send_to(&bytes[..], self.addr)?;
        Ok(())
    }
}
