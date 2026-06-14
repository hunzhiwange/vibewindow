//! 会话连接与加载流程的单元测试。

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use agent_client_protocol::{ModelInfo, SessionModelState, SetSessionConfigOptionResponse};
use async_trait::async_trait;

use super::connect_load::{
    ConnectAndLoadClient, ConnectAndLoadClientSession, ConnectAndLoadSessionError,
    ConnectAndLoadSessionOptions, connect_and_load_session,
};
use super::lifecycle::{AgentLifecycleExit, AgentLifecycleSnapshot};
use crate::error::AcpError;
use crate::queue_owner_turn_controller::{QueueControlFuture, QueueOwnerActiveSessionController};
use crate::{
    SESSION_RECORD_SCHEMA, SessionAcpxState, SessionEventLog, SessionRecord, SessionResumePolicy,
    SessionStateOptions, SessionTokenUsage,
};

#[derive(Default)]
struct TestActiveController;

impl QueueOwnerActiveSessionController for TestActiveController {
    fn has_active_prompt(&self) -> bool {
        false
    }

    fn request_cancel_active_prompt(&self) -> QueueControlFuture<bool> {
        Box::pin(async move { Ok(false) })
    }

    fn set_session_mode(&self, _mode_id: String) -> QueueControlFuture<()> {
        Box::pin(async move { Ok(()) })
    }

    fn set_session_model(&self, _model_id: String) -> QueueControlFuture<()> {
        Box::pin(async move { Ok(()) })
    }

    fn set_session_config_option(
        &self,
        _config_id: String,
        _value: String,
    ) -> QueueControlFuture<SetSessionConfigOptionResponse> {
        Box::pin(async move { unreachable!() })
    }
}

#[derive(Debug, Default)]
struct MockClientState {
    start_calls: usize,
    create_calls: usize,
    load_calls: usize,
    set_mode_calls: Vec<String>,
    set_model_calls: Vec<String>,
}

struct MockClient {
    state: Arc<Mutex<MockClientState>>,
    reusable_session: bool,
    supports_load_session: bool,
    lifecycle_snapshot: AgentLifecycleSnapshot,
    create_session: ConnectAndLoadClientSession,
    load_session: Result<ConnectAndLoadClientSession, String>,
    set_mode_error: Option<String>,
    set_model_error: Option<String>,
}

impl MockClient {
    fn new(
        create_session: ConnectAndLoadClientSession,
        load_session: Result<ConnectAndLoadClientSession, String>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockClientState::default())),
            reusable_session: false,
            supports_load_session: true,
            lifecycle_snapshot: AgentLifecycleSnapshot::default(),
            create_session,
            load_session,
            set_mode_error: None,
            set_model_error: None,
        }
    }
}

#[async_trait(?Send)]
impl ConnectAndLoadClient for MockClient {
    async fn start(&self) -> Result<(), AcpError> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).start_calls += 1;
        Ok(())
    }

    async fn create_session(&self, _cwd: &Path) -> Result<ConnectAndLoadClientSession, AcpError> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).create_calls += 1;
        Ok(self.create_session.clone())
    }

    async fn load_session_with_options(
        &self,
        _session_id: &str,
        _cwd: &Path,
    ) -> Result<ConnectAndLoadClientSession, AcpError> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).load_calls += 1;
        match &self.load_session {
            Ok(session) => Ok(session.clone()),
            Err(message) => Err(AcpError::LoadSession(message.clone())),
        }
    }

    async fn set_session_mode(
        &self,
        _session_id: &str,
        _cwd: &Path,
        mode_id: &str,
    ) -> Result<(), AcpError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .set_mode_calls
            .push(mode_id.to_string());
        match self.set_mode_error.as_ref() {
            Some(message) => Err(AcpError::SetSessionConfigOption(message.clone())),
            None => Ok(()),
        }
    }

    async fn set_session_model(
        &self,
        _session_id: &str,
        _cwd: &Path,
        model_id: &str,
    ) -> Result<(), AcpError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .set_model_calls
            .push(model_id.to_string());
        match self.set_model_error.as_ref() {
            Some(message) => Err(AcpError::SetSessionModel(message.clone())),
            None => Ok(()),
        }
    }

    fn has_reusable_session(&self, _session_id: &str) -> bool {
        self.reusable_session
    }

    fn supports_load_session(&self) -> bool {
        self.supports_load_session
    }

    fn get_agent_lifecycle_snapshot(&self) -> AgentLifecycleSnapshot {
        self.lifecycle_snapshot.clone()
    }
}

fn sample_record() -> SessionRecord {
    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "session-1".to_string(),
        agent_session_id: Some("agent-1".to_string()),
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:00:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/project/active.jsonl".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 4,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(true),
        closed_at: Some("2026-01-02T00:00:00Z".to_string()),
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
        vwacp: Some(SessionAcpxState {
            current_mode_id: None,
            desired_mode_id: Some("focus".to_string()),
            current_model_id: Some("model-a".to_string()),
            available_models: None,
            available_commands: None,
            config_options: None,
            session_options: Some(SessionStateOptions {
                model: Some("model-b".to_string()),
                allowed_tools: None,
                max_turns: None,
            }),
        }),
    }
}

