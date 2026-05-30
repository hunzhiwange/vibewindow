//! 管理设计生成计划的创建、更新和执行前状态整理。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::super::DesignPlanExecutionResult;
use super::super::canvas::{build_target_frame_options, sync_module_placeholder_status};
use super::super::logging::{
    append_design_project_log, create_design_generation_log_file, executor_step_label,
    format_design_log_stream, push_design_generation_log, push_design_stream_to_chat,
    sync_design_generation_log_editor,
};
use super::super::parser::parse_design_generation_pages;
use super::super::prompts::{
    build_design_generation_prompt, compact_multiline, design_executor_uses_gateway,
    execute_design_generation_with_streaming, format_plan_parse_error, resolve_design_acp_agent,
    resolve_design_generation_device,
};
use super::super::tasks::{
    build_design_plan_canvas, design_page_parallel_limit, next_queued_generation_pages,
};
use crate::app::message::DesignMessage;
use crate::app::task::{
    TASK_MODEL_AUTO, TaskExecutorBackend, TaskLogStream, build_executor_command,
};
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::views::design::state::{
    DesignChatMessage, DesignChatRole, DesignGenerationPlan, DesignGenerationStatus,
};
use crate::app::{App, Message};
use iced::Task;
use std::sync::mpsc;

/// design_generation_submit 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_submit(app: &mut App) -> Task<Message> {
    let project_path = app
        .project_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
    let selected_acp_agent = app.acp_agent.clone();
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let prompt = state.design_chat_input.text().trim().to_string();
    if prompt.is_empty() {
        state.design_generation_summary = Some("请先输入页面与模块需求。".to_string());
        return Task::none();
    }
    let current_log_file = create_design_generation_log_file(&project_path);
    state.design_generation_current_log_file = current_log_file.clone();
    let current_log_filename = current_log_file.as_deref();
    state.design_generation_loading = true;
    state.design_generation_brief = prompt.clone();
    state.design_generation_summary =
        Some("正在生成站点整体结构与首模块，随后按同页顺序串行生成剩余模块...".to_string());
    let executor = TaskExecutorBackend::Internal;
    let model = if state.design_generation_model.trim().is_empty() {
        TASK_MODEL_AUTO.to_string()
    } else {
        state.design_generation_model.trim().to_string()
    };
    let theme = state.design_generation_theme;
    let style = state.design_generation_style;
    let selected_device = state.design_generation_device;
    state.design_generation_logs.clear();
    sync_design_generation_log_editor(state);
    state.design_generation_stream_cursor = 0;
    state.design_generation_anim_frame = 0;
    push_design_generation_log(
        state,
        format!(
            "[PLAN] start overall_first=true theme={} executor={} model={}",
            theme.label(),
            executor.label(),
            model
        ),
    );
    state
        .design_chat_messages
        .push(DesignChatMessage { role: DesignChatRole::User, content: prompt.clone() });
    state.design_chat_messages.push(DesignChatMessage {
        role: DesignChatRole::Assistant,
        content: executor_step_label(executor),
    });
    state.design_chat_selected_message = None;
    append_design_project_log(
        &project_path,
        format!(
            "event=plan_submit executor={} model={} theme={} style={} prompt_chars={}",
            executor.label(),
            model,
            theme.label(),
            style.label(),
            prompt.chars().count()
        ),
        current_log_filename,
    );
    state.design_chat_input = iced::widget::text_editor::Content::new();
    state.sync_active_chat_session_from_legacy();
    let generation_prompt =
        build_design_generation_prompt(&prompt, executor, theme, style, selected_device);
    let (stream_tx, stream_rx) = mpsc::channel::<TaskLogStream>();
    state.design_generation_stream_rx = Some(stream_rx);
    let current_log_file_for_async = current_log_file.clone();
    // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
    Task::perform(
        async move {
            let acp_agent = resolve_design_acp_agent(executor, selected_acp_agent.as_deref());
            let route = if design_executor_uses_gateway(executor) {
                format!("gateway agent={}", acp_agent.as_deref().unwrap_or("default"))
            } else {
                let command =
                    build_executor_command(executor, &project_path, &model, &generation_prompt);
                format!("cli command={}", compact_multiline(&format!("{:?}", command)))
            };
            append_design_project_log(
                &project_path,
                format!(
                    "event=plan_dispatch executor={} model={} route={}",
                    executor.label(),
                    model,
                    route
                ),
                current_log_file_for_async.as_deref(),
            );
            let session_scope = "plan".to_string();
            let execution_project_path = project_path.clone();
            let output: (Result<String, String>, Vec<String>) =
                crate::app::message::spawn_blocking_opt(move || {
                    let (local_tx, local_rx) = mpsc::channel::<TaskLogStream>();
                    let forward_tx = stream_tx.clone();
                    let mirror = std::thread::spawn(move || {
                        let mut logs = Vec::new();
                        while let Ok(log) = local_rx.recv() {
                            let _ = forward_tx.send(log.clone());
                            if let Some(line) = format_design_log_stream(&log) {
                                logs.push(format!("[plan] {}", line));
                            }
                        }
                        logs
                    });
                    let result = execute_design_generation_with_streaming(
                        executor,
                        &execution_project_path,
                        &model,
                        &generation_prompt,
                        acp_agent,
                        local_tx,
                        &session_scope,
                    );
                    let logs = mirror.join().unwrap_or_default();
                    Some((result, logs))
                })
                .await
                .ok_or_else(|| "设计页面规划任务没有返回结果。".to_string())?;
            let (result, logs) = output;
            let raw = result?;
            for line in &logs {
                append_design_project_log(
                    &project_path,
                    format!("event=plan_stream line={}", line),
                    current_log_file_for_async.as_deref(),
                );
            }
            let plan = match parse_design_generation_pages(&raw, theme) {
                Ok(plan) => plan,
                Err(raw_error) => {
                    let streamed_raw = logs.join("\n");
                    match parse_design_generation_pages(&streamed_raw, theme) {
                        Ok(plan) => {
                            append_design_project_log(
                                &project_path,
                                "event=plan_parse_recovered_from_stream_logs",
                                current_log_file_for_async.as_deref(),
                            );
                            plan
                        }
                        Err(_) => {
                            let message = format_plan_parse_error(&raw_error, &raw);
                            append_design_project_log(
                                &project_path,
                                format!(
                                    "event=plan_parse_failed error={} raw={} stream={}",
                                    raw_error,
                                    compact_multiline(&raw),
                                    compact_multiline(&streamed_raw)
                                ),
                                current_log_file_for_async.as_deref(),
                            );
                            return Err(message);
                        }
                    }
                }
            };
            append_design_project_log(
                &project_path,
                format!(
                    "event=plan_parse_success pages={} modules={} raw={}",
                    plan.pages.len(),
                    plan.pages.iter().map(|page| page.modules.len()).sum::<usize>(),
                    compact_multiline(&raw)
                ),
                current_log_file_for_async.as_deref(),
            );
            Ok(DesignPlanExecutionResult { plan, logs })
        },
        |result| Message::Design(DesignMessage::DesignGenerationCompleted(result)),
    )
}

