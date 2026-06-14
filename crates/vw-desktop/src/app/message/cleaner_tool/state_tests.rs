#![allow(unused_must_use)]
use super::super::types::{
    CleanerScanDetail, CleanerScanGroup, CleanerScanItem, CleanerScanReport, CleanerStatusResponse,
    CleanerToolMessage,
};
use crate::app::App;
use std::sync::atomic::Ordering;

fn item(id: &str, bytes: u64) -> CleanerScanItem {
    CleanerScanItem {
        id: id.to_string(),
        title: format!("Item {id}"),
        subtitle: "subtitle".to_string(),
        sensitive: false,
        total_bytes: bytes,
        details: vec![CleanerScanDetail {
            label: "Detail".to_string(),
            path: format!("/tmp/{id}"),
            total_bytes: bytes,
        }],
    }
}

fn report() -> CleanerScanReport {
    CleanerScanReport {
        total_bytes: 3072,
        matched_items: 2,
        groups: vec![CleanerScanGroup {
            id: "system".to_string(),
            title: "System".to_string(),
            subtitle: "System junk".to_string(),
            total_bytes: 3072,
            items: vec![item("system_temp", 1024), item("downloads", 2048)],
        }],
    }
}

#[test]
fn toggle_messages_update_each_cleaner_selection_flag() {
    let (mut app, _task) = App::new();

    super::update(&mut app, CleanerToolMessage::ToggleSystemTemp(false));
    super::update(&mut app, CleanerToolMessage::ToggleDownloads(true));
    super::update(&mut app, CleanerToolMessage::ToggleChrome(true));

    assert!(!app.cleaner_clear_system_temp);
    assert!(app.cleaner_clear_downloads);
    assert!(app.cleaner_clear_chrome);
}

#[test]
fn finish_scan_success_records_report_expands_groups_and_writes_log() {
    let (mut app, _task) = App::new();

    super::update(&mut app, CleanerToolMessage::ScanFinished(Ok(report())));

    assert!(!app.cleaner_scanning);
    assert!(app.cleaner_scanned);
    assert!(app.cleaner_tree_expanded.contains("system"));
    assert_eq!(app.cleaner_scan_report.as_ref().map(|report| report.total_bytes), Some(3072));
    assert!(app.cleaner_notification.as_deref().unwrap_or_default().contains("3.00 KB"));
    assert!(app.cleaner_output_editor.text().contains("Item system_temp"));
}

#[test]
fn finish_scan_error_clears_report_and_shows_error_log() {
    let (mut app, _task) = App::new();
    app.cleaner_scan_report = Some(report());
    app.cleaner_tree_expanded.insert("system".to_string());

    super::update(&mut app, CleanerToolMessage::ScanFinished(Err("scan failed".to_string())));

    assert!(!app.cleaner_scanned);
    assert!(app.cleaner_scan_report.is_none());
    assert!(app.cleaner_tree_expanded.is_empty());
    assert_eq!(app.cleaner_output_editor.text(), "scan failed");
    assert_eq!(app.cleaner_notification.as_deref(), Some("搜索失败，请查看日志"));
}

#[test]
fn selected_scan_totals_counts_only_selected_known_items() {
    let (mut app, _task) = App::new();
    app.cleaner_scan_report = Some(report());
    app.cleaner_clear_system_temp = true;
    app.cleaner_clear_downloads = false;

    assert_eq!(super::selected_scan_totals(&app), (1, 1024));

    app.cleaner_clear_downloads = true;
    assert_eq!(super::selected_scan_totals(&app), (2, 3072));
}

#[test]
fn run_without_scan_starts_scan_instead_of_running() {
    let (mut app, _task) = App::new();
    app.cleaner_scanned = false;
    app.cleaner_scan_report = None;

    super::update(&mut app, CleanerToolMessage::Run);

    assert!(app.cleaner_scanning);
    assert!(!app.cleaner_running);
    assert_eq!(app.cleaner_notification.as_deref(), Some("正在搜索可清理文件…"));
}

