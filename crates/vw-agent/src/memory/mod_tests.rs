use super::*;
use crate::app::agent::config::{EmbeddingRouteConfig, MemoryConfig, StorageProviderConfig};
use tempfile::TempDir;

fn memory_config(backend: &str) -> MemoryConfig {
    MemoryConfig { backend: backend.to_string(), ..MemoryConfig::default() }
}

#[test]
fn effective_backend_name_trims_backend_and_blank_storage_override() {
    let blank_override =
        StorageProviderConfig { provider: "   ".to_string(), ..StorageProviderConfig::default() };
    let mysql_override = StorageProviderConfig {
        provider: " MySQL ".to_string(),
        ..StorageProviderConfig::default()
    };

    assert_eq!(effective_memory_backend_name(" SQLite ", None), "sqlite");
    assert_eq!(effective_memory_backend_name(" Lucid ", Some(&blank_override)), "lucid");
    assert_eq!(effective_memory_backend_name("sqlite", Some(&mysql_override)), "mysql");
}

#[test]
fn assistant_autosave_key_detection_trims_input() {
    assert!(is_assistant_autosave_key(" assistant_resp "));
    assert!(is_assistant_autosave_key("\tASSISTANT_RESP_42\n"));
    assert!(is_assistant_autosave_key(" assistant_resp_extra"));
}

#[test]
fn resolved_embedding_debug_redacts_api_key() {
    let resolved = ResolvedEmbeddingConfig {
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimensions: 1536,
        api_key: Some("secret-key".to_string()),
    };

    let debug = format!("{resolved:?}");

    assert!(debug.contains("openai"));
    assert!(debug.contains("text-embedding-3-small"));
    assert!(debug.contains("1536"));
    assert!(!debug.contains("secret-key"));
}

#[test]
fn resolve_embedding_config_trims_routes_and_falls_back_to_base_key() {
    let cfg = MemoryConfig {
        embedding_provider: " none ".to_string(),
        embedding_model: "hint: semantic ".to_string(),
        embedding_dimensions: 1536,
        ..MemoryConfig::default()
    };
    let routes = vec![EmbeddingRouteConfig {
        hint: " semantic ".to_string(),
        provider: " alibaba-cn ".to_string(),
        model: " text-embedding-v4 ".to_string(),
        dimensions: None,
        api_key: Some("   ".to_string()),
    }];

    let resolved = resolve_embedding_config(&cfg, &routes, Some(" base-key "));

    assert_eq!(
        resolved,
        ResolvedEmbeddingConfig {
            provider: "alibaba-cn".to_string(),
            model: "text-embedding-v4".to_string(),
            dimensions: 1536,
            api_key: Some("base-key".to_string()),
        }
    );
}

#[test]
fn create_embedding_provider_with_routes_returns_resolved_config() {
    let cfg = MemoryConfig {
        embedding_provider: "none".to_string(),
        embedding_model: "hint:test".to_string(),
        embedding_dimensions: 32,
        ..MemoryConfig::default()
    };
    let routes = vec![EmbeddingRouteConfig {
        hint: "test".to_string(),
        provider: "none".to_string(),
        model: "noop".to_string(),
        dimensions: Some(12),
        api_key: None,
    }];

    let (_provider, resolved) = create_embedding_provider_with_routes(&cfg, &routes, None);

    assert_eq!(resolved.provider, "none");
    assert_eq!(resolved.model, "noop");
    assert_eq!(resolved.dimensions, 12);
    assert_eq!(resolved.api_key, None);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn create_memory_with_builders_dispatches_postgres_and_rejects_mariadb() {
    let tmp = TempDir::new().unwrap();
    let postgres = create_memory_with_builders(
        "postgres",
        tmp.path(),
        || -> anyhow::Result<SqliteMemory> { panic!("sqlite builder should not be called") },
        || Ok(Box::new(NoneMemory::new()) as Box<dyn Memory>),
        " in test",
    )
    .unwrap();

    assert_eq!(postgres.name(), "none");

    let error = match create_memory_with_builders(
        "mariadb",
        tmp.path(),
        || -> anyhow::Result<SqliteMemory> { panic!("sqlite builder should not be called") },
        || Ok(Box::new(NoneMemory::new()) as Box<dyn Memory>),
        " in test",
    ) {
        Ok(_) => panic!("mariadb should be rejected"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("mariadb"));
    assert!(error.contains("not available"));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn migration_factory_rejects_sql_backends_and_allows_file_backends() {
    let tmp = TempDir::new().unwrap();

    let markdown = create_memory_for_migration("markdown", tmp.path()).unwrap();
    assert_eq!(markdown.name(), "markdown");

    let unknown = create_memory_for_migration("custom-experimental", tmp.path()).unwrap();
    assert_eq!(unknown.name(), "markdown");

    for backend in ["postgres", "mariadb", "mysql"] {
        let error = match create_memory_for_migration(backend, tmp.path()) {
            Ok(_) => panic!("{backend} should be rejected"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("unsupported"));
    }
}

#[test]
fn create_memory_with_storage_ignores_blank_storage_override() {
    let tmp = TempDir::new().unwrap();
    let cfg = memory_config("markdown");
    let storage =
        StorageProviderConfig { provider: " \t ".to_string(), ..StorageProviderConfig::default() };

    let memory = create_memory_with_storage(&cfg, Some(&storage), tmp.path(), None).unwrap();

    assert_eq!(memory.name(), "markdown");
}

#[test]
fn qdrant_factory_uses_lazy_backend_when_url_is_configured() {
    let tmp = TempDir::new().unwrap();
    let mut cfg = memory_config("qdrant");
    cfg.qdrant.url = Some("http://127.0.0.1:6333/".to_string());
    cfg.embedding_provider = "none".to_string();

    let memory = create_memory(&cfg, tmp.path(), None).unwrap();

    assert_eq!(memory.name(), "qdrant");
}

#[test]
fn create_response_cache_respects_enabled_flag() {
    let tmp = TempDir::new().unwrap();
    let disabled = MemoryConfig { response_cache_enabled: false, ..MemoryConfig::default() };
    let enabled = MemoryConfig {
        response_cache_enabled: true,
        response_cache_ttl_minutes: 5,
        response_cache_max_entries: 2,
        ..MemoryConfig::default()
    };

    assert!(create_response_cache(&disabled, tmp.path()).is_none());
    assert!(create_response_cache(&enabled, tmp.path()).is_some());
}
