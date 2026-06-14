#[test]
fn storage_provider_supports_legacy_db_url_aliases() {
    let parsed: super::StorageProviderConfig = serde_json::from_value(serde_json::json!({
        "provider": "postgres",
        "dbURL": "postgres://example"
    }))
    .unwrap();

    assert_eq!(parsed.db_url.as_deref(), Some("postgres://example"));
    assert_eq!(parsed.schema, "public");
    assert_eq!(parsed.table, "memories");
}

#[test]
fn memory_defaults_and_qdrant_defaults_are_stable() {
    let memory = super::MemoryConfig::default();
    assert_eq!(memory.backend, "sqlite");
    assert!(memory.auto_save);
    assert_eq!(memory.embedding_provider, "none");
    assert_eq!(memory.embedding_model, "text-embedding-3-small");
    assert_eq!(memory.embedding_dimensions, 1536);
    assert!(memory.auto_hydrate);

    let qdrant = super::QdrantConfig::default();
    assert_eq!(qdrant.collection, "vibewindow_memories");
}
