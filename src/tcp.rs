use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::auth::MoleculeAuthApi;
use crate::core::collection::MoleculeCoreCollectionApi;
use crate::core::record::MoleculeCoreRecordsApi;
use crate::molecule::Molecule;
use crate::proto::DatabaseInputType;
use crate::proto::DatabaseOutputError;
use crate::proto::DatabaseOutputMsg;
use crate::proto::HandShakeInputMsg;
use crate::proto::HandShakeOutputError;
use crate::proto::HandShakeOutputMsg;
use crate::proto::InputSource;
use crate::proto::parse_str_to_db_input_type;

trait MoleculeTcpHandle {
    async fn handshake(&self, client: &mut TcpStream) -> Result<()>;
    async fn handle_client(&self, client: &mut TcpStream) -> Result<()>;
}

trait MoleculeTcpExt {
    async fn write_db_err(
        &self,
        client: &mut TcpStream,
        database_output_error: DatabaseOutputError,
    ) -> Result<()>;
    async fn write_handshake_err(
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
            self.write_handshake_err(client, HandShakeOutputError::MalformedRequest)
                .await?;
            return Ok(());
        };
        let message = match HandShakeInputMsg::try_from(raw_message) {
            Ok(parsed_msg) => parsed_msg,
            Err(err) => return Ok(self.write_handshake_err(client, err).await?),
        };

        if let Some(auth_str) = incoming_parts.get(1) {
            let Some((username, password)) = auth_str.split_once(":") else {
                return Ok(self
                    .write_handshake_err(client, HandShakeOutputError::MalformedAuthStr)
                    .await?);
            };

            if !self.is_valid_user(username, password).await? {
                return Ok(self
                    .write_handshake_err(client, HandShakeOutputError::IncorrectAuthInfo)
                    .await?);
            };

            log::info!("Client authed with username: {}", username)
        }

        if message != HandShakeInputMsg::Ok {
            return Ok(self
                .write_handshake_err(client, HandShakeOutputError::InvalidHandShake)
                .await?);
        }

        client.write_all(HandShakeOutputMsg::Ready.into()).await?;
        Ok(())
    }

    async fn handle_client(&self, client: &mut TcpStream) -> Result<()> {
        self.handshake(client).await?;
        let mut buf: Vec<u8> = vec![0u8; 1024];
        let size = client.read(&mut buf).await?;

        let incoming_db_cmd = String::from_utf8_lossy(&buf[..size]).trim().to_string();
        log::info!("Database command: {}", incoming_db_cmd);

        let input = match parse_str_to_db_input_type(incoming_db_cmd, InputSource::Tcp) {
            Ok(parsed) => parsed,
            Err(_) => {
                return Ok(self
                    .write_db_err(client, DatabaseOutputError::InvalidInput)
                    .await?);
            }
        };

        let response = match input {
            DatabaseInputType::CollectionsList => {
                let collections = self.list_collections().await?;
                let json_str = serde_json::to_string(&collections)?;

                DatabaseOutputMsg::Collections(json_str)
            }
            DatabaseInputType::Collection(collection_id) => {
                let collection = self.get_collection_name(collection_id).await?;

                DatabaseOutputMsg::Collection(collection.unwrap_or("null".into()))
            }
            DatabaseInputType::CollectionRecords(collection_id) => {
                let records = self.get_records(collection_id).await?;
                let json_str = serde_json::to_string(&records)?;

                DatabaseOutputMsg::Records(json_str)
            }
            DatabaseInputType::IdRecord(collection_id, record_id) => {
                let record = self.get_record_by_id(collection_id, record_id).await?;
                let json_str = serde_json::to_string(&record)?;

                DatabaseOutputMsg::Records(json_str)
            }
            DatabaseInputType::CreateCollection(name) => {
                let collection_id = self.create_collection(name).await?;
                DatabaseOutputMsg::CreatedCollection(collection_id)
            }
            DatabaseInputType::CreateRecord(collection_id, contents) => {
                let collection_id = self.create_record(collection_id, contents).await?;
                DatabaseOutputMsg::CreatedRecord(collection_id)
            }
            DatabaseInputType::Noop => DatabaseOutputMsg::Noop,
            DatabaseInputType::Stop => DatabaseOutputMsg::Err(DatabaseOutputError::CmdNotAvailable),
        };

        client.write_all(&response.to_bytes()).await?;

        Ok(())
    }
}

impl MoleculeTcpExt for Molecule {
    #[inline]
    async fn write_handshake_err(
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

    #[inline]
    async fn write_db_err(
        &self,
        client: &mut TcpStream,
        database_output_error: DatabaseOutputError,
    ) -> Result<()> {
        log::error!("Database error: {}", database_output_error.as_str());
        client
            .write_all(&DatabaseOutputMsg::Err(database_output_error).to_bytes())
            .await?;
        Ok(())
    }
}
