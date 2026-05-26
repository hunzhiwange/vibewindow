//! 保存清理工具消息处理所需的界面状态和可复用状态转换。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::run::execute_cleanup;
use super::scan::scan_cleanup_targets;
use super::types::{CleanerPlatform, CleanerScanReport, CleanerToolMessage, CleanupRequest};
use super::{current_platform, format_bytes};
use crate::app::{App, Message};
use iced::Task;
use iced::widget::text_editor;
use std::sync::atomic::Ordering;

const CLEANER_TICK_MS: u64 = 120;

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub fn update(app: &mut App, message: CleanerToolMessage) -> Task<Message> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        CleanerToolMessage::EditorAction(action) => {
            app.cleaner_output_editor.perform(action);
            Task::none()
        }
        CleanerToolMessage::ToggleSystemTemp(value) => {
            app.cleaner_clear_system_temp = value;
            Task::none()
        }
        CleanerToolMessage::ToggleAppCache(value) => {
            app.cleaner_clear_app_cache = value;
            Task::none()
        }
        CleanerToolMessage::ToggleLogs(value) => {
            app.cleaner_clear_logs = value;
            Task::none()
        }
        CleanerToolMessage::TogglePackageCache(value) => {
            app.cleaner_clear_package_cache = value;
            Task::none()
        }
        CleanerToolMessage::ToggleDownloads(value) => {
            app.cleaner_clear_downloads = value;
            Task::none()
        }
        CleanerToolMessage::ToggleTrash(value) => {
            app.cleaner_empty_trash = value;
            Task::none()
        }
        CleanerToolMessage::ToggleInstallers(value) => {
            app.cleaner_clear_installers = value;
            Task::none()
        }
        CleanerToolMessage::ToggleOtherApps(value) => {
            app.cleaner_clear_other_apps = value;
            Task::none()
        }
        CleanerToolMessage::ToggleWeChatWork(value) => {
            app.cleaner_clear_wechat_work = value;
            Task::none()
        }
        CleanerToolMessage::ToggleWeChat(value) => {
            app.cleaner_clear_wechat = value;
            Task::none()
        }
        CleanerToolMessage::ToggleQq(value) => {
            app.cleaner_clear_qq = value;
            Task::none()
        }
        CleanerToolMessage::ToggleDingTalk(value) => {
            app.cleaner_clear_dingtalk = value;
            Task::none()
        }
        CleanerToolMessage::ToggleFeishu(value) => {
            app.cleaner_clear_feishu = value;
            Task::none()
        }
        CleanerToolMessage::ToggleSafari(value) => {
            app.cleaner_clear_safari = value;
            Task::none()
        }
        CleanerToolMessage::ToggleChrome(value) => {
            app.cleaner_clear_chrome = value;
            Task::none()
        }
        CleanerToolMessage::ToggleEdge(value) => {
            app.cleaner_clear_edge = value;
            Task::none()
        }
        CleanerToolMessage::ToggleFirefox(value) => {
            app.cleaner_clear_firefox = value;
            Task::none()
        }
        CleanerToolMessage::ToggleMail(value) => {
            app.cleaner_clear_mail = value;
            Task::none()
        }
        CleanerToolMessage::Scan => start_scan(app),
        CleanerToolMessage::ScanFinished(result) => finish_scan(app, result),
        CleanerToolMessage::ToggleTreeNode(id) => {
            if !app.cleaner_tree_expanded.insert(id.clone()) {
                app.cleaner_tree_expanded.remove(&id);
            }
            Task::none()
        }
        CleanerToolMessage::Run => start_run(app),
        CleanerToolMessage::Cancel => cancel_run(app),
        CleanerToolMessage::RunFinished(result) => finish_run(app, result),
        CleanerToolMessage::Tick => tick(app),
        CleanerToolMessage::Clear => {
            app.cleaner_output_editor = text_editor::Content::new();
            app.cleaner_notification = Some("已清空清理记录".to_string());
            crate::app::message::after(
                std::time::Duration::from_secs(2),
                Message::CleanerTool(CleanerToolMessage::ClearNotification),
            )
        }
        CleanerToolMessage::ClearNotification => {
            app.cleaner_notification = None;
            Task::none()
        }
    }
}

