use std::ffi::OsStr;
use std::sync::{LazyLock, Mutex, MutexGuard};

use tempfile::TempDir;
use vw_config_types::ui::{AppSystemSettingsConfig, GatewayClientSystemSettingsConfig};

const SOURCE: &str = include_str!("config_system_settings.rs");
static CONFIG_SYSTEM_SETTINGS_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set_os(key: &'static str, value: &OsStr) -> Self {
        let previous = std::env::var_os(key);
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }

    fn remove(key: &'static str) -> Self {
        let previous = std::env::var_os(key);
        unsafe { std::env::remove_var(key) };
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn env_lock() -> MutexGuard<'static, ()> {
    CONFIG_SYSTEM_SETTINGS_ENV_LOCK.lock().expect("env lock should not be poisoned")
}

fn temp_home() -> (MutexGuard<'static, ()>, TempDir, EnvGuard) {
    let lock = env_lock();
    let home = tempfile::tempdir().expect("temp home should be created");
    let guard = EnvGuard::set_os("HOME", home.path().as_os_str());
    (lock, home, guard)
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

fn gateway_config(host: &str, port: u16) -> GatewayClientSystemSettingsConfig {
    GatewayClientSystemSettingsConfig {
        host: host.to_string(),
        port,
        ..GatewayClientSystemSettingsConfig::default()
    }
}

#[test]
fn config_system_settings_tests_keeps_planned_coverage_targets() {
    for name in [
        "fetch_desktop_system_settings_via_gateway",
        "patch_desktop_system_settings_via_gateway",
        "load_legacy_system_settings_config_local",
        "normalize_system_settings_config",
        "gateway_client_bootstrap_cache_path",
        "load_gateway_client_bootstrap_config",
        "save_gateway_client_bootstrap_config",
        "load_gateway_client_config",
        "update_gateway_client_config",
        "load_system_settings_config_async",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn normalize_system_settings_config_replaces_invalid_font_size() {
    let cfg = AppSystemSettingsConfig {
        editor_font_size: f32::NAN,
        editor_line_height: 99.0,
        editor_auto_line_height: false,
        ..AppSystemSettingsConfig::default()
    };

    let normalized = super::normalize_system_settings_config(cfg);

    assert_eq!(normalized.editor_font_size, 14.0);
    assert_eq!(normalized.editor_line_height, 99.0);
}

#[test]
fn normalize_system_settings_config_auto_line_height_tracks_font_size() {
    let cfg = AppSystemSettingsConfig {
        editor_font_size: 15.0,
        editor_line_height: 1.0,
        editor_auto_line_height: true,
        ..AppSystemSettingsConfig::default()
    };

    let normalized = super::normalize_system_settings_config(cfg);

    assert_eq!(normalized.editor_font_size, 15.0);
    assert_eq!(normalized.editor_line_height, 21.0);
}

#[test]
fn normalize_system_settings_config_replaces_invalid_manual_line_height() {
    for editor_line_height in [0.0, -1.0, f32::INFINITY] {
        let cfg = AppSystemSettingsConfig {
            editor_font_size: 16.0,
            editor_line_height,
            editor_auto_line_height: false,
            ..AppSystemSettingsConfig::default()
        };

        let normalized = super::normalize_system_settings_config(cfg);

        assert_eq!(normalized.editor_font_size, 16.0);
        assert_eq!(normalized.editor_line_height, 20.0);
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn gateway_client_bootstrap_cache_path_uses_vibewindow_home_file() {
    let (_lock, home, _guard) = temp_home();

    let path = super::gateway_client_bootstrap_cache_path().expect("path should exist");

    assert_eq!(
        path,
        vw_config_types::paths::home_config_dir(home.path()).join("gateway-client-bootstrap.json")
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn save_gateway_client_bootstrap_config_writes_loadable_json() {
    let (_lock, home, _guard) = temp_home();
    let config = gateway_config("10.0.0.8", 49152);

    super::save_gateway_client_bootstrap_config(&config);

    let path =
        vw_config_types::paths::home_config_dir(home.path()).join("gateway-client-bootstrap.json");
    let saved = std::fs::read_to_string(path).expect("bootstrap json should be written");
    assert!(saved.contains("10.0.0.8"));
    assert_eq!(super::load_gateway_client_bootstrap_config(), config);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn load_gateway_client_bootstrap_config_ignores_invalid_json() {
    let (_lock, home, _guard) = temp_home();
    let cache_dir = vw_config_types::paths::home_config_dir(home.path());
    std::fs::create_dir_all(&cache_dir).expect("cache dir should be created");
    std::fs::write(cache_dir.join("gateway-client-bootstrap.json"), "{not-json")
        .expect("invalid cache should be written");

    let config = super::load_gateway_client_bootstrap_config();

    assert_eq!(config, GatewayClientSystemSettingsConfig::default());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn load_gateway_client_config_returns_bootstrap_cache() {
    let (_lock, _home, _guard) = temp_home();
    let config = gateway_config("gateway.internal", 9090);
    super::save_gateway_client_bootstrap_config(&config);

    let loaded = super::load_gateway_client_config();

    assert_eq!(loaded, config);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn load_gateway_client_bootstrap_config_falls_back_to_legacy_system_settings() {
    let lock = env_lock();
    let home = tempfile::tempdir().expect("temp home should be created");
    let config_file = home.path().join("legacy-vibewindow.json");
    let legacy = gateway_config("legacy.gateway", 7777);
    std::fs::write(
        &config_file,
        serde_json::json!({
            "app_ui": {
                "system_settings": {
                    "gateway_client": legacy
                }
            }
        })
        .to_string(),
    )
    .expect("legacy config should be written");
    let _home = EnvGuard::set_os("HOME", home.path().as_os_str());
    let _config = EnvGuard::set_os("VIBEWINDOW_CONFIG", config_file.as_os_str());
    let _config_dir = EnvGuard::remove("VIBEWINDOW_CONFIG_DIR");

    let loaded = super::load_gateway_client_bootstrap_config();

    assert_eq!(loaded.host, "legacy.gateway");
    assert_eq!(loaded.port, 7777);
    drop(lock);
}
