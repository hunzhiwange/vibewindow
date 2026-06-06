//! 处理大文件查看工具的加载、分页和界面状态更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::{App, Message};
use iced::Task;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use vw_gateway_client::vw_api_types::file::{
    LargeFileCategoryDto, LargeFileDeleteRequest, LargeFileDeleteResponse, LargeFileEntryDto,
    LargeFileScanCancelRequest, LargeFileScanProgressDto, LargeFileScanResponse,
    LargeFileScanStartRequest, LargeFileScanStatusResponse,
};

const LARGE_FILE_TICK_MS: u64 = 120;
const LARGE_FILE_NOTIFICATION_SECS: u64 = 3;

pub type LargeFileScanReport = LargeFileScanResponse;
pub type LargeFileCategory = LargeFileCategoryDto;
pub type LargeFileEntry = LargeFileEntryDto;

#[derive(Debug, Clone)]
/// LargeFileScanProgress 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub struct LargeFileScanProgress {
    pub phase_label: String,
    pub current_path: String,
    pub total_files: usize,
    pub processed_files: usize,
    pub matched_files: usize,
    pub progress_value: f32,
}

impl Default for LargeFileScanProgress {
    fn default() -> Self {
        Self {
            phase_label: "等待扫描".to_string(),
            current_path: String::new(),
            total_files: 0,
            processed_files: 0,
            matched_files: 0,
            progress_value: 0.0,
        }
    }
}

pub type LargeFileDeleteSummary = LargeFileDeleteResponse;

#[derive(Debug, Clone)]
/// LargeFileToolMessage 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum LargeFileToolMessage {
    RootChanged(String),
    PickRoot,
    RootPicked(Option<String>),
    Scan,
    CancelScan,
    ScanStarted(Result<String, String>),
    ScanStatusLoaded(Result<LargeFileScanStatusResponse, String>),
    ScanFinished(Result<LargeFileScanReport, String>),
    SelectFilter(String),
    ToggleEntrySelection { path: String, selected: bool },
    SelectVisibleEntries,
    ClearSelection,
    DeleteSelected,
    DeleteFinished(Result<LargeFileDeleteSummary, String>),
    Tick,
    ClearNotification,
}

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub fn update(app: &mut App, message: LargeFileToolMessage) -> Task<Message> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        LargeFileToolMessage::RootChanged(value) => {
            app.large_file_root = value;
            Task::none()
        }
        LargeFileToolMessage::PickRoot => pick_root(),
        LargeFileToolMessage::RootPicked(path) => {
            if let Some(path) = path.filter(|value| !value.trim().is_empty()) {
                app.large_file_root = path;
            }
            Task::none()
        }
        LargeFileToolMessage::Scan => start_scan(app),
        LargeFileToolMessage::CancelScan => cancel_scan(app),
        LargeFileToolMessage::ScanStarted(result) => scan_started(app, result),
        LargeFileToolMessage::ScanStatusLoaded(result) => scan_status_loaded(app, result),
        LargeFileToolMessage::ScanFinished(result) => finish_scan(app, result),
        LargeFileToolMessage::SelectFilter(filter) => {
            app.large_file_active_filter = filter;
            Task::none()
        }
        LargeFileToolMessage::ToggleEntrySelection { path, selected } => {
            if selected {
                app.large_file_selected_entries.insert(path);
            } else {
                app.large_file_selected_entries.remove(&path);
            }
            Task::none()
        }
        LargeFileToolMessage::SelectVisibleEntries => {
            if let Some(report) = &app.large_file_report {
                for path in visible_paths(report, &app.large_file_active_filter) {
                    app.large_file_selected_entries.insert(path);
                }
            }
            Task::none()
        }
        LargeFileToolMessage::ClearSelection => {
            app.large_file_selected_entries.clear();
            Task::none()
        }
        LargeFileToolMessage::DeleteSelected => start_delete_selected(app),
        LargeFileToolMessage::DeleteFinished(result) => finish_delete_selected(app, result),
        LargeFileToolMessage::Tick => tick(app),
        LargeFileToolMessage::ClearNotification => {
            app.large_file_notification = None;
            Task::none()
        }
    }
}

fn pick_root() -> Task<Message> {
    // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
    Task::perform(
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                None
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                rfd::FileDialog::new().pick_folder().map(|path| path.to_string_lossy().to_string())
            }
        },
        |path| Message::LargeFileTool(LargeFileToolMessage::RootPicked(path)),
    )
}

