use std::io::ErrorKind;

use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{debug, error};
use crate::Result;

use crate::handshake::{HandshakeStateBuilder, HandshakeState, SocksHandshake, reply::{SocksReply}, reply_field::ReplyField};

#[derive(Debug)]
struct Server {
  listener: TcpListener
}

impl Server {
  async fn run(&mut self) -> Result<()> {
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

  async fn accept(&mut self) -> Result<TcpStream> {
    match self.listener.accept().await {
      Ok((socket, _)) => Ok(socket),
      Err(err) => Err(err.into()),
    }
  }
}

#[derive(Debug)]
enum ConnState {
  Handshake,
  ConnEstablished(TcpStream),
}

#[derive(Debug)]
struct ConnHandler {
  socket: TcpStream,
  conn_state: ConnState,
}

impl ConnHandler {
  pub fn new(socket: TcpStream) -> Self {
    Self { socket, conn_state: ConnState::Handshake }
  }

  fn clear_buffer(&self, buf: &mut [u8; 1024]) {
    buf.iter_mut().for_each(|m| *m = 0)
  }

  #[tracing::instrument(skip(self))]
  async fn run(&mut self) -> Result<()> {
    self.read_handshake().await
  }

  async fn read_handshake(&mut self) -> Result<()> {
    let mut buf = [0_u8; 1024];
    let mut hs_builder = HandshakeStateBuilder::new();

    loop {
      let n_read = self.socket.read(&mut buf).await?;
      if n_read == 0 {
        break;
      }
      let msg = format!("Accepted: {:?}, num of bytes read: {:?} in state: {:?}", &buf[..n_read].to_vec(), n_read, self.conn_state);
      debug!(msg);

      match &mut self.conn_state {
        ConnState::Handshake => {
          match hs_builder.advance(&buf) {
            Ok(reply) => {
              // If advancing ends up in error, buffer is not cleared.
              // That could become a problem if we decide to support partial handshake
              self.handle_hs_advance(&hs_builder, reply).await?;
              self.clear_buffer(&mut buf);
            }
            Err(err) => return Err(Box::new(err)),
          };
        },
        ConnState::ConnEstablished(target_socket) => {
          // improve debug - logging buffer of bytes is not very useful in real life
          // debug!("Writing buffer: {:?}", buf);
          debug!("Sending data to target host...");
          target_socket.write(&mut buf).await?;
        }
      }
    };

    Ok(())
  }

  async fn handle_hs_advance(&mut self, hs_builder: &HandshakeStateBuilder, reply: Vec<u8>) -> Result<()> {
    match hs_builder.state() {
      HandshakeState::Wait(_, _) => {
        debug!("writing hs reply for client: {:?}", reply);
        self.reply_to_client(reply).await
      },
      HandshakeState::Finished(hs) => {
        self.verify_target_conn(hs).await
      },
      HandshakeState::Init => {
        unreachable!("this should never ever happen")
      }
    }
  }

  async fn reply_to_client(&mut self, reply: Vec<u8>) -> Result<()> {
    match self.socket.write(&reply).await {
      Ok(_n_written) => Ok(()),
      Err(err) => Err(Box::new(err))
    }
  }

  async fn connection_reply(&mut self, hs: SocksHandshake, reply_status: ReplyField) -> Result<()> {
    let reply = SocksReply::new(hs.version, reply_status, hs.atyp, hs.addr, hs.port);
    debug!("Reply for client after target connection verification: {:?}", &reply.to_reply());
    self.socket.write(&reply.to_reply()).await?;
    Ok(())
  }

  async fn connection_reply_with_error(&mut self, err: std::io::Error, hs: SocksHandshake) -> Result<()> {
    match err.kind() {
      ErrorKind::ConnectionRefused => {
        self.connection_reply(hs, ReplyField::ConnectionRefused).await?;
        Err(Box::new(err))
      }
      _ => {
        self.connection_reply(hs, ReplyField::SocksServerFailure).await?;
        debug!("Err kind: {:?}", err);
        Err(Box::new(err))
      },
    }
  }

  async fn verify_target_conn(&mut self, hs: SocksHandshake) -> Result<()> {
    let addr = hs.to_addr();
    debug!("Connecting to a target host at {:?}", addr);

    match TcpStream::connect(addr).await {
      Ok(target_socket) => {
        self.connection_reply(hs, ReplyField::Succeeded).await?;
        self.conn_state = ConnState::ConnEstablished(target_socket);
        Ok(())
      }
      Err(err) => {
        self.connection_reply_with_error(err, hs).await
      }
    }
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