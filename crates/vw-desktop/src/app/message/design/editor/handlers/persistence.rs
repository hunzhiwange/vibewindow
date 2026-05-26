//! 处理设计编辑器的持久化入口，包括保存、加载和导入导出相关状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::super::canvas::{apply_module_doc_to_canvas, find_generation_page_mut};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignDoc, compute_tree_metrics};
use crate::app::views::design::state::DesignGenerationStatus;
use crate::app::{App, Message};
use iced::Task;

/// save_design_project_pen 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn save_design_project_pen(app: &mut App) -> Task<Message> {
    let Some(state) = app.active_design_state() else {
        return Task::none();
    };
    let file_name = if let Some(path) = &state.file_path {
        path.file_name()
            .and_then(|name| name.to_str())
            .filter(|name| name.ends_with(".pen"))
            .unwrap_or("project.pen")
            .to_string()
    } else {
        "project.pen".to_string()
    };
    let doc = state.doc.clone();
    // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
    Task::perform(
        async move {
            let json = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
            let file = rfd::AsyncFileDialog::new().set_file_name(&file_name).save_file().await;
            if let Some(file) = file {
                file.write(json.as_bytes()).await.map_err(|e| e.to_string())?;
                #[cfg(not(target_arch = "wasm32"))]
                return Ok(Some(file.path().to_path_buf()));
                #[cfg(target_arch = "wasm32")]
                return Ok(None);
            }
            Ok(None)
        },
        |result| Message::Design(DesignMessage::DesignProjectPenSaved(result)),
    )
}

/// design_project_pen_saved 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_project_pen_saved(
    app: &mut App,
    result: Result<Option<std::path::PathBuf>, String>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        match result {
            Ok(Some(path)) => {
                state.file_path = Some(path.clone());
                state.design_generation_summary = Some(format!("项目 .json 已保存: {}", path.display()));
            }
            Ok(None) => {
                state.design_generation_summary = Some("已取消保存项目 .pen。".to_string());
            }
            Err(error) => {
                state.design_generation_summary = Some(error);
            }
        }
    }
    Task::none()
}

/// save_generated_page_as_pen 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn save_generated_page_as_pen(
    app: &mut App,
    page_frame_id: String,
    module_id: String,
) -> Task<Message> {
    let Some(state) = app.active_design_state() else {
        return Task::none();
    };
    let Some(page) = state.design_generation_pages.iter().find(|page| page.frame_id == page_frame_id)
    else {
        return Task::none();
    };
    let Some(module) = page.modules.iter().find(|module| module.module_id == module_id) else {
        return Task::none();
    };
    let Some(doc) = module.generated_doc.clone() else {
        return Task::none();
    };
    let file_name = format!("{}.pen", module.title.replace(['/', ' '], "-"));
    Task::perform(
        async move {
            let json = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
            let file = rfd::AsyncFileDialog::new().set_file_name(&file_name).save_file().await;
            if let Some(file) = file {
                file.write(json.as_bytes()).await.map_err(|e| e.to_string())?;
                #[cfg(not(target_arch = "wasm32"))]
                return Ok(Some(file.path().to_path_buf()));
                #[cfg(target_arch = "wasm32")]
                return Ok(None);
            }
            Ok(None)
        },
        |result| Message::Design(DesignMessage::GeneratedPagePenSaved(result)),
    )
}

/// generated_page_pen_saved 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn generated_page_pen_saved(
    app: &mut App,
    result: Result<Option<std::path::PathBuf>, String>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        match result {
            Ok(Some(path)) => {
                state.design_generation_summary = Some(format!("模块子文档已保存: {}", path.display()));
            }
            Ok(None) => {
                state.design_generation_summary = Some("已取消保存模块子文档。".to_string());
            }
            Err(error) => {
                state.design_generation_summary = Some(error);
            }
        }
    }
    Task::none()
}

/// import_generated_pen_to_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn import_generated_pen_to_page(
    page_frame_id: String,
    module_id: String,
) -> Task<Message> {
    Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Design", &["pen", "json"])
                .pick_file()
                .await;
            let Some(file) = file else {
                return Err("已取消导入".to_string());
            };
            let data = file.read().await;
            serde_json::from_slice::<DesignDoc>(&data).map_err(|e| e.to_string())
        },
        move |result| {
            Message::Design(DesignMessage::GeneratedPagePenImported {
                page_frame_id,
                page_task_id: module_id,
                result,
            })
        },
    )
}

/// generated_page_pen_imported 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn generated_page_pen_imported(
    app: &mut App,
    page_frame_id: String,
    page_task_id: String,
    result: Result<DesignDoc, String>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let mut target_frame_id = None;
        let mut title = None;
        let mut imported_doc = None;
        let mut generated_target_frame_id = None;
        if let Some(page) = find_generation_page_mut(&mut state.design_generation_pages, &page_frame_id)
            && let Some(module) = page.modules.iter_mut().find(|module| module.module_id == page_task_id)
        {
            title = Some(module.title.clone());
            target_frame_id = Some(module.target_frame_id.clone());
            match result {
                Ok(doc) => {
                    imported_doc = Some(doc.clone());
                    module.generated_doc = Some(doc);
                    module.status = DesignGenerationStatus::Generated;
                    generated_target_frame_id = Some(module.target_frame_id.clone());
                }
                Err(error) => {
                    state.design_generation_summary = Some(error);
                }
            }
        }
        if let Some(generated_target_frame_id) = generated_target_frame_id {
            super::super::canvas::sync_module_placeholder_status(
                state,
                &generated_target_frame_id,
                DesignGenerationStatus::Generated,
            );
        }
        if let (Some(target_frame_id), Some(doc), Some(title)) = (target_frame_id, imported_doc, title)
        {
            if let Err(error) = apply_module_doc_to_canvas(state, &target_frame_id, &doc) {
                state.design_generation_summary = Some(error);
            } else {
                state.design_generation_summary = Some(format!("已导入 .json 子文档到模块“{}”。", title));
                state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                state.canvas_cache.clear();
            }
        }
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}