fn start_scan(app: &mut App) -> Task<Message> {
    if app.large_file_scanning || app.large_file_deleting {
        return Task::none();
    }

    let root = normalized_root(app.large_file_root.trim());
    let progress_state = app.large_file_progress_state.clone();
    app.large_file_root = root.clone();
    app.large_file_scanning = true;
    app.large_file_scanned = false;
    app.large_file_scan_job_id = None;
    app.large_file_animation_frame = 0;
    app.large_file_report = None;
    app.large_file_selected_entries.clear();
    app.large_file_progress_label = "准备扫描".to_string();
    app.large_file_current_path = root.clone();
    app.large_file_progress_value = 0.0;
    app.large_file_processed_files = 0;
    app.large_file_total_files = 0;
    app.large_file_notification = Some("正在扫描 50MB 以上的大文件…".to_string());
    app.large_file_cancel_flag.store(false, Ordering::Relaxed);
    set_progress(
        &progress_state,
        LargeFileScanProgress {
            phase_label: "准备请求网关扫描".to_string(),
            current_path: root.clone(),
            total_files: 0,
            processed_files: 0,
            matched_files: 0,
            progress_value: 0.0,
        },
    );

    Task::batch(vec![
        Task::perform(async move { start_large_file_scan_via_gateway(root).await }, |result| {
            Message::LargeFileTool(LargeFileToolMessage::ScanStarted(result))
        }),
        crate::app::message::after(
            std::time::Duration::from_millis(LARGE_FILE_TICK_MS),
            Message::LargeFileTool(LargeFileToolMessage::Tick),
        ),
    ])
}

fn cancel_scan(app: &mut App) -> Task<Message> {
    if !app.large_file_scanning {
        return Task::none();
    }

    app.large_file_notification = Some("正在取消扫描，请稍候…".to_string());
    app.large_file_progress_label = "正在取消".to_string();
    app.large_file_cancel_flag.store(true, Ordering::Relaxed);
    let Some(job_id) = app.large_file_scan_job_id.clone() else {
        return Task::none();
    };

    Task::perform(async move { cancel_large_file_scan_via_gateway(job_id).await }, |_| {
        Message::LargeFileTool(LargeFileToolMessage::Tick)
    })
}

fn scan_started(app: &mut App, result: Result<String, String>) -> Task<Message> {
    if !app.large_file_scanning {
        return Task::none();
    }

    match result {
        Ok(job_id) => {
            app.large_file_scan_job_id = Some(job_id);
            Task::none()
        }
        Err(error) => finish_scan(app, Err(error)),
    }
}

fn scan_status_loaded(
    app: &mut App,
    result: Result<LargeFileScanStatusResponse, String>,
) -> Task<Message> {
    if !app.large_file_scanning {
        return Task::none();
    }

    let status = match result {
        Ok(status) => status,
        Err(error) => return finish_scan(app, Err(error)),
    };

    apply_gateway_progress(app, &status.progress);

    if status.finished {
        return match (status.report, status.error) {
            (Some(report), _) => finish_scan(app, Ok(report)),
            (None, Some(error)) => finish_scan(app, Err(error)),
            (None, None) => finish_scan(app, Err("扫描任务结束但未返回结果".to_string())),
        };
    }

    crate::app::message::after(
        std::time::Duration::from_millis(LARGE_FILE_TICK_MS),
        Message::LargeFileTool(LargeFileToolMessage::Tick),
    )
}

fn finish_scan(app: &mut App, result: Result<LargeFileScanReport, String>) -> Task<Message> {
    sync_progress_from_state(app);
    app.large_file_scanning = false;
    app.large_file_animation_frame = 0;
    app.large_file_scan_job_id = None;
    app.large_file_cancel_flag.store(false, Ordering::Relaxed);

    match result {
        Ok(report) => {
            let total_files = report.total_files;
            let total_bytes = report.total_bytes;
            app.large_file_scanned = true;
            app.large_file_active_filter = "all".to_string();
            app.large_file_report = Some(report);
            app.large_file_progress_label = "扫描完成".to_string();
            app.large_file_progress_value = 1.0;
            app.large_file_notification = Some(format!(
                "扫描完成，共发现 {} 个大文件，合计 {}",
                total_files,
                format_bytes(total_bytes)
            ));
            clear_notification_task()
        }
        Err(error) => {
            app.large_file_scanned = false;
            app.large_file_report = None;
            if error.contains("已取消") {
                app.large_file_progress_label = "已取消".to_string();
            }
            app.large_file_notification = Some(error);
            clear_notification_task()
        }
    }
}

fn start_delete_selected(app: &mut App) -> Task<Message> {
    if app.large_file_scanning || app.large_file_deleting {
        return Task::none();
    }
    if app.large_file_selected_entries.is_empty() {
        app.large_file_notification = Some("请先勾选要删除的文件".to_string());
        return clear_notification_task();
    }

    let selected_paths = app.large_file_selected_entries.iter().cloned().collect::<Vec<_>>();
    let root = app
        .large_file_report
        .as_ref()
        .map(|report| report.root.clone())
        .unwrap_or_else(|| normalized_root(app.large_file_root.trim()));
    app.large_file_deleting = true;
    app.large_file_notification = Some(format!("正在删除 {} 个文件…", selected_paths.len()));

    Task::perform(
        async move { delete_selected_files_via_gateway(root, selected_paths).await },
        |result| Message::LargeFileTool(LargeFileToolMessage::DeleteFinished(result)),
    )
}

