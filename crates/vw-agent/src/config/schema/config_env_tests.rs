use super::{Config, config_env::apply_env_overrides};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn apply_env_overrides_has_stable_in_place_signature() {
    let apply: fn(&mut Config) = apply_env_overrides;

    let _ = apply;
}

#[test]
fn apply_env_overrides_updates_core_gateway_and_feature_fields() {
    let _guard = env_lock();
    for key in [
        "VIBEWINDOW_API_KEY",
        "API_KEY",
        "VIBEWINDOW_PROVIDER",
        "VIBEWINDOW_MODEL",
        "VIBEWINDOW_GATEWAY_PORT",
        "VIBEWINDOW_GATEWAY_HOST",
        "VIBEWINDOW_ALLOW_PUBLIC_BIND",
        "VIBEWINDOW_TEMPERATURE",
        "VIBEWINDOW_REASONING_ENABLED",
        "VIBEWINDOW_MODEL_SUPPORT_VISION",
        "VIBEWINDOW_WEB_SEARCH_ENABLED",
        "VIBEWINDOW_WEB_SEARCH_PROVIDER",
        "VIBEWINDOW_BRAVE_API_KEY",
        "VIBEWINDOW_WEB_SEARCH_MAX_RESULTS",
        "VIBEWINDOW_WEB_SEARCH_TIMEOUT_SECS",
    ] {
        unsafe {
            std::env::remove_var(key);
        }
    }

    unsafe {
        std::env::set_var("VIBEWINDOW_API_KEY", "key");
        std::env::set_var("VIBEWINDOW_PROVIDER", "provider");
        std::env::set_var("VIBEWINDOW_MODEL", "model");
        std::env::set_var("VIBEWINDOW_GATEWAY_PORT", "19090");
        std::env::set_var("VIBEWINDOW_GATEWAY_HOST", "127.0.0.2");
        std::env::set_var("VIBEWINDOW_ALLOW_PUBLIC_BIND", "true");
        std::env::set_var("VIBEWINDOW_TEMPERATURE", "1.25");
        std::env::set_var("VIBEWINDOW_REASONING_ENABLED", "yes");
        std::env::set_var("VIBEWINDOW_MODEL_SUPPORT_VISION", "off");
        std::env::set_var("VIBEWINDOW_WEB_SEARCH_ENABLED", "1");
        std::env::set_var("VIBEWINDOW_WEB_SEARCH_PROVIDER", "brave");
        std::env::set_var("VIBEWINDOW_BRAVE_API_KEY", "brave-key");
        std::env::set_var("VIBEWINDOW_WEB_SEARCH_MAX_RESULTS", "7");
        std::env::set_var("VIBEWINDOW_WEB_SEARCH_TIMEOUT_SECS", "12");
    }

    let mut config = Config::default();
    apply_env_overrides(&mut config);

    assert_eq!(config.api_key.as_deref(), Some("key"));
    assert_eq!(config.default_provider.as_deref(), Some("provider"));
    assert_eq!(config.default_model.as_deref(), Some("model"));
    assert_eq!(config.gateway.port, 19090);
    assert_eq!(config.gateway.host, "127.0.0.2");
    assert!(config.gateway.allow_public_bind);
    assert_eq!(config.default_temperature, 1.25);
    assert_eq!(config.runtime.reasoning_enabled, Some(true));
    assert_eq!(config.model_support_vision, Some(false));
    assert!(config.web_search.enabled);
    assert_eq!(config.web_search.provider, "brave");
    assert_eq!(config.web_search.brave_api_key.as_deref(), Some("brave-key"));
    assert_eq!(config.web_search.max_results, 7);
    assert_eq!(config.web_search.timeout_secs, 12);

    for key in [
        "VIBEWINDOW_API_KEY",
        "VIBEWINDOW_PROVIDER",
        "VIBEWINDOW_MODEL",
        "VIBEWINDOW_GATEWAY_PORT",
        "VIBEWINDOW_GATEWAY_HOST",
        "VIBEWINDOW_ALLOW_PUBLIC_BIND",
        "VIBEWINDOW_TEMPERATURE",
        "VIBEWINDOW_REASONING_ENABLED",
        "VIBEWINDOW_MODEL_SUPPORT_VISION",
        "VIBEWINDOW_WEB_SEARCH_ENABLED",
        "VIBEWINDOW_WEB_SEARCH_PROVIDER",
        "VIBEWINDOW_BRAVE_API_KEY",
        "VIBEWINDOW_WEB_SEARCH_MAX_RESULTS",
        "VIBEWINDOW_WEB_SEARCH_TIMEOUT_SECS",
    ] {
        unsafe {
            std::env::remove_var(key);
        }
    }
}