fn start_scan(app: &mut App) -> Task<Message> {
    if app.cleaner_scanning || app.cleaner_running {
        return Task::none();
    }

    app.cleaner_scanning = true;
    app.cleaner_scanned = false;
    app.cleaner_last_run_completed = false;
    app.cleaner_animation_frame = 0;
    app.cleaner_notification = Some("正在搜索可清理文件…".to_string());
    app.cleaner_output_editor = text_editor::Content::with_text(&initial_scan_log());

    Task::batch(vec![
        // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
        Task::perform(
            async move {
                crate::app::message::spawn_blocking_opt(scan_cleanup_targets)
                    .await
                    .unwrap_or_else(|| Err("扫描任务执行失败或被取消".to_string()))
            },
            |result| Message::CleanerTool(CleanerToolMessage::ScanFinished(result)),
        ),
        crate::app::message::after(
            std::time::Duration::from_millis(CLEANER_TICK_MS),
            Message::CleanerTool(CleanerToolMessage::Tick),
        ),
    ])
}

fn finish_scan(app: &mut App, result: Result<CleanerScanReport, String>) -> Task<Message> {
    app.cleaner_scanning = false;
    app.cleaner_animation_frame = 0;

    match result {
        Ok(report) => {
            app.cleaner_scanned = true;
            app.cleaner_tree_expanded = default_expanded_nodes(&report);
            app.cleaner_output_editor = text_editor::Content::with_text(&scan_log(&report));
            app.cleaner_notification = Some(if report.total_bytes > 0 {
                format!("搜索完成，共发现可清理文件 {}", format_bytes(report.total_bytes))
            } else {
                "搜索完成，当前系统较干净".to_string()
            });
            app.cleaner_scan_report = Some(report);
        }
        Err(error) => {
            app.cleaner_scanned = false;
            app.cleaner_tree_expanded.clear();
            app.cleaner_scan_report = None;
            app.cleaner_output_editor = text_editor::Content::with_text(&error);
            app.cleaner_notification = Some("搜索失败，请查看日志".to_string());
        }
    }

    crate::app::message::after(
        std::time::Duration::from_secs(3),
        Message::CleanerTool(CleanerToolMessage::ClearNotification),
    )
}

fn start_run(app: &mut App) -> Task<Message> {
    if app.cleaner_running || app.cleaner_scanning {
        return Task::none();
    }
    if !app.cleaner_scanned || app.cleaner_scan_report.is_none() {
        return start_scan(app);
    }
    if !has_selected_items(app) {
        app.cleaner_notification = Some("请先勾选需要清理的项目".to_string());
        return crate::app::message::after(
            std::time::Duration::from_secs(2),
            Message::CleanerTool(CleanerToolMessage::ClearNotification),
        );
    }

    app.cleaner_running = true;
    app.cleaner_cancelling = false;
    app.cleaner_last_run_completed = false;
    app.cleaner_animation_frame = 0;
    let (_, selected_bytes) = selected_scan_totals(app);
    app.cleaner_notification = Some(if selected_bytes > 0 {
        format!("开始清理，预计释放 {}…", format_bytes(selected_bytes))
    } else {
        "开始清理，请稍候…".to_string()
    });
    app.cleaner_output_editor = text_editor::Content::with_text(&initial_log(app));
    app.cleaner_cancel_flag.store(false, Ordering::Relaxed);

    let request = CleanupRequest::from_app(app);
    let cancel_flag = app.cleaner_cancel_flag.clone();

    Task::batch(vec![
        Task::perform(
            async move {
                crate::app::message::spawn_blocking_opt(move || {
                    execute_cleanup(request, cancel_flag)
                })
                .await
                .unwrap_or_else(|| Err("清理任务执行失败或被取消".to_string()))
            },
            |result| Message::CleanerTool(CleanerToolMessage::RunFinished(result)),
        ),
        crate::app::message::after(
            std::time::Duration::from_millis(CLEANER_TICK_MS),
            Message::CleanerTool(CleanerToolMessage::Tick),
        ),
    ])
}

fn cancel_run(app: &mut App) -> Task<Message> {
    if app.cleaner_scanning {
        app.cleaner_scanning = false;
        app.cleaner_cancelling = false;
        app.cleaner_animation_frame = 0;
        app.cleaner_notification = Some("已取消搜索".to_string());
        app.cleaner_output_editor = text_editor::Content::with_text("搜索已取消。\n");
        return crate::app::message::after(
            std::time::Duration::from_secs(2),
            Message::CleanerTool(CleanerToolMessage::ClearNotification),
        );
    }

    if !app.cleaner_running {
        return Task::none();
    }

    app.cleaner_cancelling = true;
    app.cleaner_notification = Some("正在取消清理，请稍候…".to_string());
    app.cleaner_cancel_flag.store(true, Ordering::Relaxed);
    Task::none()
}

