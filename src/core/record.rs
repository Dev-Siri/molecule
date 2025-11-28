use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use tokio::fs;
use uuid::Uuid;

use crate::{constants::MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, molecule::Molecule};

pub type Record = HashMap<String, Value>;

pub trait MoleculeCoreRecordsApi {
    async fn create_record(
        &self,
        collection_id: String,
        contents: HashMap<String, Value>,
    ) -> Result<String>;
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
        let records: Vec<HashMap<String, Value>> = self.get_records(collection_id).await?;

        Ok(records
            .into_iter()
            .filter(|r| {
                r.get("_id")
                    .is_some_and(|id| id.as_str() == Some(record_id.as_str()))
            })
            .next())
    }

    async fn create_record(
        &self,
        collection_id: String,
        contents: HashMap<String, Value>,
    ) -> Result<String> {
        let collection_path = format!(
            "{}/{}.json",
            MOLECULE_DEFAULT_DATA_COLLECTIONS_PATH, collection_id
        );
        let mut records: Vec<HashMap<String, Value>> = self.get_records(collection_id).await?;
        let mut record = HashMap::new();

        record.extend(contents);

        if !record.contains_key("_id") {
            let gen_record_id = Uuid::new_v4().to_string();
            record.insert("_id".into(), gen_record_id.clone().into());
        }

        let record_id = &record
            .get("_id".into())
            .unwrap_or_default()
            .as_str()
            .unwrap_or_default()
            .to_owned();

        records.push(record);

        fs::write(collection_path, serde_json::to_vec(&records)?).await?;

        Ok(record_id.to_string())
    }
}
