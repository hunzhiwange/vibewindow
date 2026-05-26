//! # 配置管理模块
//!
//! 本模块负责 Agent 运行时配置的加载、合并、更新和持久化。
//!
//! ## 主要功能
//!
//! - **配置加载与初始化**：从文件系统加载配置，若不存在则创建默认配置
//! - **配置合并**：支持 JSON 格式的增量更新与深度合并
//! - **Provider 管理**：全局 Provider 的添加、删除和更新
//! - **插件去重**：基于名称的插件列表去重逻辑
//!
//! ## 子模块
//!
//! - `schema`：配置数据结构定义（Config 等）
//! - `traits`：配置相关 trait 定义
//!
//! ## 使用示例
//!
//! ```rust
//! use app::agent::config;
//!
//! // 获取当前配置
//! let config = config::get().await;
//!
//! // 更新配置
//! let patch = json!({"model": "gpt-4"});
//! config::update(patch).await?;
//! ```

use crate::app::agent::config::schema::load_or_init_config;
use crate::app::agent::storage;
use crate::app::agent::util::log;
use std::sync::LazyLock;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub mod schema;
pub mod traits;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "traits_tests.rs"]
mod traits_tests;
pub use schema::*;
pub(crate) use schema::{save_config, validate_config};

/// 配置模块专用日志记录器
///
/// 使用 `service: "config"` 标签，便于在日志中识别配置相关操作
static LOGGER: LazyLock<log::Logger> = LazyLock::new(|| {
    log::create(Some({
        let mut m = Map::new();
        m.insert("service".to_string(), Value::String("config".to_string()));
        m
    }))
});

/// 配置操作错误类型
///
/// 封装配置加载、解析、存储过程中可能出现的各类错误
#[derive(Debug)]
pub enum Error {
    /// I/O 错误：文件读取、写入、权限等问题
    Io(std::io::Error),
    /// JSON 解析/序列化错误
    Json(serde_json::Error),
    /// YAML 解析/序列化错误
    Yaml(serde_yaml::Error),
    /// 存储层错误
    Storage(storage::Error),
    /// 配置验证失败或格式无效
    Invalid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Json(e) => write!(f, "{}", e),
            Error::Yaml(e) => write!(f, "{}", e),
            Error::Storage(e) => write!(f, "{}", e),
            Error::Invalid(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::Yaml(value)
    }
}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Error::Storage(value)
    }
}

/// 配置状态容器
///
/// 保存运行时配置快照及相关元数据
#[derive(Debug, Clone, Default)]
pub struct State {
    /// 配置内容的 JSON 表示
    pub config: Value,
    /// 配置文件搜索路径列表
    pub directories: Vec<String>,
}

/// 插件列表去重
///
/// 基于插件名称去除重复项，保留最后一次出现的配置
///
/// # 参数
///
/// - `plugins`: 插件规格字符串列表（可能包含版本号或路径）
///
/// # 返回
///
/// 去重后的插件列表，保持原有顺序（最后出现的优先）
///
/// # 示例
///
/// ```ignore
/// let plugins = vec![
///     "foo@1.0".to_string(),
///     "bar@2.0".to_string(),
///     "foo@3.0".to_string(), // 重复，将保留此版本
/// ];
/// let deduped = deduplicate_plugins(plugins);
/// // 结果: ["bar@2.0", "foo@3.0"]
/// ```
fn deduplicate_plugins(plugins: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut out = Vec::new();
    // 逆序遍历，使最后出现的配置优先
    for spec in plugins.into_iter().rev() {
        let name = plugin_name(&spec);
        // 只有首次出现的名称会被保留
        if seen.insert(name) {
            out.push(spec);
        }
    }
    // 恢复原有顺序
    out.reverse();
    out
}

/// 从插件规格字符串中提取插件名称
///
/// 支持多种格式：
/// - `file:///path/to/plugin.wasm` → 提取文件名（不含扩展名）
/// - `package@version` → 提取包名
/// - `package` → 直接返回
///
/// # 参数
///
/// - `spec`: 插件规格字符串
///
/// # 返回
///
/// 插件名称字符串
///
/// # 示例
///
/// ```ignore
/// assert_eq!(plugin_name("foo@1.0"), "foo");
/// assert_eq!(plugin_name("file:///plugins/my-plugin.wasm"), "my-plugin");
/// assert_eq!(plugin_name("simple"), "simple");
/// ```
fn plugin_name(spec: &str) -> String {
    // 处理文件路径格式：file:///path/to/plugin.wasm
    if let Some(rest) = spec.strip_prefix("file://") {
        let p = rest.trim_start_matches('/');
        let path = Path::new("/").join(p);
        // 提取文件名（去除扩展名）
        return path.file_stem().and_then(|s| s.to_str()).unwrap_or(spec).to_string();
    }
    // 处理版本化格式：package@version
    let last_at = spec.rfind('@');
    if let Some(i) = last_at {
        if i > 0 {
            // 返回 @ 符号之前的部分
            return spec[..i].to_string();
        }
    }
    // 无特殊格式，直接返回原字符串
    spec.to_string()
}

