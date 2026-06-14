use super::*;
use crate::app::App;
use crate::app::state::{ModelSummary, ProviderModelsSummary};
use serde_json::json;

fn app() -> App {
    App::new().0
}

fn provider_summary() -> ProviderModelsSummary {
    ProviderModelsSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models: vec![ModelSummary {
            id: "gpt-5".to_string(),
            name: "GPT-5".to_string(),
            enabled: true,
            toolcall: true,
            attachment: true,
            context_limit: 200_000,
            detail: json!({
                "id": "gpt-5",
                "name": "GPT-5",
                "status": "active",
                "release_date": "2026-01-01",
                "api": { "url": "https://api.example.com", "adapter": "openai" },
                "limit": { "context": 200000, "input": 100000, "output": 20000 },
                "capabilities": {
                    "toolcall": true,
                    "attachment": true,
                    "reasoning": true,
                    "temperature": true,
                    "input": { "text": true, "audio": false, "image": true, "video": false, "pdf": true },
                    "output": { "text": true, "audio": false, "image": false, "video": false, "pdf": false },
                    "interleaved": { "field": "content" }
                },
                "cost": { "input": 1.0, "output": 2.0, "cache": { "read": 0.5, "write": 0.75 } },
                "headers": { "x-api-key": "hidden" },
                "options": { "reasoning_effort": "high" },
                "variants": { "fast": { "tier": "low-latency" } }
            }),
        }],
    }
}

#[test]
fn models_update_handles_refresh_and_errors() {
    let mut app = app();

    let _ = update(&mut app, SettingsMessage::ModelsRefresh);
    assert!(app.model_settings.loading);
    assert!(app.model_settings.save_error.is_none());

    let _ = update(&mut app, SettingsMessage::ModelsRefreshed(Ok(vec![provider_summary()])));
    assert!(!app.model_settings.loading);
    assert_eq!(app.model_settings.providers.len(), 1);

    let _ = update(&mut app, SettingsMessage::ModelsRefreshed(Err("network".to_string())));
    assert_eq!(app.model_settings.save_error.as_deref(), Some("network"));
}

#[test]
fn models_detail_modal_open_toggle_and_close() {
    let mut app = app();
    app.model_settings.providers = vec![provider_summary()];

    let _ = update(
        &mut app,
        SettingsMessage::ModelDetailOpen("missing".to_string(), "gpt-5".to_string()),
    );
    assert_eq!(app.model_settings.save_error.as_deref(), Some("未找到提供商"));

    let _ = update(
        &mut app,
        SettingsMessage::ModelDetailOpen("openai".to_string(), "gpt-5".to_string()),
    );
    let modal = app.model_settings.detail_modal.as_ref().expect("detail modal");
    assert_eq!(modal.provider_id, "openai");
    assert_eq!(modal.model_id, "gpt-5");
    assert!(modal.rows.iter().any(|row| row.label.contains("模型名称")));
    assert!(modal.raw_json.contains("\"gpt-5\""));

    let _ = update(&mut app, SettingsMessage::ModelDetailToggleRaw);
    assert!(app.model_settings.detail_modal.as_ref().expect("detail modal").show_raw);

    let _ = update(&mut app, SettingsMessage::ModelDetailClose);
    assert!(app.model_settings.detail_modal.is_none());
}