fn sample_models() -> SessionModelState {
    SessionModelState::new(
        "model-a",
        vec![ModelInfo::new("model-a", "Current Model"), ModelInfo::new("model-b", "Model B")],
    )
}

#[tokio::test]
async fn connect_and_load_session_falls_back_to_new_session_and_replays_preferences() {
    let mut record = sample_record();
    let client = Arc::new(MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("session not found".to_string()),
    ));

    let result = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::AllowNew),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect("fallback to new session should succeed");

    assert_eq!(result.session_id, "session-2");
    assert_eq!(result.agent_session_id.as_deref(), Some("agent-2"));
    assert!(!result.resumed);
    assert_eq!(result.load_error.as_deref(), Some("acp load_session failed: session not found"));
    assert_eq!(record.acp_session_id, "session-2");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-2"));
    assert_eq!(record.closed, Some(false));
    assert_eq!(record.closed_at, None);
    assert_eq!(
        record.vwacp.as_ref().and_then(|state| state.current_model_id.as_deref()),
        Some("model-b")
    );
    assert_eq!(
        record.vwacp.as_ref().and_then(|state| state.available_models.as_ref()),
        Some(&vec!["model-a".to_string(), "model-b".to_string()])
    );

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 1);
    assert_eq!(state.load_calls, 1);
    assert_eq!(state.create_calls, 1);
    assert_eq!(state.set_mode_calls, vec!["focus".to_string()]);
    assert_eq!(state.set_model_calls, vec!["model-b".to_string()]);
}

#[tokio::test]
async fn connect_and_load_session_loads_existing_session_and_updates_record_state() {
    let mut record = sample_record();
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "ignored-loaded-session".to_string(),
            agent_session_id: Some("agent-loaded".to_string()),
            models: Some(sample_models()),
        },
        Ok(ConnectAndLoadClientSession {
            session_id: "ignored-loaded-session".to_string(),
            agent_session_id: Some("agent-loaded".to_string()),
            models: Some(sample_models()),
        }),
    );
    client.lifecycle_snapshot = AgentLifecycleSnapshot {
        pid: Some(1234),
        started_at: Some("2026-01-03T00:00:00Z".to_string()),
        last_exit: Some(AgentLifecycleExit {
            exit_code: Some(0),
            signal: None,
            exited_at: Some("2026-01-03T00:01:00Z".to_string()),
            reason: Some("clean-exit".to_string()),
            unexpected_during_prompt: false,
        }),
    };
    let client = Arc::new(client);
    let client_available_calls = Arc::new(Mutex::new(0usize));
    let connected_records = Arc::new(Mutex::new(Vec::new()));
    let resolved_session_ids = Arc::new(Mutex::new(Vec::new()));

    let result = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: None,
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: Some({
            let client_available_calls = client_available_calls.clone();
            Arc::new(move |_controller| {
                *client_available_calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) +=
                    1;
            })
        }),
        on_connected_record: Some({
            let connected_records = connected_records.clone();
            Arc::new(move |record| {
                connected_records
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(record.acp_session_id.clone());
            })
        }),
        on_session_id_resolved: Some({
            let resolved_session_ids = resolved_session_ids.clone();
            Arc::new(move |session_id| {
                resolved_session_ids
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(session_id.to_string());
            })
        }),
    })
    .await
    .expect("load session should succeed");

    assert_eq!(result.session_id, "session-1");
    assert_eq!(result.agent_session_id.as_deref(), Some("agent-loaded"));
    assert!(result.resumed);
    assert_eq!(result.load_error, None);
    assert_eq!(record.acp_session_id, "session-1");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-loaded"));
    assert_eq!(record.closed, Some(false));
    assert_eq!(record.closed_at, None);
    assert_eq!(record.pid, Some(1234));
    assert_eq!(record.agent_started_at.as_deref(), Some("2026-01-03T00:00:00Z"));
    assert_eq!(record.last_agent_exit_code, Some(0));
    assert_eq!(record.last_agent_disconnect_reason.as_deref(), Some("clean-exit"));
    assert_eq!(
        record.vwacp.as_ref().and_then(|state| state.current_model_id.as_deref()),
        Some("model-a")
    );
    assert_eq!(
        record.vwacp.as_ref().and_then(|state| state.available_models.as_ref()),
        Some(&vec!["model-a".to_string(), "model-b".to_string()])
    );
    assert_eq!(*client_available_calls.lock().unwrap_or_else(|poisoned| poisoned.into_inner()), 1);
    assert_eq!(
        *connected_records.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["session-1".to_string()]
    );
    assert_eq!(
        *resolved_session_ids.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["session-1".to_string()]
    );

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 1);
    assert_eq!(state.load_calls, 1);
    assert_eq!(state.create_calls, 0);
    assert!(state.set_mode_calls.is_empty());
    assert!(state.set_model_calls.is_empty());
}

