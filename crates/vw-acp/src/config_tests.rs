//! 配置加载与解析逻辑的单元测试。

use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_client_protocol::McpServer;
use serde_json::{Value, json};

use super::*;
use crate::agent_registry::{AgentCommandSpec, DEFAULT_AGENT_NAME};
use crate::types::{AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, PermissionMode};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("vw-acp-config-tests-{label}-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.path.join(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct EnvVarGuard {
    _lock: MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvVarGuard {
    fn new(keys: &[&'static str]) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let saved = keys.iter().map(|key| (*key, std::env::var(key).ok())).collect();
        Self { _lock: lock, saved }
    }

    fn set(&self, key: &str, value: &Path) {
        unsafe { std::env::set_var(key, value) };
    }

    fn set_str(&self, key: &str, value: &str) {
        unsafe { std::env::set_var(key, value) };
    }

    fn clear(&self, key: &str) {
        unsafe { std::env::remove_var(key) };
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}

fn write_json(path: &Path, value: Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, serde_json::to_string_pretty(&value).expect("serialize json"))
        .expect("write json");
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
        .block_on(future)
}

fn assert_invalid<T: Debug>(result: Result<T, ConfigError>, expected: &str) {
    let error = result.expect_err("config should be invalid").to_string();
    assert!(error.contains(expected), "expected error to contain {expected:?}, got {error:?}");
}

fn agent(command: &str) -> Value {
    json!({ "command": command })
}

#[test]
fn parse_ttl_ms_accepts_missing_and_non_negative_seconds() {
    assert_eq!(parse_ttl_ms(None, "config.json").expect("missing ttl"), None);
    assert_eq!(parse_ttl_ms(Some(&json!(1.234)), "config.json").expect("ttl"), Some(1_234));
    assert_eq!(parse_ttl_ms(Some(&json!(0)), "config.json").expect("zero ttl"), Some(0));
}

#[test]
fn parse_ttl_ms_rejects_invalid_values() {
    assert_invalid(parse_ttl_ms(Some(&json!("1")), "config.json"), "Invalid config ttl");
    assert_invalid(parse_ttl_ms(Some(&json!(-0.1)), "config.json"), "Invalid config ttl");
}

#[test]
fn parse_timeout_ms_accepts_null_and_positive_seconds() {
    assert_eq!(parse_timeout_ms(None, "config.json").expect("missing timeout"), None);
    assert_eq!(parse_timeout_ms(Some(&Value::Null), "config.json").expect("null timeout"), None);
    assert_eq!(parse_timeout_ms(Some(&json!(2.5)), "config.json").expect("timeout"), Some(2_500));
}

#[test]
fn parse_timeout_ms_rejects_non_positive_values() {
    assert_invalid(parse_timeout_ms(Some(&json!(0)), "config.json"), "Invalid config timeout");
    assert_invalid(parse_timeout_ms(Some(&json!(-1)), "config.json"), "Invalid config timeout");
    assert_invalid(parse_timeout_ms(Some(&json!(false)), "config.json"), "Invalid config timeout");
}

#[test]
fn parse_queue_max_depth_accepts_positive_integer() {
    assert_eq!(parse_queue_max_depth(None, "config.json").expect("missing depth"), None);
    assert_eq!(
        parse_queue_max_depth(Some(&json!(3)), "config.json").expect("queue depth"),
        Some(3)
    );
}

#[test]
fn parse_queue_max_depth_rejects_zero_and_non_integer() {
    assert_invalid(
        parse_queue_max_depth(Some(&json!(0)), "config.json"),
        "Invalid config queueMaxDepth",
    );
    assert_invalid(
        parse_queue_max_depth(Some(&json!(1.5)), "config.json"),
        "Invalid config queueMaxDepth",
    );
}

#[test]
fn parse_enum_fields_accept_known_values() {
    assert_eq!(
        parse_permission_mode(Some(&json!("approve-all")), "config.json").expect("permission"),
        Some(PermissionMode::ApproveAll)
    );
    assert_eq!(
        parse_permission_mode(Some(&json!("approve-reads")), "config.json").expect("permission"),
        Some(PermissionMode::ApproveReads)
    );
    assert_eq!(
        parse_permission_mode(Some(&json!("deny-all")), "config.json").expect("permission"),
        Some(PermissionMode::DenyAll)
    );
    assert_eq!(
        parse_non_interactive_permission_policy(Some(&json!("deny")), "config.json")
            .expect("non-interactive permission"),
        Some(NonInteractivePermissionPolicy::Deny)
    );
    assert_eq!(
        parse_non_interactive_permission_policy(Some(&json!("fail")), "config.json")
            .expect("non-interactive permission"),
        Some(NonInteractivePermissionPolicy::Fail)
    );
    assert_eq!(
        parse_auth_policy(Some(&json!("skip")), "config.json").expect("auth policy"),
        Some(AuthPolicy::Skip)
    );
    assert_eq!(
        parse_auth_policy(Some(&json!("fail")), "config.json").expect("auth policy"),
        Some(AuthPolicy::Fail)
    );
    assert_eq!(
        parse_output_format(Some(&json!("text")), "config.json").expect("format"),
        Some(OutputFormat::Text)
    );
    assert_eq!(
        parse_output_format(Some(&json!("json")), "config.json").expect("format"),
        Some(OutputFormat::Json)
    );
    assert_eq!(
        parse_output_format(Some(&json!("quiet")), "config.json").expect("format"),
        Some(OutputFormat::Quiet)
    );
}

#[test]
fn parse_enum_fields_accept_missing_and_reject_unknown_values() {
    assert_eq!(parse_permission_mode(None, "config.json").expect("missing"), None);
    assert_eq!(
        parse_non_interactive_permission_policy(None, "config.json").expect("missing"),
        None
    );
    assert_eq!(parse_auth_policy(None, "config.json").expect("missing"), None);
    assert_eq!(parse_output_format(None, "config.json").expect("missing"), None);
    assert_invalid(
        parse_permission_mode(Some(&json!("bad")), "config.json"),
        "Invalid config defaultPermissions",
    );
    assert_invalid(
        parse_permission_mode(Some(&json!(1)), "config.json"),
        "Invalid config defaultPermissions",
    );
    assert_invalid(
        parse_non_interactive_permission_policy(Some(&json!("bad")), "config.json"),
        "Invalid config nonInteractivePermissions",
    );
    assert_invalid(
        parse_non_interactive_permission_policy(Some(&json!(1)), "config.json"),
        "Invalid config nonInteractivePermissions",
    );
    assert_invalid(
        parse_auth_policy(Some(&json!("bad")), "config.json"),
        "Invalid config authPolicy",
    );
    assert_invalid(parse_auth_policy(Some(&json!(1)), "config.json"), "Invalid config authPolicy");
    assert_invalid(
        parse_output_format(Some(&json!("bad")), "config.json"),
        "Invalid config format",
    );
    assert_invalid(parse_output_format(Some(&json!(1)), "config.json"), "Invalid config format");
}

#[test]
fn parse_default_agent_trims_and_normalizes_name() {
    assert_eq!(parse_default_agent(None, "config.json").expect("missing default"), None);
    assert_eq!(
        parse_default_agent(Some(&json!("  Claude Code  ")), "config.json").expect("default agent"),
        Some("claude code".to_string())
    );
}

#[test]
fn parse_default_agent_rejects_empty_and_non_string_values() {
    assert_invalid(
        parse_default_agent(Some(&json!("   ")), "config.json"),
        "Invalid config defaultAgent",
    );
    assert_invalid(
        parse_default_agent(Some(&json!(1)), "config.json"),
        "Invalid config defaultAgent",
    );
}

#[test]
fn parse_agents_accepts_named_commands_args_and_env() {
    let agents = parse_agents(
        Some(&json!({
            "Claude Code": {
                "name": " Claude ",
                "command": " claude ",
                "args": ["--model", "sonnet"],
                "env": { "TOKEN": "secret" }
            },
            "gemini": {
                "name": "",
                "command": "gemini"
            }
        })),
        "config.json",
    )
    .expect("parse agents")
    .expect("agents");

    assert_eq!(
        agents.get("claude code"),
        Some(&AgentCommandSpec {
            display_name: "Claude".to_string(),
            command: "claude".to_string(),
            args: vec!["--model".to_string(), "sonnet".to_string()],
            env: HashMap::from([("TOKEN".to_string(), "secret".to_string())]),
        })
    );
    assert_eq!(agents.get("gemini").expect("gemini").display_name, "gemini");
}

#[test]
fn parse_agents_rejects_malformed_agent_entries() {
    assert_eq!(parse_agents(None, "config.json").expect("missing agents"), None);
    assert_invalid(parse_agents(Some(&json!([])), "config.json"), "Invalid config agents");
    assert_invalid(
        parse_agents(Some(&json!({ "bad": [] })), "config.json"),
        "Invalid config agents.bad",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": { "name": 1, "command": "agent" } })), "config.json"),
        "Invalid config agents.bad.name",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": {} })), "config.json"),
        "Invalid config agents.bad.command",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": { "command": " " } })), "config.json"),
        "Invalid config agents.bad.command",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": { "command": "agent", "args": "x" } })), "config.json"),
        "Invalid config agents.bad.args",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": { "command": "agent", "args": [1] } })), "config.json"),
        "Invalid config agents.bad.args",
    );
    assert_invalid(
        parse_agents(Some(&json!({ "bad": { "command": "agent", "env": [] } })), "config.json"),
        "Invalid config agents.bad.env",
    );
    assert_invalid(
        parse_agents(
            Some(&json!({ "bad": { "command": "agent", "env": { "TOKEN": 1 } } })),
            "config.json",
        ),
        "Invalid config agents.bad.env.TOKEN",
    );
}

#[test]
fn merge_agent_maps_overlays_project_entries() {
    let base = HashMap::from([(
        "agent".to_string(),
        AgentCommandSpec {
            display_name: "Base".to_string(),
            command: "base".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
        },
    )]);
    let overlay = HashMap::from([(
        "agent".to_string(),
        AgentCommandSpec {
            display_name: "Overlay".to_string(),
            command: "overlay".to_string(),
            args: vec!["--fast".to_string()],
            env: HashMap::new(),
        },
    )]);

    let merged = merge_agent_maps(Some(base), Some(overlay));

    assert_eq!(merged.get("agent").expect("agent").command, "overlay");
}

#[test]
fn config_dir_from_workspace_handles_current_and_legacy_layouts() {
    let temp = TempDir::new("workspace-dir");
    let current = temp.join("current");
    fs::create_dir_all(&current).expect("create current");
    fs::write(current.join("vibewindow.json"), "{}").expect("write current config");
    assert_eq!(config_dir_from_workspace(&current), current);

    let workspace = temp.join("workspace");
    let legacy = temp.join(".vibewindow");
    fs::create_dir_all(&legacy).expect("create legacy");
    fs::write(legacy.join("vibewindow.json"), "{}").expect("write legacy config");
    assert_eq!(config_dir_from_workspace(&workspace), legacy);

    let fallback_temp = TempDir::new("workspace-dir-fallback");
    let standalone = fallback_temp.join("standalone");
    assert_eq!(config_dir_from_workspace(&standalone), standalone);

    let workspace_fallback = fallback_temp.join("workspace");
    assert_eq!(config_dir_from_workspace(&workspace_fallback), fallback_temp.join(".vibewindow"));

    assert_eq!(config_dir_from_workspace(Path::new("relative")), PathBuf::from("relative"));
    assert_eq!(config_dir_from_workspace(Path::new("")), PathBuf::from(""));
}

#[test]
fn parse_active_workspace_marker_handles_absent_empty_relative_and_absolute_values() {
    let temp = TempDir::new("marker");
    let marker = temp.join("active_workspace.toml");
    assert_eq!(parse_active_workspace_marker(&marker).expect("missing marker"), None);

    fs::write(&marker, "config_dir\n").expect("write marker without equals");
    assert_eq!(parse_active_workspace_marker(&marker).expect("marker without equals"), None);

    fs::write(&marker, "ignored = true\n").expect("write marker without config dir");
    assert_eq!(parse_active_workspace_marker(&marker).expect("marker without config dir"), None);

    fs::write(&marker, "config_dir = \"\"\n").expect("write empty marker");
    assert_eq!(parse_active_workspace_marker(&marker).expect("empty marker"), None);

    let env = EnvVarGuard::new(&["HOME"]);
    env.set("HOME", temp.path());
    fs::write(&marker, "ignored = true\nconfig_dir = \"relative-dir\"\n").expect("write marker");
    assert_eq!(
        parse_active_workspace_marker(&marker).expect("relative marker"),
        Some(vw_config_types::paths::home_config_dir(temp.path()).join("relative-dir"))
    );

    let absolute = temp.join("absolute-dir");
    fs::write(&marker, format!("config_dir = \"{}\"\n", absolute.display())).expect("write marker");
    assert_eq!(parse_active_workspace_marker(&marker).expect("absolute marker"), Some(absolute));
}

#[test]
fn parse_active_workspace_marker_reports_read_errors() {
    let temp = TempDir::new("marker-read-error");
    let error = parse_active_workspace_marker(temp.path()).expect_err("directory read should fail");
    assert!(matches!(error, ConfigError::Read { .. }));
}

#[test]
fn discover_vibewindow_config_path_uses_env_workspace_marker_and_project_fallbacks() {
    let temp = TempDir::new("discover");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());

    let config_dir = temp.join("explicit");
    env.set("VIBEWINDOW_CONFIG_DIR", &config_dir);
    assert_eq!(
        discover_vibewindow_config_path(&temp.join("project/.vwacprc.json"))
            .expect("explicit config dir"),
        config_dir.join("vibewindow.json")
    );
    env.set_str("VIBEWINDOW_CONFIG_DIR", "   ");

    let workspace = temp.join("workspace");
    let legacy = temp.join(".vibewindow");
    fs::create_dir_all(&legacy).expect("create legacy");
    fs::write(legacy.join("vibewindow.json"), "{}").expect("write legacy");
    env.set("VIBEWINDOW_WORKSPACE", &workspace);
    assert_eq!(
        discover_vibewindow_config_path(&temp.join("project/.vwacprc.json"))
            .expect("workspace config"),
        legacy.join("vibewindow.json")
    );
    env.set_str("VIBEWINDOW_WORKSPACE", "   ");

    let default_dir = vw_config_types::paths::home_config_dir(temp.path());
    fs::create_dir_all(&default_dir).expect("create default dir");
    fs::write(default_dir.join("active_workspace.toml"), "config_dir = \"active\"\n")
        .expect("write marker");
    assert_eq!(
        discover_vibewindow_config_path(&temp.join("project/.vwacprc.json")).expect("marker"),
        default_dir.join("active").join("vibewindow.json")
    );
    fs::remove_file(default_dir.join("active_workspace.toml")).expect("remove marker");

    let project_dir = temp.join("project");
    fs::create_dir_all(&project_dir).expect("create project");
    fs::write(project_dir.join("vibewindow.json"), "{}").expect("write project config");
    assert_eq!(
        discover_vibewindow_config_path(&project_dir.join(".vwacprc.json")).expect("project"),
        project_dir.join("vibewindow.json")
    );

    fs::remove_file(project_dir.join("vibewindow.json")).expect("remove project config");
    assert_eq!(
        discover_vibewindow_config_path(&project_dir.join(".vwacprc.json")).expect("default"),
        default_dir.join("vibewindow.json")
    );
}

#[test]
fn parse_active_workspace_marker_requires_home_for_relative_config_dir() {
    if cfg!(windows) {
        return;
    }

    let temp = TempDir::new("marker-relative-home-missing");
    let marker = temp.join("active_workspace.toml");
    fs::write(&marker, "config_dir = relative\n").expect("write marker");
    let env = EnvVarGuard::new(&["HOME"]);
    env.clear("HOME");

    assert!(matches!(parse_active_workspace_marker(&marker), Err(ConfigError::HomeDirUnavailable)));
}

#[test]
fn discover_vibewindow_config_path_handles_no_parent_and_missing_home_errors() {
    let temp = TempDir::new("discover-no-parent");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());

    assert_eq!(
        discover_vibewindow_config_path(Path::new("")).expect("discover default"),
        vw_config_types::paths::home_config_dir(temp.path()).join("vibewindow.json")
    );
    drop(env);

    if cfg!(windows) {
        return;
    }

    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("HOME");
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");

    assert!(matches!(
        discover_vibewindow_config_path(Path::new("project/.vwacprc.json")),
        Err(ConfigError::HomeDirUnavailable)
    ));
}

