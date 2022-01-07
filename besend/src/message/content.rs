use uuid::Uuid;

use crate::{Error, FromBytes, Result, ToBytes};

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
            unknown => Err(Error::UnknownMessage(unknown)),
        }
    }
}

/// Decodes u16 from 2 bytes in iter
fn decode_u16(bytes: &mut impl Iterator<Item = u8>) -> Result<u16> {
    let first = bytes.next().ok_or(Error::MessageEnded)?;
    let second = bytes.next().ok_or(Error::MessageEnded)?;
    Ok(u16::from_be_bytes([first, second]))
}

/// Decodes uuid from an unknown source
fn decode_uuid(bytes: &mut impl Iterator<Item = u8>) -> Result<Uuid> {
    const UUID_LEN: usize = 16;
    let bytes: Vec<u8> = bytes.take(UUID_LEN).collect();
    if bytes.len() != UUID_LEN {
        Err(Error::MessageEnded)
    } else {
        Ok(Uuid::from_bytes(bytes.try_into().unwrap()))
    }
}

/// Helper for encoding/decoding strings sent over the network with their length defined as a big endian [u16] beforehand
struct LenString(pub String);

impl LenString {
    fn decode(
        bytes: &mut impl Iterator<Item = u8>,
        limit: impl Into<Option<u16>>,
    ) -> Result<String> {
        let len = decode_u16(bytes)?;

        if let Some(limit) = limit.into() {
            if len > limit {
                return Err(Error::StringLimit((limit, len)));
            }
        }

        let bytes: Vec<u8> = bytes.take(len as usize).collect();
        if bytes.len() < len as usize {
            Err(Error::MessageEnded)
        } else {
            match String::from_utf8(bytes) {
                Ok(string) => Ok(string),
                Err(_) => Err(Error::InvalidString),
            }
        }
    }
}

impl ToBytes for LenString {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let len = self.0.len();
        if len > u16::MAX as usize {
            return Err(Error::StringTooLong);
        }

        let mut lenstr = (len as u16).to_be_bytes().to_vec();
        lenstr.extend(self.0.as_bytes());
        Ok(lenstr)
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
