//! 覆盖大文件工具消息处理的本地状态转换与报告整理。

use super::{
    LargeFileDeleteSummary, LargeFileEntry, LargeFileScanProgress, LargeFileScanReport,
    LargeFileToolMessage, apply_gateway_progress, format_bytes, normalized_root,
    remove_deleted_paths, set_progress, sync_progress_from_state, update, visible_paths,
};
use crate::app::App;
use std::collections::HashSet;
use vw_gateway_client::vw_api_types::file::{
    LargeFileCategoryDto, LargeFileDeleteFailureDto, LargeFileScanProgressDto,
    LargeFileScanStatusResponse,
};

fn test_app() -> App {
    App::new().0
}

fn entry(path: &str, size_bytes: u64) -> LargeFileEntry {
    LargeFileEntry {
        name: path.rsplit('/').next().unwrap_or(path).to_string(),
        path: path.to_string(),
        parent: "/tmp".to_string(),
        size_bytes,
    }
}

fn category(id: &str, files: Vec<LargeFileEntry>) -> LargeFileCategoryDto {
    LargeFileCategoryDto {
        id: id.to_string(),
        title: id.to_string(),
        subtitle: String::new(),
        total_bytes: files.iter().map(|file| file.size_bytes).sum(),
        files,
    }
}

fn report() -> LargeFileScanReport {
    let categories = vec![
        category("huge", vec![entry("/tmp/a.bin", 1024), entry("/tmp/b.bin", 2048)]),
        category("medium", vec![entry("/tmp/c.bin", 4096)]),
    ];
    LargeFileScanReport {
        root: "/tmp".to_string(),
        total_bytes: categories.iter().map(|item| item.total_bytes).sum(),
        total_files: categories.iter().map(|item| item.files.len()).sum(),
        categories,
    }
}

#[test]
fn format_bytes_uses_expected_units() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
}

#[test]
fn report_helpers_filter_and_recompute_deleted_paths() {
    let mut report = report();

    assert_eq!(visible_paths(&report, "all"), vec!["/tmp/a.bin", "/tmp/b.bin", "/tmp/c.bin"]);
    assert_eq!(visible_paths(&report, "medium"), vec!["/tmp/c.bin"]);
    assert!(visible_paths(&report, "missing").is_empty());

    remove_deleted_paths(
        &mut report,
        &HashSet::from(["/tmp/a.bin".to_string(), "/tmp/c.bin".to_string()]),
    );

    assert_eq!(report.total_files, 1);
    assert_eq!(report.total_bytes, 2048);
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].files[0].path, "/tmp/b.bin");
}

#[test]
fn progress_helpers_sync_mutex_state_into_app_fields() {
    let mut app = test_app();
    let progress = LargeFileScanProgress {
        phase_label: "扫描中".to_string(),
        current_path: "/tmp/a.bin".to_string(),
        total_files: 10,
        processed_files: 4,
        matched_files: 2,
        progress_value: 0.4,
    };

    set_progress(&app.large_file_progress_state, progress.clone());
    sync_progress_from_state(&mut app);

    assert_eq!(app.large_file_progress_label, progress.phase_label);
    assert_eq!(app.large_file_current_path, progress.current_path);
    assert_eq!(app.large_file_total_files, 10);
    assert_eq!(app.large_file_processed_files, 4);
    assert_eq!(app.large_file_progress_value, 0.4);

    apply_gateway_progress(
        &mut app,
        &LargeFileScanProgressDto {
            phase_label: "完成".to_string(),
            current_path: "/tmp/b.bin".to_string(),
            total_files: 12,
            processed_files: 12,
            matched_files: 3,
            progress_value: 1.0,
        },
    );
    assert_eq!(app.large_file_progress_label, "完成");
    assert_eq!(app.large_file_total_files, 12);
}

#[test]
fn update_handles_inputs_selection_and_scan_completion() {
    let mut app = test_app();

    let _ = update(&mut app, LargeFileToolMessage::RootChanged("/data".to_string()));
    let _ = update(&mut app, LargeFileToolMessage::RootPicked(Some(" /ignored ".to_string())));
    let _ = update(&mut app, LargeFileToolMessage::RootPicked(Some("/picked".to_string())));
    let _ = update(&mut app, LargeFileToolMessage::SelectFilter("medium".to_string()));
    assert_eq!(app.large_file_root, "/picked");
    assert_eq!(app.large_file_active_filter, "medium");

    app.large_file_report = Some(report());
    let _ = update(&mut app, LargeFileToolMessage::SelectVisibleEntries);
    assert_eq!(app.large_file_selected_entries, HashSet::from(["/tmp/c.bin".to_string()]));

    let _ = update(
        &mut app,
        LargeFileToolMessage::ToggleEntrySelection {
            path: "/tmp/a.bin".to_string(),
            selected: true,
        },
    );
    assert!(app.large_file_selected_entries.contains("/tmp/a.bin"));
    let _ = update(
        &mut app,
        LargeFileToolMessage::ToggleEntrySelection {
            path: "/tmp/a.bin".to_string(),
            selected: false,
        },
    );
    assert!(!app.large_file_selected_entries.contains("/tmp/a.bin"));

    let _ = update(&mut app, LargeFileToolMessage::ClearSelection);
    assert!(app.large_file_selected_entries.is_empty());

    app.large_file_scanning = true;
    let _ = update(&mut app, LargeFileToolMessage::ScanFinished(Ok(report())));
    assert!(app.large_file_scanned);
    assert!(!app.large_file_scanning);
    assert_eq!(app.large_file_progress_label, "扫描完成");
    assert!(
        app.large_file_notification.as_deref().unwrap_or_default().contains("共发现 3 个大文件")
    );

    app.large_file_scanning = true;
    let _ = update(&mut app, LargeFileToolMessage::ScanFinished(Err("已取消".to_string())));
    assert!(!app.large_file_scanned);
    assert_eq!(app.large_file_progress_label, "已取消");
}

