use crate::handshake::error::HandshakeError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SocksCmd {
    Connect,
}

impl TryFrom<u8> for SocksCmd {
    type Error = HandshakeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Connect),
            _ => Err(Self::Error::UnsupportedCommand),
        }
    }
}

impl From<SocksCmd> for u8 {
    fn from(item: SocksCmd) -> Self {
        match item {
            SocksCmd::Connect => 1,
        }
    }
}
