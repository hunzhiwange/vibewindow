//! 负责设计页面生成流程，把用户输入转换为异步生成任务。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::super::DesignModuleExecutionResult;
use super::super::canvas::{
    apply_module_doc_to_canvas, apply_page_doc_to_canvas, collect_retry_error_context,
    find_generation_module_index, find_generation_page_index, find_generation_page_mut,
    sync_module_placeholder_status,
};
use super::super::logging::{
    append_design_project_log, executor_step_label, push_design_stream_to_chat, push_module_log,
};
use super::super::tasks::{
    count_generation_progress, count_running_generation_pages, design_page_parallel_limit,
    next_queued_generation_pages, spawn_design_module_generation_task,
    summarize_generated_pages_for_prompt, summarize_page_modules_for_prompt,
};
use crate::app::message::DesignMessage;
use crate::app::task::{TASK_MODEL_AUTO, TaskExecutorBackend};
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::views::design::state::{DesignChatMessage, DesignChatRole, DesignGenerationStatus};
use crate::app::{App, Message};
use iced::Task;

/// generate_design_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn generate_design_page(
    app: &mut App,
    page_frame_id: String,
    module_id: String,
) -> Task<Message> {
    generate_page_task(app, page_frame_id, module_id)
}

/// regenerate_design_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn regenerate_design_page(
    app: &mut App,
    page_frame_id: String,
    module_id: String,
) -> Task<Message> {
    generate_page_task(app, page_frame_id, module_id)
}

fn generate_page_task(app: &mut App, page_frame_id: String, module_id: String) -> Task<Message> {
    let project_path = app
        .project_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
    let selected_acp_agent = app.acp_agent.clone();
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let Some(page_index) =
        find_generation_page_index(&state.design_generation_pages, &page_frame_id)
    else {
        state.design_generation_summary = Some("未找到页面规划。".to_string());
        return Task::none();
    };
    let module_index =
        find_generation_module_index(&state.design_generation_pages[page_index], &module_id)
            .or_else(|| {
                state.design_generation_pages[page_index].modules.iter().position(|module| {
                    matches!(
                        module.status,
                        DesignGenerationStatus::Queued | DesignGenerationStatus::Placeholder
                    ) && !module.is_generating
                })
            });
    let Some(module_index) = module_index else {
        return Task::none();
    };
    if state.design_generation_pages[page_index].modules.iter().any(|module| module.is_generating) {
        return Task::none();
    }
    let parallel_limit = design_page_parallel_limit(state);
    let running_count = count_running_generation_pages(state);
    if running_count >= parallel_limit {
        state.design_generation_pages[page_index].status = DesignGenerationStatus::Queued;
        state.design_generation_summary = Some("页面任务并行数已满，当前页面已排队。".to_string());
        return Task::none();
    }
    state.design_generation_pages[page_index].status = DesignGenerationStatus::Running;
    let mut running_targets = Vec::new();
    for module in &mut state.design_generation_pages[page_index].modules {
        if matches!(
            module.status,
            DesignGenerationStatus::Queued | DesignGenerationStatus::Placeholder
        ) || module.module_id == module_id
        {
            module.is_generating = true;
            module.status = DesignGenerationStatus::Running;
            running_targets.push(module.target_frame_id.clone());
        }
    }
    for target_frame_id in running_targets {
        sync_module_placeholder_status(state, &target_frame_id, DesignGenerationStatus::Running);
    }
    state.design_generation_loading = true;
    let page_snapshot = state.design_generation_pages[page_index].clone();
    let module_snapshot = state.design_generation_pages[page_index].modules[module_index].clone();
    let generated_pages_summary =
        summarize_generated_pages_for_prompt(&state.design_generation_pages, &page_frame_id);
    let page_modules_layout_summary =
        summarize_page_modules_for_prompt(&state.design_generation_pages[page_index]);
    push_module_log(
        state,
        &page_frame_id,
        &module_id,
        format!(
            "[PAGE:{}] start modules={} first_module={}",
            page_snapshot.title,
            page_snapshot.modules.len(),
            module_snapshot.title
        ),
    );
    push_module_log(state, &page_frame_id, &module_id, page_modules_layout_summary);
    let retry_context = collect_retry_error_context(&page_snapshot);
    let mut design_brief = state.design_generation_brief.clone();
    if !retry_context.is_empty() {
        design_brief = format!(
            "{}\n\n上一轮失败信息（用于修复并重试）：\n{}",
            state.design_generation_brief.trim(),
            retry_context
        );
    }
    let executor = TaskExecutorBackend::Internal;
    let theme = state.design_generation_theme;
    let style = state.design_generation_style;
    let device = state.design_generation_device;
    let model = if state.design_generation_model.trim().is_empty() {
        TASK_MODEL_AUTO.to_string()
    } else {
        state.design_generation_model.trim().to_string()
    };
    state.design_generation_summary = Some(format!("正在生成页面\"{}\"...", page_snapshot.title));
    let current_log_file = state.design_generation_current_log_file.clone();
    spawn_design_module_generation_task(
        project_path,
        design_brief,
        executor,
        selected_acp_agent,
        theme,
        style,
        device,
        model,
        generated_pages_summary,
        page_snapshot,
        page_frame_id,
        module_id,
        current_log_file,
    )
}

