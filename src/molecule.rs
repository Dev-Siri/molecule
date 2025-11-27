use tokio::sync::RwLock;

use crate::proto::AuthInfo;

#[derive(Debug)]
pub struct Molecule {
    pub addr: String,
    pub port: u32,
    pub active_user: RwLock<Option<AuthInfo>>,
}

impl Molecule {
    pub fn new(addr: String, port: u32) -> Self {
        Self {
            addr,
            port,
            active_user: RwLock::new(None),
        }
    }
}
