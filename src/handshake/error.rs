use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandshakeError {
  #[error("unsupported SOCKS version")]
  UnsupportedVersion,

  #[error("incomplete input")]
  Incomplete,

  #[error("unsupported method")]
  UnsupportedMethod,

  #[error("unsupported command")]
  UnsupportedCommand,

  #[error("unsupported atyp")]
  UnsupportedAddrType,
}
