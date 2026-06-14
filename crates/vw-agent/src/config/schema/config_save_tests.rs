use std::path::{Path, PathBuf};

use super::{CONFIG_JSON_FILENAME, Config, config_save::save_config};
use crate::app::agent::security::SecretStore;
use tokio::fs;

fn config_path(root: &Path) -> PathBuf {
    root.join("nested").join(CONFIG_JSON_FILENAME)
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().unwrap().to_string_lossy();
    path.parent().unwrap().join(format!("{file_name}.bak"))
}

async fn read_json(path: &Path) -> serde_json::Value {
    let contents = fs::read_to_string(path).await.unwrap();
    serde_json::from_str(&contents).unwrap()
}

#[tokio::test]
async fn save_config_writes_new_file_with_encrypted_secrets() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = config_path(tmp.path());
    config.default_model = Some("gpt-4o-mini".to_string());
    config.default_provider = Some("openai".to_string());
    config.api_key = Some("sk-plain".to_string());
    config.gateway.paired_tokens = vec!["pair-token".to_string()];

    save_config(&config).await.unwrap();

    let persisted = read_json(&config.config_path).await;
    assert_eq!(persisted["default_model"], "gpt-4o-mini");
    assert_eq!(persisted["default_provider"], "openai");
    assert!(SecretStore::is_encrypted(persisted["api_key"].as_str().unwrap()));
    assert!(SecretStore::is_encrypted(persisted["gateway"]["paired_tokens"][0].as_str().unwrap()));
    assert!(!backup_path(&config.config_path).exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = std::fs::metadata(&config.config_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[tokio::test]
async fn save_config_replaces_existing_payload_and_cleans_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = tmp.path().join(CONFIG_JSON_FILENAME);
    config.default_model = Some("new-model".to_string());
    config.default_provider = Some("new-provider".to_string());
    config.api_key = Some("sk-updated".to_string());

    fs::write(
        &config.config_path,
        serde_json::json!({
            "retained": true,
            "agent": { "legacy": true },
            "model": "old-model",
            "model_provider": "old-provider"
        })
        .to_string(),
    )
    .await
    .unwrap();

    save_config(&config).await.unwrap();

    let persisted = read_json(&config.config_path).await;
    assert_eq!(persisted["retained"], true);
    assert_eq!(persisted["default_model"], "new-model");
    assert_eq!(persisted["default_provider"], "new-provider");
    assert!(persisted.get("agent").is_none());
    assert!(persisted.get("model").is_none());
    assert!(persisted.get("model_provider").is_none());
    assert!(!backup_path(&config.config_path).exists());
}

#[tokio::test]
async fn save_config_removes_backup_after_replacing_large_payload() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = tmp.path().join(CONFIG_JSON_FILENAME);
    config.default_model = Some("replacement".to_string());
    config.enabled_providers = vec!["provider".repeat(100_000)];

    let original = serde_json::json!({ "preserved": "value" });
    fs::write(&config.config_path, original.to_string()).await.unwrap();

    save_config(&config).await.unwrap();

    let persisted = read_json(&config.config_path).await;
    assert_eq!(persisted["default_model"], "replacement");
    assert!(!backup_path(&config.config_path).exists());
}

#[tokio::test]
async fn save_config_rejects_paths_without_a_parent_directory() {
    let mut config = Config::default();
    config.config_path = PathBuf::new();

    let err = save_config(&config).await.unwrap_err();

    assert!(err.to_string().contains("Config path must have a parent directory"));
}
