use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{debug, error};

use crate::handshake::{HandshakeStateBuilder, HandshakeState, SocksHandshake, reply::SocksReply, reply_field::ReplyField};

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
    self.read_handshake().await
  }

  fn clear_buffer(&self, buf: &mut [u8; 1024]) {
    buf.iter_mut().for_each(|m| *m = 0)
  }

  async fn read_handshake(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
              // swap the lines? make sure buffer is always cleared to make it ready for next incoming data
              // self.clear_buffer(&mut buf);
              self.handle_hs_advance(&hs_builder, reply).await?;
              self.clear_buffer(&mut buf);
            }
            Err(err) => return Err(Box::new(err)),
          };
        },
        ConnState::ConnEstablished(target_socket) => {
          debug!("Writing buffer: {:?}", buf);
          target_socket.write(&mut buf).await?;
        }
      }
    };

    Ok(())
  }

  async fn handle_hs_advance(&mut self, hs_builder: &HandshakeStateBuilder, reply: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
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

  async fn reply_to_client(&mut self, reply: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    match self.socket.write(&reply).await {
      Ok(_n_written) => Ok(()),
      Err(err) => Err(Box::new(err))
    }
  }

  async fn verify_target_conn(&mut self, hs: SocksHandshake) -> Result<(), Box<dyn std::error::Error>> {
    let addr = hs.to_addr();

    debug!("Connecting to a target host at {:?}", addr);
    let maybe_target_socket = TcpStream::connect(addr).await;
    match maybe_target_socket {
      Ok(target_socket) => {
        let reply = SocksReply::new(hs.version, ReplyField::Succeeded, hs.atyp, hs.addr, hs.port);
        debug!("Connection to target successfull, reply for client: {:?}", &reply.to_reply());
        self.socket.write(&reply.to_reply()).await?;
        self.conn_state = ConnState::ConnEstablished(target_socket);
        Ok(())
      }
      Err(err) => {
        // handle std:io::ErrorKind, reply with error to client and terminate tcp connection
        match err.kind() {
          _ => return Err(Box::new(err)),
        }
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