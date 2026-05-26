//! question 与 todo 访问封装。
//!
//! 本模块为新 TUI 提供最小、稳定的控制面入口，覆盖：
//! - question 的拉取、过滤、回复与拒绝
//! - session todo 的拉取与覆盖更新
//! - 共享 `Todo` 与 gateway 请求体之间的窄转换逻辑

use vw_gateway_client::vw_api_types::todo::{TodoPriority, TodoStatus};
use vw_gateway_client::{GatewaySessionTodoItem, GatewaySessionTodoPutBody};
use vw_shared::question;
use vw_shared::todo::Todo;

#[cfg(not(target_arch = "wasm32"))]
use super::gateway::block_on_gateway;
use super::gateway::{GatewayUiRuntime, normalize_optional_str_ref};

impl GatewayUiRuntime {
    /// 拉取当前所有待处理 question 请求。
    pub(crate) async fn question_list_all(&self) -> Result<Vec<question::Request>, String> {
        self.client().question_list().await
    }

    /// 拉取指定会话的 question 请求；未传 `session_id` 时回退到 runtime 当前会话。
    ///
    /// 如果 runtime 当前也没有会话 ID，则返回全部 question，请由上层自行决定后续过滤策略。
    pub(crate) async fn question_list_for_session(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<question::Request>, String> {
        let session_id = normalize_optional_str_ref(session_id).or_else(|| self.session_id());
        let requests = self.question_list_all().await?;
        Ok(filter_questions_for_session(requests, session_id))
    }

    /// 回复指定的 question 请求。
    pub(crate) async fn question_reply(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
    ) -> Result<(), String> {
        let request_id = normalize_optional_str_ref(Some(request_id))
            .ok_or_else(|| "question request id is required".to_string())?;
        self.client().question_reply(request_id, answers).await.map(|_| ())
    }

    /// 拒绝指定的 question 请求。
    pub(crate) async fn question_reject(&self, request_id: &str) -> Result<(), String> {
        let request_id = normalize_optional_str_ref(Some(request_id))
            .ok_or_else(|| "question request id is required".to_string())?;
        self.client().question_reject(request_id).await.map(|_| ())
    }

    /// 拉取指定会话的 todo 列表；未显式传入时回退到 runtime 当前会话。
    pub(crate) async fn session_todo_get(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<Todo>, String> {
        let session_id = self.resolve_session_id(session_id)?;
        let directory = self.directory_value();
        self.client().session_todo_get(session_id, directory.as_deref()).await
    }

    /// 覆盖写入指定会话的 todo 列表；未显式传入时回退到 runtime 当前会话。
    pub(crate) async fn session_todo_update(
        &self,
        session_id: Option<&str>,
        todos: &[Todo],
    ) -> Result<(), String> {
        let session_id = self.resolve_session_id(session_id)?;
        let directory = self.directory_value();
        let body = todo_put_body(todos)?;
        self.client()
            .session_todo_update(session_id, directory.as_deref(), &body)
            .await
            .map(|_| ())
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式拉取所有待处理 question 请求。
    pub(crate) fn question_list_all_blocking(&self) -> Result<Vec<question::Request>, String> {
        block_on_gateway(self.question_list_all())
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式拉取指定会话的 question 请求。
    pub(crate) fn question_list_for_session_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<question::Request>, String> {
        block_on_gateway(self.question_list_for_session(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式回复 question 请求。
    pub(crate) fn question_reply_blocking(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
    ) -> Result<(), String> {
        block_on_gateway(self.question_reply(request_id, answers))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式拒绝 question 请求。
    pub(crate) fn question_reject_blocking(&self, request_id: &str) -> Result<(), String> {
        block_on_gateway(self.question_reject(request_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式拉取会话 todo 列表。
    pub(crate) fn session_todo_get_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<Todo>, String> {
        block_on_gateway(self.session_todo_get(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式覆盖写入会话 todo 列表。
    pub(crate) fn session_todo_update_blocking(
        &self,
        session_id: Option<&str>,
        todos: &[Todo],
    ) -> Result<(), String> {
        block_on_gateway(self.session_todo_update(session_id, todos))
    }
}

/// 按会话过滤待处理的 question 请求。
pub(crate) fn filter_questions_for_session(
    requests: Vec<question::Request>,
    session_id: Option<&str>,
) -> Vec<question::Request> {
    let Some(session_id) = normalize_optional_str_ref(session_id) else {
        return requests;
    };

    requests
        .into_iter()
        .filter(|request| request.session_id.trim() == session_id)
        .collect()
}

/// 将共享 `Todo` 列表转换为 gateway 覆盖写入请求体。
pub(crate) fn todo_put_body(todos: &[Todo]) -> Result<GatewaySessionTodoPutBody, String> {
    let mut items = Vec::with_capacity(todos.len());
    for todo in todos {
        items.push(todo_item(todo)?);
    }

    Ok(GatewaySessionTodoPutBody { todos: items })
}

/// 将单条共享 `Todo` 转换为 gateway 侧待办格式。
fn todo_item(todo: &Todo) -> Result<GatewaySessionTodoItem, String> {
    let id = normalize_optional_str_ref(Some(todo.id.as_str()))
        .ok_or_else(|| "todo id is required".to_string())?;

    Ok(GatewaySessionTodoItem {
        id: id.to_string(),
        content: todo.content.clone(),
        status: todo_status_from_str(&todo.status)?,
        priority: todo_priority_from_str(&todo.priority)?,
    })
}

/// 将共享 todo 状态字符串转换为 gateway 强类型枚举。
pub(crate) fn todo_status_from_str(status: &str) -> Result<TodoStatus, String> {
    let status = normalize_optional_str_ref(Some(status))
        .ok_or_else(|| "todo status is required".to_string())?;
    let normalized = status.to_ascii_lowercase();

    match normalized.as_str() {
        "pending" => Ok(TodoStatus::Pending),
        "in_progress" | "in-progress" | "inprogress" => Ok(TodoStatus::InProgress),
        "completed" | "complete" | "done" => Ok(TodoStatus::Completed),
        "cancelled" | "canceled" => Ok(TodoStatus::Cancelled),
        _ => Err(format!("unsupported todo status: {status}")),
    }
}

/// 将共享 todo 优先级字符串转换为 gateway 强类型枚举。
pub(crate) fn todo_priority_from_str(priority: &str) -> Result<TodoPriority, String> {
    let priority = normalize_optional_str_ref(Some(priority))
        .ok_or_else(|| "todo priority is required".to_string())?;
    let normalized = priority.to_ascii_lowercase();

    match normalized.as_str() {
        "low" => Ok(TodoPriority::Low),
        "medium" => Ok(TodoPriority::Medium),
        "high" => Ok(TodoPriority::High),
        _ => Err(format!("unsupported todo priority: {priority}")),
    }
}