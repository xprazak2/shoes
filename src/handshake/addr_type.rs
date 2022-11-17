use crate::handshake::error::HandshakeError;

#[derive(Clone, Copy, PartialEq)]
pub enum AddrType {
  Ipv4,
}

impl TryFrom<u8> for AddrType {
  type Error = HandshakeError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Self::Ipv4),
      _ => Err(Self::Error::UnsupportedAddrType)
    }
  }
}


