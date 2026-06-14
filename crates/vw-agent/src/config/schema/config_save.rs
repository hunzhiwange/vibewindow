use anyhow::{Context, Result};
#[cfg(not(target_arch = "wasm32"))]
use tokio::fs::{self, OpenOptions};
#[cfg(not(target_arch = "wasm32"))]
use tokio::io::AsyncWriteExt;

use vw_config_types::config::Config;

use crate::app::agent::config::schema::CONFIG_JSON_FILENAME;
use crate::app::agent::config::schema::config_io::{read_config_root_json, upsert_config_payload};
use crate::app::agent::config::schema::config_secrets::encrypt_config_secrets;
use crate::app::agent::config::schema::workspace::sync_directory;
use crate::app::agent::security::pairing::{hash_skey, masked_skey_name};

fn normalize_gateway_skeys(config: &mut Config) {
    for entry in &mut config.gateway.skeys {
        if let Some(raw) = entry.skey.take() {
            let raw = raw.trim();
            if !raw.is_empty() {
                entry.skey_hash = hash_skey(raw);
                if entry.name.trim().is_empty() {
                    entry.name = masked_skey_name(raw);
                }
            }
        }
        entry.skey_hash = entry.skey_hash.trim().to_ascii_lowercase();
        entry.name = entry.name.trim().to_string();
        entry.expires_at = entry
            .expires_at
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
    }

    config.gateway.skeys.retain(|entry| !entry.skey_hash.trim().is_empty());
}

pub async fn save_config(config: &Config) -> Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Encrypt secrets before serialization
        let mut config_to_save = config.clone();
        normalize_gateway_skeys(&mut config_to_save);
        encrypt_config_secrets(&mut config_to_save)?;

        let mut root = read_config_root_json(&config.config_path).await?;
        let config_json =
            serde_json::to_value(&config_to_save).context("Failed to serialize config")?;
        upsert_config_payload(&mut root, config_json);
        let json_str = serde_json::to_string_pretty(&root).context("Failed to serialize config")?;

        let parent_dir =
            config.config_path.parent().context("Config path must have a parent directory")?;

        fs::create_dir_all(parent_dir).await.with_context(|| {
            format!("Failed to create config directory: {}", parent_dir.display())
        })?;

        let file_name =
            config.config_path.file_name().and_then(|v| v.to_str()).unwrap_or(CONFIG_JSON_FILENAME);
        let temp_path = parent_dir.join(format!(".{file_name}.tmp-{}", uuid::Uuid::new_v4()));
        let backup_path = parent_dir.join(format!("{file_name}.bak"));

        let mut temp_file =
            OpenOptions::new().create_new(true).write(true).open(&temp_path).await.with_context(
                || format!("Failed to create temporary config file: {}", temp_path.display()),
            )?;
        #[cfg(unix)]
        {
            use std::{fs::Permissions, os::unix::fs::PermissionsExt};
            fs::set_permissions(&temp_path, Permissions::from_mode(0o600)).await.with_context(
                || {
                    format!(
                        "Failed to set secure permissions on temporary config file: {}",
                        temp_path.display()
                    )
                },
            )?;
        }
        temp_file
            .write_all(json_str.as_bytes())
            .await
            .context("Failed to write temporary config contents")?;
        temp_file.sync_all().await.context("Failed to fsync temporary config file")?;
        drop(temp_file);

        let had_existing_config = config.config_path.exists();
        if had_existing_config {
            fs::copy(&config.config_path, &backup_path).await.with_context(|| {
                format!(
                    "Failed to create config backup before atomic replace: {}",
                    backup_path.display()
                )
            })?;
        }

        if let Err(e) = fs::rename(&temp_path, &config.config_path).await {
            let _ = fs::remove_file(&temp_path).await;
            if had_existing_config && backup_path.exists() {
                fs::copy(&backup_path, &config.config_path)
                    .await
                    .context("Failed to restore config backup")?;
            }
            anyhow::bail!("Failed to atomically replace config file: {e}");
        }

        #[cfg(unix)]
        {
            use std::{fs::Permissions, os::unix::fs::PermissionsExt};
            fs::set_permissions(&config.config_path, Permissions::from_mode(0o600))
                .await
                .with_context(|| {
                    format!(
                        "Failed to enforce secure permissions on config file: {}",
                        config.config_path.display()
                    )
                })?;
        }

        sync_directory(parent_dir).await?;

        if had_existing_config {
            let _ = fs::remove_file(&backup_path).await;
        }

        Ok(())
    }
}