fn finish_run(app: &mut App, result: Result<String, String>) -> Task<Message> {
    app.cleaner_running = false;
    app.cleaner_cancelling = false;
    app.cleaner_animation_frame = 0;
    app.cleaner_cancel_flag.store(false, Ordering::Relaxed);

    match result {
        Ok(output) => {
            app.cleaner_output_editor = text_editor::Content::with_text(&output);
            let cancelled = output.contains("已取消");
            if !cancelled {
                app.cleaner_scanned = false;
                app.cleaner_scan_report = None;
                app.cleaner_tree_expanded.clear();
                app.cleaner_last_run_completed = true;
            } else {
                app.cleaner_last_run_completed = false;
            }
            app.cleaner_notification = Some(if cancelled {
                "已取消清理".to_string()
            } else {
                "清理完成，可重新搜索确认剩余项目".to_string()
            });
        }
        Err(error) => {
            app.cleaner_output_editor = text_editor::Content::with_text(&error);
            app.cleaner_last_run_completed = false;
            app.cleaner_notification = Some(if error.contains("已取消") {
                "已取消清理".to_string()
            } else {
                "清理失败，请查看日志".to_string()
            });
        }
    }

    crate::app::message::after(
        std::time::Duration::from_secs(3),
        Message::CleanerTool(CleanerToolMessage::ClearNotification),
    )
}

fn tick(app: &mut App) -> Task<Message> {
    if !app.cleaner_running && !app.cleaner_scanning {
        return Task::none();
    }

    app.cleaner_animation_frame = app.cleaner_animation_frame.wrapping_add(1);
    crate::app::message::after(
        std::time::Duration::from_millis(CLEANER_TICK_MS),
        Message::CleanerTool(CleanerToolMessage::Tick),
    )
}

fn initial_log(app: &App) -> String {
    let platform = match current_platform() {
        Some(CleanerPlatform::MacOs) => "macOS",
        Some(CleanerPlatform::Windows) => "Windows",
        None => "Unsupported",
    };

    let mut lines = vec![
        format!("准备开始清理 {platform} 垃圾文件"),
        if let Some(report) = &app.cleaner_scan_report {
            format!("- 已完成扫描，共发现 {} 可清理数据", format_bytes(report.total_bytes))
        } else {
            "- 本次将按默认规则直接执行清理".to_string()
        },
        "- 临时目录、缓存、日志等将按勾选项处理".to_string(),
        "- 某些系统目录可能触发授权弹窗，请按系统提示确认".to_string(),
        "- 清理完成后会在这里显示执行结果".to_string(),
        "".to_string(),
        "已选择项目：".to_string(),
    ];

    for item in selected_items(app) {
        lines.push(format!("  - {item}"));
    }

    lines.join("\n")
}

fn initial_scan_log() -> String {
    [
        "开始搜索可清理文件…",
        "",
        "将按系统垃圾、应用垃圾与上网垃圾三组进行扫描。",
        "下载、安装包、浏览器缓存以及敏感应用默认保持未勾选。",
        "扫描完成后可在左侧树形列表展开明细并按需勾选。",
    ]
    .join("\n")
}

fn selected_items(app: &App) -> Vec<&'static str> {
    let mut items = Vec::new();
    if app.cleaner_clear_system_temp {
        items.push("系统临时目录");
    }
    if app.cleaner_clear_app_cache {
        items.push("应用缓存");
    }
    if app.cleaner_clear_logs {
        items.push("日志与崩溃文件");
    }
    if app.cleaner_clear_package_cache {
        items.push("开发缓存");
    }
    if app.cleaner_clear_downloads {
        items.push("下载目录");
    }
    if app.cleaner_empty_trash {
        items.push("废纸篓 / 回收站");
    }
    if app.cleaner_clear_installers {
        items.push("安装包");
    }
    if app.cleaner_clear_other_apps {
        items.push("其他应用缓存");
    }
    if app.cleaner_clear_wechat_work {
        items.push("企业微信缓存");
    }
    if app.cleaner_clear_wechat {
        items.push("微信缓存");
    }
    if app.cleaner_clear_qq {
        items.push("QQ 缓存");
    }
    if app.cleaner_clear_dingtalk {
        items.push("钉钉缓存");
    }
    if app.cleaner_clear_feishu {
        items.push("飞书缓存");
    }
    if app.cleaner_clear_safari {
        items.push("Safari 上网缓存");
    }
    if app.cleaner_clear_chrome {
        items.push("Chrome 上网缓存");
    }
    if app.cleaner_clear_edge {
        items.push("Edge 上网缓存");
    }
    if app.cleaner_clear_firefox {
        items.push("Firefox 上网缓存");
    }
    if app.cleaner_clear_mail {
        items.push("Mail 邮件缓存");
    }
    if items.is_empty() {
        items.push("未选择任何项目");
    }
    items
}

