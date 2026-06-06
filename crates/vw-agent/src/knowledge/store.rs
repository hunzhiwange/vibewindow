//! SQLite-backed knowledge storage and retrieval.

use super::chunker::chunk_text;
use crate::app::agent::gateway::ApiError;
use crate::memory::embeddings::{EmbeddingProvider, NoopEmbedding};
use crate::memory::vector;
use crate::workflow::{
    WorkflowKnowledgeChunk, WorkflowKnowledgeProvider, WorkflowKnowledgeRequest,
};
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use vw_api_types::knowledge::{
    KnowledgeChunkDto, KnowledgeDatasetCreateRequest, KnowledgeDatasetDto,
    KnowledgeDocumentCreateRequest, KnowledgeDocumentDto, KnowledgeIndexingMode,
    KnowledgeRetrievalMode, KnowledgeRetrieveRequest, KnowledgeRetrieveResponse,
    KnowledgeRuntimeStatus,
};

const KNOWLEDGE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS knowledge_datasets (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    indexing_mode TEXT NOT NULL,
    retrieval_mode TEXT NOT NULL,
    embedding_model TEXT,
    rerank_model TEXT,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS knowledge_documents (
    id TEXT PRIMARY KEY NOT NULL,
    dataset_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(dataset_id) REFERENCES knowledge_datasets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS knowledge_chunks (
    id TEXT PRIMARY KEY NOT NULL,
    dataset_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    embedding BLOB,
    enabled INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY(dataset_id) REFERENCES knowledge_datasets(id) ON DELETE CASCADE,
    FOREIGN KEY(document_id) REFERENCES knowledge_documents(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_chunks_fts USING fts5(
    chunk_id UNINDEXED,
    dataset_id UNINDEXED,
    document_id UNINDEXED,
    title,
    content,
    tokenize = 'unicode61'
);

CREATE INDEX IF NOT EXISTS knowledge_documents_dataset_idx
ON knowledge_documents(dataset_id);

CREATE INDEX IF NOT EXISTS knowledge_chunks_dataset_idx
ON knowledge_chunks(dataset_id);

CREATE INDEX IF NOT EXISTS knowledge_chunks_document_idx
ON knowledge_chunks(document_id);

CREATE TABLE IF NOT EXISTS knowledge_embedding_cache (
    content_hash TEXT PRIMARY KEY NOT NULL,
    embedding BLOB NOT NULL,
    created_at_ms INTEGER NOT NULL,
    accessed_at_ms INTEGER NOT NULL
);
"#;

const MAX_TOP_K: usize = 50;
const MAX_CANDIDATES: usize = 250;

/// Local SQLite knowledge store.
#[derive(Clone)]
pub struct SqliteKnowledgeStore {
    db_path: PathBuf,
    embedder: Arc<dyn EmbeddingProvider>,
    embedding_model: Option<String>,
    vector_weight: f32,
    keyword_weight: f32,
    embedding_cache_size: usize,
}

#[derive(Debug, Clone)]
struct PreparedKnowledgeChunk {
    ordinal: usize,
    content: String,
    embedding: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct RetrievalScope {
    keyword_dataset_ids: Vec<String>,
    vector_dataset_ids: Vec<String>,
}

impl SqliteKnowledgeStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self::with_embedder(db_path, Arc::new(NoopEmbedding), None, 0.7, 0.3, 10_000)
    }

    pub fn with_embedder(
        db_path: PathBuf,
        embedder: Arc<dyn EmbeddingProvider>,
        embedding_model: Option<String>,
        vector_weight: f32,
        keyword_weight: f32,
        embedding_cache_size: usize,
    ) -> Self {
        Self {
            db_path,
            embedder,
            embedding_model,
            vector_weight,
            keyword_weight,
            embedding_cache_size,
        }
    }

    pub fn status(&self) -> KnowledgeRuntimeStatus {
        let mut notes = BTreeMap::new();
        if self.embedder.dimensions() == 0 {
            notes.insert(
                "vector".to_string(),
                "embedding provider is configured as none".to_string(),
            );
        }
        notes.insert("rerank".to_string(), "rerank provider is not configured".to_string());
        let vector_enabled = self.embedder.dimensions() > 0;
        KnowledgeRuntimeStatus {
            full_text: true,
            vector: vector_enabled,
            hybrid: vector_enabled,
            rerank: false,
            notes,
        }
    }

    pub async fn list_datasets(&self) -> Result<Vec<KnowledgeDatasetDto>, ApiError> {
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, list_datasets_blocking).await
    }

    pub async fn get_dataset(&self, dataset_id: String) -> Result<KnowledgeDatasetDto, ApiError> {
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| get_dataset_blocking(path, &dataset_id)).await
    }

    pub async fn create_dataset(
        &self,
        body: KnowledgeDatasetCreateRequest,
    ) -> Result<KnowledgeDatasetDto, ApiError> {
        validate_dataset_request(
            &body,
            self.embedder.dimensions() > 0,
            self.embedding_model.as_deref(),
        )?;
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| create_dataset_blocking(path, body)).await
    }

    pub async fn delete_dataset(
        &self,
        dataset_id: String,
    ) -> Result<KnowledgeDatasetDto, ApiError> {
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| delete_dataset_blocking(path, &dataset_id)).await
    }

    pub async fn list_documents(
        &self,
        dataset_id: String,
    ) -> Result<Vec<KnowledgeDocumentDto>, ApiError> {
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| list_documents_blocking(path, &dataset_id)).await
    }

    pub async fn create_document(
        &self,
        dataset_id: String,
        body: KnowledgeDocumentCreateRequest,
    ) -> Result<KnowledgeDocumentDto, ApiError> {
        validate_document_request(&body)?;
        let dataset = self.get_dataset(dataset_id.clone()).await?;
        let chunks = chunk_text(&body.content);
        if chunks.is_empty() {
            return Err(ApiError::bad_request("document content produced no chunks"));
        }
        let needs_embedding = dataset.indexing_mode == KnowledgeIndexingMode::HighQuality
            || dataset.retrieval_mode != KnowledgeRetrievalMode::FullText;
        if needs_embedding && self.embedder.dimensions() == 0 {
            return Err(ApiError::not_implemented(
                "knowledge dataset requires an embedding provider",
            ));
        }

        let mut prepared_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let embedding = if needs_embedding {
                self.get_or_compute_embedding(&chunk.content)
                    .await?
                    .map(|embedding| vector::vec_to_bytes(&embedding))
            } else {
                None
            };
            prepared_chunks.push(PreparedKnowledgeChunk {
                ordinal: chunk.ordinal,
                content: chunk.content,
                embedding,
            });
        }

        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| {
            create_document_blocking(path, &dataset_id, body, prepared_chunks)
        })
        .await
    }

    pub async fn delete_document(
        &self,
        document_id: String,
    ) -> Result<KnowledgeDocumentDto, ApiError> {
        let db_path = self.db_path.clone();
        spawn_store_task(db_path, move |path| delete_document_blocking(path, &document_id)).await
    }

    pub async fn retrieve(
        &self,
        request: KnowledgeRetrieveRequest,
    ) -> Result<KnowledgeRetrieveResponse, ApiError> {
        validate_retrieve_request(&request)?;
        let scope = {
            let db_path = self.db_path.clone();
            let dataset_ids = request.dataset_ids.clone();
            spawn_store_task(db_path, move |path| {
                resolve_retrieval_scope_blocking(path, &dataset_ids)
            })
            .await?
        };
        let needs_vector = !scope.vector_dataset_ids.is_empty();
        if needs_vector && self.embedder.dimensions() == 0 {
            return Err(ApiError::not_implemented(
                "knowledge retrieval requires an embedding provider for vector or hybrid datasets",
            ));
        }
        let query_embedding =
            if needs_vector { self.get_or_compute_embedding(&request.query).await? } else { None };
        let db_path = self.db_path.clone();
        let vector_weight = self.vector_weight;
        let keyword_weight = self.keyword_weight;
        spawn_store_task(db_path, move |path| {
            retrieve_blocking(path, request, scope, query_embedding, vector_weight, keyword_weight)
        })
        .await
    }

    async fn get_or_compute_embedding(&self, text: &str) -> Result<Option<Vec<f32>>, ApiError> {
        if self.embedder.dimensions() == 0 {
            return Ok(None);
        }

        let hash = content_hash(text);
        let cached = {
            let db_path = self.db_path.clone();
            let hash = hash.clone();
            spawn_store_task(db_path, move |path| read_embedding_cache_blocking(path, &hash))
                .await?
        };
        if cached.is_some() {
            return Ok(cached);
        }

        let embedding = self.embedder.embed_one(text).await.map_err(|error| {
            ApiError::bad_request(format!("knowledge embedding failed: {error}"))
        })?;
        if embedding.is_empty() {
            return Ok(None);
        }

        let db_path = self.db_path.clone();
        let bytes = vector::vec_to_bytes(&embedding);
        let cache_size = self.embedding_cache_size;
        spawn_store_task(db_path, move |path| {
            write_embedding_cache_blocking(path, &hash, bytes, cache_size)
        })
        .await?;
        Ok(Some(embedding))
    }
}

