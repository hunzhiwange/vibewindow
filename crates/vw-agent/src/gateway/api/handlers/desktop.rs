//! Desktop 专用网关接口。
//!
//! 本模块集中承载桌面端偏好、工具草稿内容、思维导图标签页、项目偏好以及
//! 外部应用打开/文件管理器定位等轻量 API。这里的处理器只负责参数校验、
//! 存储读写和平台命令分发，不承载业务策略，避免桌面 UI 与底层 agent 逻辑耦合。

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app::agent::gateway::ApiError;
use crate::storage;

const TOOL_CONTENT_TYPES: &[&str] = &["json", "sql", "html"];

/// 构建 desktop API 路由表。
///
/// # 返回值
///
/// 返回挂载了桌面偏好、工具内容、技能、外部应用与项目偏好端点的 Axum 路由。
///
/// # 错误处理
///
/// 本函数只登记路由，不执行 IO；各端点在自身处理器中将存储或平台命令错误映射为
/// [`ApiError`]。
pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/desktop/preferences", get(preferences_get).patch(preferences_patch))
        .route("/desktop/tool-content/{tool_type}", get(tool_content_get).put(tool_content_put))
        .route("/desktop/mindmap-tabs", get(mindmap_tabs_get).put(mindmap_tabs_put))
        .route("/desktop/skills", get(super::desktop_skills::catalog_get))
        .route("/desktop/skills/detail", get(super::desktop_skills::detail_get))
        .route("/desktop/skills/create", post(super::desktop_skills::create_post))
        .route(
            "/desktop/skills/install-built-in",
            post(super::desktop_skills::install_builtin_post),
        )
        .route("/desktop/skills/set-enabled", post(super::desktop_skills::set_enabled_post))
        .route("/desktop/skills/delete", post(super::desktop_skills::delete_post))
        .merge(super::desktop_cleaner::router())
        .route("/desktop/external-apps", get(external_apps_get))
        .route("/desktop/external-apps/open", post(external_apps_open_post))
        .route("/desktop/external-path/reveal", post(external_path_reveal_post))
        .route(
            "/desktop/project-preferences",
            get(project_preferences_get).put(project_preferences_put),
        )
}

async fn preferences_get() -> Result<Json<Value>, ApiError> {
    let value: Value = storage::read(&["desktop", "preferences"]).await.unwrap_or_default();
    Ok(Json(value))
}

async fn preferences_patch(Json(patch): Json<Value>) -> Result<Json<Value>, ApiError> {
    let mut current: Value = storage::read(&["desktop", "preferences"]).await.unwrap_or_default();
    merge_preferences_patch(&mut current, &patch);
    storage::write(&["desktop", "preferences"], &current)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(current))
}

fn merge_preferences_patch(current: &mut Value, patch: &Value) {
    if !current.is_object() {
        *current = serde_json::json!({});
    }

    let Some(obj) = current.as_object_mut() else {
        return;
    };
    let Some(patch_obj) = patch.as_object() else {
        return;
    };

    for (k, v) in patch_obj {
        // `null` 在偏好补丁里表示删除键，便于前端恢复默认值而不引入额外协议字段。
        if v.is_null() {
            obj.remove(k);
        } else {
            obj.insert(k.clone(), v.clone());
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ToolContentPath {
    tool_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolContentBody {
    content: String,
}

async fn tool_content_get(Path(params): Path<ToolContentPath>) -> Result<Json<Value>, ApiError> {
    let tool_type = params.tool_type;
    if !TOOL_CONTENT_TYPES.contains(&tool_type.as_str()) {
        return Err(ApiError::bad_request("unsupported tool content type"));
    }
    let key = ["desktop".to_string(), "tool_content".to_string(), tool_type.clone()];
    let key_refs: Vec<&str> = key.iter().map(String::as_str).collect();
    let content: ToolContentBody =
        storage::read(&key_refs).await.unwrap_or(ToolContentBody { content: String::new() });
    Ok(Json(serde_json::json!({ "content": content.content })))
}

async fn tool_content_put(
    Path(params): Path<ToolContentPath>,
    Json(body): Json<ToolContentBody>,
) -> Result<Json<Value>, ApiError> {
    let tool_type = params.tool_type;
    if !TOOL_CONTENT_TYPES.contains(&tool_type.as_str()) {
        return Err(ApiError::bad_request("unsupported tool content type"));
    }
    let key = ["desktop".to_string(), "tool_content".to_string(), tool_type.clone()];
    let key_refs: Vec<&str> = key.iter().map(String::as_str).collect();
    storage::write(&key_refs, &body).await.map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "content": body.content })))
}

