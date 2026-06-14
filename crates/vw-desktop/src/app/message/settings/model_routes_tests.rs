use super::*;
use crate::app::App;
use crate::app::state::ModelRoute;

fn app() -> App {
    App::new().0
}

#[test]
fn model_routes_validate_required_fields() {
    let mut app = app();
    app.model_routes_settings.routes = vec![ModelRoute {
        pattern: "  ".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5".to_string(),
        priority_input: "1".to_string(),
    }];
    assert!(validate_routes(&app).unwrap_err().contains("pattern"));

    app.model_routes_settings.routes[0].pattern = "chat".to_string();
    app.model_routes_settings.routes[0].priority_input = "oops".to_string();
    assert!(validate_routes(&app).unwrap_err().contains("priority"));
}

#[test]
fn model_routes_update_syncs_query_classification_state() {
    let mut app = app();

    let _ = update(&mut app, SettingsMessage::ModelRoutes(ModelRoutesMessage::AddRoute));
    let _ = update(
        &mut app,
        SettingsMessage::ModelRoutes(ModelRoutesMessage::PatternChanged(0, "chat".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::ModelRoutes(ModelRoutesMessage::ProviderChanged(0, "openai".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::ModelRoutes(ModelRoutesMessage::ModelChanged(0, "gpt-5".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::ModelRoutes(ModelRoutesMessage::PriorityChanged(0, "7".to_string())),
    );

    assert!(app.query_classification_settings.enabled);
    assert_eq!(app.query_classification_settings.rules.len(), 1);
    assert_eq!(app.query_classification_settings.rules[0].category, "chat");
    assert_eq!(app.model_routes_settings.routes[0].provider, "openai");
    assert_eq!(app.model_routes_settings.routes[0].model, "gpt-5");

    let _ = update(
        &mut app,
        SettingsMessage::ModelRoutes(ModelRoutesMessage::PriorityChanged(0, "bad".to_string())),
    );
    assert!(app.model_routes_settings.save_error.as_deref().unwrap_or("").contains("priority"));
}