/// 深度合并两个 JSON 值
///
/// 将 `patch` 中的内容合并到 `target` 中，支持：
/// - 对象的递归合并
/// - null 值表示删除字段
/// - 非对象值的直接覆盖
///
/// # 参数
///
/// - `target`: 目标 JSON 值（将被修改）
/// - `patch`: 补丁 JSON 值
///
/// # 合并规则
///
/// 1. 如果两者都是对象，递归合并每个字段
/// 2. 如果 patch 值为 null，删除 target 中对应的字段
/// 3. 其他情况，用 patch 值覆盖 target 值
///
/// # 示例
///
/// ```ignore
/// let mut target = json!({"a": 1, "b": {"c": 2}});
/// let patch = json!({"b": {"d": 3}, "e": 4});
/// merge_json_value(&mut target, patch);
/// // target = {"a": 1, "b": {"c": 2, "d": 3}, "e": 4}
/// ```
fn merge_json_value(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target_obj), Value::Object(patch_obj)) => {
            for (key, patch_value) in patch_obj {
                // null 值表示删除字段
                if patch_value.is_null() {
                    target_obj.remove(&key);
                    continue;
                }

                // 如果字段已存在，递归合并；否则直接插入
                if let Some(current) = target_obj.get_mut(&key) {
                    merge_json_value(current, patch_value);
                } else {
                    target_obj.insert(key, patch_value);
                }
            }
        }
        // 非对象类型：直接覆盖
        (target_value, patch_value) => {
            *target_value = patch_value;
        }
    }
}

/// 获取当前配置
///
/// 尝试加载配置，如果失败则返回默认配置并记录错误日志
///
/// # 返回
///
/// 当前配置实例（加载失败时为默认配置）
///
/// # 示例
///
/// ```ignore
/// let config = config::get().await;
/// println!("Current model: {}", config.model);
/// ```
pub async fn get() -> Config {
    match load_or_init_config().await {
        Ok(config) => config,
        Err(err) => {
            LOGGER.info(
                "load_failed",
                Some({
                    let mut m = Map::new();
                    m.insert("error".to_string(), Value::String(err.to_string()));
                    m
                }),
            );
            Config::default()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_blocking() -> Config {
    match tokio::runtime::Handle::try_current() {
        Ok(_) => std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(1)
                        .enable_all()
                        .build()
                        .map(|runtime| runtime.block_on(get()))
                        .unwrap_or_default()
                })
                .join()
                .unwrap_or_default()
        }),
        Err(_) => match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime.block_on(get()),
            Err(_) => Config::default(),
        },
    }
}

pub(crate) async fn load_from_path_without_env(
    config_path: &Path,
    workspace_dir: PathBuf,
) -> Result<Config, Error> {
    let vibewindow_dir = config_path
        .parent()
        .ok_or_else(|| Error::Invalid("config path must have a parent directory".to_string()))?;

    if let Some(config) = schema::config_load::load_config_from_json_root(
        config_path,
        workspace_dir.clone(),
        vibewindow_dir,
        schema::workspace::ConfigResolutionSource::DefaultConfigDir,
    )
    .await
    .map_err(|error| Error::Invalid(error.to_string()))?
    {
        return Ok(config);
    }

    schema::config_load::load_config_from_toml_path(
        config_path,
        workspace_dir,
        vibewindow_dir,
        schema::workspace::ConfigResolutionSource::DefaultConfigDir,
    )
    .await
    .map_err(|error| Error::Invalid(error.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn load_from_path_without_env_blocking(
    config_path: &Path,
    workspace_dir: PathBuf,
) -> Result<Config, Error> {
    match tokio::runtime::Handle::try_current() {
        Ok(_) => std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(1)
                        .enable_all()
                        .build()
                        .map_err(|error| {
                            Error::Invalid(format!("failed to build runtime: {error}"))
                        })?
                        .block_on(load_from_path_without_env(config_path, workspace_dir))
                })
                .join()
                .unwrap_or_else(|_| Err(Error::Invalid("runtime worker panicked".into())))
        }),
        Err(_) => match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime.block_on(load_from_path_without_env(config_path, workspace_dir)),
            Err(error) => Err(Error::Invalid(format!("failed to build runtime: {error}"))),
        },
    }
}