#[async_trait::async_trait]
impl WorkflowKnowledgeProvider for SqliteKnowledgeStore {
    async fn retrieve(
        &self,
        request: WorkflowKnowledgeRequest,
    ) -> Result<Vec<WorkflowKnowledgeChunk>, String> {
        let response = SqliteKnowledgeStore::retrieve(
            self,
            KnowledgeRetrieveRequest {
                query: request.query,
                dataset_ids: request.dataset_ids,
                top_k: request.top_k,
                score_threshold: request.score_threshold,
                metadata_filter: request.metadata_filter,
            },
        )
        .await
        .map_err(|error| error.to_string())?;

        Ok(response
            .chunks
            .into_iter()
            .map(|chunk| WorkflowKnowledgeChunk {
                content: chunk.content,
                title: chunk.title,
                metadata: chunk.metadata,
                score: chunk.score,
            })
            .collect())
    }
}

async fn spawn_store_task<T: Send + 'static>(
    db_path: PathBuf,
    task: impl FnOnce(PathBuf) -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(move || task(db_path))
        .await
        .map_err(|error| ApiError::internal(format!("knowledge store task failed: {error}")))?
}

fn list_datasets_blocking(db_path: PathBuf) -> Result<Vec<KnowledgeDatasetDto>, ApiError> {
    let conn = open_db(&db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, indexing_mode, retrieval_mode, embedding_model, \
             rerank_model, created_at_ms, updated_at_ms FROM knowledge_datasets \
             ORDER BY updated_at_ms DESC, name ASC",
        )
        .map_err(sql_error)?;
    let rows = stmt.query_map([], |row| dataset_from_row(&conn, row)).map_err(sql_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

fn get_dataset_blocking(
    db_path: PathBuf,
    dataset_id: &str,
) -> Result<KnowledgeDatasetDto, ApiError> {
    validate_id(dataset_id, "dataset_id")?;
    let conn = open_db(&db_path)?;
    query_dataset(&conn, dataset_id)?
        .ok_or_else(|| ApiError::not_found(format!("knowledge dataset not found: {dataset_id}")))
}

fn create_dataset_blocking(
    db_path: PathBuf,
    body: KnowledgeDatasetCreateRequest,
) -> Result<KnowledgeDatasetDto, ApiError> {
    let conn = open_db(&db_path)?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO knowledge_datasets \
         (id, name, description, indexing_mode, retrieval_mode, embedding_model, rerank_model, \
          created_at_ms, updated_at_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            body.name.trim(),
            body.description.trim(),
            indexing_mode_as_str(&body.indexing_mode),
            retrieval_mode_as_str(&body.retrieval_mode),
            body.embedding_model.as_deref(),
            body.rerank_model.as_deref(),
            now,
            now,
        ],
    )
    .map_err(sql_error)?;

    query_dataset(&conn, &id)?.ok_or_else(|| ApiError::internal("knowledge dataset was not saved"))
}

fn delete_dataset_blocking(
    db_path: PathBuf,
    dataset_id: &str,
) -> Result<KnowledgeDatasetDto, ApiError> {
    validate_id(dataset_id, "dataset_id")?;
    let conn = open_db(&db_path)?;
    let dataset = query_dataset(&conn, dataset_id)?
        .ok_or_else(|| ApiError::not_found(format!("knowledge dataset not found: {dataset_id}")))?;
    conn.execute("DELETE FROM knowledge_chunks_fts WHERE dataset_id = ?1", params![dataset_id])
        .map_err(sql_error)?;
    conn.execute("DELETE FROM knowledge_chunks WHERE dataset_id = ?1", params![dataset_id])
        .map_err(sql_error)?;
    conn.execute("DELETE FROM knowledge_documents WHERE dataset_id = ?1", params![dataset_id])
        .map_err(sql_error)?;
    conn.execute("DELETE FROM knowledge_datasets WHERE id = ?1", params![dataset_id])
        .map_err(sql_error)?;
    Ok(dataset)
}

fn list_documents_blocking(
    db_path: PathBuf,
    dataset_id: &str,
) -> Result<Vec<KnowledgeDocumentDto>, ApiError> {
    validate_id(dataset_id, "dataset_id")?;
    let conn = open_db(&db_path)?;
    ensure_dataset_exists(&conn, dataset_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, dataset_id, name, metadata_json, enabled, created_at_ms, updated_at_ms \
             FROM knowledge_documents WHERE dataset_id = ?1 ORDER BY updated_at_ms DESC, name ASC",
        )
        .map_err(sql_error)?;
    let rows = stmt
        .query_map(params![dataset_id], |row| document_from_row(&conn, row))
        .map_err(sql_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

fn create_document_blocking(
    db_path: PathBuf,
    dataset_id: &str,
    body: KnowledgeDocumentCreateRequest,
    chunks: Vec<PreparedKnowledgeChunk>,
) -> Result<KnowledgeDocumentDto, ApiError> {
    validate_id(dataset_id, "dataset_id")?;
    let conn = open_db(&db_path)?;
    ensure_dataset_exists(&conn, dataset_id)?;

    let document_id = Uuid::new_v4().to_string();
    let now = now_ms();
    let metadata_json = serde_json::to_string(&metadata_or_object(body.metadata.clone()))
        .map_err(|error| ApiError::bad_request(format!("document metadata is invalid: {error}")))?;
    conn.execute(
        "INSERT INTO knowledge_documents \
         (id, dataset_id, name, content, metadata_json, enabled, created_at_ms, updated_at_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            document_id,
            dataset_id,
            body.name.trim(),
            body.content,
            metadata_json,
            bool_to_i64(body.enabled),
            now,
            now,
        ],
    )
    .map_err(sql_error)?;

    for chunk in chunks {
        let chunk_id = Uuid::new_v4().to_string();
        let chunk_metadata = chunk_metadata(&body.metadata, body.name.trim(), chunk.ordinal);
        let chunk_metadata_json = serde_json::to_string(&chunk_metadata).map_err(|error| {
            ApiError::bad_request(format!("document chunk metadata is invalid: {error}"))
        })?;
        let content = chunk.content;
        let embedding = chunk.embedding;
        conn.execute(
            "INSERT INTO knowledge_chunks \
             (id, dataset_id, document_id, ordinal, title, content, metadata_json, embedding, enabled, \
              created_at_ms, updated_at_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                chunk_id,
                dataset_id,
                document_id,
                i64::try_from(chunk.ordinal).unwrap_or(i64::MAX),
                body.name.trim(),
                content.as_str(),
                chunk_metadata_json,
                embedding,
                bool_to_i64(body.enabled),
                now,
                now,
            ],
        )
        .map_err(sql_error)?;
        conn.execute(
            "INSERT INTO knowledge_chunks_fts \
             (chunk_id, dataset_id, document_id, title, content) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![chunk_id, dataset_id, document_id, body.name.trim(), content.as_str()],
        )
        .map_err(sql_error)?;
    }

    query_document(&conn, &document_id)?
        .ok_or_else(|| ApiError::internal("knowledge document was not saved"))
}

fn delete_document_blocking(
    db_path: PathBuf,
    document_id: &str,
) -> Result<KnowledgeDocumentDto, ApiError> {
    validate_id(document_id, "document_id")?;
    let conn = open_db(&db_path)?;
    let document = query_document(&conn, document_id)?.ok_or_else(|| {
        ApiError::not_found(format!("knowledge document not found: {document_id}"))
    })?;
    conn.execute("DELETE FROM knowledge_chunks_fts WHERE document_id = ?1", params![document_id])
        .map_err(sql_error)?;
    conn.execute("DELETE FROM knowledge_chunks WHERE document_id = ?1", params![document_id])
        .map_err(sql_error)?;
    conn.execute("DELETE FROM knowledge_documents WHERE id = ?1", params![document_id])
        .map_err(sql_error)?;
    Ok(document)
}

fn retrieve_blocking(
    db_path: PathBuf,
    request: KnowledgeRetrieveRequest,
    scope: RetrievalScope,
    query_embedding: Option<Vec<f32>>,
    vector_weight: f32,
    keyword_weight: f32,
) -> Result<KnowledgeRetrieveResponse, ApiError> {
    let conn = open_db(&db_path)?;
    let top_k = request.top_k.clamp(1, MAX_TOP_K);
    let candidate_limit = (top_k * 5).clamp(top_k, MAX_CANDIDATES);
    let mut keyword_results =
        retrieve_fts_candidates(&conn, &request, &scope.keyword_dataset_ids, candidate_limit)?;
    if keyword_results.is_empty() {
        keyword_results =
            retrieve_like_candidates(&conn, &request, &scope.keyword_dataset_ids, candidate_limit)?;
    }
    let vector_results = query_embedding
        .as_ref()
        .map(|embedding| {
            retrieve_vector_candidates(&conn, &scope.vector_dataset_ids, embedding, candidate_limit)
        })
        .transpose()?
        .unwrap_or_default();

    let merged = merge_retrieval_scores(
        &vector_results,
        &keyword_results,
        vector_weight,
        keyword_weight,
        candidate_limit,
    );
    let mut chunks = Vec::new();
    for scored in merged {
        let Some(chunk) = query_chunk(&conn, &scored.id, Some(f64::from(scored.final_score)))?
        else {
            continue;
        };
        if !metadata_matches(&chunk.metadata, &request.metadata_filter) {
            continue;
        }
        if request
            .score_threshold
            .is_some_and(|threshold| chunk.score.unwrap_or_default() < threshold)
        {
            continue;
        }
        chunks.push(chunk);
        if chunks.len() >= top_k {
            break;
        }
    }

    Ok(KnowledgeRetrieveResponse { chunks })
}

fn retrieve_fts_candidates(
    conn: &Connection,
    request: &KnowledgeRetrieveRequest,
    dataset_ids: &[String],
    limit: usize,
) -> Result<Vec<(String, f32)>, ApiError> {
    if dataset_ids.is_empty() {
        return Ok(Vec::new());
    }
    let fts_query = fts_query_from_text(&request.query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let dataset_filter = dataset_filter_sql("c", dataset_ids.len());
    let sql = format!(
        "SELECT c.id, c.dataset_id, c.document_id, c.title, c.content, c.metadata_json, \
         bm25(knowledge_chunks_fts) AS rank \
         FROM knowledge_chunks_fts \
         JOIN knowledge_chunks c ON c.id = knowledge_chunks_fts.chunk_id \
         JOIN knowledge_documents d ON d.id = c.document_id \
         WHERE knowledge_chunks_fts MATCH ? AND c.enabled = 1 AND d.enabled = 1{dataset_filter} \
         ORDER BY rank ASC LIMIT ?",
    );
    let mut params_vec = Vec::<&dyn rusqlite::ToSql>::new();
    params_vec.push(&fts_query);
    for dataset_id in dataset_ids {
        params_vec.push(dataset_id);
    }
    params_vec.push(&limit_i64);

    let mut stmt = conn.prepare(&sql).map_err(sql_error)?;
    match stmt.query_map(params_from_iter(params_vec), |row| {
        let id: String = row.get(0)?;
        let rank: f64 = row.get(6)?;
        Ok((id, fts_score(rank)))
    }) {
        Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(sql_error),
        Err(error) => {
            tracing::debug!(target: "vw_agent::knowledge", %error, "FTS query failed");
            Ok(Vec::new())
        }
    }
}

fn retrieve_like_candidates(
    conn: &Connection,
    request: &KnowledgeRetrieveRequest,
    dataset_ids: &[String],
    limit: usize,
) -> Result<Vec<(String, f32)>, ApiError> {
    if dataset_ids.is_empty() {
        return Ok(Vec::new());
    }
    let like_pattern = like_pattern(&request.query);
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let dataset_filter = dataset_filter_sql("c", dataset_ids.len());
    let sql = format!(
        "SELECT c.id, c.dataset_id, c.document_id, c.title, c.content, c.metadata_json \
         FROM knowledge_chunks c \
         JOIN knowledge_documents d ON d.id = c.document_id \
         WHERE c.enabled = 1 AND d.enabled = 1{dataset_filter} \
         AND (c.title LIKE ? ESCAPE '\\' OR c.content LIKE ? ESCAPE '\\') \
         ORDER BY c.updated_at_ms DESC LIMIT ?",
    );
    let mut params_vec = Vec::<&dyn rusqlite::ToSql>::new();
    for dataset_id in dataset_ids {
        params_vec.push(dataset_id);
    }
    params_vec.push(&like_pattern);
    params_vec.push(&like_pattern);
    params_vec.push(&limit_i64);

    let mut stmt = conn.prepare(&sql).map_err(sql_error)?;
    let rows = stmt
        .query_map(params_from_iter(params_vec), |row| {
            let chunk = chunk_from_row(row, None)?;
            let score = like_score(&chunk, &request.query);
            Ok((chunk.id, score))
        })
        .map_err(sql_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

fn retrieve_vector_candidates(
    conn: &Connection,
    dataset_ids: &[String],
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<(String, f32)>, ApiError> {
    if dataset_ids.is_empty() {
        return Ok(Vec::new());
    }
    let dataset_filter = dataset_filter_sql("c", dataset_ids.len());
    let sql = format!(
        "SELECT c.id, c.embedding \
         FROM knowledge_chunks c \
         JOIN knowledge_documents d ON d.id = c.document_id \
         WHERE c.embedding IS NOT NULL AND c.enabled = 1 AND d.enabled = 1{dataset_filter}",
    );
    let mut params_vec = Vec::<&dyn rusqlite::ToSql>::new();
    for dataset_id in dataset_ids {
        params_vec.push(dataset_id);
    }
    let mut stmt = conn.prepare(&sql).map_err(sql_error)?;
    let rows = stmt
        .query_map(params_from_iter(params_vec), |row| {
            let id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((id, blob))
        })
        .map_err(sql_error)?;

    let mut scored = Vec::new();
    for row in rows {
        let (id, blob) = row.map_err(sql_error)?;
        let embedding = vector::bytes_to_vec(&blob);
        let score = vector::cosine_similarity(query_embedding, &embedding);
        if score > 0.0 {
            scored.push((id, score));
        }
    }
    scored.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

fn merge_retrieval_scores(
    vector_results: &[(String, f32)],
    keyword_results: &[(String, f32)],
    vector_weight: f32,
    keyword_weight: f32,
    limit: usize,
) -> Vec<vector::ScoredResult> {
    if !vector_results.is_empty() && !keyword_results.is_empty() {
        return vector::hybrid_merge(
            vector_results,
            keyword_results,
            vector_weight,
            keyword_weight,
            limit,
        );
    }
    let source = if vector_results.is_empty() { keyword_results } else { vector_results };
    let mut results = source
        .iter()
        .map(|(id, score)| vector::ScoredResult {
            id: id.clone(),
            vector_score: (!vector_results.is_empty()).then_some(*score),
            keyword_score: vector_results.is_empty().then_some(*score),
            final_score: *score,
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right.final_score.partial_cmp(&left.final_score).unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

fn query_dataset(
    conn: &Connection,
    dataset_id: &str,
) -> Result<Option<KnowledgeDatasetDto>, ApiError> {
    conn.query_row(
        "SELECT id, name, description, indexing_mode, retrieval_mode, embedding_model, \
         rerank_model, created_at_ms, updated_at_ms FROM knowledge_datasets WHERE id = ?1",
        params![dataset_id],
        |row| dataset_from_row(conn, row),
    )
    .optional()
    .map_err(sql_error)
}

fn query_document(
    conn: &Connection,
    document_id: &str,
) -> Result<Option<KnowledgeDocumentDto>, ApiError> {
    conn.query_row(
        "SELECT id, dataset_id, name, metadata_json, enabled, created_at_ms, updated_at_ms \
         FROM knowledge_documents WHERE id = ?1",
        params![document_id],
        |row| document_from_row(conn, row),
    )
    .optional()
    .map_err(sql_error)
}

fn query_chunk(
    conn: &Connection,
    chunk_id: &str,
    score: Option<f64>,
) -> Result<Option<KnowledgeChunkDto>, ApiError> {
    conn.query_row(
        "SELECT id, dataset_id, document_id, title, content, metadata_json \
         FROM knowledge_chunks WHERE id = ?1 AND enabled = 1",
        params![chunk_id],
        |row| chunk_from_row(row, score),
    )
    .optional()
    .map_err(sql_error)
}

fn dataset_from_row(
    conn: &Connection,
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<KnowledgeDatasetDto> {
    let id: String = row.get(0)?;
    Ok(KnowledgeDatasetDto {
        id: id.clone(),
        name: row.get(1)?,
        description: row.get(2)?,
        indexing_mode: parse_indexing_mode(row.get::<_, String>(3)?.as_str()),
        retrieval_mode: parse_retrieval_mode(row.get::<_, String>(4)?.as_str()),
        embedding_model: row.get(5)?,
        rerank_model: row.get(6)?,
        document_count: count_by_dataset(conn, "knowledge_documents", &id)?,
        chunk_count: count_by_dataset(conn, "knowledge_chunks", &id)?,
        created_at_ms: read_u64(row, 7)?,
        updated_at_ms: read_u64(row, 8)?,
    })
}

fn document_from_row(
    conn: &Connection,
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<KnowledgeDocumentDto> {
    let id: String = row.get(0)?;
    let metadata_json: String = row.get(3)?;
    Ok(KnowledgeDocumentDto {
        id: id.clone(),
        dataset_id: row.get(1)?,
        name: row.get(2)?,
        metadata: serde_json::from_str(&metadata_json).unwrap_or(Value::Null),
        enabled: row.get::<_, i64>(4)? != 0,
        chunk_count: count_by_document(conn, &id)?,
        created_at_ms: read_u64(row, 5)?,
        updated_at_ms: read_u64(row, 6)?,
    })
}

fn chunk_from_row(
    row: &rusqlite::Row<'_>,
    score: Option<f64>,
) -> rusqlite::Result<KnowledgeChunkDto> {
    let metadata_json: String = row.get(5)?;
    Ok(KnowledgeChunkDto {
        id: row.get(0)?,
        dataset_id: row.get(1)?,
        document_id: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        metadata: serde_json::from_str(&metadata_json).unwrap_or(Value::Null),
        score,
    })
}

fn count_by_dataset(conn: &Connection, table: &str, dataset_id: &str) -> rusqlite::Result<u64> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE dataset_id = ?1");
    let count: i64 = conn.query_row(&sql, params![dataset_id], |row| row.get(0))?;
    Ok(u64::try_from(count).unwrap_or_default())
}

fn count_by_document(conn: &Connection, document_id: &str) -> rusqlite::Result<u64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM knowledge_chunks WHERE document_id = ?1",
        params![document_id],
        |row| row.get(0),
    )?;
    Ok(u64::try_from(count).unwrap_or_default())
}

fn ensure_dataset_exists(conn: &Connection, dataset_id: &str) -> Result<(), ApiError> {
    if query_dataset(conn, dataset_id)?.is_some() {
        Ok(())
    } else {
        Err(ApiError::not_found(format!("knowledge dataset not found: {dataset_id}")))
    }
}

fn validate_dataset_scope(conn: &Connection, dataset_ids: &[String]) -> Result<(), ApiError> {
    for dataset_id in dataset_ids {
        validate_id(dataset_id, "dataset_id")?;
        ensure_dataset_exists(conn, dataset_id)?;
    }
    Ok(())
}

fn resolve_retrieval_scope_blocking(
    db_path: PathBuf,
    dataset_ids: &[String],
) -> Result<RetrievalScope, ApiError> {
    let conn = open_db(&db_path)?;
    validate_dataset_scope(&conn, dataset_ids)?;
    let mut keyword_dataset_ids = Vec::new();
    let mut vector_dataset_ids = Vec::new();
    for (dataset_id, retrieval_mode) in query_retrieval_modes(&conn, dataset_ids)? {
        match retrieval_mode {
            KnowledgeRetrievalMode::FullText => keyword_dataset_ids.push(dataset_id),
            KnowledgeRetrievalMode::Vector => vector_dataset_ids.push(dataset_id),
            KnowledgeRetrievalMode::Hybrid => {
                keyword_dataset_ids.push(dataset_id.clone());
                vector_dataset_ids.push(dataset_id);
            }
        }
    }
    Ok(RetrievalScope { keyword_dataset_ids, vector_dataset_ids })
}

fn query_retrieval_modes(
    conn: &Connection,
    dataset_ids: &[String],
) -> Result<Vec<(String, KnowledgeRetrievalMode)>, ApiError> {
    let sql = if dataset_ids.is_empty() {
        "SELECT id, retrieval_mode FROM knowledge_datasets".to_string()
    } else {
        let placeholders = (0..dataset_ids.len()).map(|_| "?").collect::<Vec<_>>().join(", ");
        format!("SELECT id, retrieval_mode FROM knowledge_datasets WHERE id IN ({placeholders})")
    };
    let mut params_vec = Vec::<&dyn rusqlite::ToSql>::new();
    for dataset_id in dataset_ids {
        params_vec.push(dataset_id);
    }
    let mut stmt = conn.prepare(&sql).map_err(sql_error)?;
    let rows = stmt
        .query_map(params_from_iter(params_vec), |row| {
            let id: String = row.get(0)?;
            let retrieval_mode: String = row.get(1)?;
            Ok((id, parse_retrieval_mode(&retrieval_mode)))
        })
        .map_err(sql_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

fn validate_dataset_request(
    body: &KnowledgeDatasetCreateRequest,
    vector_available: bool,
    configured_embedding_model: Option<&str>,
) -> Result<(), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("knowledge dataset name is required"));
    }
    if body.indexing_mode == KnowledgeIndexingMode::HighQuality && !vector_available {
        return Err(ApiError::not_implemented(
            "high_quality knowledge indexing requires an embedding provider",
        ));
    }
    if body.retrieval_mode != KnowledgeRetrievalMode::FullText && !vector_available {
        return Err(ApiError::not_implemented(
            "vector and hybrid knowledge retrieval require a vector backend",
        ));
    }
    if let Some(requested_model) =
        body.embedding_model.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        let Some(configured_model) = configured_embedding_model else {
            return Err(ApiError::bad_request(
                "knowledge embedding_model requires configured memory embedding provider",
            ));
        };
        if requested_model != configured_model {
            return Err(ApiError::bad_request(format!(
                "per-dataset embedding_model override is not supported; configured model is {configured_model}"
            )));
        }
    }
    if body.rerank_model.as_deref().is_some_and(|value| !value.trim().is_empty()) {
        return Err(ApiError::not_implemented(
            "rerank_model is reserved until rerank provider support is configured",
        ));
    }
    Ok(())
}

fn validate_document_request(body: &KnowledgeDocumentCreateRequest) -> Result<(), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("knowledge document name is required"));
    }
    if body.content.trim().is_empty() {
        return Err(ApiError::bad_request("knowledge document content is required"));
    }
    if !matches!(body.metadata, Value::Null | Value::Object(_)) {
        return Err(ApiError::bad_request("knowledge document metadata must be an object"));
    }
    Ok(())
}

fn validate_retrieve_request(request: &KnowledgeRetrieveRequest) -> Result<(), ApiError> {
    if request.query.trim().is_empty() {
        return Err(ApiError::bad_request("knowledge retrieve query is required"));
    }
    if request.top_k == 0 {
        return Err(ApiError::bad_request("knowledge retrieve top_k must be greater than 0"));
    }
    if request.score_threshold.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
        return Err(ApiError::bad_request(
            "knowledge retrieve score_threshold must be between 0 and 1",
        ));
    }
    if request
        .metadata_filter
        .as_ref()
        .is_some_and(|value| !matches!(value, Value::Null | Value::Object(_)))
    {
        return Err(ApiError::bad_request("knowledge metadata_filter must be an object"));
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), ApiError> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| ApiError::bad_request(format!("knowledge {label} is invalid")))
}

fn open_db(db_path: &Path) -> Result<Connection, ApiError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApiError::internal(format!("create knowledge db dir failed: {error}"))
        })?;
    }
    let conn = Connection::open(db_path).map_err(sql_error)?;
    conn.execute_batch(KNOWLEDGE_SCHEMA).map_err(sql_error)?;
    migrate_schema(&conn)?;
    Ok(conn)
}

