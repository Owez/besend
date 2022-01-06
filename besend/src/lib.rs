mod errors;
mod message;

pub use errors::{Error, Result};
pub use message::MessageContent;

use uuid::Uuid;

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
}

impl State {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn advertise_local() -> Result<()> {
        todo!()
    }
}
