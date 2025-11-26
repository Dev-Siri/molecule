use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DatabaseInputType {
    /// Gracefully shutdown the database.
    Stop,
    /// Do nothing, empty request.
    Noop,
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

pub fn parse_str_to_db_input_type(value: &str, source: InputSource) -> Result<DatabaseInputType> {
    if value == "stop" && source != InputSource::Tcp {
        return Ok(DatabaseInputType::Stop);
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let command = *match parts.get(0) {
        Some(cmd) => cmd,
        None => return Ok(DatabaseInputType::Noop),
    };

    bail!(
        "Invalid or unsupported input type for {}: {}",
        source.as_str(),
        value
    );
}
