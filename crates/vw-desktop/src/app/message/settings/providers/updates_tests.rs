use super::*;
use crate::app::App;
use crate::app::message::settings::types::ProvidersLoaded;
use crate::app::state::{ModelCatalogEntry, ProviderSummary};
use std::collections::HashMap;

fn app() -> App {
    App::new().0
}

fn loaded() -> ProvidersLoaded {
    Ok((HashMap::new(), vec!["OpenAI".to_string()], true, Vec::new()))
}

#[test]
fn provider_update_state_machine_paths() {
    let mut app = app();
    let _ = update(&mut app, SettingsMessage::ProvidersRefresh);
    assert!(app.provider_settings.loading);
    app.provider_settings.loading = false;
    let _ = update(&mut app, SettingsMessage::ProviderModelsSyncRemote);
    assert!(app.provider_settings.models_syncing);
    let _ = update(&mut app, SettingsMessage::ProviderModelsSyncTick);
    assert!(app.provider_settings.models_sync_progress > 0.08);
    let _ = update(&mut app, SettingsMessage::ProviderModelsSyncDone(loaded()));
    assert!(!app.provider_settings.models_syncing);
    let _ =
        update(&mut app, SettingsMessage::ProviderModelsSyncDone(Err("sync failed".to_string())));
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("sync failed"));

    app.provider_settings.catalog_items = vec![ModelCatalogEntry {
        provider_id: "openai".to_string(),
        provider_name: "OpenAI Catalog".to_string(),
        model_id: "gpt".to_string(),
        model_name: "GPT".to_string(),
    }];
    let _ = update(&mut app, SettingsMessage::ProviderConnectOpen("openai".to_string()));
    assert_eq!(
        app.provider_settings.connect_modal.as_ref().unwrap().provider_name,
        "OpenAI Catalog"
    );
    let _ = update(&mut app, SettingsMessage::ProviderConnectApiKeyChanged(" ".to_string()));
    let _ = update(&mut app, SettingsMessage::ProviderConnectSubmit);
    assert_eq!(app.provider_settings.connect_error.as_deref(), Some("请输入 API 密钥"));
    let _ = update(&mut app, SettingsMessage::ProviderConnectApiKeyChanged("sk".to_string()));
    let _ = update(&mut app, SettingsMessage::ProviderConnectSubmit);
    assert!(app.provider_settings.loading);
    let _ = update(&mut app, SettingsMessage::ProviderConnectDone(loaded()));
    assert!(!app.provider_settings.loading);
    let _ = update(&mut app, SettingsMessage::ProviderConnectClose);
    assert!(app.provider_settings.connect_modal.is_none());

    let _ = update(&mut app, SettingsMessage::ProviderDisconnectRequested("openai".to_string()));
    assert_eq!(app.provider_settings.disconnect_confirm_provider_id.as_deref(), Some("openai"));
    let _ = update(&mut app, SettingsMessage::ProviderDisconnectCanceled);
    assert!(app.provider_settings.disconnect_confirm_provider_id.is_none());
    let _ = update(&mut app, SettingsMessage::ProviderDisconnectConfirmed("openai".to_string()));
    assert!(app.provider_settings.loading);
    let _ = update(&mut app, SettingsMessage::ProviderDisconnectDone(Err("failed".to_string())));
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("failed"));
}

#[test]
fn custom_provider_and_catalog_paths() {
    let mut app = app();
    let _ = update(&mut app, SettingsMessage::CustomProviderOpen);
    assert!(app.provider_settings.custom_provider_modal_open);
    let _ = update(&mut app, SettingsMessage::CustomProviderIdChanged("Bad Id".to_string()));
    let _ = update(&mut app, SettingsMessage::CustomProviderSave);
    assert!(app.provider_settings.save_error.as_deref().unwrap_or("").contains("提供商 ID"));
    let _ = update(&mut app, SettingsMessage::CustomProviderIdChanged("custom".to_string()));
    let _ = update(&mut app, SettingsMessage::CustomProviderBaseUrlChanged(" ".to_string()));
    let _ = update(&mut app, SettingsMessage::CustomProviderSave);
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("基础 URL 不能为空"));
    let _ =
        update(&mut app, SettingsMessage::CustomProviderBaseUrlChanged("https://api".to_string()));
    app.provider_settings.custom.models.clear();
    let _ = update(&mut app, SettingsMessage::CustomProviderSave);
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("至少添加一个模型"));

    let _ = update(&mut app, SettingsMessage::CustomProviderModelOpen(None));
    let _ = update(&mut app, SettingsMessage::CustomProviderModelModalSave);
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("模型 ID 不能为空"));
    let _ = update(&mut app, SettingsMessage::CustomProviderModelModalIdChanged("m1".to_string()));
    let _ = update(
        &mut app,
        SettingsMessage::CustomProviderModelModalNameChanged(" Model 1 ".to_string()),
    );
    let _ = update(&mut app, SettingsMessage::CustomProviderModelModalSave);
    assert_eq!(app.provider_settings.custom.models[0].display_name, "Model 1");
    let _ = update(&mut app, SettingsMessage::CustomProviderHeaderAdd);
    let _ = update(&mut app, SettingsMessage::CustomProviderHeaderKeyChanged(1, "X".to_string()));
    let _ = update(&mut app, SettingsMessage::CustomProviderHeaderValueChanged(1, "v".to_string()));
    let _ = update(&mut app, SettingsMessage::CustomProviderHeaderRemove(1));
    assert_eq!(app.provider_settings.custom.headers.len(), 1);
    let _ = update(&mut app, SettingsMessage::CustomProviderClose);
    assert!(!app.provider_settings.custom_provider_modal_open);

    app.provider_settings.providers = vec![ProviderSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        source_label: "API".to_string(),
        connected: true,
    }];
    let _ = update(&mut app, SettingsMessage::ProviderCatalogOpen);
    assert!(app.provider_settings.catalog_open);
    let _ = update(&mut app, SettingsMessage::ProviderCatalogQueryChanged("gpt".to_string()));
    assert_eq!(app.provider_settings.catalog_query, "gpt");
    let _ = update(
        &mut app,
        SettingsMessage::ProviderCatalogLoaded(Ok(vec![ModelCatalogEntry {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            model_id: "gpt".to_string(),
            model_name: "GPT".to_string(),
        }])),
    );
    assert_eq!(app.provider_settings.catalog_items.len(), 1);
    let _ = update(&mut app, SettingsMessage::ProviderCatalogAddToPopular("openai".to_string()));
    let _ = update(&mut app, SettingsMessage::ProviderCatalogAddToPopular("openai".to_string()));
    assert_eq!(
        app.provider_settings.popular_patterns.iter().filter(|p| p.as_str() == "openai").count(),
        1
    );
    let _ = update(&mut app, SettingsMessage::PopularProviderRemove(0));
    let _ =
        update(&mut app, SettingsMessage::PopularProvidersSaved(Err("save failed".to_string())));
    assert_eq!(app.provider_settings.save_error.as_deref(), Some("save failed"));
    let _ = update(&mut app, SettingsMessage::ProviderCatalogClose);
    assert!(!app.provider_settings.catalog_open);
}
