use std::{io::Cursor, net::Ipv4Addr};

use bytes::Buf;

use super::{
    addr_type::AddrType, error::HandshakeError, reply_field::ReplyField, version::SocksVersion,
    WithIpv4Addr, WithPort,
};

#[derive(Debug)]
pub struct SocksReply {
    pub version: SocksVersion,
    pub rep: ReplyField,
    pub atyp: AddrType,
    pub bnd_addr: Ipv4Addr,
    pub bnd_port: u16,
}

impl SocksReply {
    pub fn new(
        version: SocksVersion,
        rep: ReplyField,
        atyp: AddrType,
        addr: Ipv4Addr,
        port: u16,
    ) -> Self {
        Self {
            version,
            rep,
            atyp,
            bnd_addr: addr,
            bnd_port: port,
        }
    }

    pub fn to_reply(&self) -> Vec<u8> {
        let mut reply: Vec<u8> = vec![];
        reply.push(self.version.into());
        reply.push(self.rep.into());
        reply.push(0); // reserved
        reply.push(self.atyp.into());
        reply.append(&mut self.addr_to_bytes());
        reply.append(&mut self.port_to_bytes());
        reply
    }

    pub fn parse(buf: &[u8]) -> Result<Self, HandshakeError> {
        let mut incoming = Cursor::new(buf);
        let version: SocksVersion = incoming.get_u8().try_into()?;

        if !incoming.has_remaining() {
            return Err(HandshakeError::Incomplete);
        }
        let rep: ReplyField = incoming.get_u8().try_into()?;

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

        Ok(Self::new(version, rep, atyp, addr, port))
    }
}

impl WithIpv4Addr for SocksReply {
    fn addr(&self) -> Ipv4Addr {
        self.bnd_addr
    }
}
impl WithPort for SocksReply {
    fn port(&self) -> u16 {
        self.bnd_port
    }
}
