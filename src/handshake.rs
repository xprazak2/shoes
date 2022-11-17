use std::{io::Cursor};
use bytes::Buf;
use std::net::Ipv4Addr;

pub mod addr_type;
pub mod error;
pub mod version;
pub mod method;
pub mod cmd;

use crate::handshake::addr_type::AddrType;
use crate::handshake::error::HandshakeError;
use crate::handshake::method::SocksMethod;
use crate::handshake::version::SocksVersion;
use crate::handshake::cmd::SocksCmd;

struct SocksHandshake {
  version: SocksVersion,
  cmd: SocksCmd,
  addr: Ipv4Addr,
  port: u16,
}

#[derive(PartialEq)]
pub enum ConnState {
  Init,
  HsWait,
  HsFinished,
}

impl ConnState {
  fn next(&self) -> Self {
    use ConnState::*;
    match self {
      Init => HsWait,
      HsWait => HsFinished,
      HsFinished => HsFinished,
    }
  }
}

pub struct ConnStateBuilder {
  state: ConnState,
  version: Option<SocksVersion>,
  methods: Vec<SocksMethod>,
  reply: Vec<u8>,
  handshake: Option<SocksHandshake>,
}

impl ConnStateBuilder {
  pub fn new() -> Self {
    Self {
      state: ConnState::Init,
      version: None,
      methods: vec![],
      reply: vec![],
      handshake: None,
    }
  }

  pub fn reply(&self) -> &Vec<u8> {
    &self.reply
  }

  pub fn advance(&mut self, buf: &[u8]) -> Result<(), HandshakeError> {
    if buf.is_empty() {
      return Err(HandshakeError::Incomplete)
    }

    match self.state {
      ConnState::Init => self.advance_from_init(buf),
      ConnState::HsWait => self.advance_from_wait(buf),
      ConnState::HsFinished => Ok(()),
    }
  }

  fn advance_from_wait(&mut self, buf: &[u8]) -> Result<(), HandshakeError> {
    let mut incoming = Cursor::new(buf);
    let version: SocksVersion = incoming.get_u8().try_into()?;

    if self.version.map_or_else(|| true, |ver| ver != version) {
      return Err(HandshakeError::UnsupportedVersion)
    }

    if !incoming.has_remaining() {
      return Err(HandshakeError::Incomplete);
    }
    let cmd: SocksCmd = incoming.get_u8().try_into()?;

    if !incoming.has_remaining() {
      return Err(HandshakeError::Incomplete);
    }
    let _rsv = incoming.get_u8();
    if !incoming.has_remaining() {
      return Err(HandshakeError::Incomplete);
    }

    let _atyp: AddrType = incoming.get_u8().try_into()?;

    if incoming.remaining() < 4 {
      return Err(HandshakeError::Incomplete);
    }

    let addr = Ipv4Addr::from(incoming.get_u32());

    if incoming.remaining() < 2 {
      return Err(HandshakeError::Incomplete);
    }

    let port = incoming.get_u16();
    self.handshake = Some(SocksHandshake{ version, cmd, addr, port });
    self.state = self.state.next();

    Ok(())
  }

  fn advance_from_init(&mut self, buf: &[u8]) -> Result<(), HandshakeError> {
    let mut incoming = Cursor::new(buf);
    let version = incoming.get_u8().try_into()?;
    self.version = Some(version);

    if !incoming.has_remaining() {
      return Err(HandshakeError::Incomplete);
    }

    let n_methods = incoming.get_u8();

    for _ in 0..n_methods {
      if !incoming.has_remaining() {
        return Err(HandshakeError::Incomplete);
      }

      let method: SocksMethod = incoming.get_u8().into();
      self.methods.push(method);
    }

    // make sure we read everything from buffer, drain?
    // take not of selected method?
    self.reply = vec![version.into(), self.select_method().into()];
    self.state = self.state.next();

    Ok(())
  }

  fn select_method(&self) -> SocksMethod {
    if self.methods.contains(&SocksMethod::NoAuth) {
      return SocksMethod::NoAuth
    }
    return SocksMethod::NoAcceptableMethod
  }
}