fn migrate_schema(conn: &Connection) -> Result<(), ApiError> {
    if !column_exists(conn, "knowledge_chunks", "embedding")? {
        conn.execute("ALTER TABLE knowledge_chunks ADD COLUMN embedding BLOB", [])
            .map_err(sql_error)?;
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, ApiError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})")).map_err(sql_error)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1)).map_err(sql_error)?;
    for row in rows {
        if row.map_err(sql_error)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn metadata_or_object(value: Value) -> Value {
    match value {
        Value::Null => Value::Object(Map::new()),
        other => other,
    }
}

fn chunk_metadata(metadata: &Value, document_name: &str, ordinal: usize) -> Value {
    let mut object = match metadata {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };
    object.insert("document_name".to_string(), Value::String(document_name.to_string()));
    object.insert("chunk_index".to_string(), json!(ordinal));
    Value::Object(object)
}

fn metadata_matches(metadata: &Value, filter: &Option<Value>) -> bool {
    let Some(filter) = filter.as_ref().filter(|value| !value.is_null()) else {
        return true;
    };
    let (Value::Object(metadata), Value::Object(filter)) = (metadata, filter) else {
        return metadata == filter;
    };
    filter.iter().all(|(key, value)| metadata.get(key) == Some(value))
}

fn dataset_filter_sql(alias: &str, dataset_count: usize) -> String {
    if dataset_count == 0 {
        return String::new();
    }
    let placeholders = (0..dataset_count).map(|_| "?").collect::<Vec<_>>().join(", ");
    format!(" AND {alias}.dataset_id IN ({placeholders})")
}

fn fts_query_from_text(query: &str) -> String {
    query
        .split_whitespace()
        .map(quote_fts_term)
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn quote_fts_term(term: &str) -> String {
    let escaped = term.trim().replace('"', "\"\"");
    if escaped.is_empty() { String::new() } else { format!("\"{escaped}\"") }
}

fn like_pattern(query: &str) -> String {
    let mut escaped = String::from("%");
    for ch in query.chars() {
        match ch {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            other => escaped.push(other),
        }
    }
    escaped.push('%');
    escaped
}

fn like_score(chunk: &KnowledgeChunkDto, query: &str) -> f32 {
    let query = query.to_lowercase();
    let title = chunk.title.to_lowercase();
    let content = chunk.content.to_lowercase();
    if title.contains(&query) {
        0.65
    } else if content.contains(&query) {
        0.55
    } else {
        0.1
    }
}

fn fts_score(rank: f64) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    let score = 1.0 / (1.0 + rank.abs());
    score as f32
}

fn read_embedding_cache_blocking(
    db_path: PathBuf,
    hash: &str,
) -> Result<Option<Vec<f32>>, ApiError> {
    let conn = open_db(&db_path)?;
    let blob = conn
        .query_row(
            "SELECT embedding FROM knowledge_embedding_cache WHERE content_hash = ?1",
            params![hash],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()
        .map_err(sql_error)?;
    if let Some(blob) = blob {
        conn.execute(
            "UPDATE knowledge_embedding_cache SET accessed_at_ms = ?1 WHERE content_hash = ?2",
            params![now_ms(), hash],
        )
        .map_err(sql_error)?;
        return Ok(Some(vector::bytes_to_vec(&blob)));
    }
    Ok(None)
}

fn write_embedding_cache_blocking(
    db_path: PathBuf,
    hash: &str,
    embedding: Vec<u8>,
    cache_size: usize,
) -> Result<(), ApiError> {
    let conn = open_db(&db_path)?;
    let now = now_ms();
    conn.execute(
        "INSERT OR REPLACE INTO knowledge_embedding_cache \
         (content_hash, embedding, created_at_ms, accessed_at_ms) VALUES (?1, ?2, ?3, ?4)",
        params![hash, embedding, now, now],
    )
    .map_err(sql_error)?;
    let cache_size = i64::try_from(cache_size).unwrap_or(i64::MAX);
    conn.execute(
        "DELETE FROM knowledge_embedding_cache WHERE content_hash IN (
            SELECT content_hash FROM knowledge_embedding_cache
            ORDER BY accessed_at_ms ASC
            LIMIT MAX(0, (SELECT COUNT(*) FROM knowledge_embedding_cache) - ?1)
        )",
        params![cache_size],
    )
    .map_err(sql_error)?;
    Ok(())
}

fn content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn indexing_mode_as_str(mode: &KnowledgeIndexingMode) -> &'static str {
    match mode {
        KnowledgeIndexingMode::Economy => "economy",
        KnowledgeIndexingMode::HighQuality => "high_quality",
    }
}

fn retrieval_mode_as_str(mode: &KnowledgeRetrievalMode) -> &'static str {
    match mode {
        KnowledgeRetrievalMode::FullText => "full_text",
        KnowledgeRetrievalMode::Vector => "vector",
        KnowledgeRetrievalMode::Hybrid => "hybrid",
    }
}

fn parse_indexing_mode(value: &str) -> KnowledgeIndexingMode {
    match value {
        "high_quality" => KnowledgeIndexingMode::HighQuality,
        _ => KnowledgeIndexingMode::Economy,
    }
}

fn parse_retrieval_mode(value: &str) -> KnowledgeRetrievalMode {
    match value {
        "vector" => KnowledgeRetrievalMode::Vector,
        "hybrid" => KnowledgeRetrievalMode::Hybrid,
        _ => KnowledgeRetrievalMode::FullText,
    }
}

fn read_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    Ok(u64::try_from(value).unwrap_or_default())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn sql_error(error: rusqlite::Error) -> ApiError {
    ApiError::internal(format!("knowledge sqlite error: {error}"))
}