/// 删除全局 Provider 配置
///
/// 从全局配置中移除指定的 Provider，并保存更新后的配置
///
/// # 参数
///
/// - `provider_id`: Provider 标识符（不能为空）
///
/// # 返回
///
/// - `Ok(())`: 删除成功
/// - `Err(Error)`: 删除失败（provider_id 为空、配置加载失败或保存失败）
///
/// # 错误
///
/// - `Error::Invalid`: provider_id 为空字符串
/// - `Error::Invalid`: 配置加载或保存失败
///
/// # 示例
///
/// ```ignore
/// config::remove_global_provider("openai").await?;
/// ```
pub async fn remove_global_provider(provider_id: &str) -> Result<(), Error> {
    let provider_id = provider_id.trim();
    // 验证 provider_id 不为空
    if provider_id.is_empty() {
        return Err(Error::Invalid("provider_id must not be empty".to_string()));
    }

    let mut config = load_or_init_config().await.map_err(|e| Error::Invalid(e.to_string()))?;
    config.providers.remove(provider_id);
    save_config(&config).await.map_err(|e| Error::Invalid(e.to_string()))
}

/// 更新全局配置（增量合并）
///
/// 将补丁配置深度合并到当前配置中，验证后保存
///
/// # 参数
///
/// - `patch`: JSON 对象形式的配置补丁
///
/// # 返回
///
/// - `Ok(())`: 更新成功
/// - `Err(Error)`: 更新失败
///
/// # 错误
///
/// - `Error::Invalid`: patch 不是 JSON 对象
/// - `Error::Json`: JSON 序列化/反序列化失败
/// - `Error::Invalid`: 配置验证失败或保存失败
///
/// # 合并规则
///
/// 使用 `merge_json_value` 进行深度合并：
/// - 对象递归合并
/// - null 值删除对应字段
/// - 其他值直接覆盖
///
/// # 保留字段
///
/// `workspace_dir` 和 `config_path` 字段不会被 patch 覆盖
///
/// # 示例
///
/// ```ignore
/// let patch = json!({
///     "model": "gpt-4",
///     "temperature": 0.7
/// });
/// config::update_global(patch).await?;
/// ```
pub async fn update_global(patch: Value) -> Result<(), Error> {
    // 验证 patch 必须是对象
    if !patch.is_object() {
        return Err(Error::Invalid("config patch must be a JSON object".to_string()));
    }

    let current = load_or_init_config().await.map_err(|e| Error::Invalid(e.to_string()))?;
    // 将配置转换为 JSON 值
    let mut merged = serde_json::to_value(&current).map_err(Error::Json)?;
    // 深度合并补丁
    merge_json_value(&mut merged, patch);

    // 反序列化回 Config 结构
    let mut next = serde_json::from_value::<Config>(merged).map_err(Error::Json)?;
    // 保留路径字段（不被 patch 覆盖）
    next.workspace_dir = current.workspace_dir;
    next.config_path = current.config_path;
    validate_config(&next).map_err(|e| Error::Invalid(e.to_string()))?;
    save_config(&next).await.map_err(|e| Error::Invalid(e.to_string()))
}

/// 更新配置（update_global 的别名）
///
/// # 参数
///
/// - `patch`: JSON 对象形式的配置补丁
///
/// # 返回
///
/// - `Ok(())`: 更新成功
/// - `Err(Error)`: 更新失败
///
/// # 示例
///
/// ```ignore
/// config::update(json!({"model": "claude-3"})).await?;
/// ```
pub async fn update(patch: Value) -> Result<(), Error> {
    update_global(patch).await
}

/// 获取全局配置（get 的别名）
///
/// # 返回
///
/// 当前配置实例
///
/// # 示例
///
/// ```ignore
/// let config = config::get_global().await;
/// ```
pub async fn get_global() -> Config {
    get().await
}

/// 获取配置文件搜索目录列表
///
/// 当前为占位实现，返回空列表
///
/// # 返回
///
/// 配置文件搜索路径列表（当前为空）
///
/// # 未来规划
///
/// 将返回按优先级排序的配置目录列表，用于多级配置合并
pub async fn directories() -> Vec<String> {
    Vec::new()
}
