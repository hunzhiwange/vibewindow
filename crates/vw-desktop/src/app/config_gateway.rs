use crate::app::{Message, message::SettingsMessage};
use iced::Task;
#[cfg(not(target_arch = "wasm32"))]
use serde::Deserialize;
use serde::de::DeserializeOwned;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
use vw_config_types::{config::Config, security::IdentityConfig};
use vw_gateway_client::{GatewayClient, GatewayEndpoint};

use super::system_settings::{
    load_gateway_client_bootstrap_config, save_gateway_client_bootstrap_config,
};

fn normalize_identity_format(raw: Option<&str>) -> String {
    let _ = raw;
    "openclaw".to_string()
}

fn normalize_gateway_host(raw: &str) -> String {
    let value = raw.trim();
    if value.is_empty() {
        "127.0.0.1".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct GatewayPairCodeResponse {
    require_pairing: bool,
    paired: bool,
    pairing_code: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
struct GatewayPairResponse {
    token: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn is_loopback_host(host: &str) -> bool {
    matches!(host.trim().to_ascii_lowercase().as_str(), "127.0.0.1" | "localhost" | "::1")
}

#[cfg(not(target_arch = "wasm32"))]
fn gateway_bootstrap_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn endpoint_has_gateway_auth(endpoint: &GatewayEndpoint) -> bool {
    endpoint.auth.as_ref().is_some_and(|auth| {
        auth.bearer_token.as_deref().is_some_and(|value| !value.trim().is_empty())
            || auth.password.as_deref().is_some_and(|value| !value.trim().is_empty())
            || auth.skey.as_deref().is_some_and(|value| !value.trim().is_empty())
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn fetch_gateway_pair_code_response(
    endpoint: &GatewayEndpoint,
) -> Result<GatewayPairCodeResponse, String> {
    let pair_code_url = format!("{}/v1/pair-code", endpoint.base_url());

    run_gateway_call(async {
        let client = gateway_bootstrap_http_client()?;
        let response = client.get(&pair_code_url).send().await.map_err(|err| err.to_string())?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("status={status} body={}", body.trim()));
        }
        response.json::<GatewayPairCodeResponse>().await.map_err(|err| err.to_string())
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn should_attempt_tools_list_request(endpoint: &GatewayEndpoint) -> bool {
    if endpoint_has_gateway_auth(endpoint) || !is_loopback_host(endpoint.normalized_host()) {
        return true;
    }

    match fetch_gateway_pair_code_response(endpoint) {
        Ok(pairing) if !pairing.require_pairing => true,
        Ok(pairing) => {
            tracing::debug!(
                target: "vw_desktop",
                endpoint = %endpoint.describe(),
                paired = pairing.paired,
                "skipping tools list load until gateway client authentication is available"
            );
            false
        }
        Err(err) => {
            tracing::debug!(
                target: "vw_desktop",
                endpoint = %endpoint.describe(),
                error = %err,
                "skipping tools list load because gateway pairing state is unavailable without credentials"
            );
            false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn maybe_auto_pair_gateway_client(cfg: &mut vw_config_types::ui::GatewayClientSystemSettingsConfig) {
    if !cfg.bearer_token.trim().is_empty()
        || !cfg.username.trim().is_empty()
        || !cfg.password.trim().is_empty()
        || !cfg.skey.trim().is_empty()
        || !is_loopback_host(&cfg.host)
    {
        return;
    }

    let host = normalize_gateway_host(&cfg.host);
    let endpoint = GatewayEndpoint::new(host, cfg.port.clamp(1, u16::MAX));
    let pair_code_response = match fetch_gateway_pair_code_response(&endpoint) {
        Ok(response) => response,
        Err(err) => {
            tracing::warn!(target: "vw_desktop", error = %err, endpoint = %endpoint.describe(), "failed to fetch gateway pairing bootstrap state");
            return;
        }
    };

    if !pair_code_response.require_pairing {
        return;
    }

    let Some(pairing_code) = pair_code_response
        .pairing_code
        .filter(|value| !value.trim().is_empty())
    else {
        tracing::warn!(target: "vw_desktop", endpoint = %endpoint.describe(), "gateway requires pairing but did not expose a usable loopback pairing code");
        return;
    };

    let pair_url = format!("{}/v1/pair", endpoint.base_url());
    let pair_response = match run_gateway_call(async {
        let client = gateway_bootstrap_http_client()?;
        let response = client
            .post(&pair_url)
            .header("X-Pairing-Code", pairing_code)
            .send()
            .await
            .map_err(|err| err.to_string())?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("status={status} body={}", body.trim()));
        }
        response.json::<GatewayPairResponse>().await.map_err(|err| err.to_string())
    }) {
        Ok(response) => response,
        Err(err) => {
            tracing::warn!(target: "vw_desktop", error = %err, endpoint = %endpoint.describe(), "failed to pair desktop gateway client automatically");
            return;
        }
    };

    let Some(token) = pair_response.token.filter(|value| !value.trim().is_empty()) else {
        tracing::warn!(target: "vw_desktop", endpoint = %endpoint.describe(), "automatic gateway pairing succeeded without a bearer token payload");
        return;
    };

    cfg.bearer_token = token;
    save_gateway_client_bootstrap_config(cfg);
}

pub(super) fn apply_main_agent_overrides(config: &mut Config) {
    let Some(main) = config.agents.get("main") else {
        return;
    };

    let provider = main.provider.trim();
    if !provider.is_empty() {
        config.default_provider = Some(provider.to_string());
    }

    let model = main.model.trim();
    if !provider.is_empty() && !model.is_empty() {
        config.default_model = Some(format!("{provider}/{model}"));
    }

    if let Some(temperature) = main.temperature {
        config.default_temperature = temperature;
    }

    if main.identity_format.is_some() {
        config.identity = IdentityConfig {
            format: normalize_identity_format(main.identity_format.as_deref()),
            aieos_path: None,
            aieos_inline: None,
        };
    }
}

fn vibewindow_home_config_path() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        if let Some(home) = std::env::var_os("USERPROFILE") {
            return Some(
                std::path::PathBuf::from(home).join(".vibewindow").join("vibewindow.json"),
            );
        }
    }
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .map(|home| home.join(".vibewindow").join("vibewindow.json"))
}

fn vibewindow_config_path() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("VIBEWINDOW_CONFIG") {
        let path = std::path::PathBuf::from(path);
        if path.is_file() || path.extension().is_some() {
            return Some(path);
        }
    }
    if let Some(dir) = std::env::var_os("VIBEWINDOW_CONFIG_DIR") {
        return Some(std::path::PathBuf::from(dir).join("vibewindow.json"));
    }
    if let Some(home_path) = vibewindow_home_config_path() {
        return Some(home_path);
    }
    crate::app::project_dirs().map(|d| d.config_dir().join("vibewindow.json"))
}

fn vibewindow_legacy_config_path() -> Option<std::path::PathBuf> {
    crate::app::project_dirs().map(|d| d.config_dir().join("vibewindow.json"))
}

fn load_vibewindow_root_json() -> serde_json::Value {
    let mut candidates = Vec::new();
    if let Some(home) = vibewindow_home_config_path() {
        candidates.push(home);
    }
    if let Some(primary) = vibewindow_config_path()
        && !candidates.iter().any(|p| p == &primary)
    {
        candidates.push(primary);
    }
    if let Some(legacy) = vibewindow_legacy_config_path()
        && !candidates.iter().any(|p| p == &legacy)
    {
        candidates.push(legacy);
    }

    for path in candidates {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            return v;
        }
    }

    serde_json::json!({})
}

pub(super) fn set_config_value_at_path(
    root: &mut serde_json::Value,
    path: &[&str],
    value: serde_json::Value,
) {
    if path.is_empty() {
        *root = value;
        return;
    }

    if !root.is_object() {
        *root = serde_json::json!({});
    }

    let mut current = root;
    for key in &path[..path.len() - 1] {
        if current.get(*key).and_then(|v| v.as_object()).is_none()
            && let Some(obj) = current.as_object_mut()
        {
            obj.insert((*key).to_string(), serde_json::json!({}));
        }
        current = current
            .as_object_mut()
            .and_then(|obj| obj.get_mut(*key))
            .expect("path object exists");
    }

    if let Some(obj) = current.as_object_mut() {
        obj.insert(path[path.len() - 1].to_string(), value);
    }
}

pub(super) fn load_config_value_at_path<T: DeserializeOwned>(path: &[&str]) -> Option<T> {
    let root = load_vibewindow_root_json();
    let mut current = &root;
    for key in path {
        current = current.get(*key)?;
    }
    serde_json::from_value::<T>(current.clone()).ok()
}

pub fn server_config_unreachable_error(err: impl AsRef<str>) -> String {
    let detail = err.as_ref().trim();
    if detail.is_empty() {
        "服务端配置不可达，请检查 Gateway 连接状态。".to_string()
    } else {
        format!("服务端配置不可达，请检查 Gateway 连接状态。{detail}")
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_gateway_task(
    tag: &'static str,
    future: impl std::future::Future<Output = Result<(), String>> + 'static,
) -> Task<Message> {
    Task::perform(async move { future.await }, move |result| {
        Message::Settings(SettingsMessage::AgentConfigSaved { tag, result })
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_gateway_task(
    tag: &'static str,
    future: impl std::future::Future<Output = Result<(), String>> + Send + 'static,
) -> Task<Message> {
    Task::perform(future, move |result| {
        Message::Settings(SettingsMessage::AgentConfigSaved { tag, result })
    })
}

pub fn gateway_client_endpoint() -> GatewayEndpoint {
    let mut cfg = load_gateway_client_bootstrap_config();
    #[cfg(not(target_arch = "wasm32"))]
    maybe_auto_pair_gateway_client(&mut cfg);
    let host = normalize_gateway_host(&cfg.host);
    let auth = vw_gateway_client::GatewayAuth {
        bearer_token: Some(cfg.bearer_token.trim().to_string()).filter(|value| !value.is_empty()),
        username: Some(cfg.username.trim().to_string()).filter(|value| !value.is_empty()),
        password: Some(cfg.password.trim().to_string()).filter(|value| !value.is_empty()),
        skey: Some(cfg.skey.trim().to_string()).filter(|value| !value.is_empty()),
    };
    GatewayEndpoint::new(host, cfg.port.clamp(1, u16::MAX)).with_auth(auth)
}

pub fn gateway_client() -> Result<GatewayClient, String> {
    GatewayClient::new(gateway_client_endpoint())
}

fn normalize_tool_ids(mut tools: Vec<String>) -> Vec<String> {
    tools.retain(|tool_id| !tool_id.trim().is_empty());
    tools.sort();
    tools.dedup();
    tools
}

pub fn load_tools_list_via_gateway() -> Vec<String> {
    let endpoint = gateway_client_endpoint();

    #[cfg(not(target_arch = "wasm32"))]
    if !should_attempt_tools_list_request(&endpoint) {
        return Vec::new();
    }

    let client = match GatewayClient::new(endpoint) {
        Ok(client) => client,
        Err(err) => {
            tracing::warn!(target: "vw_desktop", error = %err, "gateway client unavailable for tools list");
            return Vec::new();
        }
    };

    let result = run_gateway_call(async { client.tools_list().await });

    match result {
        Ok(tools) => normalize_tool_ids(tools),
        Err(err) => {
            tracing::warn!(target: "vw_desktop", error = %err, "failed to load tools list via gateway");
            Vec::new()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn gateway_blocking_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();

    match RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .max_blocking_threads(8)
            .enable_all()
            .build()
            .map_err(|err| err.to_string())
    }) {
        Ok(runtime) => Ok(runtime),
        Err(err) => Err(err.clone()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn run_gateway_call<T: Send>(
    future: impl std::future::Future<Output = Result<T, String>> + Send,
) -> Result<T, String> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle)
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread =>
        {
            tokio::task::block_in_place(|| handle.block_on(future))
        }
        Ok(_) => std::thread::scope(|scope| {
            scope
                .spawn(|| gateway_blocking_runtime()?.block_on(future))
                .join()
                .unwrap_or_else(|_| Err("gateway blocking thread panicked".to_string()))
        }),
        Err(_) => gateway_blocking_runtime()?.block_on(future),
    }
}

#[cfg(target_arch = "wasm32")]
/// WASM keeps this shim only for legacy synchronous load sites and returns
/// `Default` immediately; synchronous save flows now run through
/// `spawn_gateway_task` and the async gateway APIs.
pub(super) fn run_gateway_call<T: Default>(
    _future: impl std::future::Future<Output = Result<T, String>>,
) -> Result<T, String> {
    tracing::warn!(target: "vw_desktop", "sync gateway call on wasm, returning default");
    Ok(T::default())
}

#[cfg(test)]
mod tests {
    use super::normalize_tool_ids;

    #[test]
    fn normalize_tool_ids_sorts_and_dedups() {
        let tools = normalize_tool_ids(vec![
            "bash".to_string(),
            String::new(),
            "file_read".to_string(),
            "bash".to_string(),
        ]);

        assert_eq!(tools, vec!["bash".to_string(), "file_read".to_string()]);
    }
}
