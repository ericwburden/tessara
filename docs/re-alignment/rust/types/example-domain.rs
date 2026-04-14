use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetRevisionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub dataset_id: DatasetId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetRevision {
    pub dataset_revision_id: DatasetRevisionId,
    pub dataset_id: DatasetId,
    pub revision_number: i32,
    pub query_ir_json: serde_json::Value,
    pub contract_json: serde_json::Value,
    pub materialized_relation_name: String,
    pub materialization_status: String,
    pub created_at: DateTime<Utc>,
}

pub trait DatasetService {
    fn create_revision(&self, dataset_id: DatasetId) -> anyhow::Result<DatasetRevision>;
    fn materialize_revision(&self, dataset_revision_id: DatasetRevisionId) -> anyhow::Result<()>;
    fn generated_sql(&self, dataset_revision_id: DatasetRevisionId) -> anyhow::Result<String>;
}
