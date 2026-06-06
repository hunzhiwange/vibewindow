//! 任务执行器的 runner.rs 子模块。
//!
//! 该模块聚焦任务运行过程中的一个局部职责，供执行器入口组合调用。注释说明边界、错误传播和平台差异，避免调用方需要阅读完整执行链才能理解行为。

#[cfg(not(target_arch = "wasm32"))]
use super::backend_output::build_model_prompt;
#[cfg(not(target_arch = "wasm32"))]
use super::process_utils::{emit_stderr_log, emit_stdout_log};
use super::state::TaskLogStream;
#[cfg(not(target_arch = "wasm32"))]
use super::state::{
    GIT_SOURCE_BRANCH_TAG, GIT_SUMMARY_TAG, GIT_TARGET_BRANCH_TAG, GIT_WORKTREE_PATH_TAG,
    SelectedExecutionWorkspace,
};
#[cfg(not(target_arch = "wasm32"))]
use super::worktree_admin::resolve_task_execution_workspace;
#[cfg(not(target_arch = "wasm32"))]
use super::worktree_pool::{lock_merge_target, task_merge_lock_holder, unlock_merge_target};
use super::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::task::{SubTask, TASK_AGENT_MAIN, normalize_task_acp_agent_input};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::{Map, Value, json};

#[cfg(not(target_arch = "wasm32"))]
fn flush_gateway_output_lines(
    pending: &mut String,
    sender: Option<&Sender<TaskLogStream>>,
    force: bool,
) {
    while let Some(pos) = pending.find('\n') {
        let line = pending[..pos].trim_end_matches('\r').to_string();
        pending.drain(..pos + 1);
        if !line.trim().is_empty() {
            emit_stdout_log(sender, line);
        }
    }

    if force {
        let line = pending.trim();
        if !line.is_empty() {
            emit_stdout_log(sender, line.to_string());
        }
        pending.clear();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn gateway_prompt_options(execution_path: &str, acp_enabled: bool) -> Map<String, Value> {
    let mut options = Map::new();
    options.insert("acp_test".to_string(), json!(acp_enabled));
    options.insert("cwd".to_string(), json!(execution_path));
    options.insert("full_access".to_string(), json!(true));
    if acp_enabled {
        options.insert("acp_permission_mode".to_string(), json!("approve-all"));
        options.insert("acp_force_new_session".to_string(), json!(true));
        options.insert("acp_history_strategy".to_string(), json!("discard"));
        options.insert("acp_history_recent_count".to_string(), json!(1));
    }
    options
}

/// 公开的 execute_gateway_prompt_with_streaming 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn execute_gateway_prompt_with_streaming(
    session_id: &str,
    execution_path: &str,
    model: &str,
    prompt: &str,
    agent: Option<String>,
    acp_agent: Option<String>,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    let endpoint = crate::app::config::gateway_client_endpoint();
    let acp_enabled = acp_agent.is_some();
    let agent_enabled = agent.is_some();
    let route_label = if acp_enabled {
        "acp"
    } else if agent_enabled {
        "agent"
    } else {
        "model"
    };
    let selected_agent = acp_agent.clone().unwrap_or_else(|| "disabled".to_string());
    let selected_delegate_agent = agent.clone().unwrap_or_else(|| "disabled".to_string());

    emit_stdout_log(
        sender,
        format!(
            "[GATEWAY] endpoint={} session={} route={} agent={} acp_agent={} model={} cwd={}",
            endpoint.describe(),
            session_id,
            route_label,
            selected_delegate_agent,
            selected_agent,
            model,
            execution_path
        ),
    );

    let mut output = String::new();
    let mut pending = String::new();
    let mut stream_error: Option<String> = None;
    let mut options = gateway_prompt_options(execution_path, acp_enabled);
    if agent_enabled {
        options.insert("agent".to_string(), json!(agent));
    }
    if acp_enabled {
        options.insert("acp_agent".to_string(), json!(acp_agent));
    }
    let request = vw_gateway_client::GatewayChatStreamRequest {
        session_id: Some(session_id.into()),
        messages: vec![json!({ "role": "user", "content": prompt })],
        system: None,
        model: (model != "auto").then(|| model.to_string()),
        agent,
        allowed_tools: None,
        acp_agent: acp_enabled.then_some(acp_agent).flatten(),
        acp_allowed_tools: None,
        options: Some(serde_json::Value::Object(options)),
    };

    let result = vw_gateway_client::GatewayClient::stream_chat_blocking(
        &endpoint,
        Some(execution_path),
        &request,
        |event| match event {
            vw_gateway_client::GatewayChatStreamEvent::Delta(delta) => {
                output.push_str(&delta);
                pending.push_str(&delta);
                flush_gateway_output_lines(&mut pending, sender, false);
                true
            }
            vw_gateway_client::GatewayChatStreamEvent::Done { finish_reason, .. } => {
                flush_gateway_output_lines(&mut pending, sender, true);
                emit_stdout_log(
                    sender,
                    format!(
                        "[GATEWAY] done finish_reason={}",
                        finish_reason.unwrap_or_else(|| "unknown".to_string())
                    ),
                );
                true
            }
            vw_gateway_client::GatewayChatStreamEvent::Error(error) => {
                flush_gateway_output_lines(&mut pending, sender, true);
                emit_stderr_log(sender, format!("[GATEWAY] error {}", error));
                stream_error = Some(error);
                false
            }
            vw_gateway_client::GatewayChatStreamEvent::Other(payload) => {
                match payload.get("type").and_then(serde_json::Value::as_str) {
                    Some("chat.step_start") => {
                        let step_index = payload
                            .get("step_index")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default();
                        emit_stdout_log(sender, format!("[GATEWAY] step_start {}", step_index));
                    }
                    Some("chat.step_finish") => {
                        let step_index = payload
                            .get("step_index")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or_default();
                        emit_stdout_log(sender, format!("[GATEWAY] step_finish {}", step_index));
                    }
                    _ => {}
                }
                true
            }
        },
    );

    if let Err(error) = result {
        let _ = sender.map(|stream| {
            stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
        });
        return Err(format!("gateway 调度失败: {}", error));
    }

    if let Some(error) = stream_error {
        let _ = sender.map(|stream| {
            stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
        });
        return Err(error);
    }

    flush_gateway_output_lines(&mut pending, sender, true);
    let _ = sender.map(|stream| {
        stream.send(TaskLogStream::ExitStatus { success: true, code: Some(0), signal: None })
    });
    Ok(output)
}

/// 公开的 execute_gateway_prompt_with_streaming 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn execute_gateway_prompt_with_streaming(
    _session_id: &str,
    _execution_path: &str,
    _model: &str,
    _prompt: &str,
    _agent: Option<String>,
    _acp_agent: Option<String>,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    let _ = sender.map(|stream| {
        stream.send(TaskLogStream::ExitStatus { success: false, code: None, signal: None })
    });
    Err("WASM 平台暂不支持通过网关执行流式任务".to_string())
}

