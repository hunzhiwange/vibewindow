//! Knowledge base gateway routes.

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use std::path::PathBuf;
use vw_api_types::knowledge::{
    KnowledgeDatasetCreateRequest, KnowledgeDatasetDto, KnowledgeDocumentCreateRequest,
    KnowledgeDocumentDto, KnowledgeRetrieveRequest, KnowledgeRetrieveResponse,
    KnowledgeRuntimeStatus,
};

use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::state::AppState;
use crate::knowledge::SqliteKnowledgeStore;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/knowledge/status", get(knowledge_status))
        .route("/knowledge/datasets", get(knowledge_datasets_list).post(knowledge_dataset_create))
        .route(
            "/knowledge/datasets/{dataset_id}",
            get(knowledge_dataset_get).delete(knowledge_dataset_delete),
        )
        .route(
            "/knowledge/datasets/{dataset_id}/documents",
            get(knowledge_documents_list).post(knowledge_document_create),
        )
        .route(
            "/knowledge/documents/{document_id}",
            axum::routing::delete(knowledge_document_delete),
        )
        .route("/knowledge/retrieve", post(knowledge_retrieve))
        .route("/knowledge/retrieval-test", post(knowledge_retrieve))
}

async fn knowledge_status() -> Result<Json<KnowledgeRuntimeStatus>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.status()))
}

async fn knowledge_datasets_list() -> Result<Json<Vec<KnowledgeDatasetDto>>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.list_datasets().await?))
}

async fn knowledge_dataset_get(
    Path(dataset_id): Path<String>,
) -> Result<Json<KnowledgeDatasetDto>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.get_dataset(dataset_id).await?))
}

async fn knowledge_dataset_create(
    Json(body): Json<KnowledgeDatasetCreateRequest>,
) -> Result<Json<KnowledgeDatasetDto>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.create_dataset(body).await?))
}

async fn knowledge_dataset_delete(
    Path(dataset_id): Path<String>,
) -> Result<Json<KnowledgeDatasetDto>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.delete_dataset(dataset_id).await?))
}

async fn knowledge_documents_list(
    Path(dataset_id): Path<String>,
) -> Result<Json<Vec<KnowledgeDocumentDto>>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.list_documents(dataset_id).await?))
}

async fn knowledge_document_create(
    Path(dataset_id): Path<String>,
    Json(body): Json<KnowledgeDocumentCreateRequest>,
) -> Result<Json<KnowledgeDocumentDto>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.create_document(dataset_id, body).await?))
}

async fn knowledge_document_delete(
    Path(document_id): Path<String>,
) -> Result<Json<KnowledgeDocumentDto>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.delete_document(document_id).await?))
}

async fn knowledge_retrieve(
    Json(body): Json<KnowledgeRetrieveRequest>,
) -> Result<Json<KnowledgeRetrieveResponse>, ApiError> {
    Ok(Json(knowledge_store_from_global_config().await.retrieve(body).await?))
}

pub(super) fn knowledge_store(state: &AppState) -> SqliteKnowledgeStore {
    let config = state.config.lock().clone();
    knowledge_store_from_config(config)
}

async fn knowledge_store_from_global_config() -> SqliteKnowledgeStore {
    knowledge_store_from_config(crate::config::get().await)
}

fn knowledge_store_from_config(config: crate::config::Config) -> SqliteKnowledgeStore {
    let (embedder, resolved_embedding) = crate::memory::create_embedding_provider_with_routes(
        &config.memory,
        &config.embedding_routes,
        config.api_key.as_deref(),
    );
    let embedding_model = (embedder.dimensions() > 0).then_some(resolved_embedding.model);
    SqliteKnowledgeStore::with_embedder(
        knowledge_db_path(),
        embedder,
        embedding_model,
        config.memory.vector_weight as f32,
        config.memory.keyword_weight as f32,
        config.memory.embedding_cache_size,
    )
}

pub(super) fn knowledge_db_path() -> PathBuf {
    crate::global::paths().data.join("knowledge").join("knowledge.sqlite")
}

#[cfg(test)]
#[path = "knowledge_tests.rs"]
mod knowledge_tests;
