//! 设计生成任务调度与画布草图构建。

use super::logging::{append_design_project_log, collect_design_log_lines};
use super::parser::parse_design_generation_module_doc;
use super::prompts::{
    build_page_generation_prompt, compact_multiline, design_executor_uses_gateway,
    design_plan_page_width, design_reference_tokens_and_theme,
    execute_design_generation_with_streaming, resolve_design_acp_agent,
};
use super::DesignModuleExecutionResult;
use crate::app::message::DesignMessage;
use crate::app::task::{TaskExecutorBackend, TaskLogStream};
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationPage, DesignGenerationStatus,
    DesignGenerationTheme, DesignStyle, sanitize_design_generation_parallel_pages,
};
use crate::app::Message;
use iced::Task;
use std::sync::mpsc;

pub(super) fn next_queued_generation_pages(
    state: &crate::app::views::design::state::DesignState,
    limit: usize,
) -> Vec<(String, String)> {
    if limit == 0 {
        return Vec::new();
    }
    state
        .design_generation_pages
        .iter()
        .filter(|page| {
            page.modules.iter().any(|module| {
                matches!(
                    module.status,
                    DesignGenerationStatus::Queued | DesignGenerationStatus::Placeholder
                ) && module.generated_doc.is_none()
                    && !module.is_generating
            })
        })
        .take(limit)
        .map(|page| {
            let anchor_module_id = page
                .modules
                .iter()
                .find(|module| {
                    matches!(
                        module.status,
                        DesignGenerationStatus::Queued | DesignGenerationStatus::Placeholder
                    ) && module.generated_doc.is_none()
                        && !module.is_generating
                })
                .map(|module| module.module_id.clone())
                .unwrap_or_default();
            (page.frame_id.clone(), anchor_module_id)
        })
        .collect()
}

pub(super) fn count_running_generation_pages(
    state: &crate::app::views::design::state::DesignState,
) -> usize {
    state
        .design_generation_pages
        .iter()
        .filter(|page| {
            page.modules.iter().any(|module| {
                module.is_generating || matches!(module.status, DesignGenerationStatus::Running)
            })
        })
        .count()
}

pub(super) fn design_page_parallel_limit(
    state: &crate::app::views::design::state::DesignState,
) -> usize {
    sanitize_design_generation_parallel_pages(state.design_generation_parallel_pages)
}

pub(super) fn summarize_generated_pages_for_prompt(
    pages: &[DesignGenerationPage],
    current_page_frame_id: &str,
) -> String {
    let mut lines = Vec::new();
    for page in pages {
        if page.frame_id == current_page_frame_id {
            continue;
        }
        let mut filled_count = 0usize;
        for module in &page.modules {
            if matches!(module.status, DesignGenerationStatus::Filled) {
                filled_count += 1;
            }
        }
        lines.push(format!(
            "- 页面:{} | status={} | modules={} | filled_modules={}",
            page.title,
            prompt_status_label(page.status),
            page.modules.len(),
            filled_count
        ));
    }
    if lines.is_empty() {
        "暂无可参考的已完成页面，请严格遵循当前页面目标并保持主题风格一致。".to_string()
    } else {
        lines.into_iter().take(8).collect::<Vec<_>>().join("\n")
    }
}

fn prompt_status_label(status: DesignGenerationStatus) -> &'static str {
    match status {
        DesignGenerationStatus::Placeholder => "placeholder",
        DesignGenerationStatus::Queued => "queued",
        DesignGenerationStatus::Running => "running",
        DesignGenerationStatus::Generated => "generated",
        DesignGenerationStatus::Filled => "filled",
        DesignGenerationStatus::Failed => "failed",
        DesignGenerationStatus::Planned => "planned",
        DesignGenerationStatus::Aggregated => "aggregated",
    }
}

fn summarize_generated_root_height(doc: &DesignDoc) -> Option<f32> {
    let root = doc.children.first()?;
    let theme_mode = doc.theme.as_ref().map(|theme| theme.mode.as_str());
    crate::app::views::design::models::parse_val(&root.height, &doc.variables, theme_mode)
}

pub(super) fn summarize_page_modules_for_prompt(page: &DesignGenerationPage) -> String {
    let mut lines = Vec::new();
    for (index, module) in page.modules.iter().enumerate() {
        let mut line = format!(
            "{}. {} | module_id={} | status={}",
            index + 1,
            module.title,
            module.module_id,
            prompt_status_label(module.status)
        );
        if let Some(doc) = &module.generated_doc {
            if let Some(height) = summarize_generated_root_height(doc) {
                line.push_str(&format!(" | 已生成高度≈{:.0}px", height));
            } else {
                line.push_str(" | 已生成高度=unknown");
            }
        } else {
            line.push_str(" | 待生成");
        }
        lines.push(line);
    }
    if lines.is_empty() {
        format!("页面“{}”暂无模块摘要。", page.title)
    } else {
        format!(
            "页面“{}”模块顺序与状态：\n{}",
            page.title,
            lines.into_iter().take(12).collect::<Vec<_>>().join("\n")
        )
    }
}

