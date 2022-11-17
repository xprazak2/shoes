
use crate::handshake::error::HandshakeError;

#[derive(Clone, Copy, PartialEq)]
pub enum SocksVersion {
  V5
}

impl TryFrom<u8> for SocksVersion {
  type Error = HandshakeError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      5 => Ok(Self::V5),
      _ => Err(Self::Error::UnsupportedVersion)
    }
  }
}

impl From <SocksVersion> for u8 {
  fn from(ver: SocksVersion) -> Self {
    match ver {
      SocksVersion::V5 => 5
    }
  }
}