#[tokio::test]
async fn connect_and_load_session_reuses_loaded_session_without_starting_client() {
    let mut record = sample_record();
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("load should not be called".to_string()),
    );
    client.reusable_session = true;
    let client = Arc::new(client);

    let result = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::SameSessionOnly),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect("reusable session should be accepted");

    assert_eq!(result.session_id, "session-1");
    assert_eq!(result.agent_session_id.as_deref(), Some("agent-1"));
    assert!(result.resumed);
    assert_eq!(result.load_error, None);

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 0);
    assert_eq!(state.load_calls, 0);
    assert_eq!(state.create_calls, 0);
}

#[tokio::test]
async fn connect_and_load_session_creates_new_session_when_load_is_unsupported() {
    let mut record = sample_record();
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: None,
            models: None,
        },
        Err("load should not be called".to_string()),
    );
    client.supports_load_session = false;
    let client = Arc::new(client);

    let result = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::AllowNew),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect("unsupported load should create a fresh session when allowed");

    assert_eq!(result.session_id, "session-2");
    assert_eq!(result.agent_session_id.as_deref(), Some("agent-1"));
    assert!(!result.resumed);
    assert_eq!(record.acp_session_id, "session-2");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-1"));

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 1);
    assert_eq!(state.load_calls, 0);
    assert_eq!(state.create_calls, 1);
}

#[tokio::test]
async fn connect_and_load_session_requires_same_session_when_load_is_unsupported() {
    let mut record = sample_record();
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: None,
            models: None,
        },
        Err("load should not be called".to_string()),
    );
    client.supports_load_session = false;
    let client = Arc::new(client);

    let error = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::SameSessionOnly),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect_err("same-session-only should reject unsupported load");

    assert!(matches!(error, ConnectAndLoadSessionError::SessionResumeRequired(_)));
    assert_eq!(record.acp_session_id, "session-1");

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 1);
    assert_eq!(state.load_calls, 0);
    assert_eq!(state.create_calls, 0);
}

#[tokio::test]
async fn connect_and_load_session_returns_non_fallback_load_error() {
    let mut record = sample_record();
    let client = Arc::new(MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("permission denied".to_string()),
    ));

    let error = connect_and_load_session(ConnectAndLoadSessionOptions {
        client: client.clone(),
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::AllowNew),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect_err("non-fallback load errors should be returned");

    assert!(matches!(error, ConnectAndLoadSessionError::Acp(AcpError::LoadSession(_))));
    assert_eq!(record.acp_session_id, "session-1");

    let state = client.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.start_calls, 1);
    assert_eq!(state.load_calls, 1);
    assert_eq!(state.create_calls, 0);
}

#[tokio::test]
async fn connect_and_load_session_requires_same_session_when_load_fails() {
    let mut record = sample_record();
    let client = Arc::new(MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("session not found".to_string()),
    ));

    let error = connect_and_load_session(ConnectAndLoadSessionOptions {
        client,
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::SameSessionOnly),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect_err("same-session-only should reject fallback");

    assert!(matches!(error, ConnectAndLoadSessionError::SessionResumeRequired(_)));
    assert_eq!(record.acp_session_id, "session-1");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-1"));
}

#[tokio::test]
async fn connect_and_load_session_restores_original_ids_when_mode_replay_fails() {
    let mut record = sample_record();
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("session not found".to_string()),
    );
    client.set_mode_error = Some("mode replay failed".to_string());
    let client = Arc::new(client);

    let error = connect_and_load_session(ConnectAndLoadSessionOptions {
        client,
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::AllowNew),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect_err("mode replay failure should be surfaced");

    assert!(matches!(error, ConnectAndLoadSessionError::SessionModeReplay(_)));
    assert_eq!(record.acp_session_id, "session-1");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-1"));
}

#[tokio::test]
async fn connect_and_load_session_restores_original_ids_when_model_replay_fails() {
    let mut record = sample_record();
    record.vwacp.as_mut().expect("vwacp state").desired_mode_id = None;
    let mut client = MockClient::new(
        ConnectAndLoadClientSession {
            session_id: "session-2".to_string(),
            agent_session_id: Some("agent-2".to_string()),
            models: Some(sample_models()),
        },
        Err("session not found".to_string()),
    );
    client.set_model_error = Some("model replay failed".to_string());
    let client = Arc::new(client);

    let error = connect_and_load_session(ConnectAndLoadSessionOptions {
        client,
        record: &mut record,
        resume_policy: Some(SessionResumePolicy::AllowNew),
        timeout_ms: Some(5_000),
        verbose: false,
        active_controller: Arc::new(TestActiveController),
        on_client_available: None,
        on_connected_record: None,
        on_session_id_resolved: None,
    })
    .await
    .expect_err("model replay failure should be surfaced");

    assert!(matches!(error, ConnectAndLoadSessionError::SessionModelReplay(_)));
    assert_eq!(record.acp_session_id, "session-1");
    assert_eq!(record.agent_session_id.as_deref(), Some("agent-1"));
}
