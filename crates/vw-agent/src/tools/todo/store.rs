use anyhow::anyhow;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use vw_shared::todo::{default_todo_priority, default_todo_status};

use super::Todo;
use super::normalize::normalize_todos;
use super::schema::{TodoInput, TodoPatch, WriteArgs};

/// 全局待办事项存储
///
/// 使用互斥锁保护的哈希表，键为会话标识，值为该会话的待办事项列表。
static TODOS: LazyLock<Mutex<HashMap<String, Vec<Todo>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn load_todos(session: &str) -> Vec<Todo> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return crate::session::ui_store::load_session_todos(session)
            .into_iter()
            .map(|todo| Todo {
                content: todo.content,
                status: todo.status,
                priority: todo.priority,
                id: todo.id,
            })
            .collect();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let Some(path) = ProjectDirs::from("dev", "VibeWindow", "vibe-window")
            .map(|d| d.data_local_dir().join("todos").join(format!("{}.json", session)))
        else {
            return vec![];
        };
        let Ok(content) = std::fs::read_to_string(path) else {
            return vec![];
        };
        serde_json::from_str::<Vec<Todo>>(&content).unwrap_or_default()
    }
}

fn save_todos(session: &str, todos: &[Todo]) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mapped = todos
            .iter()
            .map(|todo| crate::session::ui_types::SessionTodoItem {
                content: todo.content.clone(),
                status: todo.status.clone(),
                priority: todo.priority.clone(),
                id: todo.id.clone(),
            })
            .collect::<Vec<_>>();
        let _ = crate::session::ui_store::save_session_todos(session, &mapped);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let Some(path) = ProjectDirs::from("dev", "VibeWindow", "vibe-window")
            .map(|d| d.data_local_dir().join("todos").join(format!("{}.json", session)))
        else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let content = serde_json::to_string(todos).unwrap_or_else(|_| "[]".to_string());
        let _ = std::fs::write(path, content);
    }
}

/// 供 UI 层使用的待办事项读取接口
pub(super) fn read_for_ui(session: &str) -> Vec<Todo> {
    let mut lock = match TODOS.lock() {
        Ok(lock) => lock,
        Err(e) => e.into_inner(),
    };
    if !lock.contains_key(session) {
        lock.insert(session.to_string(), normalize_todos(load_todos(session)));
    }
    lock.get(session).cloned().unwrap_or_default()
}

pub(super) fn read_for_tool(session: &str) -> anyhow::Result<Vec<Todo>> {
    let mut lock = TODOS.lock().map_err(|_| anyhow!("Todo 存储已被锁定"))?;
    if !lock.contains_key(session) {
        lock.insert(session.to_string(), normalize_todos(load_todos(session)));
    }

    let todos = lock.get(session).cloned().unwrap_or_default();
    let normalized = normalize_todos(todos);
    lock.insert(session.to_string(), normalized.clone());
    save_todos(session, &normalized);
    Ok(normalized)
}

/// 核心写入逻辑：更新或替换待办事项列表
pub(super) fn write_todos(session: &str, args: serde_json::Value) -> anyhow::Result<String> {
    let args: WriteArgs = serde_json::from_value(args)?;

    let mut lock = TODOS.lock().map_err(|_| anyhow!("Todo 存储已被锁定"))?;
    let mut current = lock.get(session).cloned().unwrap_or_else(|| load_todos(session));
    current = normalize_todos(current);

    let next = if args.merge {
        let patches = args
            .todos
            .iter()
            .cloned()
            .map(|value| {
                serde_json::from_value::<TodoPatch>(value).map_err(|e| anyhow!(e.to_string()))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        for patch in patches {
            let patch_content = patch.content.as_deref().map(|s| s.trim().to_string());
            let patch_id = patch.id.as_deref().map(|s| s.trim().to_string());

            let mut target_idx = patch_content
                .as_deref()
                .and_then(|content| current.iter().position(|todo| todo.content == content));
            if target_idx.is_none() {
                target_idx = patch_id
                    .as_deref()
                    .and_then(|id| current.iter().position(|todo| todo.id == id));
            }

            if let Some(idx) = target_idx {
                if let Some(content) = patch_content.clone() {
                    current[idx].content = content;
                }
                if let Some(status) = patch.status {
                    current[idx].status = status.trim().to_string();
                }
                if let Some(priority) = patch.priority {
                    current[idx].priority = priority.trim().to_string();
                }
                continue;
            }

            let Some(content) = patch_content else { continue };
            let status = patch.status.unwrap_or_else(default_todo_status);
            let priority = patch.priority.unwrap_or_else(default_todo_priority);
            let mut next_todos =
                vec![Todo { id: patch_id.unwrap_or_default(), content, status, priority }];
            next_todos.extend(current);
            current = normalize_todos(next_todos);
        }

        normalize_todos(current)
    } else {
        let parsed = args
            .todos
            .iter()
            .cloned()
            .map(|value| {
                let todo_input = serde_json::from_value::<TodoInput>(value)
                    .map_err(|e| anyhow!(e.to_string()))?;
                Ok::<Todo, anyhow::Error>(Todo {
                    id: todo_input.id.unwrap_or_default(),
                    content: todo_input.content,
                    status: todo_input.status,
                    priority: todo_input.priority,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        normalize_todos(parsed)
    };

    lock.insert(session.to_string(), next.clone());
    save_todos(session, &next);
    serde_json::to_string_pretty(&next).map_err(|e| anyhow!(e))
}
