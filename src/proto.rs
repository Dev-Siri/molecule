use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseInputType {
    /// Gracefully shutdown the database.
    Stop,
    /// Do nothing, empty request.
    Noop,
    /// List all collections in the database.
    CollectionsList,
    /// Get collection name from ID.
    Collection(String),
    /// Get records of a collection by the collection ID.
    CollectionRecords(String),
    /// Get record of a collection (referenced by collection_id) by the record ID.
    IdRecord(String, String),
    /// Create a collection with a provided collection_name.
    CreateCollection(String),
    /// Create a record in a specific collection (referenced by collection_id) with contents.
    CreateRecord(String, HashMap<String, Value>),
    /// Delete a collection referenced by it's collection ID.
    DeleteCollection(String),
    /// Delete a record in a specific collection (referenced by collection_id) with it's record ID.
    DeleteRecord(String, String),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DatabaseOutputMsg {
    Noop,
    Err(DatabaseOutputError),
    /// Collections(Stringified JSON of the collections)
    Collections(String),
    /// Collections(Name of collection)
    Collection(String),
    /// Records(Stringified JSON of the records)
    Records(String),
    /// CreatedCollection(ID of the collection)
    CreatedCollection(String),
    /// CreatedRecord(ID of the record)
    CreatedRecord(String),
    /// DeletedCollection(ID of the collection)
    DeletedCollection(String),
    /// DeletedRecord(ID of the record)
    DeletedRecord(String),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DatabaseOutputError {
    InvalidInput,
    CmdNotAvailable,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HandShakeInputMsg {
    Ok,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HandShakeOutputMsg {
    InitConn,
    Err(HandShakeOutputError),
    Ready,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HandShakeOutputError {
    InvalidHandShake,
    InvalidHandShakeMsg,
    MalformedAuthStr,
    MalformedRequest,
    IncorrectAuthInfo,
}

impl TryFrom<&str> for HandShakeInputMsg {
    type Error = HandShakeOutputError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OK" => Ok(Self::Ok),
            _ => Err(HandShakeOutputError::InvalidHandShakeMsg),
        }
    }
}

impl HandShakeOutputError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidHandShake => "ERR invalid_handshake\n",
            Self::InvalidHandShakeMsg => "ERR invalid_handshake_msg\n",
            Self::MalformedAuthStr => "ERR malformed_auth_str\n",
            Self::MalformedRequest => "ERR malformed_request\n",
            Self::IncorrectAuthInfo => "ERR incorrect_auth_info\n",
        }
    }
}

impl HandShakeOutputMsg {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InitConn => "INITCONN\n",
            Self::Err(err) => err.as_str(),
            Self::Ready => "READY\n",
        }
    }
}

impl<'a> Into<&'a [u8]> for HandShakeOutputMsg {
    fn into(self) -> &'a [u8] {
        self.as_str().as_bytes()
    }
}

impl DatabaseOutputError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidInput => "ERR invalid_input\n",
            Self::CmdNotAvailable => "ERR cmd_not_available",
        }
    }
}

impl DatabaseOutputMsg {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Noop => Vec::new(),
            Self::Err(err) => err.as_str().as_bytes().to_vec(),
            Self::Collections(collection) => collection.as_bytes().to_vec(),
            Self::Collection(collection_name) => collection_name.as_bytes().to_vec(),
            Self::Records(records) => records.as_bytes().to_vec(),
            Self::CreatedCollection(collection_id) => collection_id.as_bytes().to_vec(),
            Self::CreatedRecord(record_id) => record_id.as_bytes().to_vec(),
            Self::DeletedCollection(collection_id) => collection_id.as_bytes().to_vec(),
            Self::DeletedRecord(record_id) => record_id.as_bytes().to_vec(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum InputSource {
    Cli,
    Tcp,
}

impl InputSource {
    fn as_str(&self) -> &'static str {
        match &self {
            Self::Cli => "CLI",
            Self::Tcp => "TCP",
        }
    }
}

pub fn parse_str_to_db_input_type(value: String, source: InputSource) -> Result<DatabaseInputType> {
    if value == "STOP" && source != InputSource::Tcp {
        return Ok(DatabaseInputType::Stop);
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let command = *match parts.get(0) {
        Some(cmd) => cmd,
        None => return Ok(DatabaseInputType::Noop),
    };

    match command {
        "COLLECTIONS_LIST" => Ok(DatabaseInputType::CollectionsList),
        "COLLECTION" => {
            if let Some(collection_id) = parts.get(1) {
                return Ok(DatabaseInputType::Collection(collection_id.to_string()));
            }

            bail!("Input type COLLECTION is missing required argument for collection_id.");
        }
        "CLN_GET" => {
            if let Some(collection_id) = parts.get(1) {
                return Ok(DatabaseInputType::CollectionRecords(
                    collection_id.to_string(),
                ));
            }

            bail!("Input type CLN_GET is missing required argument for collection_id.");
        }
        "REC_GET" => {
            if let (Some(collection_id), Some(record_id)) = (parts.get(1), parts.get(2)) {
                return Ok(DatabaseInputType::IdRecord(
                    collection_id.to_string(),
                    record_id.to_string(),
                ));
            }

            bail!("Input type REC_GET is missing required argument for collection_id, record_id.");
        }
        "CLN_CREATE" => {
            if let Some(name) = parts.get(1) {
                return Ok(DatabaseInputType::CreateCollection(name.to_string()));
            }

            bail!("Input type CLN_CREATE is missing required argument for name.");
        }
        "REC_CREATE" => {
            if let (Some(collection_id), content) = (parts.get(1), parts[2..].join(" ")) {
                return Ok(DatabaseInputType::CreateRecord(
                    collection_id.to_string(),
                    serde_json::from_str(&content)?,
                ));
            }

            bail!(
                "Input type REC_CREATE is missing required argument for collection_id, contents."
            );
        }
        "CLN_DELETE" => {
            if let Some(collection_id) = parts.get(1) {
                return Ok(DatabaseInputType::DeleteCollection(
                    collection_id.to_string(),
                ));
            }

            bail!("Input type CLN_DELETE is missing required argument for collection_id.");
        }
        "REC_DELETE" => {
            if let (Some(collection_id), Some(record_id)) = (parts.get(1), parts.get(2)) {
                return Ok(DatabaseInputType::DeleteRecord(
                    collection_id.to_string(),
                    record_id.to_string(),
                ));
            }

            bail!(
                "Input type REC_DELETE is missing required argument for collection_id, record_id."
            );
        }
        _ => bail!(
            "Invalid or unsupported input type for {}: {}",
            source.as_str(),
            value
        ),
    }
}
