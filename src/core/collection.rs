use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
use uuid::Uuid;

use crate::{
    constants::{
        MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH, MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH,
    },
    molecule::Molecule,
};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Collection {
    pub collection_id: String,
    pub name: String,
}

pub trait MoleculeCoreCollectionApi {
    async fn create_collection(&self, name: String) -> Result<String>;
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

    async fn create_collection(&self, name: String) -> Result<String> {
        let mut meta_contents = self.list_collections().await?;
        let collection_id = Uuid::new_v4().to_string();

        meta_contents.push(Collection {
            collection_id: collection_id.clone(),
            name,
        });

        let collection_path = format!(
            "{}/{}.json",
            MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, collection_id
        );

        fs::write(collection_path, b"[]").await?;

        fs::write(
            MOLECULE_DEFAULT_DATA_COLLECTION_META_PATH,
            serde_json::to_vec(&meta_contents)?,
        )
        .await?;

        Ok(collection_id)
    }
}
