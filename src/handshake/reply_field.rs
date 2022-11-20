use super::error::HandshakeError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReplyField {
  Succeeded,
  SocksServerFailure,
  ConnectionNotAllowed,
  NetworkUnreachable,
  HostUnreachable,
  ConnectionRefused,
  TtlExpired,
  CommandNotSupported,
  AddrTypeNotSupported,
}

impl TryFrom<u8> for ReplyField {
  type Error = HandshakeError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Succeeded),
      1 => Ok(Self::SocksServerFailure),
      2 => Ok(Self::ConnectionNotAllowed),
      3 => Ok(Self::NetworkUnreachable),
      4 => Ok(Self::HostUnreachable),
      5 => Ok(Self::ConnectionRefused),
      6 => Ok(Self::TtlExpired),
      7 => Ok(Self::CommandNotSupported),
      8 => Ok(Self::AddrTypeNotSupported),
      _ => Err(HandshakeError::UnsupportedRepType),
    }
  }
}

impl From<ReplyField> for u8 {

  fn from(val: ReplyField) -> Self {
    use ReplyField::*;
    match val {
      Succeeded => 0,
      SocksServerFailure => 1,
      ConnectionNotAllowed => 2,
      NetworkUnreachable => 3,
      HostUnreachable => 4,
      ConnectionRefused => 5,
      TtlExpired => 6,
      CommandNotSupported => 7,
      AddrTypeNotSupported => 8,
    }
  }
}