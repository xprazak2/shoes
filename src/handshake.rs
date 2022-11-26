use bytes::Buf;
use std::io::Cursor;
use std::net::Ipv4Addr;

pub mod addr_type;
pub mod cmd;
pub mod error;
pub mod method;
pub mod reply;
pub mod reply_field;
pub mod version;

use crate::handshake::addr_type::AddrType;
use crate::handshake::cmd::SocksCmd;
use crate::handshake::error::HandshakeError;
use crate::handshake::method::SocksMethod;
use crate::handshake::version::SocksVersion;

pub trait WithIpv4Addr {
    fn addr(&self) -> Ipv4Addr;

    fn addr_to_bytes(&self) -> Vec<u8> {
        self.addr().octets().to_vec()
    }
}

pub trait WithPort {
    fn port(&self) -> u16;

    fn port_to_bytes(&self) -> Vec<u8> {
        self.port().to_be_bytes().to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct SocksHandshake {
    pub version: SocksVersion,
    pub cmd: SocksCmd,
    pub addr: Ipv4Addr,
    pub port: u16,
    pub atyp: AddrType, // do we need this one if the information can be derived from addr?
}

impl SocksHandshake {
    pub fn to_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }

    pub fn to_request(&self) -> Vec<u8> {
        let mut req: Vec<u8> = vec![];
        req.push(self.version.into());
        req.push(self.cmd.into());
        req.push(0); // reserved
        req.push(self.atyp.into());
        req.append(&mut self.addr_to_bytes());
        req.append(&mut self.port_to_bytes());
        req
    }
}

impl WithIpv4Addr for SocksHandshake {
    fn addr(&self) -> Ipv4Addr {
        self.addr
    }
}
impl WithPort for SocksHandshake {
    fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Clone, Debug)]
pub enum HandshakeState {
    Init,
    Wait(SocksVersion, Vec<SocksMethod>),
    Finished(SocksHandshake),
}

#[derive(Clone)]
pub struct HandshakeStateBuilder {
    state: HandshakeState,
}

impl HandshakeStateBuilder {
    pub fn new() -> Self {
        Self {
            state: HandshakeState::Init,
        }
    }

    pub fn state(&self) -> HandshakeState {
        self.state.clone()
    }

    pub fn advance(&mut self, buf: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        if buf.is_empty() {
            return Err(HandshakeError::Incomplete);
        }

        match self.state {
            HandshakeState::Init => self.advance_from_init(buf),
            HandshakeState::Wait(current_version, _) => {
                self.advance_from_wait(buf, current_version)
            }
            HandshakeState::Finished(_) => Ok(vec![]),
        }
    }

    fn advance_from_wait(
        &mut self,
        buf: &[u8],
        current_version: SocksVersion,
    ) -> Result<Vec<u8>, HandshakeError> {
        let mut incoming = Cursor::new(buf);
        let version: SocksVersion = incoming.get_u8().try_into()?;

        if current_version != version {
            return Err(HandshakeError::UnsupportedVersion);
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

        let atyp: AddrType = incoming.get_u8().try_into()?;

        if incoming.remaining() < 4 {
            return Err(HandshakeError::Incomplete);
        }

        let addr = Ipv4Addr::from(incoming.get_u32());

        if incoming.remaining() < 2 {
            return Err(HandshakeError::Incomplete);
        }

        let port = incoming.get_u16();
        self.state = HandshakeState::Finished(SocksHandshake {
            version,
            cmd,
            addr,
            port,
            atyp,
        });

        Ok(vec![])
    }

    fn advance_from_init(&mut self, buf: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        let mut incoming = Cursor::new(buf);
        let version: SocksVersion = incoming.get_u8().try_into()?;

        if !incoming.has_remaining() {
            return Err(HandshakeError::Incomplete);
        }

        let n_methods = incoming.get_u8();

        let mut methods = vec![];

        for _ in 0..n_methods {
            if !incoming.has_remaining() {
                return Err(HandshakeError::Incomplete);
            }

            let method: SocksMethod = incoming.get_u8().into();
            methods.push(method);
        }

        // should we check we read everything from buffeer?
        // add selected method to state?
        let reply = vec![version.into(), self.select_method(&methods).into()];
        self.state = HandshakeState::Wait(version, methods);

        Ok(reply)
    }

    fn select_method(&self, methods: &Vec<SocksMethod>) -> SocksMethod {
        if methods.contains(&SocksMethod::NoAuth) {
            return SocksMethod::NoAuth;
        }
        SocksMethod::NoAcceptableMethod
    }
}
