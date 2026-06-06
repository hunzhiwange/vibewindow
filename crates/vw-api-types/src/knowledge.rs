//! Knowledge base API DTOs.
//!
//! This module only defines data crossing the gateway boundary. Storage,
//! indexing and retrieval behavior live in `vw-agent`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

fn default_indexing_mode() -> KnowledgeIndexingMode {
    KnowledgeIndexingMode::Economy
}

fn default_retrieval_mode() -> KnowledgeRetrievalMode {
    KnowledgeRetrievalMode::FullText
}

fn default_top_k() -> usize {
    10
}

fn default_enabled() -> bool {
    true
}

/// Knowledge indexing strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeIndexingMode {
    /// Local lexical indexing only.
    Economy,
    /// Reserved for embedding-backed indexing.
    HighQuality,
}

/// Knowledge retrieval strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRetrievalMode {
    FullText,
    Vector,
    Hybrid,
}

/// Dataset metadata returned by the gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDatasetDto {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_indexing_mode")]
    pub indexing_mode: KnowledgeIndexingMode,
    #[serde(default = "default_retrieval_mode")]
    pub retrieval_mode: KnowledgeRetrievalMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_model: Option<String>,
    pub document_count: u64,
    pub chunk_count: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

/// Create a dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDatasetCreateRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_indexing_mode")]
    pub indexing_mode: KnowledgeIndexingMode,
    #[serde(default = "default_retrieval_mode")]
    pub retrieval_mode: KnowledgeRetrievalMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_model: Option<String>,
}

/// Document metadata returned by the gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDocumentDto {
    pub id: String,
    pub dataset_id: String,
    pub name: String,
    pub metadata: Value,
    pub enabled: bool,
    pub chunk_count: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

/// Add one text document to a dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDocumentCreateRequest {
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// A retrieved knowledge chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeChunkDto {
    pub id: String,
    pub dataset_id: String,
    pub document_id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// Retrieval request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeRetrieveRequest {
    pub query: String,
    #[serde(default)]
    pub dataset_ids: Vec<String>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<Value>,
}

/// Retrieval response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeRetrieveResponse {
    #[serde(default)]
    pub chunks: Vec<KnowledgeChunkDto>,
}

/// Capability status for the current local knowledge runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeRuntimeStatus {
    pub full_text: bool,
    pub vector: bool,
    pub hybrid: bool,
    pub rerank: bool,
    #[serde(default)]
    pub notes: BTreeMap<String, String>,
}
