use super::*;
use crate::app::App;
use crate::app::state::EmbeddingRouteDraft;

fn app() -> App {
    App::new().0
}

#[test]
fn embedding_routes_trim_to_option_trims_blank_values() {
    assert_eq!(trim_to_option(" key "), Some("key".to_string()));
    assert_eq!(trim_to_option("   "), None);
}

#[test]
fn embedding_routes_update_manages_rows_and_validation() {
    let mut app = app();

    let _ = update(&mut app, SettingsMessage::EmbeddingRoutes(EmbeddingRoutesMessage::AddRoute));
    assert_eq!(app.embedding_routes_settings.routes.len(), 1);
    assert!(!app.embedding_routes_settings.save_success);

    let _ = update(&mut app, SettingsMessage::EmbeddingRoutes(EmbeddingRoutesMessage::Save));
    assert!(app.embedding_routes_settings.save_error.as_deref().unwrap_or("").contains("pattern"));

    app.embedding_routes_settings.routes = vec![EmbeddingRouteDraft {
        pattern: "docs".to_string(),
        provider: "openai".to_string(),
        model: "text-embedding-3-large".to_string(),
        dimensions: "0".to_string(),
        api_key_input: "  key  ".to_string(),
    }];
    let _ = update(&mut app, SettingsMessage::EmbeddingRoutes(EmbeddingRoutesMessage::Save));
    assert!(
        app.embedding_routes_settings.save_error.as_deref().unwrap_or("").contains("dimensions")
    );

    let _ = update(
        &mut app,
        SettingsMessage::EmbeddingRoutes(EmbeddingRoutesMessage::DimensionsChanged(
            0,
            "1536".to_string(),
        )),
    );
    let _ = update(&mut app, SettingsMessage::EmbeddingRoutes(EmbeddingRoutesMessage::Save));
    assert!(app.embedding_routes_settings.save_error.is_none());
    assert!(app.embedding_routes_settings.save_success);
}
