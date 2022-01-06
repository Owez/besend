//! Messaging constructs

use crate::{Error, FromBytes, Result, ToBytes};

/// String length limit for filenames being sent over the network
const FILENAME_LIMIT: u16 = 512;

/// String length limit for sound names being sent over the network
const SOUNDNAME_LIMIT: u16 = 40;

/// Represents each [Message] variant without any potential data as it's designator byte
macro_rules! message_byte {
    (Message::AdvertiseFile) => {
        0u8
    };
    (Message::AdvertiseSound) => {
        1u8
    };
}

/// Internally constructed message, containing the message type and it's contents
#[derive(Debug, PartialEq)]
pub enum Message {
    AdvertiseFile(String),
    AdvertiseSound(String),
}

impl ToBytes for Message {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::AdvertiseFile(filename) => {
                let mut bytes = vec![message_byte!(Message::AdvertiseFile)];
                bytes.extend(LenString(filename.clone()).to_bytes()?);
                Ok(bytes)
            }
            Self::AdvertiseSound(soundname) => {
                let mut bytes = vec![message_byte!(Message::AdvertiseSound)];
                bytes.extend(LenString(soundname.clone()).to_bytes()?);
                Ok(bytes)
            }
        }
    }
}

impl FromBytes for Message {
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self> {
        let mut bytes = bytes.into_iter();
        let designator = bytes.next().ok_or(Error::MessageEnded)?;

        match designator {
            0 => Ok(Self::AdvertiseFile(LenString::decode(
                &mut bytes,
                FILENAME_LIMIT,
            )?)),
            1 => Ok(Self::AdvertiseSound(LenString::decode(
                &mut bytes,
                SOUNDNAME_LIMIT,
            )?)),
            unknown => Err(Error::UnknownMessage(unknown)),
        }
    }
}

/// Helper for encoding/decoding strings sent over the network with their length defined as a big endian [u16] beforehand
struct LenString(pub String);

impl LenString {
    fn decode(
        bytes: &mut impl Iterator<Item = u8>,
        limit: impl Into<Option<u16>>,
    ) -> Result<String> {
        let first = bytes.next().ok_or(Error::MessageEnded)?;
        let second = bytes.next().ok_or(Error::MessageEnded)?;

        let len = u16::from_be_bytes([first, second]);

        if let Some(limit) = limit.into() {
            if len > limit {
                return Err(Error::StringLimit((limit, len)));
            }
        }

        let bytes: Vec<u8> = bytes.collect();
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
    fn file_ad_encode_decode() -> Result<()> {
        let string = "abcdefg".to_string();

        let mut bytes = vec![0, 0, 7];
        bytes.extend(string.as_bytes());

        assert_eq!(Message::AdvertiseFile(string.clone()).to_bytes()?, bytes);
        assert_eq!(Message::from_bytes(bytes)?, Message::AdvertiseFile(string));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn file_ad_limit() {
        let payload = vec![0, 255, 255]; // 255 and 255 go to 65535, limit is less so this will error
        Message::from_bytes(payload).unwrap();
    }

    #[test]
    fn sound_ad_encode_decode() -> Result<()> {
        let string = "abcdefg".to_string();

        let mut bytes = vec![1, 0, 7];
        bytes.extend(string.as_bytes());

        assert_eq!(Message::AdvertiseSound(string.clone()).to_bytes()?, bytes);
        assert_eq!(Message::from_bytes(bytes)?, Message::AdvertiseSound(string));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn sound_ad_limit() {
        let payload = vec![0, 255, 255]; // 255 and 255 go to 65535, limit is less so this will error
        Message::from_bytes(payload).unwrap();
    }
}
