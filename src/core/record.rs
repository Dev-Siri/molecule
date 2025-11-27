use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use tokio::fs;

use crate::{constants::MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, molecule::Molecule};

pub type Record = HashMap<String, Value>;

pub trait MoleculeCoreRecordsApi {
    async fn get_record_by_id(
        &self,
        collection_id: String,
        record_id: String,
    ) -> Result<Option<Record>>;
    async fn get_records(&self, collection_id: String) -> Result<Vec<Record>>;
}

impl MoleculeCoreRecordsApi for Molecule {
    async fn get_records(&self, collection_id: String) -> Result<Vec<Record>> {
        let collection_path = format!(
            "{}/{}.json",
            MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, collection_id
        );
        let items = fs::read(collection_path).await?;
        let records: Vec<Record> = serde_json::from_slice(&items)?;

        Ok(records)
    }

    async fn get_record_by_id(
        &self,
        collection_id: String,
        record_id: String,
    ) -> Result<Option<Record>> {
        let records = self.get_records(collection_id).await?;

        Ok(records
            .into_iter()
            .filter(|r| {
                r.get("_id")
                    .is_some_and(|id| id.as_str() == Some(record_id.as_str()))
            })
            .next())
    }
}
