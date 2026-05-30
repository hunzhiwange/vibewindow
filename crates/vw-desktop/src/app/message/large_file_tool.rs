//! 处理大文件查看工具的加载、分页和界面状态更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::{App, Message};
use iced::Task;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const LARGE_FILE_MIN_BYTES: u64 = 50 * 1024 * 1024;
const LARGE_FILE_TICK_MS: u64 = 120;
const LARGE_FILE_NOTIFICATION_SECS: u64 = 3;
const ONE_GB: u64 = 1024 * 1024 * 1024;
const FIVE_HUNDRED_MB: u64 = 500 * 1024 * 1024;
const ONE_HUNDRED_MB: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone)]
/// LargeFileScanReport 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub struct LargeFileScanReport {
    pub root: String,
    pub total_bytes: u64,
    pub total_files: usize,
    pub categories: Vec<LargeFileCategory>,
}

#[derive(Debug, Clone)]
/// LargeFileCategory 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub struct LargeFileCategory {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub total_bytes: u64,
    pub files: Vec<LargeFileEntry>,
}

#[derive(Debug, Clone)]
/// LargeFileEntry 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub struct LargeFileEntry {
    pub name: String,
    pub path: String,
    pub parent: String,
    pub size_bytes: u64,
}

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

#[derive(Debug, Clone)]
/// LargeFileDeleteSummary 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub struct LargeFileDeleteSummary {
    pub deleted_paths: Vec<String>,
    pub failed_paths: Vec<(String, String)>,
}

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
    let cancel_flag = app.large_file_cancel_flag.clone();
    app.large_file_root = root.clone();
    app.large_file_scanning = true;
    app.large_file_scanned = false;
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
            phase_label: "准备扫描".to_string(),
            current_path: root.clone(),
            total_files: 0,
            processed_files: 0,
            matched_files: 0,
            progress_value: 0.0,
        },
    );

    Task::batch(vec![
        Task::perform(
            async move {
                crate::app::message::spawn_blocking_opt(move || {
                    Some(scan_large_files(root, progress_state, cancel_flag))
                })
                .await
                .unwrap_or_else(|| Err("扫描任务执行失败或被取消".to_string()))
            },
            |result| Message::LargeFileTool(LargeFileToolMessage::ScanFinished(result)),
        ),
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
    Task::none()
}

fn finish_scan(app: &mut App, result: Result<LargeFileScanReport, String>) -> Task<Message> {
    sync_progress_from_state(app);
    app.large_file_scanning = false;
    app.large_file_animation_frame = 0;
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
    app.large_file_deleting = true;
    app.large_file_notification = Some(format!("正在删除 {} 个文件…", selected_paths.len()));

    Task::perform(
        async move {
            crate::app::message::spawn_blocking_opt(move || {
                Some(delete_selected_files(selected_paths))
            })
            .await
            .unwrap_or_else(|| Err("删除任务执行失败或被取消".to_string()))
        },
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
    crate::app::message::after(
        std::time::Duration::from_millis(LARGE_FILE_TICK_MS),
        Message::LargeFileTool(LargeFileToolMessage::Tick),
    )
}

fn normalized_root(root: &str) -> String {
    if !root.is_empty() {
        return root.to_string();
    }

    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
}

fn scan_large_files(
    root: String,
    progress_state: Arc<Mutex<LargeFileScanProgress>>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<LargeFileScanReport, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        return Err("扫描目录不存在".to_string());
    }
    if !root_path.is_dir() {
        return Err("扫描目标不是目录".to_string());
    }

    update_phase(&progress_state, "预扫描目录结构", root.clone(), 0, 0, 0, 0.02);

    let mut file_paths = Vec::new();
    let mut seen_files = 0_usize;

    for entry in
        walkdir::WalkDir::new(&root_path).follow_links(false).into_iter().filter_map(Result::ok)
    {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("已取消扫描".to_string());
        }

        let entry_path = entry.path().to_string_lossy().to_string();
        if entry.file_type().is_file() {
            file_paths.push(entry.path().to_path_buf());
            seen_files += 1;
        }

        if seen_files.is_multiple_of(64) || !entry.file_type().is_file() {
            let pulse = 0.02 + ((seen_files % 240) as f32 / 240.0) * 0.18;
            update_phase(
                &progress_state,
                "预扫描目录结构",
                entry_path,
                seen_files,
                seen_files,
                0,
                pulse.min(0.20),
            );
        }

        if !entry.file_type().is_file() {
            continue;
        }
    }

    let total_candidates = file_paths.len();
    update_phase(
        &progress_state,
        "并行扫描文件大小",
        root.clone(),
        0,
        total_candidates,
        0,
        if total_candidates == 0 { 1.0 } else { 0.20 },
    );

    if total_candidates == 0 {
        return Ok(LargeFileScanReport {
            root,
            total_bytes: 0,
            total_files: 0,
            categories: Vec::new(),
        });
    }

    let worker_count =
        std::thread::available_parallelism().map(|count| count.get()).unwrap_or(4).clamp(1, 8);
    let chunk_size = total_candidates.div_ceil(worker_count).max(1);
    let mut handles = Vec::new();

    for chunk in file_paths.chunks(chunk_size) {
        let files = chunk.to_vec();
        let progress_state = progress_state.clone();
        let cancel_flag = cancel_flag.clone();
        handles.push(std::thread::spawn(move || scan_chunk(files, progress_state, cancel_flag)));
    }

    let mut giga_files = Vec::new();
    let mut large_files = Vec::new();
    let mut medium_files = Vec::new();
    let mut small_files = Vec::new();
    let mut total_bytes = 0_u64;
    let mut total_files = 0_usize;

    for handle in handles {
        let worker_result = handle.join().map_err(|_| "扫描线程异常退出".to_string())??;
        total_bytes = total_bytes.saturating_add(worker_result.total_bytes);
        total_files += worker_result.total_files;
        giga_files.extend(worker_result.giga_files);
        large_files.extend(worker_result.large_files);
        medium_files.extend(worker_result.medium_files);
        small_files.extend(worker_result.small_files);
    }

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("已取消扫描".to_string());
    }

    update_phase(
        &progress_state,
        "整理结果",
        root.clone(),
        total_candidates,
        total_candidates,
        total_files,
        0.98,
    );

    let mut categories = vec![
        build_category("giga", "1GB 以上", "优先检查虚拟机镜像、素材包、数据库快照", giga_files),
        build_category(
            "500m",
            "500MB - 1GB",
            "通常是安装包、视频缓存、训练数据或构建产物",
            large_files,
        ),
        build_category("100m", "100MB - 500MB", "常见于导出文件、依赖缓存、下载目录", medium_files),
        build_category("50m", "50MB - 100MB", "适合作为首轮整理补充项", small_files),
    ];
    categories.retain(|category| !category.files.is_empty());

    update_phase(
        &progress_state,
        "扫描完成",
        root.clone(),
        total_candidates,
        total_candidates,
        total_files,
        1.0,
    );

    Ok(LargeFileScanReport { root, total_bytes, total_files, categories })
}

