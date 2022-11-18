use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{debug, error};

use std::str;
use std::io::Cursor;

use crate::handshake::{HandshakeStateBuilder, HandshakeState, SocksHandshake};


#[derive(Debug)]
struct Server {
  listener: TcpListener
}

impl Server {
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    loop {
      let socket = self.accept().await?;

      let mut handler = ConnHandler::new(socket);

      tokio::spawn(async move {
        if let Err(err) = handler.run().await {
          error!(err);
        }
      });
    }
  }

  async fn accept(&mut self) -> Result<TcpStream, Box<dyn std::error::Error>> {
    match self.listener.accept().await {
      Ok((socket, _)) => Ok(socket),
      Err(err) => Err(err.into()),
    }
  }
}

#[derive(Debug)]
enum ConnState {
  Handshake,
  VerifyTarget(SocksHandshake),
  ConnEstablished(TcpStream),
}

#[derive(Debug)]
struct ConnHandler {
  socket: TcpStream,
  conn_state: ConnState,
}

impl ConnHandler {
  pub fn new(socket: TcpStream) -> Self {
    Self { socket: socket, conn_state: ConnState::Handshake }
  }

  #[tracing::instrument(skip(self))]
  async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    println!("processing connection...");
    let res = self.read_handshake().await?;

    Ok(())
  }

  async fn read_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0_u8; 1024];
    let mut n_read = 0;
    let mut hs_builder = HandshakeStateBuilder::new();

    let req = loop {
      n_read += self.socket.read(&mut buf[n_read..]).await?;
      let msg = format!("Accepted: {:?}, bytes read: {:?}", String::from_utf8_lossy(&buf[0..n_read]), n_read);
      debug!(msg);

      match &mut self.conn_state {
        ConnState::Handshake => {
          // should advancing hs return reply instead of keeping it in state?
          let res = hs_builder.advance(&buf);

          match res {
            Ok(_) =>  self.reply(&hs_builder).await?,
            Err(err) => return Err(Box::new(err)),
          };
        },
        ConnState::VerifyTarget(hs) => {
          let addr = hs.to_addr();

          // let verify_res = self.verify_target(&addr);
          let maybe_target_socket = TcpStream::connect(addr).await;
          match maybe_target_socket {
            Ok(target_socket) => {
              let mut reply: Vec<u8> = vec![];
              reply.push(hs.version.into());
              reply.push(0);
              reply.push(0);
              reply.push(hs.atyp.into());
              reply.append(&mut hs.addr.octets().to_vec());
              reply.append(&mut hs.port.to_be_bytes().to_vec());
              self.socket.write(&reply).await?;
              self.conn_state = ConnState::ConnEstablished(target_socket)
            }
            Err(err) => {
              // handle std:io::ErrorKind, reply with error and terminate tcp connection
              match err.kind() {
                _ => return Err(Box::new(err)),
              }
            }
          }
        },
        ConnState::ConnEstablished(target_socket) => {
          target_socket.write(&mut buf).await?;
        }
      }
    };
  }

  fn conn_reply(&mut self, hs_builder: &HandshakeStateBuilder) -> Vec<u8> {
    let mut reply = vec![];
    // reply.push()
    reply
  }

  async fn verify_target(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  async fn reply(&mut self, hs_builder: &HandshakeStateBuilder) -> Result<(), Box<dyn std::error::Error>> {
    let reply = hs_builder.reply();
    if !reply.is_empty() {
      self.socket.write(&reply).await?;
    }

    match hs_builder.state() {
      HandshakeState::Finished(hs) => self.conn_state = ConnState::VerifyTarget(hs),
      _ => {}
    }

    Ok(())
  }

}

pub async fn run(listener: TcpListener) {
  let mut server = Server { listener };

  tokio::select! {
    res = server.run() => {
      if let Err(err) = res {
        error!(cause = %err, "running server failed");
      }
    }
  }
}