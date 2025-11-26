use anyhow::Result;
use tokio::fs::{self};

use crate::{constants::MOLECULE_AUTH_FILE_PATH, molecule::Molecule, proto::AuthInfo};

pub trait MoleculeAuthApi {
    async fn setup_user(&self, username: String, password: String) -> Result<()>;
    async fn is_valid_user(&self, username: &str, password: &str) -> Result<bool>;
}

impl MoleculeAuthApi for Molecule {
    async fn setup_user(&self, username: String, password: String) -> Result<()> {
        if fs::try_exists(MOLECULE_AUTH_FILE_PATH).await? {
            let existing_auth_info_bytes = fs::read(MOLECULE_AUTH_FILE_PATH).await?;
            let existing_auth_info: AuthInfo = serde_json::from_slice(&existing_auth_info_bytes)?;

            log::info!("Found existing user: {}", existing_auth_info.username);
            *self.active_user.write().await = Some(existing_auth_info);

            return Ok(());
        }

        log::info!("Setting up user with username: {}", username);
        let hashed_password = bcrypt::hash(password, 12)?;
        let auth_info = AuthInfo {
            username,
            password: hashed_password,
        };
        let auth_info_bytes = serde_json::to_vec(&auth_info)?;

        fs::write(MOLECULE_AUTH_FILE_PATH, auth_info_bytes).await?;
        log::info!("Auth store created for the current session.");

        *self.active_user.write().await = Some(auth_info);

        Ok(())
    }

    async fn is_valid_user(&self, username: &str, password: &str) -> Result<bool> {
        if let Some(auth_info) = &*self.active_user.read().await {
            if auth_info.username != username {
                return Ok(false);
            }

            return Ok(bcrypt::verify(password, &auth_info.password)?);
        }

        Ok(false)
    }
}