fn has_selected_items(app: &App) -> bool {
    app.cleaner_clear_system_temp
        || app.cleaner_clear_app_cache
        || app.cleaner_clear_logs
        || app.cleaner_clear_package_cache
        || app.cleaner_clear_downloads
        || app.cleaner_empty_trash
        || app.cleaner_clear_installers
        || app.cleaner_clear_other_apps
        || app.cleaner_clear_wechat_work
        || app.cleaner_clear_wechat
        || app.cleaner_clear_qq
        || app.cleaner_clear_dingtalk
        || app.cleaner_clear_feishu
        || app.cleaner_clear_safari
        || app.cleaner_clear_chrome
        || app.cleaner_clear_edge
        || app.cleaner_clear_firefox
        || app.cleaner_clear_mail
}

/// selected_scan_totals 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub fn selected_scan_totals(app: &App) -> (usize, u64) {
    let Some(report) = &app.cleaner_scan_report else {
        return (0, 0);
    };

    let mut count = 0usize;
    let mut bytes = 0u64;
    for group in &report.groups {
        for item in &group.items {
            if is_item_selected(app, &item.id) {
                count += 1;
                bytes = bytes.saturating_add(item.total_bytes);
            }
        }
    }
    (count, bytes)
}

fn is_item_selected(app: &App, id: &str) -> bool {
    match id {
        "system_temp" => app.cleaner_clear_system_temp,
        "app_cache" => app.cleaner_clear_app_cache,
        "logs" => app.cleaner_clear_logs,
        "package_cache" => app.cleaner_clear_package_cache,
        "downloads" => app.cleaner_clear_downloads,
        "trash" => app.cleaner_empty_trash,
        "installers" => app.cleaner_clear_installers,
        "other_apps" => app.cleaner_clear_other_apps,
        "wechat_work" => app.cleaner_clear_wechat_work,
        "wechat" => app.cleaner_clear_wechat,
        "qq" => app.cleaner_clear_qq,
        "dingtalk" => app.cleaner_clear_dingtalk,
        "feishu" => app.cleaner_clear_feishu,
        "safari" => app.cleaner_clear_safari,
        "chrome" => app.cleaner_clear_chrome,
        "edge" => app.cleaner_clear_edge,
        "firefox" => app.cleaner_clear_firefox,
        "mail" => app.cleaner_clear_mail,
        _ => false,
    }
}

fn default_expanded_nodes(report: &CleanerScanReport) -> std::collections::HashSet<String> {
    report.groups.iter().map(|group| group.id.clone()).collect()
}

fn scan_log(report: &CleanerScanReport) -> String {
    let mut lines = vec![
        format!("扫描完成，共发现可清理文件 {}", format_bytes(report.total_bytes)),
        format!("命中 {} 个可清理项", report.matched_items),
        "".to_string(),
    ];

    for group in &report.groups {
        lines.push(format!(
            "{}：{}",
            group.title,
            if group.total_bytes > 0 {
                format_bytes(group.total_bytes)
            } else {
                "很干净".to_string()
            }
        ));
        for item in &group.items {
            lines.push(format!(
                "  - {}：{}",
                item.title,
                if item.total_bytes > 0 {
                    format_bytes(item.total_bytes)
                } else {
                    "很干净".to_string()
                }
            ));
            for detail in &item.details {
                if detail.total_bytes > 0 {
                    lines.push(format!(
                        "      · {}：{} ({})",
                        detail.label,
                        format_bytes(detail.total_bytes),
                        detail.path
                    ));
                }
            }
        }
        lines.push(String::new());
    }

    lines.join("\n")
}
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
