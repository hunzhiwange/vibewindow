use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{
    AcpAgentConfig, AcpSessionOptions, PermissionMode, SessionEventLog, SessionTokenUsage,
};

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
        let path = std::env::temp_dir().join(format!("vw-acp-session-records-{label}-{unique}"));
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

struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    saved_home: Option<String>,
}

impl EnvGuard {
    fn set_home(path: &Path) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let saved_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", path) };
        Self { _lock: lock, saved_home }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.saved_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }
}

struct MockAgent {
    _dir: TempDir,
    script: PathBuf,
}

impl MockAgent {
    fn new() -> Self {
        let dir = TempDir::new("agent-script");
        let script = dir.join("agent.py");
        fs::write(&script, mock_agent_script()).expect("write mock agent script");
        Self { _dir: dir, script }
    }

    fn config(&self) -> AcpAgentConfig {
        AcpAgentConfig {
            command: "python3".to_string(),
            args: vec!["-u".to_string(), self.script.display().to_string()],
            env: HashMap::new(),
        }
    }
}

fn mock_agent_script() -> &'static str {
    r#"import json
import sys

session_counter = 0

for raw_line in sys.stdin:
    line = raw_line.strip()
    if not line:
        continue
    message = json.loads(line)
    method = message.get("method")
    request_id = message.get("id")
    if request_id is None:
        continue

    if method == "initialize":
        result = {
            "protocolVersion": message["params"]["protocolVersion"],
            "agentCapabilities": {},
            "authMethods": [],
            "agentInfo": {"name": "mock-acp-agent", "version": "0.1.0"}
        }
    elif method == "session/new":
        session_counter += 1
        result = {"sessionId": f"session-{session_counter}"}
    else:
        result = {}

    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}) + "\n")
    sys.stdout.flush()
"#
}

fn create_options(
    cwd: &Path,
    agent: &MockAgent,
    session_options: Option<AcpSessionOptions>,
) -> SessionCreateOptions {
    SessionCreateOptions {
        agent_command: "mock".to_string(),
        agent_config: Some(agent.config()),
        cwd: cwd.display().to_string(),
        name: Some("  work  ".to_string()),
        resume_session_id: None,
        mcp_servers: None,
        permission_mode: PermissionMode::DenyAll,
        non_interactive_permissions: None,
        auth_credentials: None,
        auth_policy: None,
        verbose: false,
        session_options,
        timeout_ms: Some(5_000),
    }
}

fn ensure_options(cwd: &Path, agent: &MockAgent) -> SessionEnsureOptions {
    SessionEnsureOptions {
        agent_command: "mock".to_string(),
        agent_config: Some(agent.config()),
        cwd: cwd.display().to_string(),
        name: Some("work".to_string()),
        resume_session_id: None,
        mcp_servers: None,
        permission_mode: PermissionMode::DenyAll,
        non_interactive_permissions: None,
        auth_credentials: None,
        auth_policy: None,
        verbose: false,
        walk_boundary: None,
        session_options: None,
        timeout_ms: Some(5_000),
    }
}

fn ensure_options_with_boundary(
    cwd: &Path,
    agent: &MockAgent,
    walk_boundary: &Path,
) -> SessionEnsureOptions {
    SessionEnsureOptions {
        walk_boundary: Some(walk_boundary.display().to_string()),
        ..ensure_options(cwd, agent)
    }
}

