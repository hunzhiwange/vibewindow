//! Provider 相关配置读取与网关端点解析。
//!
//! 本模块负责：
//! - 定位 `vibewindow.json` 配置文件
//! - 在本地平台读取配置文件内容
//! - 在 wasm 平台通过网关读取配置
//! - 提取 provider 覆盖项与默认模型配置
//! - 推导浏览器环境下的默认网关端点

use directories::UserDirs;
#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
use vw_config_types::config::Config;
#[cfg(target_arch = "wasm32")]
use vw_gateway_client::{GatewayClient, GatewayEndpoint};

/// 配置文件名。
const CONFIG_JSON_FILENAME: &str = "vibewindow.json";

#[cfg(target_arch = "wasm32")]
static WASM_GATEWAY_ENDPOINT: Lazy<Mutex<Option<GatewayEndpoint>>> = Lazy::new(|| Mutex::new(None));

/// 解析默认配置目录，优先使用测试或外部显式指定的目录。
///
/// 若未显式指定，则默认落在用户主目录下的活跃 VibeWindow 配置目录。
fn default_config_dir() -> Option<PathBuf> {
    std::env::var_os("VIBEWINDOW_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| UserDirs::new().map(|u| vw_config_types::paths::home_config_dir(u.home_dir())))
}

/// 返回配置文件路径。
///
/// # 返回值
///
/// 成功时返回配置文件绝对路径；若当前环境无法解析用户目录则返回 `None`
pub fn config_path() -> Option<PathBuf> {
    default_config_dir().map(|d| d.join(CONFIG_JSON_FILENAME))
}

#[cfg(not(target_arch = "wasm32"))]
/// 从本地配置文件读取配置；读取失败时回退到默认配置。
///
/// 该函数会对以下场景统一回退：
/// - 配置路径不可用
/// - 配置文件不存在
/// - 配置文件为空
/// - JSON 反序列化失败
pub async fn get() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(contents) = tokio::fs::read_to_string(&path).await else {
        return Config::default();
    };
    if contents.trim().is_empty() {
        return Config::default();
    }
    match serde_json::from_str::<Config>(&contents) {
        Ok(config) => config,
        Err(err) => {
            tracing::warn!(path = %path.display(), error = %err, "Failed to parse config file, using defaults");
            Config::default()
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// 通过网关读取配置；读取失败时回退到默认配置。
pub async fn get() -> Config {
    let Ok(client) = gateway_client() else {
        return Config::default();
    };
    let Ok(value) = client.config_get(None).await else {
        return Config::default();
    };
    serde_json::from_value(value).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
/// 显式设置 wasm 环境下的网关端点。
///
/// 当宿主环境已知网关地址时，可通过该函数覆盖浏览器地址推导逻辑。
pub fn set_wasm_gateway_endpoint(endpoint: GatewayEndpoint) {
    if let Ok(mut slot) = WASM_GATEWAY_ENDPOINT.lock() {
        *slot = Some(endpoint);
    }
}

#[cfg(target_arch = "wasm32")]
/// 基于当前网关端点创建客户端。
pub fn gateway_client() -> Result<GatewayClient, String> {
    GatewayClient::new(current_wasm_gateway_endpoint())
}

#[cfg(target_arch = "wasm32")]
/// 获取 wasm 环境当前应使用的网关端点。
///
/// 优先使用显式设置的端点；若不存在，则回退到浏览器地址推导结果。
fn current_wasm_gateway_endpoint() -> GatewayEndpoint {
    if let Ok(slot) = WASM_GATEWAY_ENDPOINT.lock()
        && let Some(endpoint) = slot.clone()
    {
        return endpoint;
    }

    browser_location_gateway_endpoint()
}

#[cfg(target_arch = "wasm32")]
/// 根据浏览器地址推导默认网关端点。
///
/// 当页面运行在 `https` 上且 URL 未显式携带端口时，默认使用 `443`；
/// 其他情况下默认使用 `80`。
fn browser_location_gateway_endpoint() -> GatewayEndpoint {
    let Some(window) = web_sys::window() else {
        return GatewayEndpoint::new("127.0.0.1", 42617);
    };
    let location = window.location();
    let host = location.hostname().ok().filter(|value| !value.trim().is_empty());
    let protocol = location.protocol().ok().unwrap_or_default();
    let port = location
        .port()
        .ok()
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed.to_string())
        })
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(if protocol == "https:" { 443 } else { 80 });

    GatewayEndpoint::new(host.unwrap_or_else(|| "127.0.0.1".to_string()), port)
}

#[cfg(not(target_arch = "wasm32"))]
/// 在同步上下文中读取配置，必要时自行创建运行时。
///
/// 适合供无法直接 `await` 的同步调用点使用。
pub fn get_blocking() -> Config {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(get())),
        Err(_) => match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime.block_on(get()),
            Err(_) => Config::default(),
        },
    }
}

#[cfg(target_arch = "wasm32")]
/// wasm 环境下不支持阻塞读取，统一返回默认配置。
pub fn get_blocking() -> Config {
    Config::default()
}

/// 读取配置中的 provider 覆盖项，并转换为 JSON 对象列表。
///
/// 返回值保留 provider_id 与其对应的原始 JSON 对象，便于后续按需合并字段。
pub async fn load_provider_overrides() -> Vec<(String, serde_json::Value)> {
    get()
        .await
        .providers
        .into_iter()
        .filter_map(|(provider_id, provider_cfg)| {
            provider_cfg
                .as_object()
                .map(|obj| (provider_id, serde_json::Value::Object(obj.clone())))
        })
        .collect()
}

/// 读取配置中的默认模型标识。
pub async fn read_default_model() -> Option<String> {
    get().await.default_model
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