fn scan_chunk(
    files: Vec<PathBuf>,
    progress_state: Arc<Mutex<LargeFileScanProgress>>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<WorkerScanResult, String> {
    let mut result = WorkerScanResult::default();

    for path in files {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("已取消扫描".to_string());
        }

        let path_display = path.to_string_lossy().to_string();
        let entry = classify_large_file(&path)?;
        let matched = entry.is_some();
        advance_processing_progress(&progress_state, path_display, matched);

        if let Some(entry) = entry {
            result.total_bytes = result.total_bytes.saturating_add(entry.size_bytes);
            result.total_files += 1;

            if entry.size_bytes >= ONE_GB {
                result.giga_files.push(entry);
            } else if entry.size_bytes >= FIVE_HUNDRED_MB {
                result.large_files.push(entry);
            } else if entry.size_bytes >= ONE_HUNDRED_MB {
                result.medium_files.push(entry);
            } else {
                result.small_files.push(entry);
            }
        }
    }

    Ok(result)
}

fn build_category(
    id: &str,
    title: &str,
    subtitle: &str,
    mut files: Vec<LargeFileEntry>,
) -> LargeFileCategory {
    files.sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes));
    let total_bytes = files.iter().map(|file| file.size_bytes).sum();

    LargeFileCategory {
        id: id.to_string(),
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        total_bytes,
        files,
    }
}

fn classify_large_file(path: &Path) -> Result<Option<LargeFileEntry>, String> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    if !metadata.is_file() {
        return Ok(None);
    }

    let size_bytes = metadata.len();
    if size_bytes < LARGE_FILE_MIN_BYTES {
        return Ok(None);
    }

    Ok(Some(LargeFileEntry {
        name: path.file_name().and_then(|name| name.to_str()).unwrap_or("未知文件").to_string(),
        path: path.to_string_lossy().to_string(),
        parent: path.parent().unwrap_or_else(|| Path::new("")).to_string_lossy().to_string(),
        size_bytes,
    }))
}

fn delete_selected_files(paths: Vec<String>) -> Result<LargeFileDeleteSummary, String> {
    let mut deleted_paths = Vec::new();
    let mut failed_paths = Vec::new();

    for path in paths {
        match std::fs::remove_file(&path) {
            Ok(_) => deleted_paths.push(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => deleted_paths.push(path),
            Err(error) => failed_paths.push((path, error.to_string())),
        }
    }

    Ok(LargeFileDeleteSummary { deleted_paths, failed_paths })
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

fn update_phase(
    progress_state: &Arc<Mutex<LargeFileScanProgress>>,
    phase_label: &str,
    current_path: String,
    processed_files: usize,
    total_files: usize,
    matched_files: usize,
    progress_value: f32,
) {
    if let Ok(mut state) = progress_state.lock() {
        state.phase_label = phase_label.to_string();
        state.current_path = current_path;
        state.processed_files = processed_files;
        state.total_files = total_files;
        state.matched_files = matched_files;
        state.progress_value = progress_value.clamp(0.0, 1.0);
    }
}

fn advance_processing_progress(
    progress_state: &Arc<Mutex<LargeFileScanProgress>>,
    current_path: String,
    matched: bool,
) {
    if let Ok(mut state) = progress_state.lock() {
        state.phase_label = "并行扫描文件大小".to_string();
        state.current_path = current_path;
        state.processed_files += 1;
        if matched {
            state.matched_files += 1;
        }
        let total = state.total_files.max(1) as f32;
        state.progress_value = (0.2 + (state.processed_files as f32 / total) * 0.8).clamp(0.2, 1.0);
    }
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

#[derive(Default)]
struct WorkerScanResult {
    total_bytes: u64,
    total_files: usize,
    giga_files: Vec<LargeFileEntry>,
    large_files: Vec<LargeFileEntry>,
    medium_files: Vec<LargeFileEntry>,
    small_files: Vec<LargeFileEntry>,
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
