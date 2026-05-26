use super::*;
use crate::app::agent::memory::{MemoryCategory, MemoryEntry};

#[derive(Default)]
struct FakeMemory;

#[async_trait::async_trait]
impl Memory for FakeMemory {
    fn name(&self) -> &str {
        "fake"
    }

    async fn store(
        &self,
        _key: &str,
        _content: &str,
        _category: MemoryCategory,
        _session_id: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn recall(
        &self,
        _query: &str,
        _limit: usize,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(vec![
            MemoryEntry {
                id: "1".to_string(),
                key: "keep".to_string(),
                content: "use concise answers".to_string(),
                category: MemoryCategory::Core,
                timestamp: "now".to_string(),
                session_id: None,
                score: Some(0.9),
            },
            MemoryEntry {
                id: "2".to_string(),
                key: "drop".to_string(),
                content: "too weak".to_string(),
                category: MemoryCategory::Core,
                timestamp: "now".to_string(),
                session_id: None,
                score: Some(0.1),
            },
        ])
    }

    async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(None)
    }

    async fn list(
        &self,
        _category: Option<&MemoryCategory>,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(0)
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn build_context_filters_low_relevance_entries() {
    let context = build_context(&FakeMemory, "style", 0.5).await;

    assert!(context.contains("- keep: use concise answers"));
    assert!(!context.contains("too weak"));
}
