//! 处理 Figma 导入消息，将外部设计数据接入本地设计模型。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::DesignMessage;
use crate::app::views::design::models::{DesignTool, compute_tree_metrics};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::models::DesignDoc;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::state::{FigmaProgressStage, FigmaProgressState};
use crate::app::{App, Message};
use iced::Task;

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn update(app: &mut App, message: DesignMessage) -> Option<Task<Message>> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        DesignMessage::ToolSelected(DesignTool::ImportFigma) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                Some(Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Figma", &["fig"])
                            .pick_file()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |path| Message::Design(DesignMessage::FigmaImportFilePicked(path)),
                ))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(Task::none())
            }
        }
        DesignMessage::FigmaImportFilePicked(path) => {
            let Some(path) = path else {
                return Some(Task::none());
            };
            #[cfg(target_arch = "wasm32")]
            let _ = &path;

            #[cfg(not(target_arch = "wasm32"))]
            {
                let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                if let Some(state) = app.active_design_state_mut() {
                    state.figma_progress = Some(FigmaProgressState::new(
                        FigmaProgressStage::Importing,
                        0,
                        1,
                        "正在读取 Figma 文件…",
                    ));
                    state.figma_progress_rx = Some(progress_rx);
                }

                Some(Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || -> Result<Option<DesignDoc>, String> {
                            let bytes =
                                std::fs::read(path.as_path()).map_err(|error| error.to_string())?;
                            crate::app::views::design::import::figma_to_design_doc_with_elements_progress(
                                &bytes,
                                |progress| {
                                    let _ = progress_tx.send(FigmaProgressState::new(
                                        FigmaProgressStage::Importing,
                                        progress.completed_pages,
                                        progress.total_pages,
                                        progress.detail,
                                    ));
                                },
                            )
                            .map(Some)
                            .map_err(|error| error.to_string())
                        })
                        .await
                        .unwrap_or_else(|error| Err(error.to_string()))
                    },
                    |result| Message::Design(DesignMessage::FigmaFileImported(result)),
                ))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(Task::none())
            }
        }
        DesignMessage::FigmaProgressTick => {
            if let Some(state) = app.active_design_state_mut()
                && let Some(receiver) = state.figma_progress_rx.take()
            {
                let mut active_receiver = Some(receiver);
                for _ in 0..24 {
                    let Some(current_receiver) = active_receiver.as_ref() else {
                        break;
                    };
                    match current_receiver.try_recv() {
                        Ok(progress) => state.figma_progress = Some(progress),
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            active_receiver = None;
                            break;
                        }
                    }
                }
                state.figma_progress_rx = active_receiver;
            }
            Some(Task::none())
        }
        DesignMessage::FigmaFileImported(result) => {
            if let Some(state) = app.active_design_state_mut() {
                state.figma_progress = None;
                state.figma_progress_rx = None;
                match result {
                    Ok(Some(elements)) => {
                        let mut imported_doc = elements;
                        imported_doc.normalize_groups();
                        if state.has_single_empty_page() {
                            state.doc = imported_doc;
                            state.ensure_valid_group();
                            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                            state.canvas_cache.clear();
                            state.active_tool = DesignTool::Move;
                            state.focus_first_element_in_active_group();
                            return Some(Task::done(Message::Design(DesignMessage::Snapshot)));
                        }
                        let mut group_id_map = std::collections::BTreeMap::new();
                        for imported_group in imported_doc.groups {
                            let target_group_id = if state
                                .doc
                                .groups
                                .iter()
                                .any(|group| group.id == imported_group.id)
                            {
                                state.doc.next_group_id()
                            } else {
                                imported_group.id
                            };
                            group_id_map.insert(imported_group.id, target_group_id);
                            state.doc.groups.push(crate::app::views::design::models::DesignGroup {
                                id: target_group_id,
                                name: imported_group.name,
                            });
                        }
                        for element in &mut imported_doc.children {
                            if let Some(mapped_group_id) =
                                group_id_map.get(&element.group_id).copied()
                            {
                                element.set_group_id_recursive(mapped_group_id);
                            }
                        }
                        state.doc.children.extend(imported_doc.children);
                        state.doc.normalize_groups();
                        state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                        state.canvas_cache.clear();
                        state.active_tool = DesignTool::Move;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!("Failed to import Figma file: {}", error);
                    }
                }
            }
            Some(Task::done(Message::Design(DesignMessage::Snapshot)))
        }
        _ => None,
    }
}

