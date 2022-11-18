use std::{net::Ipv4Addr, str::FromStr};

use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};
use shoes::{server, cli::ClientCli, handshake::{SocksHandshake, version::SocksVersion, cmd::SocksCmd, addr_type::AddrType, method::SocksMethod}, client::ClientConnectMsg};
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let default_port = 7474;
  let default_host = "127.0.0.1".to_string();
  let args = ClientCli::parse();
  let port = args.port.unwrap_or(default_port);
  let host = args.host.unwrap_or(default_host);

  let mut socket = TcpStream::connect(format!("{}:{}", host, port)).await?;

  let connect_msg = ClientConnectMsg::new(SocksVersion::V5, 1, vec![SocksMethod::NoAuth]);

  let req_written = socket.write(&connect_msg.to_request()).await?;
  let mut buf = [0_u8; 1024];
  socket.read(&mut buf).await?;
  let version: SocksVersion = buf[0].try_into()?;
  let method: SocksMethod = buf[1].try_into()?;

  let hs_req = SocksHandshake{
    version: version,
    cmd: SocksCmd::Connect,
    addr: Ipv4Addr::from_str(&host).expect("should be a valid IPv4 addr"),
    port: port,
    atyp: AddrType::Ipv4,
  };

  let req_written = socket.write(&hs_req.to_request()).await?;


  // socket.
  Ok(())
}

