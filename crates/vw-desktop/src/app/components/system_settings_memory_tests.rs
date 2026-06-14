use super::*;
use crate::app::App;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_memory_tests_are_wired() {
    assert!(module_path!().contains("system_settings_memory_tests"));
}

#[test]
fn view_builds_default_and_custom_memory_states() {
    let app = test_app();
    let _ = view(&app);

    let mut custom = test_app();
    custom.memory_settings.backend = "qdrant".to_string();
    custom.memory_settings.auto_save = false;
    custom.memory_settings.hygiene_enabled = false;
    custom.memory_settings.response_cache_enabled = true;
    custom.memory_settings.snapshot_enabled = true;
    custom.memory_settings.snapshot_on_hygiene = true;
    custom.memory_settings.auto_hydrate = false;
    custom.memory_settings.archive_after_days = 0;
    custom.memory_settings.purge_after_days = 3650;
    custom.memory_settings.conversation_retention_days = 90;
    custom.memory_settings.embedding_provider = "hint:semantic".to_string();
    custom.memory_settings.embedding_model = "text-embedding-v4".to_string();
    custom.memory_settings.embedding_dimensions = 2048;
    custom.memory_settings.vector_weight = 1.0;
    custom.memory_settings.keyword_weight = 0.0;
    custom.memory_settings.min_relevance_score = 0.95;
    custom.memory_settings.embedding_cache_size = 0;
    custom.memory_settings.chunk_max_tokens = 32_768;
    custom.memory_settings.response_cache_ttl_minutes = 0;
    custom.memory_settings.response_cache_max_entries = 0;
    custom.memory_settings.sqlite_open_timeout_secs = 3600;
    custom.memory_settings.qdrant_url_input = "http://localhost:6333".to_string();
    custom.memory_settings.qdrant_collection = "memories".to_string();
    custom.memory_settings.qdrant_api_key_input = "secret".to_string();
    custom.memory_settings.save_error = Some("memory save failed".to_string());
    let _ = view(&custom);
}