/// 模块内部可见的 resolve_task_execution_acp_agent 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn resolve_task_execution_acp_agent(task: &Task) -> Option<String> {
    task.acp_agent.as_deref().and_then(normalize_task_acp_agent_input)
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_with_selected_backend(
    task: &Task,
    execution_path: &str,
    model: &str,
    prompt: &str,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<String, String> {
    execute_gateway_prompt_with_streaming(
        &task_session_id(&task.id),
        execution_path,
        model,
        prompt,
        task.agent.clone().or_else(|| Some(TASK_AGENT_MAIN.to_string())),
        resolve_task_execution_acp_agent(task),
        sender,
    )
}

#[derive(Debug, Clone)]
pub struct TaskPlanGenerationOutcome {
    pub subtasks: Vec<TaskPlanSubTask>,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPlanSubTask {
    pub title: String,
    pub boundary: String,
    pub acceptance_criteria: Vec<String>,
    pub target_files: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TaskPlanSubTask {
    fn from_title(title: String) -> Self {
        Self {
            title,
            boundary: String::new(),
            acceptance_criteria: Vec::new(),
            target_files: Vec::new(),
        }
    }

    fn fallback(fallback: &str) -> Self {
        let title = fallback.trim();
        let title =
            if title.is_empty() { "完成原始需求".to_string() } else { title.to_string() };
        Self {
            title,
            boundary: "完成原始需求中当前任务范围内的全部必要变更。".to_string(),
            acceptance_criteria: vec!["原始需求描述的行为可以被验证。".to_string()],
            target_files: Vec::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn task_plan_root_dir(project_path: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(project_path);
    path.push(".vibewindow");
    path.push("tasks");
    path.push("plan");
    path
}

#[cfg(not(target_arch = "wasm32"))]
fn task_plan_dir(project_path: &str, task_id: &str) -> std::path::PathBuf {
    let mut path = task_plan_root_dir(project_path);
    path.push(task_id);
    path
}

#[cfg(not(target_arch = "wasm32"))]
fn task_plan_file_path(project_path: &str, task_id: &str) -> std::path::PathBuf {
    let mut path = task_plan_dir(project_path, task_id);
    path.push("plan.md");
    path
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_task_plan_file_part(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            output.push(ch);
        } else if ch == '.' {
            output.push('-');
        }
    }
    if output.is_empty() { "subtask".to_string() } else { output }
}

#[cfg(not(target_arch = "wasm32"))]
fn subtask_plan_file_name(index: usize, subtask: &SubTask) -> String {
    format!("{:03}-{}.md", index + 1, sanitize_task_plan_file_part(&subtask.id))
}

#[cfg(not(target_arch = "wasm32"))]
fn subtask_plan_file_path(
    project_path: &str,
    task_id: &str,
    index: usize,
    subtask: &SubTask,
) -> std::path::PathBuf {
    let mut path = task_plan_dir(project_path, task_id);
    path.push(subtask_plan_file_name(index, subtask));
    path
}

#[cfg(not(target_arch = "wasm32"))]
fn format_task_plan_duration(ms: Option<u64>) -> String {
    let Some(ms) = ms else {
        return "-".to_string();
    };
    let total_secs = ms / 1000;
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    if minutes > 0 { format!("{minutes}m{seconds}s") } else { format!("{seconds}s") }
}

#[cfg(not(target_arch = "wasm32"))]
fn format_task_plan_started_at(ms: Option<u64>) -> String {
    static FMT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
        once_cell::sync::Lazy::new(|| {
            time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .unwrap_or_default()
        });

    let Some(ms) = ms else {
        return "-".to_string();
    };
    let nanos = (ms as i128).saturating_mul(1_000_000);
    time::OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .ok()
        .and_then(|dt| dt.format(&FMT).ok())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn subtask_status_label(status: crate::app::task::SubTaskStatus) -> &'static str {
    match status {
        crate::app::task::SubTaskStatus::Pending => "pending",
        crate::app::task::SubTaskStatus::Running => "running",
        crate::app::task::SubTaskStatus::Completed => "completed",
        crate::app::task::SubTaskStatus::Failed => "failed",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn format_markdown_list(items: &[String], empty: &str) -> String {
    if items.is_empty() {
        format!("- {empty}\n")
    } else {
        let mut output = String::new();
        for item in items {
            output.push_str("- ");
            output.push_str(item.trim());
            output.push('\n');
        }
        output
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_task_plan_files(project_path: &str, task: &Task) -> Result<(), String> {
    let dir = task_plan_dir(project_path, &task.id);
    std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;

    let expected_files = task
        .subtasks
        .iter()
        .enumerate()
        .map(|(index, subtask)| subtask_plan_file_name(index, subtask))
        .collect::<std::collections::HashSet<_>>();

    std::fs::write(task_plan_file_path(project_path, &task.id), build_task_plan_markdown(task))
        .map_err(|err| err.to_string())?;
    for (index, subtask) in task.subtasks.iter().enumerate() {
        std::fs::write(
            subtask_plan_file_path(project_path, &task.id, index, subtask),
            build_subtask_plan_markdown(task, subtask, index, task.subtasks.len()),
        )
        .map_err(|err| err.to_string())?;
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if file_name != "plan.md" && !expected_files.contains(file_name) {
                std::fs::remove_file(path).map_err(|err| err.to_string())?;
            }
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn write_task_plan_files(_project_path: &str, _task: &Task) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_task_plan_markdown(task: &Task) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {}\n\n", task.id));
    output.push_str(&format!("- 状态: {}\n", task.status.label()));
    output.push_str(&format!("- 描述: {}\n\n", task.description.trim()));
    output.push_str("## 原始需求\n\n");
    output.push_str(task.prompt.trim());
    output.push_str("\n\n## 子任务\n\n");
    for (index, subtask) in task.subtasks.iter().enumerate() {
        let file_name = subtask_plan_file_name(index, subtask);
        output.push_str(&format!(
            "{}. [{}] [{}]({}) {}\n",
            index + 1,
            subtask_status_label(subtask.status),
            subtask.content.trim(),
            file_name,
            subtask.boundary.trim()
        ));
        output.push_str(&format!(
            "   - 目标文件: {}\n   - 开始时间: {}\n   - 执行耗时: {}\n",
            if subtask.target_files.is_empty() {
                "未指定".to_string()
            } else {
                subtask.target_files.join(", ")
            },
            format_task_plan_started_at(subtask.execution_started_at_ms),
            format_task_plan_duration(
                subtask.display_execution_duration_ms(crate::app::time::now_ms())
            )
        ));
    }
    output
}

#[cfg(not(target_arch = "wasm32"))]
fn build_subtask_plan_markdown(
    task: &Task,
    subtask: &SubTask,
    index: usize,
    total: usize,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("# 子任务 {:03}: {}\n\n", index + 1, subtask.content.trim()));
    output.push_str(&format!("- 任务ID: {}\n", task.id));
    output.push_str(&format!("- 子任务ID: {}\n", subtask.id));
    output.push_str(&format!("- 状态: {}\n", subtask_status_label(subtask.status)));
    output.push_str(&format!("- 顺序: {}/{}\n", index + 1, total));
    output.push_str(&format!(
        "- 开始时间: {}\n",
        format_task_plan_started_at(subtask.execution_started_at_ms)
    ));
    output.push_str(&format!(
        "- 执行耗时: {}\n\n",
        format_task_plan_duration(
            subtask.display_execution_duration_ms(crate::app::time::now_ms())
        )
    ));

    output.push_str("## 边界\n\n");
    let boundary = subtask.boundary.trim();
    if boundary.is_empty() {
        output.push_str("完成本子任务标题所描述的范围，不处理后续子任务。\n\n");
    } else {
        output.push_str(boundary);
        output.push_str("\n\n");
    }

    output.push_str("## 需要修改的文件\n\n");
    output.push_str(&format_markdown_list(&subtask.target_files, "未指定，由执行者按边界判断"));
    output.push_str("\n## 验收条件\n\n");
    output.push_str(&format_markdown_list(
        &subtask.acceptance_criteria,
        "完成后能用本子任务边界中的行为或检查方式验证",
    ));
    output.push_str("\n## 执行约束\n\n");
    output.push_str("- 只执行当前子任务，不开始后续子任务。\n");
    output.push_str("- 如发现必须扩大范围，先在结果中说明阻塞原因和建议边界。\n");
    output
}

#[cfg(not(target_arch = "wasm32"))]
fn build_task_split_prompt(task: &Task) -> String {
    format!(
        "请把下面开发需求拆分成顺序执行的子任务。只输出 JSON，不要解释。\nJSON 格式必须是：\n{{\"subtasks\":[{{\"title\":\"...\",\"boundary\":\"...\",\"acceptance_criteria\":[\"...\"],\"target_files\":[\"path/to/file.rs\"]}}]}}\n要求：\n- 不要拆分太细；每个子任务应该是一个清晰可交付的工作单元。\n- 子任务必须按执行顺序排列，每个子任务单独请求执行。\n- 每个子任务必须有明确边界、验收条件、预计需要修改的文件或目录。\n- target_files 使用仓库相对路径；不确定时写最可能的目录或文件，不要泛写“所有文件”。\n- 不要并行任务，不要生成泛泛的检查项。\n- 不要把执行状态写入 JSON；状态由任务结构维护。\n- 如果需求很小，拆成 1 个子任务。\n\n任务ID: {}\n标题: {}\n需求:\n{}",
        task.id,
        task.description.trim(),
        task.prompt.trim()
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_string_list(value: Option<&serde_json::Value>) -> Vec<String> {
    match value {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
        Some(serde_json::Value::String(value)) => value
            .lines()
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(|item| {
                item.trim_start_matches(|ch: char| {
                    ch == '-' || ch == '*' || ch == '•' || ch.is_ascii_digit() || ch == '.'
                })
                .trim()
                .to_string()
            })
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn first_string_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    names: &[&str],
) -> &'a str {
    for name in names {
        if let Some(value) = object.get(*name).and_then(serde_json::Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    ""
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_task_plan_item(value: &serde_json::Value) -> Option<TaskPlanSubTask> {
    if let Some(title) = value.as_str().map(str::trim).filter(|value| !value.is_empty()) {
        return Some(TaskPlanSubTask::from_title(title.to_string()));
    }

    let object = value.as_object()?;
    let title = first_string_field(object, &["title", "content", "task", "name"]);
    if title.is_empty() {
        return None;
    }
    let boundary = first_string_field(object, &["boundary", "scope"]).to_string();
    let acceptance_criteria = parse_string_list(
        object
            .get("acceptance_criteria")
            .or_else(|| object.get("acceptance"))
            .or_else(|| object.get("验收条件")),
    );
    let target_files = parse_string_list(
        object
            .get("target_files")
            .or_else(|| object.get("files"))
            .or_else(|| object.get("modified_files"))
            .or_else(|| object.get("需要修改的文件")),
    );

    Some(TaskPlanSubTask { title: title.to_string(), boundary, acceptance_criteria, target_files })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_task_plan_items(value: &serde_json::Value) -> Vec<TaskPlanSubTask> {
    let items = value.get("subtasks").and_then(serde_json::Value::as_array);
    let Some(items) = items else {
        return Vec::new();
    };

    items.iter().filter_map(parse_task_plan_item).collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_fenced_json_candidates(output: &str, candidates: &mut Vec<String>) {
    let mut in_fence = false;
    let mut current = String::new();
    for line in output.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                if !current.trim().is_empty() {
                    candidates.push(current.trim().to_string());
                }
                current.clear();
                in_fence = false;
            } else {
                in_fence = true;
            }
            continue;
        }
        if in_fence {
            current.push_str(line);
            current.push('\n');
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_balanced_json_candidates(output: &str, candidates: &mut Vec<String>) {
    for (start, first) in output.char_indices() {
        if !matches!(first, '{' | '[') {
            continue;
        }
        let mut stack = vec![first];
        let mut in_string = false;
        let mut escaped = false;
        for (offset, ch) in output[start + first.len_utf8()..].char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '{' | '[' => stack.push(ch),
                '}' => {
                    if stack.pop() != Some('{') {
                        break;
                    }
                }
                ']' => {
                    if stack.pop() != Some('[') {
                        break;
                    }
                }
                _ => {}
            }

            if stack.is_empty() {
                let end = start + first.len_utf8() + offset + ch.len_utf8();
                candidates.push(output[start..end].to_string());
                break;
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_task_split_output(output: &str, fallback: &str) -> Vec<TaskPlanSubTask> {
    let trimmed = output.trim();
    let mut candidates = vec![trimmed.to_string()];
    collect_fenced_json_candidates(trimmed, &mut candidates);
    collect_balanced_json_candidates(trimmed, &mut candidates);
    candidates.reverse();

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate) {
            let parsed = parse_task_plan_items(&value);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    vec![TaskPlanSubTask::fallback(fallback)]
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_plan_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<TaskPlanGenerationOutcome, String>) {
    let sender_ref = log_sender.as_ref();
    let model = if task.model == "auto" { "auto".to_string() } else { task.model.clone() };
    let prompt = build_task_split_prompt(&task);
    let result = execute_gateway_prompt_with_streaming(
        &format!("{}-plan", task_session_id(&task.id)),
        &project_path,
        &model,
        &prompt,
        task.agent.clone().or_else(|| Some(TASK_AGENT_MAIN.to_string())),
        resolve_task_execution_acp_agent(&task),
        sender_ref,
    )
    .map(|raw_output| TaskPlanGenerationOutcome {
        subtasks: parse_task_split_output(&raw_output, &task.prompt),
        raw_output,
    });
    (task.id, result)
}

pub async fn execute_task_plan_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<TaskPlanGenerationOutcome, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_plan_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("任务拆分线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Task planning not supported on Web".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct GitPrepareOutcome {
    summary: String,
    source_branch: Option<String>,
    target_branch: Option<String>,
    worktree_path: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
struct GatewayGitContext {
    project_id: vw_gateway_client::vw_api_types::id::ProjectId,
    worktree_id: Option<vw_gateway_client::vw_api_types::id::WorktreeId>,
}

#[cfg(not(target_arch = "wasm32"))]
fn format_git_summary(add: &str, commit: &str, merge: &str) -> String {
    format!("Git动作摘要: add={}; commit={}; merge={}", add, commit, merge)
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_gateway_path(value: &str) -> String {
    value.replace('\\', "/").trim_end_matches('/').to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_gateway_git_context(
    project_path: &str,
    git_directory: &str,
) -> Result<GatewayGitContext, String> {
    let client = crate::app::config::gateway_client()?;
    let project = super::block_on_gateway(client.project_resolve(
        &vw_gateway_client::vw_api_types::project::ResolveProjectRequest {
            directory: project_path.to_string(),
            create_if_missing: true,
        },
    ))?
    .project;

    let target_directory = normalize_gateway_path(git_directory);
    if target_directory == normalize_gateway_path(&project.directory) {
        return Ok(GatewayGitContext { project_id: project.id, worktree_id: None });
    }

    let worktree_id = super::block_on_gateway(client.project_worktrees(&project.id.0))?
        .items
        .into_iter()
        .find(|worktree| normalize_gateway_path(&worktree.directory) == target_directory)
        .map(|worktree| worktree.id);

    let Some(worktree_id) = worktree_id else {
        return Err(format!("网关未登记任务 worktree，拒绝绕过网关执行 git: {}", git_directory));
    };

    Ok(GatewayGitContext { project_id: project.id, worktree_id: Some(worktree_id) })
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_git_prepare_actions(
    task: &Task,
    project_path: &str,
    workspace: &SelectedExecutionWorkspace,
    sender: Option<&Sender<TaskLogStream>>,
) -> Result<GitPrepareOutcome, String> {
    emit_stdout_log(sender, "[GATEWAY GIT] 请求网关执行 add/commit 自动流程".to_string());
    let context = resolve_gateway_git_context(project_path, &workspace.execution_path)?;
    let client = crate::app::config::gateway_client()?;
    let commit_message = build_task_commit_message(task, &format!("task({})", task.id));
    let response = super::block_on_gateway(client.git_commit(
        &vw_gateway_client::vw_api_types::git::GitCommitRequest {
            project_id: context.project_id,
            worktree_id: context.worktree_id,
            message: commit_message.clone(),
            stage_all: true,
            selected_files: Vec::new(),
            selected_hunks: Vec::new(),
            selected_lines: Vec::new(),
            selected_old_lines: Vec::new(),
        },
    ))?;
    if !response.ok {
        return Err("网关未确认 git commit 成功".to_string());
    }
    emit_stdout_log(
        sender,
        format!(
            "[GATEWAY GIT] commit 成功 sha={} message={}",
            response.commit.sha, response.commit.message
        ),
    );
    let add_status = "成功(网关)".to_string();
    let commit_status = format!("成功({})", commit_message);
    let merge_status = "跳过(等待审核后再合并)".to_string();

    Ok(GitPrepareOutcome {
        summary: format_git_summary(&add_status, &commit_status, &merge_status),
        source_branch: workspace.selected_worktree_branch.clone(),
        target_branch: workspace.merge_target_branch.clone(),
        worktree_path: workspace.selected_worktree_path.clone(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn read_plan_context(
    project_path: &str,
    task: &Task,
    subtask: &SubTask,
    index: usize,
    total: usize,
) -> (String, String) {
    let plan = std::fs::read_to_string(task_plan_file_path(project_path, &task.id))
        .unwrap_or_else(|_| build_task_plan_markdown(task));
    let subtask_plan =
        std::fs::read_to_string(subtask_plan_file_path(project_path, &task.id, index, subtask))
            .unwrap_or_else(|_| build_subtask_plan_markdown(task, subtask, index, total));
    (plan, subtask_plan)
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_subtask_status(
    project_path: &str,
    task: &mut Task,
    subtask_id: &str,
    update: impl FnOnce(&mut SubTask),
    sender: Option<&Sender<TaskLogStream>>,
) {
    if let Some(subtask) = task.subtasks.iter_mut().find(|item| item.id == subtask_id) {
        update(subtask);
        if let Err(error) = write_task_plan_files(project_path, task) {
            emit_stderr_log(sender, format!("[PLAN] 子任务状态写入失败: {}", error));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_subtask_execution_prompt(
    task: &Task,
    plan_markdown: &str,
    subtask_markdown: &str,
) -> String {
    format!(
        "请单独执行当前子任务.md 中定义的子任务。不要开始后续子任务，也不要重新拆分任务。\n\n# plan.md\n{}\n\n# 子任务.md\n{}\n\n# 原始需求\n{}",
        plan_markdown.trim(),
        subtask_markdown.trim(),
        task.prompt.trim()
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    let sender_ref = log_sender.as_ref();
    let (workspace, _claim_guard) =
        match resolve_task_execution_workspace(&task, &project_path, sender_ref) {
            Ok(result) => result,
            Err(error) => return (task.id, Err(error)),
        };

    let model = if task.model == "auto" { "auto".to_string() } else { task.model.clone() };
    let subtasks = if task.subtasks.is_empty() {
        vec![SubTask::new(task.prompt.clone())]
    } else {
        task.subtasks.clone()
    };
    let mut task_snapshot = task.clone();
    if task_snapshot.subtasks.is_empty() {
        task_snapshot.subtasks = subtasks.clone();
    }
    let mut execution_output = String::new();
    let mut execution_result: Result<String, String> = Ok(String::new());
    for (index, subtask) in subtasks.iter().enumerate() {
        persist_subtask_status(
            &project_path,
            &mut task_snapshot,
            &subtask.id,
            SubTask::start_execution,
            sender_ref,
        );
        if let Some(sender) = sender_ref {
            let _ = sender.send(TaskLogStream::SubTaskStarted {
                subtask_id: subtask.id.clone(),
                content: subtask.content.clone(),
            });
        }
        let (plan_markdown, subtask_markdown) =
            read_plan_context(&project_path, &task_snapshot, subtask, index, subtasks.len());
        let subtask_prompt =
            build_subtask_execution_prompt(&task, &plan_markdown, &subtask_markdown);
        let model_prompt = build_model_prompt(&task, &subtask_prompt);
        match execute_task_with_selected_backend(
            &task,
            &workspace.execution_path,
            &model,
            &model_prompt,
            sender_ref,
        ) {
            Ok(output) => {
                persist_subtask_status(
                    &project_path,
                    &mut task_snapshot,
                    &subtask.id,
                    SubTask::mark_completed,
                    sender_ref,
                );
                if let Some(sender) = sender_ref {
                    let _ = sender
                        .send(TaskLogStream::SubTaskCompleted { subtask_id: subtask.id.clone() });
                }
                if !execution_output.is_empty() {
                    execution_output.push_str("\n\n");
                }
                execution_output.push_str(&output);
            }
            Err(error) => {
                persist_subtask_status(
                    &project_path,
                    &mut task_snapshot,
                    &subtask.id,
                    SubTask::mark_failed,
                    sender_ref,
                );
                if let Some(sender) = sender_ref {
                    let _ = sender.send(TaskLogStream::SubTaskFailed {
                        subtask_id: subtask.id.clone(),
                        error: error.clone(),
                    });
                }
                execution_result = Err(error);
                break;
            }
        }
    }
    if execution_result.is_ok() {
        execution_result = Ok(execution_output);
    }

    let final_result = match execution_result {
        Ok(exec) => match execute_git_prepare_actions(&task, &project_path, &workspace, sender_ref)
        {
            Ok(git) => {
                let mut output = format!("{}\n{}:{}", exec, GIT_SUMMARY_TAG, git.summary);
                if let Some(source) = git.source_branch {
                    output.push_str(&format!("\n{}:{}", GIT_SOURCE_BRANCH_TAG, source));
                }
                if let Some(target) = git.target_branch {
                    output.push_str(&format!("\n{}:{}", GIT_TARGET_BRANCH_TAG, target));
                }
                if let Some(worktree_path) = git.worktree_path {
                    output.push_str(&format!("\n{}:{}", GIT_WORKTREE_PATH_TAG, worktree_path));
                }
                Ok(output)
            }
            Err(git_err) => Err(format!("任务执行成功，但 Git 流程失败: {}", git_err)),
        },
        Err(exec_err) => {
            emit_stdout_log(
                sender_ref,
                format!("[GATEWAY GIT] 跳过 add/commit/merge，原因=执行失败: {}", exec_err),
            );
            Err(exec_err)
        }
    };

    (task.id, final_result)
}

/// 公开的 execute_task_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || execute_task_blocking(task, project_path, log_sender))
            .await
            .unwrap_or_else(|error| (task_id, Err(format!("任务执行线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Task execution not supported on Web".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_review_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    let sender_ref = log_sender.as_ref();
    let model = if task.model == "auto" { "auto".to_string() } else { task.model.clone() };
    let model_prompt = build_model_prompt(&task, &task.prompt);
    let execution_result =
        execute_task_with_selected_backend(&task, &project_path, &model, &model_prompt, sender_ref);
    (task.id, execution_result)
}

/// 公开的 execute_task_review_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_review_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_review_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("代码审核线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Code review not supported on Web".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute_task_merge_blocking(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    struct MergeTargetLockGuard<'a> {
        project_path: &'a str,
        task: &'a Task,
        sender: Option<&'a Sender<TaskLogStream>>,
        target_branch: &'a str,
    }

    impl Drop for MergeTargetLockGuard<'_> {
        fn drop(&mut self) {
            unlock_merge_target(self.project_path, self.task);
            emit_stdout_log(
                self.sender,
                format!(
                    "[GATEWAY GIT MERGE] lock released task_id={} target={} holder_after_release={}",
                    self.task.id,
                    self.target_branch,
                    task_merge_lock_holder(self.project_path, self.task)
                        .unwrap_or_else(|| "none".to_string())
                ),
            );
        }
    }

    let sender_ref = log_sender.as_ref();
    let task_id = task.id.clone();
    lock_merge_target(&project_path, &task);
    let source_branch = task.merge_source_branch.clone().unwrap_or_default();
    let target_branch = task.merge_target_branch.clone().unwrap_or_default();
    let _merge_lock_guard = MergeTargetLockGuard {
        project_path: &project_path,
        task: &task,
        sender: sender_ref,
        target_branch: &target_branch,
    };
    emit_stdout_log(
        sender_ref,
        format!(
            "[GATEWAY GIT MERGE] start task={} {} -> {}",
            task.id, source_branch, target_branch
        ),
    );

    let merge_result = if source_branch.trim().is_empty() || target_branch.trim().is_empty() {
        emit_stdout_log(sender_ref, "[GATEWAY GIT MERGE] 未发现可合并分支，跳过".to_string());
        Ok("合并跳过: 无可合并分支".to_string())
    } else if source_branch == target_branch || source_branch == "HEAD" {
        emit_stdout_log(
            sender_ref,
            format!(
                "[GATEWAY GIT MERGE] 合并分支无效，跳过 source={} target={}",
                source_branch, target_branch
            ),
        );
        Ok(format!("合并跳过: source={} target={}", source_branch, target_branch))
    } else {
        let client = match crate::app::config::gateway_client() {
            Ok(client) => client,
            Err(error) => return (task_id, Err(error)),
        };
        let context = match resolve_gateway_git_context(&project_path, &project_path) {
            Ok(context) => context,
            Err(error) => return (task_id, Err(error)),
        };
        let request = vw_gateway_client::vw_api_types::git::GitMergeRequest {
            project_id: context.project_id,
            source_branch: source_branch.clone(),
            target_branch: target_branch.clone(),
        };
        match super::block_on_gateway(client.git_merge(&request)) {
            Ok(response) if response.ok && response.already_merged => {
                emit_stdout_log(
                    sender_ref,
                    format!(
                        "[GATEWAY GIT MERGE] source 已在 target 中，跳过 source={} target={} workspace={}",
                        response.source_branch, response.target_branch, response.workspace
                    ),
                );
                Ok(format!(
                    "合并跳过: source={} 已包含于 target={} workspace={}",
                    response.source_branch, response.target_branch, response.workspace
                ))
            }
            Ok(response) if response.ok => {
                emit_stdout_log(
                    sender_ref,
                    format!(
                        "[GATEWAY GIT MERGE] merge 成功 workspace={} source={} target={}",
                        response.workspace, response.source_branch, response.target_branch
                    ),
                );
                Ok(format!(
                    "Git动作摘要: add=跳过; commit=跳过; merge=成功({}->{}) workspace={}",
                    response.source_branch, response.target_branch, response.workspace
                ))
            }
            Ok(_) => Err("网关未确认 merge 成功".to_string()),
            Err(error) => Err(format!(
                "网关合并失败 source={} target={} err={}",
                source_branch, target_branch, error
            )),
        }
    };

    if let Err(error) = &merge_result {
        emit_stderr_log(
            sender_ref,
            format!(
                "[GATEWAY GIT MERGE] completed task_id={} source={} target={} result=error err={}",
                task_id, source_branch, target_branch, error
            ),
        );
    }

    (task_id, merge_result)
}

/// 公开的 execute_task_merge_async 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub async fn execute_task_merge_async(
    task: Task,
    project_path: String,
    log_sender: Option<Sender<TaskLogStream>>,
) -> (String, Result<String, String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let task_id = task.id.clone();
        tokio::task::spawn_blocking(move || {
            execute_task_merge_blocking(task, project_path, log_sender)
        })
        .await
        .unwrap_or_else(|error| (task_id, Err(format!("代码合并线程异常: {}", error))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_path, log_sender);
        (task.id, Err("Task merge not supported on Web".to_string()))
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod runner_tests;
