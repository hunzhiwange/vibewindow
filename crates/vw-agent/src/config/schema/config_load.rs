//! 配置加载与首次初始化流程。
//!
//! 本模块负责在运行时解析配置目录、读取 JSON 或旧 TOML 配置、处理兼容迁移、解密密钥、
//! 展开环境变量引用并执行最终校验。加载失败会显式返回错误，避免 agent 在部分配置状态下启动。

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use tokio::fs;
use vw_config_types::config::Config;

use crate::app::agent::config::schema::CONFIG_JSON_FILENAME;
use crate::app::agent::config::schema::config_env::apply_env_overrides;
use crate::app::agent::config::schema::config_helpers::config_dir_creation_error;
use crate::app::agent::config::schema::config_io::{
    extract_config_payload, normalize_legacy_alias_conflicts, normalize_top_level_table_aliases,
    read_config_root_json,
};
use crate::app::agent::config::schema::config_save::save_config;
use crate::app::agent::config::schema::config_secrets::decrypt_config_secrets;
use crate::app::agent::config::schema::config_validate::validate_config;
use crate::app::agent::config::schema::resolve_telegram_allowed_users_env_refs;
use crate::app::agent::config::schema::workspace::ConfigResolutionSource;
use crate::app::agent::config::schema::{
    default_config_and_workspace_dirs, resolve_runtime_config_dirs,
};

/// 返回当前配置目录下的 JSON 配置文件路径。
///
/// 参数 `vibewindow_dir` 是已经解析好的 VibeWindow 配置目录；函数只拼接文件名，不访问文件系统。
pub(crate) fn config_json_path(vibewindow_dir: &Path) -> PathBuf {
    vibewindow_dir.join(CONFIG_JSON_FILENAME)
}

/// 从旧 TOML 配置文件加载完整配置。
///
/// `config_path` 指向 TOML 文件，`workspace_dir` 和 `vibewindow_dir` 分别提供运行时 workspace 与密钥目录。
/// 加载过程会归一化历史表名、记录未知键、解密密钥、解析 Telegram 环境变量引用、应用环境变量覆盖并校验配置。
///
/// WASM 目标不支持本地 TOML 文件读取，会直接返回不支持错误；桌面/服务端目标上的读取、解析、解密或校验失败
/// 都会返回带上下文的错误。
pub(crate) async fn load_config_from_toml_path(
    config_path: &Path,
    workspace_dir: PathBuf,
    vibewindow_dir: &Path,
    resolution_source: ConfigResolutionSource,
) -> Result<Config> {
    #[cfg(target_arch = "wasm32")]
    anyhow::bail!("Not supported in WASM");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let contents =
            fs::read_to_string(config_path).await.context("Failed to read config file")?;
        let mut raw_toml: toml::Value =
            toml::from_str(&contents).context("Failed to parse config file")?;
        normalize_top_level_table_aliases(&mut raw_toml);
        let normalized_contents =
            toml::to_string(&raw_toml).context("Failed to normalize config file")?;

        let mut ignored_paths: Vec<String> = Vec::new();
        let deserializer = toml::Deserializer::new(&normalized_contents);
        // 使用 serde_ignored 保持向后兼容：未知键不阻断启动，但必须记录，方便用户发现拼写错误。
        let mut config: Config = serde_ignored::deserialize(deserializer, |path| {
            ignored_paths.push(path.to_string());
        })
        .context("Failed to deserialize config file")?;

        for path in ignored_paths {
            tracing::warn!(
                "Unknown config key ignored: \"{}\". Check vibewindow.json for typos or deprecated options.",
                path
            );
        }
        config.config_path = config_path.to_path_buf();
        config.workspace_dir = workspace_dir;
        decrypt_config_secrets(&mut config, vibewindow_dir)?;
        resolve_telegram_allowed_users_env_refs(&mut config.channels_config)?;
        apply_env_overrides(&mut config);
        validate_config(&config)?;
        tracing::info!(
            path = %config.config_path.display(),
            workspace = %config.workspace_dir.display(),
            source = resolution_source.as_str(),
            initialized = false,
            "Config loaded"
        );
        Ok(config)
    }
}