async fn mindmap_tabs_get() -> Result<Json<Value>, ApiError> {
    let value: Value = storage::read(&["desktop", "mindmap_tabs"]).await.unwrap_or_default();
    Ok(Json(value))
}

async fn mindmap_tabs_put(Json(body): Json<Value>) -> Result<Json<Value>, ApiError> {
    storage::write(&["desktop", "mindmap_tabs"], &body)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(body))
}

#[derive(Debug, Deserialize)]
struct ProjectPreferencesQuery {
    project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectPreferencesBody {
    model: String,
    auto_model: bool,
    #[serde(default)]
    acp_agent: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProjectPreferencesResponse {
    model: String,
    auto_model: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    acp_agent: Option<String>,
}

async fn project_preferences_get(
    Query(query): Query<ProjectPreferencesQuery>,
) -> Result<Json<ProjectPreferencesResponse>, ApiError> {
    let key = project_prefs_key(&query.project_path);
    let key_refs: Vec<&str> = key.iter().map(String::as_str).collect();
    let prefs: ProjectPreferencesBody = storage::read(&key_refs)
        .await
        .map_err(|_| ApiError::not_found("project preferences not found"))?;
    Ok(Json(ProjectPreferencesResponse {
        model: prefs.model,
        auto_model: prefs.auto_model,
        acp_agent: prefs.acp_agent,
    }))
}

async fn project_preferences_put(
    Query(query): Query<ProjectPreferencesQuery>,
    Json(body): Json<ProjectPreferencesBody>,
) -> Result<Json<ProjectPreferencesResponse>, ApiError> {
    let key = project_prefs_key(&query.project_path);
    let key_refs: Vec<&str> = key.iter().map(String::as_str).collect();
    storage::write(&key_refs, &body).await.map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ProjectPreferencesResponse {
        model: body.model,
        auto_model: body.auto_model,
        acp_agent: body.acp_agent,
    }))
}

fn project_prefs_key(project_path: &str) -> Vec<String> {
    // 存储层 key 是路径段数组；将路径字符收敛为稳定 id，避免项目路径中的分隔符改变层级。
    let normalized = project_path.replace(['/', '\\', ':', ' ', '.'], "_");
    let trimmed = normalized.trim_matches('_');
    let id = if trimmed.is_empty() { "_root".to_string() } else { trimmed.to_string() };
    vec!["desktop".to_string(), "project_prefs".to_string(), id]
}

#[cfg(test)]
#[path = "desktop_tests.rs"]
mod desktop_tests;

#[derive(Debug, Serialize)]
struct ExternalAppState {
    id: String,
    available: bool,
}

#[derive(Debug, Serialize)]
struct ExternalAppsResponse {
    platform: &'static str,
    apps: Vec<ExternalAppState>,
}

#[derive(Debug, Deserialize)]
struct ExternalOpenRequest {
    path: String,
    target: String,
}

#[derive(Debug, Deserialize)]
struct ExternalRevealRequest {
    path: String,
}

async fn external_apps_get() -> Result<Json<ExternalAppsResponse>, ApiError> {
    Ok(Json(ExternalAppsResponse { platform: host_platform(), apps: detect_external_apps() }))
}

