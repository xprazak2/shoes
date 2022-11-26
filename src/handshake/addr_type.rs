use crate::handshake::error::HandshakeError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl From<AddrType> for u8 {
  fn from(method: AddrType) -> Self {
    match method {
      AddrType::Ipv4 => 1,
    }
  }
}
