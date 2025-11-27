use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{constants::MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH, molecule::Molecule};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Collection {
    pub collection_id: String,
    pub name: String,
}

pub trait MoleculeCoreCollectionApi {
    async fn list_collections(&self) -> Result<Vec<Collection>>;
    async fn get_collection_name(&self, collection_id: String) -> Result<Option<String>>;
}

impl MoleculeCoreCollectionApi for Molecule {
    async fn list_collections(&self) -> Result<Vec<Collection>> {
        let collection_meta_content = fs::read(MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH).await?;
        let parsed_meta: Vec<Collection> = serde_json::from_slice(&collection_meta_content)?;

        Ok(parsed_meta)
    }

    async fn get_collection_name(&self, collection_id: String) -> Result<Option<String>> {
        let meta_contents = self.list_collections().await?;
        Ok(meta_contents
            .into_iter()
            .filter(|c| c.collection_id == collection_id)
            .next()
            .and_then(|c| Some(c.name)))
    }
}