#[test]
fn discover_vibewindow_config_path_propagates_marker_read_errors() {
    let temp = TempDir::new("discover-marker-read-error");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let default_dir = vw_config_types::paths::home_config_dir(temp.path());
    fs::create_dir_all(default_dir.join("active_workspace.toml")).expect("create marker directory");

    assert!(matches!(
        discover_vibewindow_config_path(Path::new("project/.vwacprc.json")),
        Err(ConfigError::Read { .. })
    ));
}

#[test]
fn path_wrappers_report_current_dir_errors() {
    let temp = TempDir::new("current-dir-error");
    let deleted = temp.join("deleted");
    fs::create_dir_all(&deleted).expect("create deleted cwd");
    let original = std::env::current_dir().expect("current dir");
    std::env::set_current_dir(&deleted).expect("enter deleted cwd");
    fs::remove_dir_all(&deleted).expect("remove cwd");

    let resolve_result = resolve_path("config.json");
    let project_result = project_config_path("project");
    let load_result = block_on(load_resolved_config("project"));

    std::env::set_current_dir(original).expect("restore cwd");

    assert!(matches!(resolve_result, Err(ConfigError::Read { .. })));
    assert!(matches!(project_result, Err(ConfigError::Read { .. })));
    assert!(matches!(load_result, Err(ConfigError::Read { .. })));
}