fn stored_record(id: &str, cwd: &Path, name: Option<&str>, last_used_at: &str) -> SessionRecord {
    SessionRecord {
        schema: crate::SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: id.to_string(),
        acp_session_id: id.to_string(),
        agent_session_id: None,
        agent_command: "mock".to_string(),
        agent_config: None,
        cwd: absolute_path(&cwd.display().to_string()).to_string_lossy().into_owned(),
        name: name.map(ToOwned::to_owned),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: last_used_at.to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: format!("/tmp/{id}.ndjson"),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 3,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: None,
        agent_started_at: None,
        last_prompt_at: None,
        last_agent_exit_code: None,
        last_agent_exit_signal: None,
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: None,
        messages: Vec::new(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: None,
    }
}

#[test]
fn walk_boundary_accepts_descendants_only() {
    let boundary = Path::new("/tmp/project");

    assert!(is_within_walk_boundary(boundary, Path::new("/tmp/project/src")));
    assert!(is_within_walk_boundary(boundary, Path::new("/tmp/project")));
    assert!(!is_within_walk_boundary(boundary, Path::new("/tmp/project-other")));
}

#[tokio::test(flavor = "current_thread")]
async fn create_session_writes_normalized_record_and_requested_options() {
    let home = TempDir::new("home");
    let _env = EnvGuard::set_home(home.path());
    let cwd = TempDir::new("cwd");
    let agent = MockAgent::new();

    let record = create_session(create_options(
        cwd.path(),
        &agent,
        Some(AcpSessionOptions {
            model: Some("fast-model".to_string()),
            allowed_tools: Some(vec!["shell".to_string()]),
            max_turns: Some(4),
        }),
    ))
    .await
    .expect("session record should be created");

    assert_eq!(record.vwacp_record_id, "session-1");
    assert_eq!(record.acp_session_id, "session-1");
    let expected_cwd = absolute_path(&cwd.path().display().to_string());
    assert_eq!(record.cwd, expected_cwd.to_string_lossy().as_ref());
    assert_eq!(record.name.as_deref(), Some("work"));
    let state = record.vwacp.as_ref().expect("vwacp state should be stored");
    assert_eq!(state.current_model_id.as_deref(), Some("fast-model"));
    let options = state.session_options.as_ref().expect("session options should be stored");
    assert_eq!(options.model.as_deref(), Some("fast-model"));
    assert_eq!(options.allowed_tools.as_deref(), Some(&["shell".to_string()][..]));
    assert_eq!(options.max_turns, Some(4));

    let sessions = crate::session_persistence::list_sessions()
        .await
        .expect("created session should be listed");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].vwacp_record_id, "session-1");
}

#[tokio::test(flavor = "current_thread")]
async fn create_session_uses_load_when_resume_session_id_is_requested() {
    let home = TempDir::new("home");
    let _env = EnvGuard::set_home(home.path());
    let cwd = TempDir::new("cwd");
    let agent = MockAgent::new();
    let mut options = create_options(cwd.path(), &agent, None);
    options.resume_session_id = Some("resume-77".to_string());

    let record = create_session(options).await.expect("resumed session should be recorded");

    assert_eq!(record.vwacp_record_id, "resume-77");
    assert_eq!(record.acp_session_id, "resume-77");
    assert_eq!(record.event_log.active_path, fallback_event_log("resume-77").active_path);
}

#[tokio::test(flavor = "current_thread")]
async fn ensure_session_reuses_directory_walk_record_without_creating() {
    let home = TempDir::new("home");
    let _env = EnvGuard::set_home(home.path());
    let workspace = TempDir::new("workspace");
    let parent = workspace.join("project");
    let child = parent.join("src");
    fs::create_dir_all(&child).expect("create nested cwd");
    let agent = MockAgent::new();
    let existing = stored_record("existing-session", &parent, Some("work"), "2026-01-02T00:00:00Z");
    crate::session_persistence::write_session_record(&existing)
        .await
        .expect("write existing record");

    let result = ensure_session(ensure_options_with_boundary(&child, &agent, workspace.path()))
        .await
        .expect("existing session should load");

    assert!(!result.created);
    assert_eq!(result.record.vwacp_record_id, "existing-session");
}

#[tokio::test(flavor = "current_thread")]
async fn live_directory_walk_ignores_boundary_outside_start_subtree() {
    let home = TempDir::new("home");
    let _env = EnvGuard::set_home(home.path());
    let workspace = TempDir::new("workspace");
    let start = workspace.join("project/src");
    let outside = workspace.join("other");
    fs::create_dir_all(&start).expect("create start cwd");
    fs::create_dir_all(&outside).expect("create outside boundary");

    let found = find_live_session_by_directory_walk(
        &start.display().to_string(),
        Some("work"),
        Some(&outside.display().to_string()),
    )
    .await
    .expect("empty live walk should succeed");

    assert!(found.is_none());
}
