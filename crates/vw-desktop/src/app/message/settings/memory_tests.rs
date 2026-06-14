use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn memory_normalizes_backend_aliases() {
    assert_eq!(normalize_backend("postgres"), "postgres");
    assert_eq!(normalize_backend("null"), "none");
    assert_eq!(normalize_backend("unknown"), "sqlite");
}

#[test]
fn memory_update_clamps_and_refreshes_state() {
    let mut app = app();
    app.memory_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::Refresh));
    assert!(app.memory_settings.save_error.is_none());

    let _ = update(
        &mut app,
        SettingsMessage::Memory(MemoryMessage::BackendChanged("null".to_string())),
    );
    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::EmbeddingDimensionsChanged(0)));
    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::VectorWeightChanged(3.0)));
    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::KeywordWeightChanged(-1.0)));
    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::MinRelevanceScoreChanged(2.0)));
    let _ = update(&mut app, SettingsMessage::Memory(MemoryMessage::ChunkMaxTokensChanged(0)));
    let _ = update(
        &mut app,
        SettingsMessage::Memory(MemoryMessage::QdrantCollectionChanged("  memories  ".to_string())),
    );

    assert_eq!(app.memory_settings.backend, "none");
    assert_eq!(app.memory_settings.embedding_dimensions, 1);
    assert_eq!(app.memory_settings.vector_weight, 1.0);
    assert_eq!(app.memory_settings.keyword_weight, 0.0);
    assert_eq!(app.memory_settings.min_relevance_score, 1.0);
    assert_eq!(app.memory_settings.chunk_max_tokens, 1);
    assert_eq!(app.memory_settings.qdrant_collection, "  memories  ");
    assert!(app.memory_settings.save_error.is_none());
}