#[test]
fn load_resolved_config_reports_default_path_errors() {
    if cfg!(windows) {
        return;
    }

    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("HOME");
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");

    assert!(matches!(block_on(load_resolved_config(".")), Err(ConfigError::HomeDirUnavailable)));
    assert!(matches!(
        block_on(load_resolved_config_from_paths("global.json", "project/.vwacprc.json")),
        Err(ConfigError::HomeDirUnavailable)
    ));
}

#[cfg(unix)]
#[test]
fn init_global_config_file_at_reports_try_exists_errors() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let temp = TempDir::new("try-exists-error");
    let mut path = temp.path().as_os_str().as_bytes().to_vec();
    path.extend_from_slice(b"/bad\0path");
    let path = PathBuf::from(OsString::from_vec(path));

    assert!(matches!(block_on(init_global_config_file_at(path)), Err(ConfigError::Read { .. })));
}

#[test]
fn load_resolved_config_uses_default_paths_and_global_fallback_values() {
    let temp = TempDir::new("load-wrapper");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());

    let global_path =
        vw_config_types::paths::home_config_dir(temp.path()).join("acp").join("config.json");
    write_json(
        &global_path,
        json!({
            "defaultAgent": "global",
            "defaultPermissions": "approve-all",
            "nonInteractivePermissions": "fail",
            "queueMaxDepth": 9,
            "disableExec": true
        }),
    );
    let cwd = temp.join("project");
    fs::create_dir_all(&cwd).expect("create project dir");

    let config = block_on(load_resolved_config(&cwd)).expect("load config");

    assert_eq!(config.default_agent, "global");
    assert_eq!(config.default_permissions, PermissionMode::ApproveAll);
    assert_eq!(config.non_interactive_permissions, NonInteractivePermissionPolicy::Fail);
    assert_eq!(config.queue_max_depth, 9);
    assert!(config.disable_exec);
    assert_eq!(config.global_path, path_to_string(&global_path));
    assert_eq!(config.project_path, path_to_string(&cwd.join(".vwacprc.json")));
}

