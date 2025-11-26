use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::auth::MoleculeAuthApi;
use crate::molecule::Molecule;
use crate::proto::HandShakeInputMsg;
use crate::proto::HandShakeOutputError;
use crate::proto::HandShakeOutputMsg;

trait MoleculeTcpHandle {
    async fn handshake(&self, client: &mut TcpStream) -> Result<()>;
    async fn handle_client(&self, client: &mut TcpStream) -> Result<()>;
}

trait MoleculeTcpExt {
    async fn write_err(
        &self,
        client: &mut TcpStream,
        handshake_output_error: HandShakeOutputError,
    ) -> Result<()>;
}

pub trait MoleculeTcpApi {
    async fn start_tcp(self: Arc<Self>) -> Result<()>;
}

impl MoleculeTcpApi for Molecule {
    async fn start_tcp(self: Arc<Self>) -> Result<()> {
        let bind_addr = format!("{}:{}", self.addr, self.port);
        let tcp_listener = TcpListener::bind(&bind_addr).await?;

        loop {
            let (mut stream, socket) = tcp_listener.accept().await?;
            log::info!("Client connected with IP: {}", socket.ip());

            let this = self.clone();
            tokio::spawn(async move {
                if let Err(e) = this.handle_client(&mut stream).await {
                    eprintln!("Client error: {}", e.to_string());
                }
            });
        }
    }
}

impl MoleculeTcpHandle for Molecule {
    async fn handshake(&self, client: &mut TcpStream) -> Result<()> {
        client
            .write_all(HandShakeOutputMsg::InitConn.into())
            .await?;

        let mut buf = vec![0u8; 1024];
        let size = client.read(&mut buf).await?;

        let incoming = String::from_utf8_lossy(&buf[..size]).trim().to_string();
        log::info!("Handshake: {}", incoming);

        let incoming_parts: Vec<&str> = incoming.split_whitespace().collect();
        let Some(&raw_message) = incoming_parts.get(0) else {
            self.write_err(client, HandShakeOutputError::MalformedRequest)
                .await?;
            return Ok(());
        };
        let message = match HandShakeInputMsg::try_from(raw_message) {
            Ok(parsed_msg) => parsed_msg,
            Err(err) => return Ok(self.write_err(client, err).await?),
        };

        if let Some(auth_str) = incoming_parts.get(1) {
            let Some((username, password)) = auth_str.split_once(":") else {
                return Ok(self
                    .write_err(client, HandShakeOutputError::MalformedAuthStr)
                    .await?);
            };

            if !self.is_valid_user(username, password).await? {
                return Ok(self
                    .write_err(client, HandShakeOutputError::IncorrectAuthInfo)
                    .await?);
            };

            log::info!("Client authed with username: {}", username)
        }

        if message != HandShakeInputMsg::Ok {
            return Ok(self
                .write_err(client, HandShakeOutputError::InvalidHandShake)
                .await?);
        }

        client.write_all(HandShakeOutputMsg::Ready.into()).await?;
        Ok(())
    }

    async fn handle_client(&self, client: &mut TcpStream) -> Result<()> {
        self.handshake(client).await?;
        let mut buf: Vec<u8> = vec![0u8; 1024];
        let buf_size = client.read(&mut buf).await?;

        client.write_all(&buf[..buf_size]).await?;

        Ok(())
    }
}

impl MoleculeTcpExt for Molecule {
    #[inline]
    async fn write_err(
        &self,
        client: &mut TcpStream,
        handshake_output_error: HandShakeOutputError,
    ) -> Result<()> {
        log::error!("Handshake error: {}", handshake_output_error.as_str());
        client
            .write_all(HandShakeOutputMsg::Err(handshake_output_error).into())
            .await?;
        Ok(())
    }
}