#[test]
fn update_handles_status_and_delete_results() {
    let mut app = test_app();
    app.large_file_scanning = true;

    let _ = update(
        &mut app,
        LargeFileToolMessage::ScanStatusLoaded(Ok(LargeFileScanStatusResponse {
            job_id: "job".to_string(),
            progress: LargeFileScanProgressDto {
                phase_label: "阶段".to_string(),
                current_path: "/tmp/a.bin".to_string(),
                total_files: 2,
                processed_files: 1,
                matched_files: 1,
                progress_value: 0.5,
            },
            finished: false,
            report: None,
            error: None,
        })),
    );
    assert_eq!(app.large_file_progress_label, "阶段");
    assert!(app.large_file_scanning);

    let _ = update(
        &mut app,
        LargeFileToolMessage::ScanStatusLoaded(Ok(LargeFileScanStatusResponse {
            job_id: "job".to_string(),
            progress: LargeFileScanProgressDto {
                phase_label: "结束".to_string(),
                current_path: String::new(),
                total_files: 0,
                processed_files: 0,
                matched_files: 0,
                progress_value: 1.0,
            },
            finished: true,
            report: None,
            error: None,
        })),
    );
    assert!(!app.large_file_scanning);
    assert_eq!(app.large_file_notification.as_deref(), Some("扫描任务结束但未返回结果"));

    app.large_file_report = Some(report());
    app.large_file_selected_entries =
        HashSet::from(["/tmp/a.bin".to_string(), "/tmp/c.bin".to_string()]);
    let _ = update(
        &mut app,
        LargeFileToolMessage::DeleteFinished(Ok(LargeFileDeleteSummary {
            deleted_paths: vec!["/tmp/a.bin".to_string()],
            failed_paths: vec![LargeFileDeleteFailureDto {
                path: "/tmp/c.bin".to_string(),
                error: "denied".to_string(),
            }],
        })),
    );
    assert!(!app.large_file_selected_entries.contains("/tmp/a.bin"));
    assert!(app.large_file_selected_entries.contains("/tmp/c.bin"));
    assert_eq!(app.large_file_report.as_ref().map(|item| item.total_files), Some(2));
    assert_eq!(app.large_file_notification.as_deref(), Some("已删除 1 个文件，1 个删除失败"));

    let _ =
        update(&mut app, LargeFileToolMessage::DeleteFinished(Err("delete failed".to_string())));
    assert_eq!(app.large_file_notification.as_deref(), Some("delete failed"));
}

#[test]
fn update_guards_busy_delete_tick_and_clear_notification() {
    let mut app = test_app();

    let _ = update(&mut app, LargeFileToolMessage::DeleteSelected);
    assert_eq!(app.large_file_notification.as_deref(), Some("请先勾选要删除的文件"));

    let _ = update(&mut app, LargeFileToolMessage::ClearNotification);
    assert!(app.large_file_notification.is_none());

    app.large_file_scanning = false;
    let _ = update(&mut app, LargeFileToolMessage::Tick);
    assert_eq!(app.large_file_animation_frame, 0);

    assert!(!normalized_root(" /x ").is_empty());

    app.large_file_scanning = true;
    app.large_file_progress_state.lock().expect("progress lock").phase_label = "tick".to_string();
    let _ = update(&mut app, LargeFileToolMessage::Tick);
    assert_eq!(app.large_file_animation_frame, 1);
    assert_eq!(app.large_file_progress_label, "tick");

    let _ = update(&mut app, LargeFileToolMessage::CancelScan);
    assert_eq!(app.large_file_progress_label, "正在取消");
    assert!(app.large_file_cancel_flag.load(std::sync::atomic::Ordering::Relaxed));

    let _ = update(&mut app, LargeFileToolMessage::ScanStarted(Ok("job-1".to_string())));
    assert_eq!(app.large_file_scan_job_id.as_deref(), Some("job-1"));
}
