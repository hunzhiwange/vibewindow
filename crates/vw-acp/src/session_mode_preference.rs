//! 会话模式与模型偏好的读取和写回逻辑。

use agent_client_protocol::SessionModelState;

use crate::{SessionAcpxState, SessionRecord, SessionStateOptions};

#[cfg(test)]
#[path = "session_mode_preference_tests.rs"]
mod session_mode_preference_tests;

fn empty_session_vwacp_state() -> SessionAcpxState {
    SessionAcpxState {
        current_mode_id: None,
        desired_mode_id: None,
        current_model_id: None,
        available_models: None,
        available_commands: None,
        config_options: None,
        session_options: None,
    }
}

fn empty_session_state_options() -> SessionStateOptions {
    SessionStateOptions { model: None, allowed_tools: None, max_turns: None }
}

fn ensure_vwacp_state(record: &mut SessionRecord) -> &mut SessionAcpxState {
    record.vwacp.get_or_insert_with(empty_session_vwacp_state)
}

pub fn normalize_mode_id(mode_id: Option<&str>) -> Option<String> {
    let trimmed = mode_id?.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn normalize_model_id(model_id: Option<&str>) -> Option<String> {
    let trimmed = model_id?.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub fn get_desired_mode_id(state: Option<&SessionAcpxState>) -> Option<String> {
    normalize_mode_id(state.and_then(|state| state.desired_mode_id.as_deref()))
}

pub fn set_desired_mode_id(record: &mut SessionRecord, mode_id: Option<&str>) {
    let normalized = normalize_mode_id(mode_id);
    let vwacp = ensure_vwacp_state(record);

    vwacp.desired_mode_id = normalized;
}

pub fn get_desired_model_id(state: Option<&SessionAcpxState>) -> Option<String> {
    normalize_model_id(
        state
            .and_then(|state| state.session_options.as_ref())
            .and_then(|options| options.model.as_deref()),
    )
}

pub fn set_desired_model_id(record: &mut SessionRecord, model_id: Option<&str>) {
    let normalized = normalize_model_id(model_id);
    let vwacp = ensure_vwacp_state(record);
    let mut session_options =
        vwacp.session_options.clone().unwrap_or_else(empty_session_state_options);

    session_options.model = normalized;

    if session_options.model.is_some()
        || session_options.allowed_tools.is_some()
        || session_options.max_turns.is_some()
    {
        vwacp.session_options = Some(session_options);
    } else {
        vwacp.session_options = None;
    }
}

pub fn set_current_model_id(record: &mut SessionRecord, model_id: Option<&str>) {
    let normalized = normalize_model_id(model_id);
    let vwacp = ensure_vwacp_state(record);

    vwacp.current_model_id = normalized;
}

pub fn sync_advertised_model_state(record: &mut SessionRecord, models: Option<&SessionModelState>) {
    let Some(models) = models else {
        return;
    };

    let vwacp = ensure_vwacp_state(record);
    vwacp.current_model_id = Some(models.current_model_id.to_string());
    vwacp.available_models =
        Some(models.available_models.iter().map(|model| model.model_id.to_string()).collect());
}
