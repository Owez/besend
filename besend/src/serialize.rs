use std::net::SocketAddr;

use crate::{Error, FromBytes, Result, ToBytes};
use uuid::Uuid;

/// Decodes u16 from 2 bytes in iter
pub(crate) fn decode_u16(bytes: &mut impl Iterator<Item = u8>) -> Result<u16> {
    let first = bytes.next().ok_or(Error::MessageEnded)?;
    let second = bytes.next().ok_or(Error::MessageEnded)?;
    Ok(u16::from_be_bytes([first, second]))
}

/// Decodes uuid from an unknown source
pub(crate) fn decode_uuid(bytes: &mut impl Iterator<Item = u8>) -> Result<Uuid> {
    const UUID_LEN: usize = 16;
    let bytes: Vec<u8> = bytes.take(UUID_LEN).collect();
    if bytes.len() != UUID_LEN {
        Err(Error::MessageEnded)
    } else {
        Ok(Uuid::from_bytes(bytes.try_into().unwrap()))
    }
}

/// Helper for encoding/decoding strings sent over the network with their length defined as a big endian [u16] beforehand
pub(crate) struct LenString(pub String);

impl LenString {
    pub(crate) fn decode(
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

pub(crate) struct Address(pub SocketAddr);

impl ToBytes for Address {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        match self.0 {
            SocketAddr::V4(ipv4) => {
                bytes.push(0);
                bytes.extend(ipv4.ip().octets());
                bytes.extend(ipv4.port().to_be_bytes());
            }
            SocketAddr::V6(ipv6) => {
                bytes.push(1);
                bytes.extend(ipv6.ip().octets());
                bytes.extend(ipv6.port().to_be_bytes());
            }
        }
        Ok(bytes)
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self> {
        let mut bytes = bytes.into_iter();
        match bytes.next() {
            Some(0) => todo!("ipv4"),
            Some(1) => todo!("ipv6"),
            Some(unknown) => Err(Error::UnknownAddressType(unknown)),
            None => Err(Error::MessageEnded),
        }
    }
}