fn finish_delete_selected(
    app: &mut App,
    result: Result<LargeFileDeleteSummary, String>,
) -> Task<Message> {
    app.large_file_deleting = false;

    match result {
        Ok(summary) => {
            if summary.deleted_paths.is_empty() && summary.failed_paths.is_empty() {
                app.large_file_notification = Some("没有删除任何文件".to_string());
                return clear_notification_task();
            }

            let deleted_set = summary.deleted_paths.iter().cloned().collect::<HashSet<_>>();
            app.large_file_selected_entries.retain(|path| !deleted_set.contains(path));

            if let Some(report) = app.large_file_report.as_mut() {
                remove_deleted_paths(report, &deleted_set);
            }

            if summary.failed_paths.is_empty() {
                app.large_file_notification =
                    Some(format!("已删除 {} 个文件", summary.deleted_paths.len()));
            } else {
                app.large_file_notification = Some(format!(
                    "已删除 {} 个文件，{} 个删除失败",
                    summary.deleted_paths.len(),
                    summary.failed_paths.len()
                ));
            }
            clear_notification_task()
        }
        Err(error) => {
            app.large_file_notification = Some(error);
            clear_notification_task()
        }
    }
}

fn tick(app: &mut App) -> Task<Message> {
    if !app.large_file_scanning {
        return Task::none();
    }

    app.large_file_animation_frame = app.large_file_animation_frame.wrapping_add(1);
    sync_progress_from_state(app);
    let Some(job_id) = app.large_file_scan_job_id.clone() else {
        return crate::app::message::after(
            std::time::Duration::from_millis(LARGE_FILE_TICK_MS),
            Message::LargeFileTool(LargeFileToolMessage::Tick),
        );
    };

    Task::perform(async move { large_file_scan_status_via_gateway(job_id).await }, |result| {
        Message::LargeFileTool(LargeFileToolMessage::ScanStatusLoaded(result))
    })
}

fn normalized_root(root: &str) -> String {
    if !root.is_empty() {
        return root.to_string();
    }

    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
}

async fn start_large_file_scan_via_gateway(root: String) -> Result<String, String> {
    let client = crate::app::gateway_client()?;
    client
        .large_file_scan_start(&LargeFileScanStartRequest { root })
        .await
        .map(|response| response.job_id)
}

async fn large_file_scan_status_via_gateway(
    job_id: String,
) -> Result<LargeFileScanStatusResponse, String> {
    let client = crate::app::gateway_client()?;
    client.large_file_scan_status(&job_id).await
}

async fn cancel_large_file_scan_via_gateway(job_id: String) -> Result<(), String> {
    let client = crate::app::gateway_client()?;
    client.large_file_scan_cancel(&LargeFileScanCancelRequest { job_id }).await.map(|_| ())
}

async fn delete_selected_files_via_gateway(
    root: String,
    paths: Vec<String>,
) -> Result<LargeFileDeleteSummary, String> {
    let client = crate::app::gateway_client()?;
    client.large_file_delete(&LargeFileDeleteRequest { root, paths }).await
}

fn remove_deleted_paths(report: &mut LargeFileScanReport, deleted_paths: &HashSet<String>) {
    for category in &mut report.categories {
        category.files.retain(|file| !deleted_paths.contains(&file.path));
        category.total_bytes = category.files.iter().map(|file| file.size_bytes).sum();
    }
    report.categories.retain(|category| !category.files.is_empty());
    report.total_bytes = report.categories.iter().map(|category| category.total_bytes).sum();
    report.total_files = report.categories.iter().map(|category| category.files.len()).sum();
}

fn visible_paths(report: &LargeFileScanReport, filter: &str) -> Vec<String> {
    report
        .categories
        .iter()
        .filter(|category| filter == "all" || category.id == filter)
        .flat_map(|category| category.files.iter().map(|file| file.path.clone()))
        .collect()
}

fn set_progress(progress_state: &Arc<Mutex<LargeFileScanProgress>>, value: LargeFileScanProgress) {
    if let Ok(mut state) = progress_state.lock() {
        *state = value;
    }
}

fn apply_gateway_progress(app: &mut App, progress: &LargeFileScanProgressDto) {
    let value = LargeFileScanProgress {
        phase_label: progress.phase_label.clone(),
        current_path: progress.current_path.clone(),
        total_files: progress.total_files,
        processed_files: progress.processed_files,
        matched_files: progress.matched_files,
        progress_value: progress.progress_value,
    };
    set_progress(&app.large_file_progress_state, value);
    sync_progress_from_state(app);
}

fn sync_progress_from_state(app: &mut App) {
    if let Ok(state) = app.large_file_progress_state.lock() {
        app.large_file_progress_label = state.phase_label.clone();
        app.large_file_current_path = state.current_path.clone();
        app.large_file_progress_value = state.progress_value;
        app.large_file_processed_files = state.processed_files;
        app.large_file_total_files = state.total_files;
    }
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(LARGE_FILE_NOTIFICATION_SECS),
        Message::LargeFileTool(LargeFileToolMessage::ClearNotification),
    )
}

/// format_bytes 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0_usize;

    while value >= 1024.0 && unit_index + 1 < UNITS.len() {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}