async fn external_apps_open_post(
    Json(body): Json<ExternalOpenRequest>,
) -> Result<Json<Value>, ApiError> {
    open_project_path_in_external(&body.path, &body.target).map_err(ApiError::bad_request)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn external_path_reveal_post(
    Json(body): Json<ExternalRevealRequest>,
) -> Result<Json<Value>, ApiError> {
    reveal_path_in_file_manager(&body.path).map_err(ApiError::bad_request)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn detect_external_apps() -> Vec<ExternalAppState> {
    let mut apps: Vec<ExternalAppState> = Vec::new();
    apps.push(ExternalAppState { id: "finder".to_string(), available: true });

    #[cfg(target_os = "macos")]
    {
        let app_exists = |bundle_path: &str| std::path::Path::new(bundle_path).exists();
        apps.push(ExternalAppState {
            id: "vscode".to_string(),
            available: app_exists("/Applications/Visual Studio Code.app"),
        });
        apps.push(ExternalAppState {
            id: "cursor".to_string(),
            available: app_exists("/Applications/Cursor.app"),
        });
        apps.push(ExternalAppState {
            id: "trae".to_string(),
            available: app_exists("/Applications/Trae.app")
                || app_exists("/Applications/Trae IDE.app"),
        });
        apps.push(ExternalAppState {
            id: "windsurf".to_string(),
            available: app_exists("/Applications/Windsurf.app"),
        });
        apps.push(ExternalAppState {
            id: "kiro".to_string(),
            available: app_exists("/Applications/Kiro.app"),
        });
        apps.push(ExternalAppState {
            id: "zed".to_string(),
            available: app_exists("/Applications/Zed.app"),
        });
        apps.push(ExternalAppState {
            id: "textmate".to_string(),
            available: app_exists("/Applications/TextMate.app"),
        });
        apps.push(ExternalAppState {
            id: "antigravity".to_string(),
            available: app_exists("/Applications/Antigravity.app"),
        });
        apps.push(ExternalAppState {
            id: "terminal".to_string(),
            available: app_exists("/System/Applications/Utilities/Terminal.app")
                || app_exists("/Applications/Utilities/Terminal.app"),
        });
        apps.push(ExternalAppState {
            id: "iterm2".to_string(),
            available: app_exists("/Applications/iTerm.app"),
        });
        apps.push(ExternalAppState {
            id: "ghostty".to_string(),
            available: app_exists("/Applications/Ghostty.app"),
        });
        apps.push(ExternalAppState {
            id: "xcode".to_string(),
            available: app_exists("/Applications/Xcode.app"),
        });
        apps.push(ExternalAppState {
            id: "android-studio".to_string(),
            available: app_exists("/Applications/Android Studio.app"),
        });
        apps.push(ExternalAppState {
            id: "sublime-text".to_string(),
            available: app_exists("/Applications/Sublime Text.app"),
        });
    }

    #[cfg(windows)]
    {
        let cmd_exists = |cmd: &str| {
            std::process::Command::new("cmd")
                .args(["/C", "where", cmd])
                .status()
                .is_ok_and(|status| status.success())
        };
        apps.push(ExternalAppState { id: "trae".to_string(), available: cmd_exists("trae") });
        apps.push(ExternalAppState {
            id: "windsurf".to_string(),
            available: cmd_exists("windsurf"),
        });
        apps.push(ExternalAppState { id: "kiro".to_string(), available: cmd_exists("kiro") });
        apps.push(ExternalAppState { id: "cursor".to_string(), available: cmd_exists("cursor") });
        apps.push(ExternalAppState { id: "vscode".to_string(), available: cmd_exists("code") });
        apps.push(ExternalAppState { id: "zed".to_string(), available: cmd_exists("zed") });
        apps.push(ExternalAppState {
            id: "sublime-text".to_string(),
            available: cmd_exists("subl"),
        });
        apps.push(ExternalAppState { id: "powershell".to_string(), available: true });
    }

    #[cfg(target_os = "linux")]
    {
        let cmd_exists = |cmd: &str| {
            std::process::Command::new("sh")
                .args(["-lc", &format!("command -v {cmd} >/dev/null 2>&1")])
                .status()
                .is_ok_and(|status| status.success())
        };
        apps.push(ExternalAppState { id: "trae".to_string(), available: cmd_exists("trae") });
        apps.push(ExternalAppState {
            id: "windsurf".to_string(),
            available: cmd_exists("windsurf"),
        });
        apps.push(ExternalAppState { id: "kiro".to_string(), available: cmd_exists("kiro") });
        apps.push(ExternalAppState { id: "cursor".to_string(), available: cmd_exists("cursor") });
        apps.push(ExternalAppState { id: "vscode".to_string(), available: cmd_exists("code") });
        apps.push(ExternalAppState { id: "zed".to_string(), available: cmd_exists("zed") });
        apps.push(ExternalAppState {
            id: "sublime-text".to_string(),
            available: cmd_exists("subl"),
        });
    }

    apps
}

fn host_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos"
    }

    #[cfg(windows)]
    {
        "windows"
    }

    #[cfg(all(not(target_os = "macos"), not(windows)))]
    {
        "linux"
    }
}