/// 从 JSON 根对象加载配置。
///
/// 返回 `Ok(None)` 表示配置文件不存在或为空，调用方可以继续初始化默认配置。返回 `Ok(Some(config))`
/// 表示已经完成反序列化、旧别名归一化、密钥解密、环境变量覆盖和配置校验。
///
/// WASM 目标不读取本地文件，因此始终返回 `Ok(None)`；非 WASM 目标上，读取、解析、解密或校验失败会返回错误。
pub(crate) async fn load_config_from_json_root(
    config_path: &Path,
    workspace_dir: PathBuf,
    vibewindow_dir: &Path,
    resolution_source: ConfigResolutionSource,
) -> Result<Option<Config>> {
    #[cfg(target_arch = "wasm32")]
    return Ok(None);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = read_config_root_json(config_path).await?;
        let Some(mut config_payload) = extract_config_payload(&root) else {
            return Ok(None);
        };

        normalize_legacy_alias_conflicts(&mut config_payload);
        let config_str =
            serde_json::to_string(&config_payload).context("Failed to serialize config file")?;
        let mut ignored_paths: Vec<String> = Vec::new();
        let mut deserializer = serde_json::Deserializer::from_str(&config_str);
        // 未知字段只警告不失败，兼顾老版本配置和手写配置；真正危险或无效的值交给 validate_config 拒绝。
        let mut config: Config = serde_ignored::deserialize(&mut deserializer, |path| {
            ignored_paths.push(path.to_string());
        })
        .context("Failed to deserialize config file")?;

        for path in ignored_paths {
            tracing::warn!(
                "Unknown config key ignored: \"{}\". Check vibewindow.json for typos or deprecated options.",
                path
            );
        }
        config.config_path = config_path.to_path_buf();
        config.workspace_dir = workspace_dir;
        decrypt_config_secrets(&mut config, vibewindow_dir)?;
        resolve_telegram_allowed_users_env_refs(&mut config.channels_config)?;
        apply_env_overrides(&mut config);
        validate_config(&config)?;
        tracing::info!(
            path = %config.config_path.display(),
            workspace = %config.workspace_dir.display(),
            source = resolution_source.as_str(),
            initialized = false,
            "Config loaded"
        );
        Ok(Some(config))
    }
}

/// 加载现有配置，或在首次运行时创建默认配置。
///
/// 非 WASM 环境会解析默认与运行时目录，优先读取当前 JSON 配置；当发现旧 `vibewindow.json` TOML 配置时会加载并
/// 保存为当前 JSON 路径。新建配置会尝试设置为仅当前用户可读写。
///
/// 返回完整可用的 `Config`。目录创建、文件读写、密钥处理或校验失败都会返回错误；WASM 目标直接返回默认配置。
pub async fn load_or_init_config() -> Result<Config> {
    #[cfg(target_arch = "wasm32")]
    {
        Ok(Config::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (default_vibewindow_dir, default_workspace_dir) = default_config_and_workspace_dirs()?;

        let (vibewindow_dir, workspace_dir, resolution_source) =
            resolve_runtime_config_dirs(&default_vibewindow_dir, &default_workspace_dir).await?;

        let config_path = config_json_path(&vibewindow_dir);
        let legacy_config_path = vibewindow_dir.join("vibewindow.json");

        fs::create_dir_all(&vibewindow_dir)
            .await
            .with_context(|| config_dir_creation_error(&vibewindow_dir))?;
        fs::create_dir_all(&workspace_dir).await.context("Failed to create workspace directory")?;

        if config_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // 配置中可能保存 provider 密钥引用；世界可读不会阻断启动，但必须提示用户收紧权限。
                if let Ok(meta) = fs::metadata(&config_path).await {
                    if meta.permissions().mode() & 0o004 != 0 {
                        tracing::warn!(
                            "Config file {:?} is world-readable (mode {:o}). \
                             Consider restricting with: chmod 600 {:?}",
                            config_path,
                            meta.permissions().mode() & 0o777,
                            config_path,
                        );
                    }
                }
            }

            if let Some(config) = load_config_from_json_root(
                &config_path,
                workspace_dir.clone(),
                &vibewindow_dir,
                resolution_source,
            )
            .await?
            {
                return Ok(config);
            }

            if legacy_config_path.exists() {
                // JSON 文件存在但为空时，仍允许从旧 TOML 迁移，避免升级后生成空文件导致旧配置被忽略。
                let mut config = load_config_from_toml_path(
                    &legacy_config_path,
                    workspace_dir.clone(),
                    &vibewindow_dir,
                    resolution_source,
                )
                .await?;
                config.config_path = config_path.clone();
                save_config(&config).await?;
                return Ok(config);
            }
        } else if legacy_config_path.exists() {
            let mut config = load_config_from_toml_path(
                &legacy_config_path,
                workspace_dir.clone(),
                &vibewindow_dir,
                resolution_source,
            )
            .await?;
            config.config_path = config_path.clone();
            save_config(&config).await?;
            return Ok(config);
        }

        let mut config = Config::default();
        config.config_path = config_path.clone();
        config.workspace_dir = workspace_dir;
        save_config(&config).await?;

        #[cfg(unix)]
        {
            use std::{fs::Permissions, os::unix::fs::PermissionsExt};
            let _ = fs::set_permissions(&config_path, Permissions::from_mode(0o600)).await;
        }

        apply_env_overrides(&mut config);
        validate_config(&config)?;
        tracing::info!(
            path = %config.config_path.display(),
            workspace = %config.workspace_dir.display(),
            source = resolution_source.as_str(),
            initialized = true,
            "Config loaded"
        );
        Ok(config)
    }
}
