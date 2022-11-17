use crate::handshake::error::HandshakeError;

#[derive(Clone, Copy, PartialEq)]
pub enum SocksCmd {
  Connect,
}

impl TryFrom<u8> for SocksCmd {
  type Error = HandshakeError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Self::Connect),
      _ => Err(Self::Error::UnsupportedCommand)
    }
  }
}


