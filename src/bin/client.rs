use std::{net::Ipv4Addr, str::FromStr};

use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};
use shoes::{cli::ClientCli, handshake::{SocksHandshake, version::SocksVersion, cmd::SocksCmd, addr_type::AddrType, method::SocksMethod, reply_field::ReplyField, reply::SocksReply}, client::ClientConnectMsg};
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let default_port = 7474;
  let target_port = 6666;
  let default_host = "127.0.0.1".to_string();
  let args = ClientCli::parse();
  let port = args.port.unwrap_or(default_port);
  let host = args.host.unwrap_or(default_host);

  let mut socket = TcpStream::connect(format!("{}:{}", host, port)).await?;

  let connect_msg = ClientConnectMsg::new(SocksVersion::V5, 1, vec![SocksMethod::NoAuth]);

  socket.write(&connect_msg.to_request()).await?;
  let mut buf = [0_u8; 1024];
  socket.read(&mut buf).await?;
  let version: SocksVersion = buf[0].try_into()?;
  let _method: SocksMethod = buf[1].try_into()?;

  let hs_req = SocksHandshake{
    version: version,
    cmd: SocksCmd::Connect,
    addr: Ipv4Addr::from_str(&host).expect("should be a valid IPv4 addr"),
    port: target_port,
    atyp: AddrType::Ipv4,
  };

  socket.write(&hs_req.to_request()).await?;
  socket.read(&mut buf).await?;
  println!("reply: {:?}", buf);
  let socks_reply = SocksReply::parse(&buf)?;

  println!("socks reply: {:?}", socks_reply);
  if socks_reply.rep != ReplyField::Succeeded {
    panic!("Handshake failed, server replied: {:?}", socks_reply.rep)
  }

  let mut input = String::new();

  match std::io::stdin().read_line(&mut input) {
    Ok(_) => {
      println!("Sending data to server: {:?}", input);
      socket.write(input.as_bytes()).await?;
    },
    Err(e) => panic!("Failed to read user input: {:?}", e),
  };

  Ok(())
}
