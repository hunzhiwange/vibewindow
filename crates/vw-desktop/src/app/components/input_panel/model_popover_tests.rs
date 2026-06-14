use super::model_popover::{model_popover_content, model_toggle_button, normalize_model_input};
use crate::app::state::{ModelSummary, ProviderModelsSummary};
use iced::Length;

fn model(id: &str, name: &str, enabled: bool) -> ModelSummary {
    ModelSummary {
        id: id.to_string(),
        name: name.to_string(),
        enabled,
        toolcall: true,
        attachment: false,
        context_limit: 128_000,
        detail: serde_json::json!({}),
    }
}

fn provider(id: &str, name: &str, models: Vec<ModelSummary>) -> ProviderModelsSummary {
    ProviderModelsSummary { id: id.to_string(), name: name.to_string(), models }
}

#[test]
fn normalize_model_input_defaults_blank_values_to_auto() {
    assert_eq!(normalize_model_input(""), "auto");
    assert_eq!(normalize_model_input("   "), "auto");
    assert_eq!(normalize_model_input(" openai/gpt-4.1 "), "openai/gpt-4.1");
}

#[test]
fn model_toggle_button_uses_auto_label_and_expanded_state() {
    let app = crate::app::App::new().0;

    let auto = model_toggle_button(&app, true, "openai/gpt-4.1", false);
    let expanded = model_toggle_button(&app, false, "openai/gpt-4.1", true);

    assert_eq!(auto.as_widget().size().width, Length::Shrink);
    assert_eq!(expanded.as_widget().size().width, Length::Shrink);
}

#[test]
fn model_toggle_button_resolves_provider_model_and_bare_model_names() {
    let mut app = crate::app::App::new().0;
    app.model_settings.providers = vec![provider(
        "openai",
        "OpenAI",
        vec![model("gpt-4.1", "GPT 4.1", true), model("z-disabled", "Disabled", false)],
    )];

    let provider_model = model_toggle_button(&app, false, "openai/gpt-4.1", false);
    let bare_model = model_toggle_button(&app, false, "gpt-4.1", false);
    let fallback = model_toggle_button(&app, false, "custom/model", false);

    assert_eq!(provider_model.as_widget().size().width, Length::Shrink);
    assert_eq!(bare_model.as_widget().size().width, Length::Shrink);
    assert_eq!(fallback.as_widget().size().width, Length::Shrink);
}

#[test]
fn model_popover_content_renders_loading_empty_and_filtered_states() {
    let mut app = crate::app::App::new().0;

    app.model_settings.loading = true;
    let loading = model_popover_content(&app, true, "auto", false);
    assert_eq!(loading.as_widget().size().width, Length::Fixed(320.0));
    drop(loading);

    app.model_settings.loading = false;
    let empty = model_popover_content(&app, false, "missing", true);
    assert_eq!(empty.as_widget().size().width, Length::Fixed(320.0));
    drop(empty);

    app.model_settings.providers = vec![
        provider(
            "openai",
            "OpenAI",
            vec![
                model("gpt-4.1", "GPT 4.1", true),
                model(
                    "a-very-long-model-id",
                    "A very long model display name that will be truncated",
                    true,
                ),
            ],
        ),
        provider("disabled", "Disabled", vec![model("hidden", "Hidden", false)]),
    ];
    app.model_settings.query = "gpt".to_string();
    let filtered = model_popover_content(&app, false, "openai/gpt-4.1", false);
    assert_eq!(filtered.as_widget().size().width, Length::Fixed(320.0));
    drop(filtered);

    app.model_settings.query = "no-match".to_string();
    let no_match = model_popover_content(&app, false, "openai/gpt-4.1", false);
    assert_eq!(no_match.as_widget().size().width, Length::Fixed(320.0));
}