/// design_generation_completed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_completed(
    app: &mut App,
    result: Result<DesignPlanExecutionResult, String>,
) -> Task<Message> {
    let project_path = app
        .project_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
    if let Some(state) = app.active_design_state_mut() {
        let current_log_file = state.design_generation_current_log_file.clone();
        if !state.design_generation_loading {
            return Task::done(Message::Design(DesignMessage::Snapshot));
        }
        state.design_generation_loading = false;
        state.design_generation_stream_rx = None;
        state.design_generation_anim_frame = 0;
        match result {
            Ok(execution) => {
                append_design_project_log(
                    &project_path,
                    format!("event=plan_completed_ok log_lines={}", execution.logs.len()),
                    current_log_file.as_deref(),
                );
                let logs = execution.logs;
                let remaining_logs = logs
                    .iter()
                    .skip(state.design_generation_stream_cursor)
                    .cloned()
                    .collect::<Vec<_>>();
                for line in &remaining_logs {
                    push_design_generation_log(state, line.clone());
                }
                push_design_stream_to_chat(state, &remaining_logs, 12);
                state.design_generation_stream_cursor = logs.len();
                let plan = execution.plan;
                let DesignGenerationPlan { summary, pages } = plan;
                let page_count = pages.len();
                let module_count = pages.iter().map(|page| page.modules.len()).sum::<usize>();
                let summary_for_canvas = summary.clone();
                state.design_generation_pages = pages.clone();
                push_design_generation_log(
                    state,
                    format!("[PLAN] parsed pages={} modules={}", page_count, module_count),
                );
                state.design_generation_summary = Some(match summary {
                    Some(summary) if !summary.trim().is_empty() => format!(
                        "整体结构已生成：{} 个页面、{} 个模块。{}",
                        page_count, module_count, summary
                    ),
                    _ => {
                        format!("整体结构已生成：{} 个页面、{} 个模块。", page_count, module_count)
                    }
                });
                state.design_chat_messages.push(DesignChatMessage {
                    role: DesignChatRole::Assistant,
                    content: executor_step_label(TaskExecutorBackend::Internal),
                });
                state.doc = build_design_plan_canvas(
                    &pages,
                    state.design_generation_theme,
                    resolve_design_generation_device(
                        state.design_generation_device,
                        &state.design_generation_brief,
                    ),
                    summary_for_canvas.as_deref(),
                );
                state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                state.canvas_cache.clear();
                let mut queued_count = 0usize;
                for page_index in 0..state.design_generation_pages.len() {
                    state.design_generation_pages[page_index].status =
                        DesignGenerationStatus::Queued;
                    for module_index in 0..state.design_generation_pages[page_index].modules.len() {
                        let target_frame_id = state.design_generation_pages[page_index].modules
                            [module_index]
                            .target_frame_id
                            .clone();
                        state.design_generation_pages[page_index].modules[module_index]
                            .logs
                            .clear();
                        state.design_generation_pages[page_index].modules[module_index]
                            .is_generating = false;
                        state.design_generation_pages[page_index].modules[module_index]
                            .generated_doc = None;
                        state.design_generation_pages[page_index].modules[module_index]
                            .target_frame_options = build_target_frame_options(
                            &state.doc,
                            page_index,
                            module_index,
                            Some(&target_frame_id),
                        );
                        state.design_generation_pages[page_index].modules[module_index].status =
                            DesignGenerationStatus::Queued;
                        sync_module_placeholder_status(
                            state,
                            &target_frame_id,
                            DesignGenerationStatus::Queued,
                        );
                        queued_count += 1;
                    }
                }
                state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                state.canvas_cache.clear();
                let parallel_limit = design_page_parallel_limit(state);
                let queued_batch = next_queued_generation_pages(state, parallel_limit);
                if !queued_batch.is_empty() {
                    state.design_generation_loading = true;
                    state.design_generation_summary = Some(format!(
                        "整体结构已就绪：{} 个模块待生成，已启动按页面并行生成。",
                        queued_count
                    ));
                    let mut tasks = vec![Task::done(Message::Design(DesignMessage::Snapshot))];
                    tasks.extend(queued_batch.into_iter().map(
                        |(next_page_frame_id, next_module_id)| {
                            Task::done(Message::Design(DesignMessage::GenerateDesignPage(
                                next_page_frame_id,
                                next_module_id,
                            )))
                        },
                    ));
                    return Task::batch(tasks);
                }
                state.design_generation_loading = false;
                state.design_generation_anim_frame = 0;
                state.design_generation_summary = Some(format!(
                    "结构规划完成：{} 个模块已排队，等待页面任务生成。",
                    queued_count
                ));
                return Task::done(Message::Design(DesignMessage::Snapshot));
            }
            Err(error) => {
                state.design_generation_loading = false;
                state.design_generation_anim_frame = 0;
                append_design_project_log(
                    &project_path,
                    format!("event=plan_completed_failed error={}", error),
                    current_log_file.as_deref(),
                );
                state.design_generation_summary = Some(error.clone());
                push_design_generation_log(state, format!("[PLAN] failed {}", error));
                state.design_chat_messages.push(DesignChatMessage {
                    role: DesignChatRole::Assistant,
                    content: format!("生成失败：{}", error),
                });
            }
        }
        state.design_generation_stream_cursor = 0;
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}
