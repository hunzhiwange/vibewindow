//! 权限请求的决策分类与模式匹配逻辑。
//!
//! 本模块集中处理工具调用时的权限决策，负责根据当前权限模式、
//! 非交互策略和请求内容判断是自动放行、拒绝，还是进入确认流程。
//!
//! 该层的目标是让权限语义保持可预测，避免运行时各处重复实现分散的判断分支。

use agent_client_protocol::{
    PermissionOption, PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome,
};
use serde_json::Value;

use crate::errors::PermissionPromptUnavailableError;
use crate::permission_prompt::{
    PermissionPromptOptions, can_prompt_for_permission, prompt_for_permission,
};
use crate::types::{NonInteractivePermissionPolicy, PermissionMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Approved,
    Denied,
    Cancelled,
}

fn permission_mode_rank(mode: PermissionMode) -> u8 {
    match mode {
        PermissionMode::DenyAll => 0,
        PermissionMode::ApproveReads => 1,
        PermissionMode::ApproveAll => 2,
    }
}

fn selected(option_id: String) -> RequestPermissionResponse {
    serde_json::from_value(serde_json::json!({
        "outcome": {
            "outcome": "selected",
            "optionId": option_id
        }
    }))
    .expect("valid ACP permission response")
}

fn cancelled() -> RequestPermissionResponse {
    serde_json::from_value(serde_json::json!({
        "outcome": {
            "outcome": "cancelled"
        }
    }))
    .expect("valid ACP permission cancellation response")
}

fn pick_option<'a>(
    options: &'a [PermissionOption],
    kinds: &[PermissionOptionKind],
) -> Option<&'a PermissionOption> {
    for kind in kinds {
        if let Some(option) = options.iter().find(|option| option.kind == *kind) {
            return Some(option);
        }
    }
    None
}

fn infer_tool_kind_from_title(title: &str) -> Option<&'static str> {
    let head = title
        .trim()
        .to_ascii_lowercase()
        .split(':')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if head.is_empty() {
        return None;
    }
    if head.contains("read") || head.contains("cat") {
        return Some("read");
    }
    if head.contains("search") || head.contains("find") || head.contains("grep") {
        return Some("search");
    }
    if head.contains("write") || head.contains("edit") || head.contains("patch") {
        return Some("edit");
    }
    if head.contains("delete") || head.contains("remove") {
        return Some("delete");
    }
    if head.contains("move") || head.contains("rename") {
        return Some("move");
    }
    if head.contains("run") || head.contains("execute") || head.contains("bash") {
        return Some("execute");
    }
    if head.contains("fetch") || head.contains("http") || head.contains("url") {
        return Some("fetch");
    }
    if head.contains("think") {
        return Some("think");
    }
    Some("other")
}

fn tool_call_record(params: &RequestPermissionRequest) -> Option<serde_json::Map<String, Value>> {
    serde_json::to_value(&params.tool_call).ok()?.as_object().cloned()
}

fn tool_call_string_field(params: &RequestPermissionRequest, field: &str) -> Option<String> {
    tool_call_record(params)?.get(field)?.as_str().map(ToOwned::to_owned)
}

fn infer_tool_kind(params: &RequestPermissionRequest) -> Option<String> {
    tool_call_string_field(params, "kind").or_else(|| {
        tool_call_string_field(params, "title")
            .and_then(|value| infer_tool_kind_from_title(&value).map(str::to_string))
    })
}

fn is_auto_approved_read_kind(kind: Option<&str>) -> bool {
    matches!(kind, Some("read" | "search"))
}

fn prompt_for_tool_permission(params: &RequestPermissionRequest) -> Result<bool, std::io::Error> {
    let tool_name = tool_call_string_field(params, "title").unwrap_or_else(|| "tool".to_string());
    let tool_kind = infer_tool_kind(params).unwrap_or_else(|| "other".to_string());
    prompt_for_permission(&PermissionPromptOptions {
        prompt: format!("\n[permission] Allow {tool_name} [{tool_kind}]? (y/N) "),
        header: None,
        details: None,
    })
}

pub fn permission_mode_satisfies(actual: PermissionMode, required: PermissionMode) -> bool {
    permission_mode_rank(actual) >= permission_mode_rank(required)
}

#[allow(clippy::result_large_err)]
pub fn resolve_permission_request(
    params: &RequestPermissionRequest,
    mode: PermissionMode,
    non_interactive_policy: Option<NonInteractivePermissionPolicy>,
) -> Result<RequestPermissionResponse, PermissionPromptUnavailableError> {
    let options = &params.options;
    if options.is_empty() {
        return Ok(cancelled());
    }

    let allow_option =
        pick_option(options, &[PermissionOptionKind::AllowOnce, PermissionOptionKind::AllowAlways]);
    let reject_option = pick_option(
        options,
        &[PermissionOptionKind::RejectOnce, PermissionOptionKind::RejectAlways],
    );

    match mode {
        PermissionMode::ApproveAll => {
            return Ok(selected(allow_option.unwrap_or(&options[0]).option_id.to_string()));
        }
        PermissionMode::DenyAll => {
            return Ok(reject_option
                .map(|option| selected(option.option_id.to_string()))
                .unwrap_or_else(cancelled));
        }
        PermissionMode::ApproveReads => {}
    }

    let kind = infer_tool_kind(params);
    if is_auto_approved_read_kind(kind.as_deref())
        && let Some(option) = allow_option
    {
        return Ok(selected(option.option_id.to_string()));
    }

    if let Some(policy) = non_interactive_policy {
        return match policy {
            NonInteractivePermissionPolicy::Deny => Ok(reject_option
                .map(|option| selected(option.option_id.to_string()))
                .unwrap_or_else(cancelled)),
            NonInteractivePermissionPolicy::Fail => Err(PermissionPromptUnavailableError::new()),
        };
    }

    if !can_prompt_for_permission() {
        return Ok(reject_option
            .map(|option| selected(option.option_id.to_string()))
            .unwrap_or_else(cancelled));
    }

    let approved = prompt_for_tool_permission(params).unwrap_or(false);
    if approved && let Some(option) = allow_option {
        return Ok(selected(option.option_id.to_string()));
    }
    if !approved && let Some(option) = reject_option {
        return Ok(selected(option.option_id.to_string()));
    }
    Ok(cancelled())
}

pub fn classify_permission_decision(
    params: &RequestPermissionRequest,
    response: &RequestPermissionResponse,
) -> PermissionDecision {
    let RequestPermissionOutcome::Selected(SelectedPermissionOutcome { option_id, .. }) =
        &response.outcome
    else {
        return PermissionDecision::Cancelled;
    };

    let Some(selected_option) = params.options.iter().find(|option| &option.option_id == option_id)
    else {
        return PermissionDecision::Cancelled;
    };

    match selected_option.kind {
        PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways => {
            PermissionDecision::Approved
        }
        PermissionOptionKind::RejectOnce | PermissionOptionKind::RejectAlways => {
            PermissionDecision::Denied
        }
        _ => PermissionDecision::Cancelled,
    }
}

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod permissions_tests;