#[test]
fn default_vibewindow_config_dir_requires_home_on_unix() {
    if cfg!(windows) {
        return;
    }

    let env = EnvVarGuard::new(&["HOME"]);
    env.clear("HOME");
    assert!(matches!(default_vibewindow_config_dir(), Err(ConfigError::HomeDirUnavailable)));
    assert!(matches!(default_global_config_path(), Err(ConfigError::HomeDirUnavailable)));
    assert!(matches!(block_on(init_global_config_file()), Err(ConfigError::HomeDirUnavailable)));
}

#[test]
fn parse_auth_accepts_trimmed_credentials_and_rejects_invalid_values() {
    assert_eq!(parse_auth(None, "config.json").expect("missing auth"), None);
    assert_eq!(
        parse_auth(Some(&json!({ "github": " token " })), "config.json")
            .expect("auth")
            .expect("auth"),
        HashMap::from([("github".to_string(), "token".to_string())])
    );
    assert_invalid(parse_auth(Some(&json!([])), "config.json"), "Invalid config auth");
    assert_invalid(
        parse_auth(Some(&json!({ "github": 1 })), "config.json"),
        "Invalid config auth.github",
    );
    assert_invalid(
        parse_auth(Some(&json!({ "github": " " })), "config.json"),
        "Invalid config auth.github",
    );
}