pub(super) fn count_generation_progress(
    state: &crate::app::views::design::state::DesignState,
) -> (usize, usize, usize) {
    let mut filled_count = 0usize;
    let mut queued_count = 0usize;
    let mut failed_count = 0usize;
    for page in &state.design_generation_pages {
        for module in &page.modules {
            match module.status {
                DesignGenerationStatus::Filled => filled_count += 1,
                DesignGenerationStatus::Failed => failed_count += 1,
                DesignGenerationStatus::Queued | DesignGenerationStatus::Running => {
                    queued_count += 1
                }
                DesignGenerationStatus::Placeholder => {
                    if module.generated_doc.is_none() {
                        queued_count += 1;
                    }
                }
                DesignGenerationStatus::Generated => queued_count += 1,
                DesignGenerationStatus::Planned | DesignGenerationStatus::Aggregated => {}
            }
        }
    }
    (filled_count, queued_count, failed_count)
}

pub(super) fn spawn_design_module_generation_task(
    project_path: String,
    design_brief: String,
    executor: TaskExecutorBackend,
    selected_acp_agent: Option<String>,
    theme: DesignGenerationTheme,
    style: DesignStyle,
    device: DesignGenerationDevice,
    model: String,
    generated_pages_summary: String,
    page_snapshot: DesignGenerationPage,
    page_frame_id: String,
    module_id: String,
    current_log_file: Option<String>,
) -> Task<Message> {
    let (log_tx, log_rx) = mpsc::channel::<TaskLogStream>();
    let log_scope = format!("page:{}", page_snapshot.title);
    Task::perform(
        async move {
            let prompt = build_page_generation_prompt(
                &design_brief,
                executor,
                theme,
                style,
                device,
                &page_snapshot,
                &generated_pages_summary,
            );
            let acp_agent = resolve_design_acp_agent(executor, selected_acp_agent.as_deref());
            let route = if design_executor_uses_gateway(executor) {
                format!("gateway agent={}", acp_agent.as_deref().unwrap_or("default"))
            } else {
                let command =
                    crate::app::task::build_executor_command(executor, &project_path, &model, &prompt);
                format!("cli command={}", compact_multiline(&format!("{:?}", command)))
            };
            append_design_project_log(
                &project_path,
                format!(
                    "event=page_generate_submit page={} executor={} model={} theme={} prompt_chars={}",
                    page_snapshot.title,
                    executor.label(),
                    model,
                    theme.label(),
                    prompt.chars().count()
                ),
                current_log_file.as_deref(),
            );
            append_design_project_log(
                &project_path,
                format!(
                    "event=page_generate_dispatch page={} executor={} route={}",
                    page_snapshot.title,
                    executor.label(),
                    route
                ),
                current_log_file.as_deref(),
            );
            let session_scope = format!("page-{}", page_snapshot.frame_id);
            let execution_project_path = project_path.clone();
            let output: (Result<String, String>, Vec<String>) =
                crate::app::message::spawn_blocking_opt(move || {
                    let result = execute_design_generation_with_streaming(
                        executor,
                        &execution_project_path,
                        &model,
                        &prompt,
                        acp_agent,
                        log_tx,
                        &session_scope,
                    );
                    let logs = collect_design_log_lines(&log_scope, &log_rx);
                    Some((result, logs))
                })
                .await
                .ok_or_else(|| format!("页面\"{}\"生成任务没有返回结果。", page_snapshot.title))?;

            let (result, logs) = output;
            for line in &logs {
                append_design_project_log(
                    &project_path,
                    format!(
                        "event=page_generate_stream page={} line={}",
                        page_snapshot.title, line
                    ),
                    current_log_file.as_deref(),
                );
            }
            let raw = result?;
            let doc = parse_design_generation_module_doc(&raw, &logs, theme).map_err(|error| {
                let message = format!(
                    "页面\"{}\"生成结果不是合法 .json JSON: {}",
                    page_snapshot.title, error
                );
                append_design_project_log(
                    &project_path,
                    format!(
                        "event=page_generate_parse_failed page={} error={} raw={} stream={}",
                        page_snapshot.title,
                        error,
                        compact_multiline(&raw),
                        compact_multiline(&logs.join("\n"))
                    ),
                    current_log_file.as_deref(),
                );
                message
            })?;
            append_design_project_log(
                &project_path,
                format!(
                    "event=page_generate_success page={} root_children={} raw={}",
                    page_snapshot.title,
                    doc.children.len(),
                    compact_multiline(&raw)
                ),
                current_log_file.as_deref(),
            );

            Ok(DesignModuleExecutionResult { doc, logs })
        },
        move |result| {
            Message::Design(DesignMessage::DesignPageGenerated {
                page_frame_id,
                page_task_id: module_id,
                result,
            })
        },
    )
}

