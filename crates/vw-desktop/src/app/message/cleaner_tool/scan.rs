//! 承接系统清理工具的扫描阶段，发现可清理目标并生成面向界面的扫描明细。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

#[cfg(not(target_arch = "wasm32"))]
use super::fs::{directory_size, matching_file_size};
use super::types::CleanerScanReport;
#[cfg(not(target_arch = "wasm32"))]
use super::types::{
    CleanerPlatform, CleanerScanDetail, CleanerScanGroup, CleanerScanItem, current_platform,
    unsupported_platform_message,
};

#[cfg(not(target_arch = "wasm32"))]
#[path = "scan_macos.rs"]
mod macos;
#[cfg(not(target_arch = "wasm32"))]
#[path = "scan_windows.rs"]
mod windows;

#[cfg(not(target_arch = "wasm32"))]
use self::{macos::macos_scan_blueprints, windows::windows_scan_blueprints};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// ScanDetailKind 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub(super) enum ScanDetailKind {
    Directory,
    FileExtensions(&'static [&'static str]),
}

#[cfg(not(target_arch = "wasm32"))]
/// ScanGroupBlueprint 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub(super) struct ScanGroupBlueprint {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) items: Vec<ScanItemBlueprint>,
}

#[cfg(not(target_arch = "wasm32"))]
/// ScanItemBlueprint 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub(super) struct ScanItemBlueprint {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) sensitive: bool,
    pub(super) details: Vec<ScanDetailBlueprint>,
}

#[cfg(not(target_arch = "wasm32"))]
/// ScanDetailBlueprint 保存该流程中跨函数传递的结构化数据。
///
/// 使用具名字段保留领域含义，避免在消息链路中传递松散的动态数据。
pub(super) struct ScanDetailBlueprint {
    pub(super) label: &'static str,
    pub(super) path: &'static str,
    pub(super) kind: ScanDetailKind,
}

#[cfg(target_arch = "wasm32")]
/// scan_cleanup_targets 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn scan_cleanup_targets() -> Option<Result<CleanerScanReport, String>> {
    Some(Err("Web 平台暂不支持扫描系统垃圾。".to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
/// scan_cleanup_targets 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn scan_cleanup_targets() -> Option<Result<CleanerScanReport, String>> {
    Some(match current_platform() {
        Some(CleanerPlatform::MacOs) => Ok(scan_platform_groups(macos_scan_blueprints())),
        Some(CleanerPlatform::Windows) => Ok(scan_platform_groups(windows_scan_blueprints())),
        None => Err(unsupported_platform_message()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
/// scan_dir 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn scan_dir(label: &'static str, path: &'static str) -> ScanDetailBlueprint {
    ScanDetailBlueprint { label, path, kind: ScanDetailKind::Directory }
}

#[cfg(not(target_arch = "wasm32"))]
/// scan_files 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn scan_files(
    label: &'static str,
    path: &'static str,
    extensions: &'static [&'static str],
) -> ScanDetailBlueprint {
    ScanDetailBlueprint { label, path, kind: ScanDetailKind::FileExtensions(extensions) }
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_platform_groups(blueprints: Vec<ScanGroupBlueprint>) -> CleanerScanReport {
    let mut total_bytes = 0u64;
    let mut matched_items = 0usize;
    let mut groups = Vec::new();

    for group in blueprints {
        let mut group_total = 0u64;
        let mut items = Vec::new();

        for item in group.items {
            let mut item_total = 0u64;
            let mut details = Vec::new();

            for detail in item.details {
                let detail_bytes = scan_detail_bytes(&detail);
                item_total = item_total.saturating_add(detail_bytes);
                details.push(CleanerScanDetail {
                    label: detail.label.to_string(),
                    path: detail.path.to_string(),
                    total_bytes: detail_bytes,
                });
            }

            if item_total > 0 {
                matched_items += 1;
            }

            group_total = group_total.saturating_add(item_total);
            items.push(CleanerScanItem {
                id: item.id.to_string(),
                title: item.title.to_string(),
                subtitle: item.subtitle.to_string(),
                sensitive: item.sensitive,
                total_bytes: item_total,
                details,
            });
        }

        total_bytes = total_bytes.saturating_add(group_total);
        groups.push(CleanerScanGroup {
            id: group.id.to_string(),
            title: group.title.to_string(),
            subtitle: group.subtitle.to_string(),
            total_bytes: group_total,
            items,
        });
    }

    CleanerScanReport { total_bytes, matched_items, groups }
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_detail_bytes(detail: &ScanDetailBlueprint) -> u64 {
    match detail.kind {
        ScanDetailKind::Directory => directory_size(detail.path),
        ScanDetailKind::FileExtensions(extensions) => matching_file_size(detail.path, extensions),
    }
}
#[cfg(test)]
#[path = "scan_tests.rs"]
mod scan_tests;