#[test]
fn parse_disable_exec_accepts_boolean_and_rejects_other_values() {
    assert_eq!(parse_disable_exec(None, "config.json").expect("missing disable exec"), None);
    assert_eq!(
        parse_disable_exec(Some(&json!(true)), "config.json").expect("disable exec"),
        Some(true)
    );
    assert_invalid(
        parse_disable_exec(Some(&json!("true")), "config.json"),
        "Invalid config disableExec",
    );
}

#[test]
fn read_config_file_handles_missing_valid_and_invalid_files() {
    let temp = TempDir::new("read-file");
    let missing = temp.join("missing.json");
    let missing_result = block_on(read_config_file(&missing)).expect("missing config");
    assert!(!missing_result.exists);
    assert_eq!(missing_result.config, None);

    let valid = temp.join("valid.json");
    write_json(&valid, json!({ "ttl": 1 }));
    let valid_result = block_on(read_config_file(&valid)).expect("valid config");
    assert!(valid_result.exists);
    assert_eq!(valid_result.config.expect("config").get("ttl"), Some(&json!(1)));

    let invalid = temp.join("invalid.json");
    fs::write(&invalid, "{").expect("write invalid json");
    assert!(matches!(
        block_on(read_config_file(&invalid)).expect_err("invalid json"),
        ConfigError::InvalidJson { .. }
    ));

    let non_object = temp.join("array.json");
    fs::write(&non_object, "[]").expect("write non-object json");
    assert_invalid(block_on(read_config_file(&non_object)), "expected top-level JSON object");

    assert!(matches!(
        block_on(read_config_file(temp.path())).expect_err("directory read should fail"),
        ConfigError::Read { .. }
    ));
}

#[test]
fn merge_maps_overlays_project_values() {
    let merged = merge_maps(
        Some(HashMap::from([
            ("github".to_string(), "global".to_string()),
            ("slack".to_string(), "global".to_string()),
        ])),
        Some(HashMap::from([("github".to_string(), "project".to_string())])),
    );

    assert_eq!(merged.get("github"), Some(&"project".to_string()));
    assert_eq!(merged.get("slack"), Some(&"global".to_string()));
}

#[test]
fn resolve_path_preserves_absolute_and_resolves_relative_from_current_dir() {
    let absolute = std::env::current_dir().expect("current dir").join("config.json");
    assert_eq!(resolve_path(&absolute).expect("absolute"), absolute);
    assert!(resolve_path("config.json").expect("relative").ends_with("config.json"));
}

#[test]
fn default_and_project_config_paths_point_to_expected_files() {
    let temp = TempDir::new("paths");
    let env = EnvVarGuard::new(&["HOME"]);
    env.set("HOME", temp.path());

    assert_eq!(
        default_global_config_path().expect("global path"),
        vw_config_types::paths::home_config_dir(temp.path()).join("acp").join("config.json")
    );
    assert_eq!(
        project_config_path(temp.join("project")).expect("project path"),
        temp.join("project").join(".vwacprc.json")
    );
}

