use crate::app::components::mind_map;
use crate::app::{App, Message};
use iced::Task;

use super::super::super::persist::persist;
use super::super::super::tabs::{ensure_top_tab, new_blank_tab};
use super::super::super::types::MindMapMessage;
use super::json_format::tab_to_json;
use super::tab_restore::{new_tab_from_json, new_tab_from_md};

fn save_finished_message(res: Result<(), String>) -> Message {
    Message::MindMapTool(MindMapMessage::SaveFinished(res))
}

pub(super) fn is_json_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|segment| segment.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

/// 创建新的空白思维导图标签页
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 空的 Task（标签页创建是同步操作）
pub(crate) fn new_tab(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.show_action_menu = false;
    }
    new_blank_tab(app)
}

/// 打开文件对话框以选择思维导图文件
///
/// 在非 WASM 平台上显示异步文件选择对话框，
/// 支持 `.md` 和 `.json` 格式。
///
/// # 返回
///
/// - WASM 平台：返回空 Task
/// - 其他平台：返回异步任务，完成时发送 `FileOpened` 消息
pub(crate) fn open() -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        Task::none()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Task::perform(
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("MindMap", &["md", "json"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let data = file.read().await;
                    let text = String::from_utf8_lossy(&data).to_string();
                    let path = Some(file.path().to_string_lossy().to_string());
                    Ok((path, text))
                } else {
                    Err("Cancelled".to_string())
                }
            },
            |res| Message::MindMapTool(MindMapMessage::FileOpened(res)),
        )
    }
}

/// 处理文件打开结果
///
/// 根据文件扩展名（.json 或 .md）选择相应的解析方式。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `res`: 文件打开结果，包含文件路径和内容，或错误信息
///
/// # 返回
///
/// 空的 Task
pub(crate) fn file_opened(
    app: &mut App,
    res: Result<(Option<String>, String), String>,
) -> Task<Message> {
    match res {
        Ok((path, text)) => {
            if path.as_deref().is_some_and(is_json_path) {
                if let Err(error) = new_tab_from_json(app, path, text) {
                    app.error_message = Some(error);
                }
            } else {
                new_tab_from_md(app, path, text);
            }
        }
        Err(error) => {
            if error != "Cancelled" {
                app.error_message = Some(error);
            }
        }
    }
    Task::none()
}

/// 保存当前思维导图
///
/// 如果文件已有路径，则直接保存；否则调用"另存为"对话框。
/// 根据文件扩展名选择保存格式（JSON 或 Markdown）。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步保存任务，完成时发送 `SaveFinished` 消息
pub(crate) fn save(app: &mut App) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab() else {
        return Task::none();
    };
    let Some(path) = tab.file_path.clone() else {
        return save_as(app);
    };

    let payload = if is_json_path(&path) {
        let file = tab_to_json(tab);
        match serde_json::to_string_pretty(&file) {
            Ok(json) => json,
            Err(error) => {
                return Task::perform(async move { Err(error.to_string()) }, save_finished_message);
            }
        }
    } else {
        mind_map::to_markdown(&tab.doc)
    };
    #[cfg(target_arch = "wasm32")]
    let _ = &payload;

    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.show_action_menu = false;
    }

    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use std::io::Write;
                match std::fs::File::create(&path) {
                    Ok(mut file) => match file.write_all(payload.as_bytes()) {
                        Ok(_) => Ok(()),
                        Err(error) => Err(error.to_string()),
                    },
                    Err(error) => Err(error.to_string()),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                Ok(())
            }
        },
        save_finished_message,
    )
}

/// 处理保存完成结果
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `res`: 保存操作结果
///
/// # 返回
///
/// 空的 Task
pub(crate) fn save_finished(app: &mut App, res: Result<(), String>) -> Task<Message> {
    if let Err(error) = res {
        app.error_message = Some(error);
    }
    Task::none()
}

/// 另存为 Markdown 文件
///
/// 显示文件保存对话框，默认文件名为 `mindmap.md`。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步保存任务，完成时发送 `FileSaved` 消息
pub(crate) fn save_as(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.show_action_menu = false;
    }

    let Some(tab) = app.active_mindmap_tab() else {
        return Task::none();
    };
    let markdown = mind_map::to_markdown(&tab.doc);
    Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new().set_file_name("mindmap.md").save_file().await;
            if let Some(file) = file {
                if let Err(error) = file.write(markdown.as_bytes()).await {
                    return Err(error.to_string());
                }
                #[cfg(not(target_arch = "wasm32"))]
                return Ok(Some(file.path().to_string_lossy().to_string()));
                #[cfg(target_arch = "wasm32")]
                return Ok(None);
            }
            Ok(None)
        },
        |res| Message::MindMapTool(MindMapMessage::FileSaved(res.ok().flatten())),
    )
}

/// 另存为 JSON 文件
///
/// 显示文件保存对话框，默认文件名为 `mindmap.json`。
/// JSON 格式可保存完整的思维导图状态。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步保存任务，完成时发送 `FileSaved` 消息
pub(crate) fn save_as_json(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.show_action_menu = false;
    }

    let Some(tab) = app.active_mindmap_tab() else {
        return Task::none();
    };
    let file = tab_to_json(tab);
    let json = match serde_json::to_string_pretty(&file) {
        Ok(json) => json,
        Err(error) => {
            return Task::perform(async move { Err(error.to_string()) }, save_finished_message);
        }
    };

    Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new().set_file_name("mindmap.json").save_file().await;
            if let Some(file) = file {
                if let Err(error) = file.write(json.as_bytes()).await {
                    return Err(error.to_string());
                }
                #[cfg(not(target_arch = "wasm32"))]
                return Ok(Some(file.path().to_string_lossy().to_string()));
                #[cfg(target_arch = "wasm32")]
                return Ok(None);
            }
            Ok(None)
        },
        |res| Message::MindMapTool(MindMapMessage::FileSaved(res.ok().flatten())),
    )
}

/// 处理文件保存完成
///
/// 更新标签页的文件路径和标题，并持久化应用状态。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `path`: 保存成功的文件路径（用户取消时为 None）
///
/// # 返回
///
/// 空的 Task
pub(crate) fn file_saved(app: &mut App, path: Option<String>) -> Task<Message> {
    if let Some(path) = path {
        let top = if let Some(tab) = app.active_mindmap_tab_mut() {
            tab.file_path = Some(path.clone());
            if let Some(name) =
                std::path::Path::new(&path).file_name().and_then(|segment| segment.to_str())
            {
                tab.title = name.to_string();
            }
            Some((tab.id.clone(), tab.title.clone()))
        } else {
            None
        };
        if let Some((id, title)) = top {
            ensure_top_tab(app, &id, &title);
            let _ = persist(app);
        }
    }
    Task::none()
}
