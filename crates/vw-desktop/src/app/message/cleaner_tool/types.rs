//! 定义清理工具在扫描、执行和界面展示之间传递的领域类型。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use iced::widget::text_editor;
pub use vw_gateway_client::{
    CleanerCleanupRequest, CleanerScanDetail, CleanerScanGroup, CleanerScanItem, CleanerScanReport,
    CleanerStatusResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// CleanerPlatform 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum CleanerPlatform {
    MacOs,
    Windows,
}

#[derive(Debug, Clone)]
/// CleanerToolMessage 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum CleanerToolMessage {
    EditorAction(text_editor::Action),
    ToggleSystemTemp(bool),
    ToggleAppCache(bool),
    ToggleLogs(bool),
    TogglePackageCache(bool),
    ToggleDownloads(bool),
    ToggleTrash(bool),
    ToggleInstallers(bool),
    ToggleOtherApps(bool),
    ToggleWeChatWork(bool),
    ToggleWeChat(bool),
    ToggleQq(bool),
    ToggleDingTalk(bool),
    ToggleFeishu(bool),
    ToggleSafari(bool),
    ToggleChrome(bool),
    ToggleEdge(bool),
    ToggleFirefox(bool),
    ToggleMail(bool),
    InfoLoaded(Result<vw_gateway_client::CleanerInfoResponse, String>),
    Scan,
    ScanFinished(Result<CleanerScanReport, String>),
    ToggleTreeNode(String),
    Run,
    Cancel,
    CancelFinished(Result<(), String>),
    StatusLoaded(Result<CleanerStatusResponse, String>),
    RunFinished(Result<String, String>),
    Tick,
    Clear,
    ClearNotification,
}

/// current_platform 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub fn current_platform() -> Option<CleanerPlatform> {
    if cfg!(target_os = "macos") {
        Some(CleanerPlatform::MacOs)
    } else if cfg!(target_os = "windows") {
        Some(CleanerPlatform::Windows)
    } else {
        None
    }
}

/// format_bytes 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

#[allow(dead_code)]
/// unsupported_platform_message 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn unsupported_platform_message() -> String {
    [
        "当前系统暂不支持该清理工具。",
        "目前仅为 macOS 和 Windows 内置了直接清理逻辑。",
        "如需支持 Linux，可继续扩展一套对应执行策略。",
    ]
    .join("\n")
}
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