#[test]
fn load_resolved_config_defaults_queue_owner_ttl_to_five_minutes() {
    let temp = TempDir::new("defaults");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let global_path = temp.join("global.json");
    let project_path = temp.join("project.json");

    let config = block_on(load_resolved_config_from_paths(&global_path, &project_path))
        .expect("load default config");

    assert_eq!(config.default_agent, DEFAULT_AGENT_NAME);
    assert_eq!(config.default_permissions, PermissionMode::ApproveReads);
    assert_eq!(config.non_interactive_permissions, NonInteractivePermissionPolicy::Deny);
    assert_eq!(config.auth_policy, AuthPolicy::Skip);
    assert_eq!(config.ttl_ms, 300_000);
    assert_eq!(config.timeout_ms, None);
    assert_eq!(config.queue_max_depth, 16);
    assert_eq!(config.format, OutputFormat::Text);
    assert!(!config.disable_exec);
    assert!(!config.has_global_config);
    assert!(!config.has_project_config);
}

#[test]
fn load_resolved_config_project_values_override_global_values() {
    let temp = TempDir::new("overrides");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let global_path = temp.join("global.json");
    let project_path = temp.join("project/.vwacprc.json");
    let vibewindow_path = temp.join("project/vibewindow.json");
    write_json(
        &vibewindow_path,
        json!({
            "acp": {
                "shared": agent("from-vibewindow"),
                "vibe-only": agent("vibe")
            }
        }),
    );
    write_json(
        &global_path,
        json!({
            "defaultAgent": " global agent ",
            "defaultPermissions": "approve-all",
            "nonInteractivePermissions": "fail",
            "authPolicy": "fail",
            "ttl": 10,
            "timeout": 20,
            "queueMaxDepth": 2,
            "format": "json",
            "disableExec": true,
            "auth": {
                "github": "global",
                "slack": "global"
            },
            "acp": {
                "shared": agent("from-global-acp")
            },
            "agents": {
                "global-only": agent("global")
            },
            "mcpServers": [
                { "name": "global-mcp", "command": "global-mcp" }
            ]
        }),
    );
    write_json(
        &project_path,
        json!({
            "defaultAgent": " Project Agent ",
            "defaultPermissions": "deny-all",
            "nonInteractivePermissions": "deny",
            "authPolicy": "skip",
            "ttl": 1.5,
            "timeout": null,
            "queueMaxDepth": 4,
            "format": "quiet",
            "disableExec": false,
            "auth": {
                "github": "project"
            },
            "agents": {
                "shared": {
                    "command": "from-project",
                    "args": ["--one"],
                    "env": { "A": "B" }
                },
                "project-only": agent("project")
            },
            "mcpServers": [
                { "name": "project-mcp", "type": "http", "url": "https://example.test/mcp" }
            ]
        }),
    );

    let config = block_on(load_resolved_config_from_paths(&global_path, &project_path))
        .expect("load config");

    assert_eq!(config.default_agent, "project agent");
    assert_eq!(config.default_permissions, PermissionMode::DenyAll);
    assert_eq!(config.non_interactive_permissions, NonInteractivePermissionPolicy::Deny);
    assert_eq!(config.auth_policy, AuthPolicy::Skip);
    assert_eq!(config.ttl_ms, 1_500);
    assert_eq!(config.timeout_ms, None);
    assert_eq!(config.queue_max_depth, 4);
    assert_eq!(config.format, OutputFormat::Quiet);
    assert!(!config.disable_exec);
    assert!(config.has_global_config);
    assert!(config.has_project_config);
    assert_eq!(config.auth.get("github"), Some(&"project".to_string()));
    assert_eq!(config.auth.get("slack"), Some(&"global".to_string()));
    assert_eq!(config.agents.get("shared").expect("shared").command, "from-project");
    assert_eq!(config.agents.get("vibe-only").expect("vibe").command, "vibe");
    assert_eq!(config.agents.get("global-only").expect("global").command, "global");
    assert_eq!(config.agents.get("project-only").expect("project").command, "project");
    assert!(matches!(config.mcp_servers.as_slice(), [McpServer::Http(_)]));
}

#[test]
fn load_resolved_config_uses_global_timeout_and_mcp_when_project_omits_fields() {
    let temp = TempDir::new("global-fields");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let global_path = temp.join("global.json");
    let project_path = temp.join("project/.vwacprc.json");
    write_json(
        &global_path,
        json!({
            "timeout": 3,
            "mcpServers": [
                { "name": "global-mcp", "command": "global-mcp" }
            ]
        }),
    );
    write_json(&project_path, json!({}));

    let config = block_on(load_resolved_config_from_paths(&global_path, &project_path))
        .expect("load config");

    assert_eq!(config.timeout_ms, Some(3_000));
    assert!(matches!(config.mcp_servers.as_slice(), [McpServer::Stdio(_)]));
}

