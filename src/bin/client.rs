use std::{net::Ipv4Addr, str::FromStr};

use clap::Parser;
use shoes::{
    cli::ClientCli,
    client::ClientConnectMsg,
    handshake::{
        addr_type::AddrType, cmd::SocksCmd, method::SocksMethod, reply::SocksReply,
        reply_field::ReplyField, version::SocksVersion, SocksHandshake,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

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

    let hs_req = SocksHandshake {
        version,
        cmd: SocksCmd::Connect,
        addr: Ipv4Addr::from_str(&host).expect("should be a valid IPv4 addr"),
        port: target_port,
        atyp: AddrType::Ipv4,
    };

    socket.write(&hs_req.to_request()).await?;
    socket.read(&mut buf).await?;
    let socks_reply = SocksReply::parse(&buf)?;

    println!("socks reply: {:?}", socks_reply);
    if socks_reply.rep != ReplyField::Succeeded {
        println!(
            "Handshake failed, exiting. Server replied: {:?}",
            socks_reply.rep
        );
        return Ok(());
    }

    let mut input = String::new();
    println!("Input the data which will be sent to server:");

    // this example client works in 'oneshot' mode - it disconnects after sending one message
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            println!("Sending data to server: {:?}", input);
            socket.write(input.as_bytes()).await?;
        }
        Err(e) => panic!("Failed to read user input: {:?}", e),
    };

    Ok(())
}