#[test]
fn run_without_selected_items_shows_prompt() {
    let (mut app, _task) = App::new();
    app.cleaner_scanned = true;
    app.cleaner_scan_report = Some(report());
    app.cleaner_clear_system_temp = false;
    app.cleaner_clear_app_cache = false;
    app.cleaner_clear_logs = false;
    app.cleaner_clear_package_cache = false;
    app.cleaner_clear_other_apps = false;

    super::update(&mut app, CleanerToolMessage::Run);

    assert!(!app.cleaner_running);
    assert_eq!(app.cleaner_notification.as_deref(), Some("请先勾选需要清理的项目"));
}

#[test]
fn cancel_scanning_resets_scan_state_and_writes_cancel_log() {
    let (mut app, _task) = App::new();
    app.cleaner_scanning = true;
    app.cleaner_animation_frame = 7;

    super::update(&mut app, CleanerToolMessage::Cancel);

    assert!(!app.cleaner_scanning);
    assert_eq!(app.cleaner_animation_frame, 0);
    assert_eq!(app.cleaner_notification.as_deref(), Some("已取消搜索"));
    assert!(app.cleaner_output_editor.text().contains("搜索已取消"));
}

#[test]
fn finish_run_success_clears_scan_and_marks_completed() {
    let (mut app, _task) = App::new();
    app.cleaner_running = true;
    app.cleaner_cancelling = true;
    app.cleaner_scan_report = Some(report());
    app.cleaner_tree_expanded.insert("system".to_string());
    app.cleaner_cancel_flag.store(true, Ordering::Relaxed);

    super::update(&mut app, CleanerToolMessage::RunFinished(Ok("removed files".to_string())));

    assert!(!app.cleaner_running);
    assert!(!app.cleaner_cancelling);
    assert!(!app.cleaner_cancel_flag.load(Ordering::Relaxed));
    assert!(app.cleaner_scan_report.is_none());
    assert!(app.cleaner_tree_expanded.is_empty());
    assert!(app.cleaner_last_run_completed);
    assert_eq!(app.cleaner_notification.as_deref(), Some("清理完成，可重新搜索确认剩余项目"));
}

#[test]
fn finish_run_cancelled_keeps_completion_false() {
    let (mut app, _task) = App::new();
    app.cleaner_running = true;
    app.cleaner_scan_report = Some(report());

    super::update(&mut app, CleanerToolMessage::RunFinished(Ok("用户已取消".to_string())));

    assert!(!app.cleaner_last_run_completed);
    assert!(app.cleaner_scan_report.is_some());
    assert_eq!(app.cleaner_notification.as_deref(), Some("已取消清理"));
}

#[test]
fn status_loaded_updates_output_only_while_running_and_non_empty() {
    let (mut app, _task) = App::new();
    app.cleaner_running = true;

    super::update(
        &mut app,
        CleanerToolMessage::StatusLoaded(Ok(CleanerStatusResponse {
            running: true,
            output: "live output".to_string(),
        })),
    );

    assert_eq!(app.cleaner_output_editor.text(), "live output");

    app.cleaner_running = false;
    super::update(
        &mut app,
        CleanerToolMessage::StatusLoaded(Ok(CleanerStatusResponse {
            running: true,
            output: "ignored".to_string(),
        })),
    );
    assert_eq!(app.cleaner_output_editor.text(), "live output");
}

#[test]
fn tick_advances_animation_only_when_busy() {
    let (mut app, _task) = App::new();

    super::update(&mut app, CleanerToolMessage::Tick);
    assert_eq!(app.cleaner_animation_frame, 0);

    app.cleaner_scanning = true;
    super::update(&mut app, CleanerToolMessage::Tick);
    assert_eq!(app.cleaner_animation_frame, 1);
}

#[test]
fn clear_and_clear_notification_update_log_and_notification() {
    let (mut app, _task) = App::new();
    app.cleaner_output_editor = iced::widget::text_editor::Content::with_text("old");

    super::update(&mut app, CleanerToolMessage::Clear);
    assert_eq!(app.cleaner_output_editor.text(), "");
    assert_eq!(app.cleaner_notification.as_deref(), Some("已清空清理记录"));

    super::update(&mut app, CleanerToolMessage::ClearNotification);
    assert_eq!(app.cleaner_notification, None);
}