#[test]
fn load_resolved_config_reports_read_errors_for_each_config_source() {
    let temp = TempDir::new("load-read-errors");

    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    assert!(matches!(
        block_on(load_resolved_config_from_paths(temp.path(), temp.join("project/.vwacprc.json"))),
        Err(ConfigError::Read { .. })
    ));
    drop(env);

    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    assert!(matches!(
        block_on(load_resolved_config_from_paths(temp.join("global.json"), temp.path())),
        Err(ConfigError::Read { .. })
    ));
    drop(env);

    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.set("HOME", temp.path());
    let config_dir = temp.join("vibewindow-dir");
    fs::create_dir_all(config_dir.join("vibewindow.json")).expect("create vibewindow dir path");
    env.set("VIBEWINDOW_CONFIG_DIR", &config_dir);
    env.clear("VIBEWINDOW_WORKSPACE");
    assert!(matches!(
        block_on(load_resolved_config_from_paths(
            temp.join("global.json"),
            temp.join("project/.vwacprc.json")
        )),
        Err(ConfigError::Read { .. })
    ));
}

#[test]
fn load_resolved_config_reports_invalid_vibewindow_agents() {
    let temp = TempDir::new("invalid-vibewindow-agents");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.set("HOME", temp.path());
    env.clear("VIBEWINDOW_WORKSPACE");
    let config_dir = temp.join("vibewindow");
    env.set("VIBEWINDOW_CONFIG_DIR", &config_dir);
    write_json(&config_dir.join("vibewindow.json"), json!({ "acp": [] }));

    assert_invalid(
        block_on(load_resolved_config_from_paths(
            temp.join("global.json"),
            temp.join("project/.vwacprc.json"),
        )),
        "Invalid config agents",
    );
}

#[test]
fn load_resolved_config_reports_invalid_project_mcp_servers() {
    let temp = TempDir::new("invalid-mcp");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let global_path = temp.join("global.json");
    let project_path = temp.join("project/.vwacprc.json");
    write_json(&project_path, json!({ "mcpServers": {} }));

    let error = block_on(load_resolved_config_from_paths(&global_path, &project_path))
        .expect_err("invalid mcp servers");

    assert!(matches!(error, ConfigError::McpServers(_)));
}

#[test]
fn load_resolved_config_reports_invalid_project_fields() {
    let cases = [
        ("project-default-agent", "defaultAgent", json!(" "), "Invalid config defaultAgent"),
        (
            "project-default-permissions",
            "defaultPermissions",
            json!("bad"),
            "Invalid config defaultPermissions",
        ),
        (
            "project-non-interactive-permissions",
            "nonInteractivePermissions",
            json!("bad"),
            "Invalid config nonInteractivePermissions",
        ),
        ("project-auth-policy", "authPolicy", json!("bad"), "Invalid config authPolicy"),
        ("project-ttl", "ttl", json!(-1), "Invalid config ttl"),
        ("project-timeout", "timeout", json!(0), "Invalid config timeout"),
        ("project-format", "format", json!("bad"), "Invalid config format"),
        ("project-queue-depth", "queueMaxDepth", json!(0), "Invalid config queueMaxDepth"),
        ("project-acp", "acp", json!([]), "Invalid config agents"),
        ("project-agents", "agents", json!([]), "Invalid config agents"),
        ("project-auth", "auth", json!([]), "Invalid config auth"),
        ("project-disable-exec", "disableExec", json!("yes"), "Invalid config disableExec"),
    ];

    for (label, field, value, expected) in cases {
        let temp = TempDir::new(label);
        let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
        env.clear("VIBEWINDOW_CONFIG_DIR");
        env.clear("VIBEWINDOW_WORKSPACE");
        env.set("HOME", temp.path());
        let global_path = temp.join("global.json");
        let project_path = temp.join("project/.vwacprc.json");
        let mut config = serde_json::Map::new();
        config.insert(field.to_string(), value);
        write_json(&project_path, Value::Object(config));

        assert_invalid(
            block_on(load_resolved_config_from_paths(&global_path, &project_path)),
            expected,
        );
    }
}

#[test]
fn load_resolved_config_reports_invalid_global_fallback_values() {
    let cases = [
        ("default-agent", "defaultAgent", json!(" "), "Invalid config defaultAgent"),
        (
            "default-permissions",
            "defaultPermissions",
            json!("bad"),
            "Invalid config defaultPermissions",
        ),
        (
            "non-interactive-permissions",
            "nonInteractivePermissions",
            json!("bad"),
            "Invalid config nonInteractivePermissions",
        ),
        ("auth-policy", "authPolicy", json!("bad"), "Invalid config authPolicy"),
        ("ttl", "ttl", json!(-1), "Invalid config ttl"),
        ("timeout", "timeout", json!(0), "Invalid config timeout"),
        ("format", "format", json!("bad"), "Invalid config format"),
        ("queue-depth", "queueMaxDepth", json!(0), "Invalid config queueMaxDepth"),
        ("acp", "acp", json!([]), "Invalid config agents"),
        ("agents", "agents", json!([]), "Invalid config agents"),
        ("auth", "auth", json!([]), "Invalid config auth"),
        ("disable-exec", "disableExec", json!("yes"), "Invalid config disableExec"),
    ];

    for (label, field, value, expected) in cases {
        let temp = TempDir::new(label);
        let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
        env.clear("VIBEWINDOW_CONFIG_DIR");
        env.clear("VIBEWINDOW_WORKSPACE");
        env.set("HOME", temp.path());
        let global_path = temp.join("global.json");
        let project_path = temp.join("project/.vwacprc.json");
        let mut config = serde_json::Map::new();
        config.insert(field.to_string(), value);
        write_json(&global_path, Value::Object(config));

        assert_invalid(
            block_on(load_resolved_config_from_paths(&global_path, &project_path)),
            expected,
        );
    }

    let temp = TempDir::new("global-invalid-mcp");
    let env = EnvVarGuard::new(&["HOME", "VIBEWINDOW_CONFIG_DIR", "VIBEWINDOW_WORKSPACE"]);
    env.clear("VIBEWINDOW_CONFIG_DIR");
    env.clear("VIBEWINDOW_WORKSPACE");
    env.set("HOME", temp.path());
    let global_path = temp.join("global.json");
    let project_path = temp.join("project/.vwacprc.json");
    write_json(&global_path, json!({ "mcpServers": {} }));

    let error = block_on(load_resolved_config_from_paths(&global_path, &project_path))
        .expect_err("invalid global mcp servers");

    assert!(matches!(error, ConfigError::McpServers(_)));
}