/// design_page_generated 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_page_generated(
    app: &mut App,
    page_frame_id: String,
    page_task_id: String,
    result: Result<DesignModuleExecutionResult, String>,
) -> Task<Message> {
    let project_path = app
        .project_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
    if let Some(state) = app.active_design_state_mut() {
        let current_log_file = state.design_generation_current_log_file.clone();
        let mut pending_logs: Vec<String> = Vec::new();
        let Some(page_index) =
            find_generation_page_index(&state.design_generation_pages, &page_frame_id)
        else {
            return Task::done(Message::Design(DesignMessage::Snapshot));
        };
        if !state.design_generation_pages[page_index]
            .modules
            .iter()
            .any(|module| module.is_generating)
        {
            return Task::done(Message::Design(DesignMessage::Snapshot));
        }
        let page_title = state.design_generation_pages[page_index].title.clone();
        let module_targets = state.design_generation_pages[page_index]
            .modules
            .iter()
            .map(|module| (module.module_id.clone(), module.target_frame_id.clone()))
            .collect::<Vec<_>>();
        for module in &mut state.design_generation_pages[page_index].modules {
            module.is_generating = false;
        }
        match result {
            Ok(execution) => {
                pending_logs.extend(execution.logs);
                pending_logs.push(format!("[PAGE:{}] parsed design doc", page_title));
                match apply_page_doc_to_canvas(state, &page_frame_id, &execution.doc) {
                    Ok(()) => {
                        if let Some(page) = find_generation_page_mut(
                            &mut state.design_generation_pages,
                            &page_frame_id,
                        ) {
                            page.status = DesignGenerationStatus::Filled;
                            for module in &mut page.modules {
                                module.status = DesignGenerationStatus::Filled;
                            }
                        }
                        for (item_module_id, target_frame_id) in &module_targets {
                            sync_module_placeholder_status(
                                state,
                                target_frame_id,
                                DesignGenerationStatus::Filled,
                            );
                            push_module_log(
                                state,
                                &page_frame_id,
                                item_module_id,
                                format!("[PAGE:{}] filled by page task", page_title),
                            );
                        }
                        append_design_project_log(
                            &project_path,
                            format!(
                                "event=page_completed_ok page_frame={} page_title={} generated_children={}",
                                page_frame_id,
                                page_title,
                                execution.doc.children.len()
                            ),
                            current_log_file.as_deref(),
                        );
                        state.design_generation_summary =
                            Some(format!("页面“{}”已生成并导入画布。", page_title));
                        state.design_chat_messages.push(DesignChatMessage {
                            role: DesignChatRole::Assistant,
                            content: executor_step_label(TaskExecutorBackend::Internal),
                        });
                        state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                        state.canvas_cache.clear();
                    }
                    Err(error) => {
                        append_design_project_log(
                            &project_path,
                            format!(
                                "event=page_fill_failed page_frame={} page_title={} error={}",
                                page_frame_id, page_title, error
                            ),
                            current_log_file.as_deref(),
                        );
                        if let Some(page) = find_generation_page_mut(
                            &mut state.design_generation_pages,
                            &page_frame_id,
                        ) {
                            page.status = DesignGenerationStatus::Failed;
                            for module in &mut page.modules {
                                module.status = DesignGenerationStatus::Failed;
                            }
                        }
                        for (item_module_id, target_frame_id) in &module_targets {
                            sync_module_placeholder_status(
                                state,
                                target_frame_id,
                                DesignGenerationStatus::Failed,
                            );
                            push_module_log(
                                state,
                                &page_frame_id,
                                item_module_id,
                                format!("[PAGE:{}] fill_failed {}", page_title, error),
                            );
                        }
                        state.design_generation_summary =
                            Some(format!("页面“{}”生成成功，但导入失败：{}", page_title, error));
                    }
                }
            }
            Err(error) => {
                append_design_project_log(
                    &project_path,
                    format!(
                        "event=page_completed_failed page_frame={} page_task_id={} page_title={} error={}",
                        page_frame_id, page_task_id, page_title, error
                    ),
                    current_log_file.as_deref(),
                );
                if let Some(page) =
                    find_generation_page_mut(&mut state.design_generation_pages, &page_frame_id)
                {
                    page.status = DesignGenerationStatus::Failed;
                    for module in &mut page.modules {
                        module.status = DesignGenerationStatus::Failed;
                    }
                }
                for (item_module_id, target_frame_id) in &module_targets {
                    sync_module_placeholder_status(
                        state,
                        target_frame_id,
                        DesignGenerationStatus::Failed,
                    );
                    push_module_log(
                        state,
                        &page_frame_id,
                        item_module_id,
                        format!("[PAGE:{}] failed {}", page_title, error),
                    );
                }
                state.design_generation_summary = Some(error.clone());
                pending_logs.push(format!("[PAGE:{}] failed {}", page_title, error));
            }
        }
        push_design_stream_to_chat(state, &pending_logs, 8);
        for line in pending_logs {
            push_module_log(state, &page_frame_id, &page_task_id, line);
        }
        let parallel_limit = design_page_parallel_limit(state);
        let running_count = count_running_generation_pages(state);
        let queued_batch = if running_count >= parallel_limit {
            Vec::new()
        } else {
            next_queued_generation_pages(state, parallel_limit - running_count)
        };
        if !queued_batch.is_empty() {
            state.design_generation_loading = true;
            state.design_generation_summary = Some("继续按页面并行生成剩余页面...".to_string());
            let mut tasks = vec![Task::done(Message::Design(DesignMessage::Snapshot))];
            tasks.extend(queued_batch.into_iter().map(|(next_page_frame_id, next_module_id)| {
                Task::done(Message::Design(DesignMessage::GenerateDesignPage(
                    next_page_frame_id,
                    next_module_id,
                )))
            }));
            return Task::batch(tasks);
        }
        if running_count > 0 {
            state.design_generation_loading = true;
            state.design_generation_summary =
                Some(format!("仍有 {} 个页面任务正在生成中...", running_count));
            return Task::done(Message::Design(DesignMessage::Snapshot));
        }
        let (filled_count, queued_count, failed_count) = count_generation_progress(state);
        state.design_generation_loading = false;
        state.design_generation_anim_frame = 0;
        state.design_generation_summary = Some(format!(
            "页面并行生成完成：{} 个模块已回填，{} 个失败，{} 个未完成。",
            filled_count, failed_count, queued_count
        ));
        if failed_count == 0 {
            state.design_chat_messages.push(DesignChatMessage {
                role: DesignChatRole::Assistant,
                content: format!(
                    "Designed {}.\n\nWhat’s included:\n- Filled modules: {}\n- Pending modules: {}\n- Failed modules: {}",
                    state.design_generation_brief, filled_count, queued_count, failed_count
                ),
            });
        }
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}

/// aggregate_design_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn aggregate_design_page(
    app: &mut App,
    page_frame_id: String,
    module_id: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let mut aggregate_doc = None;
        let mut target_frame_id = None;
        let mut module_title = None;
        if let Some(page_index) =
            find_generation_page_index(&state.design_generation_pages, &page_frame_id)
            && let Some(module_index) =
                find_generation_module_index(&state.design_generation_pages[page_index], &module_id)
        {
            let page = &mut state.design_generation_pages[page_index];
            let module = &mut page.modules[module_index];
            aggregate_doc = module.generated_doc.clone();
            target_frame_id = Some(module.target_frame_id.clone());
            module_title = Some(module.title.clone());
            module.status = DesignGenerationStatus::Aggregated;
        }
        match (target_frame_id, aggregate_doc, module_title) {
            (Some(target_frame_id), Some(doc), Some(title)) => {
                if let Err(error) = apply_module_doc_to_canvas(state, &target_frame_id, &doc) {
                    state.design_generation_summary = Some(error);
                } else {
                    state.design_generation_summary =
                        Some(format!("模块“{}”已导入到指定画布位置。", title));
                    state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                    state.canvas_cache.clear();
                }
            }
            (_, _, Some(title)) => {
                state.design_generation_summary =
                    Some(format!("模块“{}”还没有可导入的生成结果。", title));
            }
            _ => {
                state.design_generation_summary = Some("未找到模块规划。".to_string());
            }
        }
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}
