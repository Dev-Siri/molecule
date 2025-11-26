use tokio::sync::RwLock;

use crate::proto::AuthInfo;

#[derive(Debug)]
pub struct Molecule {
    pub addr: String,
    pub port: u32,
    pub data_path: String,
    pub active_user: RwLock<Option<AuthInfo>>,
}

impl Molecule {
    pub fn new(addr: String, port: u32, data_path: String) -> Self {
        Self {
            addr,
            port,
            data_path,
            active_user: RwLock::new(None),
        }
    }
}