#[test]
fn to_config_display_hides_auth_values_and_sorts_auth_methods() {
    let config = ResolvedAcpxConfig {
        default_agent: "agent".to_string(),
        default_permissions: PermissionMode::ApproveAll,
        non_interactive_permissions: NonInteractivePermissionPolicy::Fail,
        auth_policy: AuthPolicy::Fail,
        ttl_ms: 12_000,
        timeout_ms: Some(34_000),
        queue_max_depth: 5,
        format: OutputFormat::Json,
        agents: HashMap::from([(
            "agent".to_string(),
            AgentCommandSpec {
                display_name: "Agent".to_string(),
                command: "agent".to_string(),
                args: vec!["--flag".to_string()],
                env: HashMap::from([("ENV".to_string(), "value".to_string())]),
            },
        )]),
        auth: HashMap::from([
            ("zeta".to_string(), "secret".to_string()),
            ("alpha".to_string(), "secret".to_string()),
        ]),
        disable_exec: true,
        mcp_servers: Vec::new(),
        global_path: "global".to_string(),
        project_path: "project".to_string(),
        has_global_config: true,
        has_project_config: true,
    };

    let display = to_config_display(&config);

    assert_eq!(display.ttl, 12);
    assert_eq!(display.timeout, Some(34));
    assert_eq!(display.auth_methods, vec!["alpha".to_string(), "zeta".to_string()]);
    assert_eq!(display.agents.get("agent").expect("agent").command, "agent");
    assert!(display.disable_exec);
}

#[test]
fn init_global_config_file_at_creates_parent_dirs_and_default_payload() {
    let temp = TempDir::new("init-create");
    let config_path = temp.join("nested/config.json");

    let result = block_on(init_global_config_file_at(&config_path)).expect("init config");

    assert!(result.created);
    assert_eq!(result.path, path_to_string(&config_path));
    let contents = fs::read_to_string(&config_path).expect("read config");
    let payload: Value = serde_json::from_str(&contents).expect("parse config");
    assert_eq!(payload.get("defaultAgent"), Some(&json!(DEFAULT_AGENT_NAME)));
    assert_eq!(payload.get("timeout"), Some(&Value::Null));
}

#[test]
fn init_global_config_file_at_leaves_existing_file_unchanged() {
    let temp = TempDir::new("init-existing");
    let config_path = temp.join("config.json");
    fs::write(&config_path, "{}").expect("write existing config");

    let result = block_on(init_global_config_file_at(&config_path)).expect("init existing");

    assert!(!result.created);
    assert_eq!(fs::read_to_string(&config_path).expect("read existing"), "{}");
}

#[test]
fn init_global_config_file_at_reports_create_dir_errors() {
    let temp = TempDir::new("init-create-error");
    let file_parent = temp.join("parent-file");
    fs::write(&file_parent, "not a dir").expect("write parent file");

    let error = block_on(init_global_config_file_at(file_parent.join("config.json")))
        .expect_err("create dir should fail");

    assert!(matches!(error, ConfigError::CreateDir { .. }));
}

#[test]
fn init_global_config_file_at_reports_write_errors_without_parent_path() {
    let error =
        block_on(init_global_config_file_at(Path::new(""))).expect_err("empty path should fail");

    assert!(matches!(error, ConfigError::Write { .. }));
}

#[test]
fn init_global_config_file_uses_default_global_path() {
    let temp = TempDir::new("init-wrapper");
    let env = EnvVarGuard::new(&["HOME"]);
    env.set("HOME", temp.path());

    let result = block_on(init_global_config_file()).expect("init default config");

    assert!(result.created);
    assert_eq!(
        result.path,
        path_to_string(
            &vw_config_types::paths::home_config_dir(temp.path()).join("acp").join("config.json")
        )
    );
}
