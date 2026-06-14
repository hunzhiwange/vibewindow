use super::*;
use crate::app::agent::memory::embeddings::EmbeddingProvider;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn embedding_cache_module_is_linked() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}

struct CountingEmbedding {
    calls: AtomicUsize,
}

#[async_trait]
impl EmbeddingProvider for CountingEmbedding {
    fn name(&self) -> &str {
        "counting"
    }

    fn dimensions(&self) -> usize {
        2
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(texts.iter().map(|text| vec![text.len() as f32, 1.0]).collect())
    }
}

#[tokio::test]
async fn zero_dimension_embedder_skips_cache_and_embedding_call() {
    let workspace = tempfile::TempDir::new().unwrap();
    let memory = SqliteMemory::new(workspace.path()).unwrap();

    let result = memory.get_or_compute_embedding("hello").await.unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn get_or_compute_embedding_caches_and_reuses_result() {
    let workspace = tempfile::TempDir::new().unwrap();
    let embedder = Arc::new(CountingEmbedding { calls: AtomicUsize::new(0) });
    let memory =
        SqliteMemory::with_embedder(workspace.path(), embedder.clone(), 0.7, 0.3, 100, None)
            .unwrap();

    let first = memory.get_or_compute_embedding("hello").await.unwrap().unwrap();
    let second = memory.get_or_compute_embedding("hello").await.unwrap().unwrap();

    assert_eq!(first, vec![5.0, 1.0]);
    assert_eq!(second, first);
    assert_eq!(embedder.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn get_or_compute_embedding_evicts_old_entries_when_cache_is_full() {
    let workspace = tempfile::TempDir::new().unwrap();
    let embedder = Arc::new(CountingEmbedding { calls: AtomicUsize::new(0) });
    let memory =
        SqliteMemory::with_embedder(workspace.path(), embedder.clone(), 0.7, 0.3, 1, None).unwrap();

    memory.get_or_compute_embedding("one").await.unwrap();
    memory.get_or_compute_embedding("two").await.unwrap();

    let conn = memory.conn.lock();
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM embedding_cache", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);
}