fn reveal_path_in_file_manager(path: &str) -> Result<(), String> {
    let path = decode_file_url_path(path);
    let p = std::path::Path::new(&path);
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("open").args(["-R", &path]).spawn().is_err() {
            if let Some(dir) = p.parent() {
                os_default_open(&dir.to_string_lossy())?;
            } else {
                os_default_open(&path)?;
            }
        }
    }
    #[cfg(windows)]
    {
        if std::process::Command::new("explorer").args(["/select,", &path]).spawn().is_err() {
            if let Some(dir) = p.parent() {
                os_default_open(&dir.to_string_lossy())?;
            } else {
                os_default_open(&path)?;
            }
        }
    }
    #[cfg(all(not(target_os = "macos"), not(windows)))]
    {
        if let Some(dir) = p.parent() {
            os_default_open(&dir.to_string_lossy())?;
        } else {
            os_default_open(&path)?;
        }
    }
    Ok(())
}

fn decode_file_url_path(path_or_url: &str) -> String {
    let raw = path_or_url
        .strip_prefix("file:///")
        .or_else(|| path_or_url.strip_prefix("file://"))
        .unwrap_or(path_or_url);
    let decoded = urlencoding::decode(raw)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| raw.to_string());
    #[cfg(not(windows))]
    {
        // macOS/Linux 的 file:///foo 在去前缀后会丢掉根斜杠，这里显式补回绝对路径语义。
        let has_file_scheme = path_or_url.starts_with("file://");
        let has_triple_slash = path_or_url.starts_with("file:///");
        if has_file_scheme && has_triple_slash && !decoded.starts_with('/') {
            return format!("/{}", decoded);
        }
    }
    #[cfg(windows)]
    {
        if decoded.starts_with('/') {
            let bytes = decoded.as_bytes();
            if bytes.len() > 2 && bytes[2] == b':' {
                return decoded[1..].to_string();
            }
        }
    }
    decoded
}

fn os_default_open(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
    #[cfg(all(not(target_os = "macos"), not(windows)))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

fn open_project_path_in_external(path: &str, target: &str) -> Result<(), String> {
    match target {
        "finder" => os_default_open(path),
        "vscode" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Visual Studio Code", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("code").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "cursor" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Cursor", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("cursor").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "trae" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Trae", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("trae").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "windsurf" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Windsurf", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("windsurf").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "kiro" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Kiro", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("kiro").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "zed" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Zed", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("zed").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        "iterm2" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "iTerm", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "terminal" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Terminal", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "textmate" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "TextMate", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "antigravity" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Antigravity", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "ghostty" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Ghostty", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "xcode" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Xcode", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "android-studio" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Android Studio", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                os_default_open(path)
            }
        }
        "powershell" => {
            #[cfg(windows)]
            {
                std::process::Command::new("powershell")
                    .args(["-NoExit", "-Command", &format!("Set-Location -LiteralPath \"{path}\"")])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(windows))]
            {
                os_default_open(path)
            }
        }
        "sublime-text" => {
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .args(["-a", "Sublime Text", path])
                    .spawn()
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if std::process::Command::new("subl").arg(path).spawn().is_err() {
                    os_default_open(path)
                } else {
                    Ok(())
                }
            }
        }
        _ => Err("unsupported external app target".to_string()),
    }
}
