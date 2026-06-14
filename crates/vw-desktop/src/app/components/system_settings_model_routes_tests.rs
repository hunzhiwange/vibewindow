use super::*;
use crate::app::App;
use crate::app::state::ModelRoute;

const SOURCE: &str = include_str!("system_settings_model_routes.rs");

fn test_app() -> App {
    App::new().0
}

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn system_settings_model_routes_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn view_builds_empty_error_and_populated_model_routes_states() {
    let app = test_app();
    let _ = view(&app);

    let mut with_error = test_app();
    with_error.model_routes_settings.save_error = Some("model route save failed".to_string());
    let _ = view(&with_error);

    let mut populated = test_app();
    populated.model_routes_settings.routes = vec![
        ModelRoute {
            pattern: "reasoning".to_string(),
            provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            priority_input: "10".to_string(),
        },
        ModelRoute {
            pattern: "fast".to_string(),
            provider: "anthropic".to_string(),
            model: "claude".to_string(),
            priority_input: "1".to_string(),
        },
    ];
    let _ = view(&populated);
}
