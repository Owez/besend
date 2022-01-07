use crate::{Error, FromBytes, Peer, Result, ToBytes, serialize::{LenString, decode_uuid, decode_u16}};
use uuid::Uuid;

/// String length limit for sound names being sent over the network
const SOUNDNAME_LIMIT: u16 = 40;

/// Represents each [MessageContent] variant without any potential data as it's designator byte
macro_rules! message_byte {
    (Message::AdvertiseSound) => {
        0u8
    };
    (Message::AdvertiseAvailability) => {
        1u8
    };
    (Message::Interested) => {
        2u8
    };
}

/// Internally constructed message, containing the message type and it's contents
#[derive(Debug, PartialEq)]
pub enum MessageContent {
    /// Advertisement message for a sound with provided sound source message
    AdvertiseSound(String),
    /// Advertise your availability to pick up sources
    AdvertiseAvailability,
    /// Interested message relaying the advertisers id, as well as the pin code generated on the advertisers device
    Interested((Uuid, u16)),
    /// Someone sent [MessageContent::AdvertiseAvailability] and we know information on another peer
    WeKnow(Peer),
}

impl ToBytes for MessageContent {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::AdvertiseSound(soundname) => {
                let mut bytes = vec![message_byte!(Message::AdvertiseSound)];
                bytes.extend(LenString(soundname.clone()).to_bytes()?);
                Ok(bytes)
            }
            Self::AdvertiseAvailability => Ok(vec![message_byte!(Message::AdvertiseAvailability)]),
            Self::Interested((uuid, pin)) => {
                let mut bytes = vec![message_byte!(Message::Interested)];
                bytes.extend(uuid.as_bytes());
                bytes.extend(pin.to_be_bytes());
                Ok(bytes)
            }
            Self::WeKnow(peer) => peer.to_bytes(),
        }
    }
}

impl FromBytes for MessageContent {
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self> {
        let mut bytes = bytes.into_iter();
        let designator = bytes.next().ok_or(Error::MessageEnded)?;

        match designator {
            0 => Ok(Self::AdvertiseSound(LenString::decode(
                &mut bytes,
                SOUNDNAME_LIMIT,
            )?)),
            1 => Ok(Self::AdvertiseAvailability),
            2 => Ok(Self::Interested((
                decode_uuid(&mut bytes)?,
                decode_u16(&mut bytes)?,
            ))),
            3 => Ok(Self::WeKnow(Peer::from_bytes(bytes)?)),
            unknown => Err(Error::UnknownMessage(unknown)),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_ad_encode_decode() -> Result<()> {
        let string = "abcdefg".to_string();

        let mut bytes = vec![1, 0, 7];
        bytes.extend(string.as_bytes());

        assert_eq!(
            MessageContent::AdvertiseSound(string.clone()).to_bytes()?,
            bytes
        );
        assert_eq!(
            MessageContent::from_bytes(bytes)?,
            MessageContent::AdvertiseSound(string)
        );
        Ok(())
    }

    #[test]
    #[should_panic]
    fn sound_ad_limit() {
        let payload = vec![0, 255, 255]; // 255 and 255 go to 65535, limit is less so this will error
        MessageContent::from_bytes(payload).unwrap();
    }
}
