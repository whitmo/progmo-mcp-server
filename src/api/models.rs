use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub entries: Vec<KnowledgeEntry>,
    pub total: usize,
}