pub(super) fn build_design_plan_canvas(
    pages: &[DesignGenerationPage],
    theme: DesignGenerationTheme,
    device: DesignGenerationDevice,
    summary: Option<&str>,
) -> DesignDoc {
    let _ = summary;
    let mut doc_children = Vec::new();
    let page_width = design_plan_page_width(device);
    let page_gap = 32.0;
    let page_start_x = 0.0;
    let page_start_y = 0.0;
    let (variables, doc_theme) = design_reference_tokens_and_theme(theme);

    for (page_index, page) in pages.iter().enumerate() {
        let page_x = page_start_x + page_index as f32 * (page_width + page_gap);
        let page_y = page_start_y;
        let mut page_children = Vec::new();

        page_children.push(crate::app::views::design::models::DesignElement {
            kind: "text".to_string(),
            id: format!("page-title-{}", page_index),
            x: 24.0,
            y: 24.0,
            name: Some("Page Title".to_string()),
            content: Some(page.title.clone()),
            font_size: Some(serde_json::json!(22)),
            font_family: Some("$--font-primary".to_string()),
            font_weight: Some(serde_json::json!(700)),
            color: Some("$--foreground".to_string()),
            ..Default::default()
        });

        page_children.push(crate::app::views::design::models::DesignElement {
            kind: "text".to_string(),
            id: format!("page-objective-{}", page_index),
            x: 24.0,
            y: 62.0,
            name: Some("Page Objective".to_string()),
            width: Some(serde_json::json!(page_width - 48.0)),
            content: Some(page.objective.clone()),
            font_size: Some(serde_json::json!(13)),
            font_family: Some("$--font-primary".to_string()),
            line_height: Some(serde_json::json!(1.5)),
            color: Some("$--muted-foreground".to_string()),
            ..Default::default()
        });

        page_children.push(crate::app::views::design::models::DesignElement {
            kind: "text".to_string(),
            id: format!("page-status-{}", page_index),
            x: 24.0,
            y: 112.0,
            name: Some("Page Status".to_string()),
            content: Some(format!("状态: {}", page.status.label())),
            font_size: Some(serde_json::json!(12)),
            font_family: Some("$--font-primary".to_string()),
            font_weight: Some(serde_json::json!(600)),
            color: Some("$--primary".to_string()),
            ..Default::default()
        });

        let mut current_y = 150.0;
        for module in &page.modules {
            let module_height = 148.0;
            let module_id = module.target_frame_id.clone();
            let module_children = vec![
                crate::app::views::design::models::DesignElement {
                    kind: "text".to_string(),
                    id: format!("{}-title", module_id),
                    x: 18.0,
                    y: 16.0,
                    name: Some("Module Title".to_string()),
                    content: Some(module.title.clone()),
                    font_size: Some(serde_json::json!(18)),
                    font_family: Some("$--font-primary".to_string()),
                    font_weight: Some(serde_json::json!(700)),
                    color: Some("$--card-foreground".to_string()),
                    ..Default::default()
                },
                crate::app::views::design::models::DesignElement {
                    kind: "text".to_string(),
                    id: format!("{}-description", module_id),
                    x: 18.0,
                    y: 48.0,
                    width: Some(serde_json::json!(page_width - 84.0)),
                    name: Some("Module Description".to_string()),
                    content: Some(module.description.clone()),
                    font_size: Some(serde_json::json!(12)),
                    font_family: Some("$--font-primary".to_string()),
                    line_height: Some(serde_json::json!(1.45)),
                    color: Some("$--muted-foreground".to_string()),
                    ..Default::default()
                },
                crate::app::views::design::models::DesignElement {
                    kind: "text".to_string(),
                    id: format!("{}-status", module_id),
                    x: 18.0,
                    y: 102.0,
                    name: Some("Module Status".to_string()),
                    content: Some(format!("{} · 实时预览占位", module.status.label())),
                    font_size: Some(serde_json::json!(11)),
                    font_family: Some("$--font-primary".to_string()),
                    font_weight: Some(serde_json::json!(600)),
                    color: Some("$--primary".to_string()),
                    ..Default::default()
                },
                crate::app::views::design::models::DesignElement {
                    kind: "frame".to_string(),
                    id: format!("{}-badge", module_id),
                    x: page_width - 130.0,
                    y: 14.0,
                    name: Some("Module Badge".to_string()),
                    width: Some(serde_json::json!(88)),
                    height: Some(serde_json::json!(26)),
                    fill: Some(serde_json::json!(match module.status {
                        DesignGenerationStatus::Queued => "#F59E0B",
                        DesignGenerationStatus::Running => "#9333EA",
                        DesignGenerationStatus::Filled => "#0891B2",
                        DesignGenerationStatus::Failed => "#DC2626",
                        DesignGenerationStatus::Generated => "#059669",
                        _ => "$--secondary",
                    })),
                    corner_radius: Some(serde_json::json!(999)),
                    children: vec![crate::app::views::design::models::DesignElement {
                        kind: "text".to_string(),
                        id: format!("{}-badge-text", module_id),
                        x: 12.0,
                        y: 6.0,
                        name: Some("Module Badge Text".to_string()),
                        content: Some(
                            match module.status {
                                DesignGenerationStatus::Queued => "queued",
                                DesignGenerationStatus::Running => "running",
                                DesignGenerationStatus::Filled => "filled",
                                DesignGenerationStatus::Failed => "failed",
                                DesignGenerationStatus::Generated => "generated",
                                _ => "preview",
                            }
                            .to_string(),
                        ),
                        font_size: Some(serde_json::json!(10)),
                        font_family: Some("$--font-secondary".to_string()),
                        font_weight: Some(serde_json::json!(700)),
                        color: Some("#FFFFFF".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                crate::app::views::design::models::DesignElement {
                    kind: "text".to_string(),
                    id: format!("{}-slot-hint", module_id),
                    x: 18.0,
                    y: 122.0,
                    name: Some("Module Slot Hint".to_string()),
                    content: Some("占位模块，可继续生成并汇总到当前页面".to_string()),
                    font_size: Some(serde_json::json!(10)),
                    font_family: Some("$--font-primary".to_string()),
                    color: Some("$--muted-foreground".to_string()),
                    ..Default::default()
                },
                crate::app::views::design::models::DesignElement {
                    kind: "text".to_string(),
                    id: format!("{}-status-id", module_id),
                    x: page_width - 212.0,
                    y: 104.0,
                    name: Some("Module Status Id".to_string()),
                    width: Some(serde_json::json!(188.0)),
                    content: Some(format!("id: {} · {}", module.module_id, module.status.label())),
                    font_size: Some(serde_json::json!(10)),
                    font_family: Some("$--font-secondary".to_string()),
                    color: Some("$--muted-foreground".to_string()),
                    ..Default::default()
                },
            ];

            page_children.push(crate::app::views::design::models::DesignElement {
                kind: "frame".to_string(),
                id: module_id,
                x: 24.0,
                y: current_y,
                name: Some(module.title.clone()),
                context: Some(module.description.clone()),
                width: Some(serde_json::json!(page_width - 48.0)),
                height: Some(serde_json::json!(module_height)),
                fill: Some(serde_json::json!("$--card")),
                corner_radius: Some(serde_json::json!(16)),
                stroke: Some(crate::app::views::design::models::Stroke {
                    align: Some("inside".to_string()),
                    thickness: Some(serde_json::json!(1)),
                    fill: Some("$--border".to_string()),
                }),
                layout: Some("vertical".to_string()),
                gap: Some(serde_json::json!(8)),
                padding: Some(serde_json::json!(16)),
                children: module_children,
                ..Default::default()
            });
            current_y += module_height + 18.0;
        }

        let page_height = current_y + 24.0;
        doc_children.push(crate::app::views::design::models::DesignElement {
            kind: "frame".to_string(),
            id: page.frame_id.clone(),
            x: page_x,
            y: page_y,
            name: Some(page.title.clone()),
            context: Some(page.objective.clone()),
            width: Some(serde_json::json!(page_width)),
            height: Some(serde_json::json!(page_height)),
            fill: Some(serde_json::json!("$--background")),
            corner_radius: Some(serde_json::json!(24)),
            stroke: Some(crate::app::views::design::models::Stroke {
                align: Some("inside".to_string()),
                thickness: Some(serde_json::json!(1)),
                fill: Some("$--border".to_string()),
            }),
            clip: Some(true),
            children: page_children,
            ..Default::default()
        });
    }

    DesignDoc {
        version: "2.6".to_string(),
        children: doc_children,
        variables,
        theme: doc_theme.and_then(|value| serde_json::from_value(value).ok()),
        ..Default::default()
    }
}

