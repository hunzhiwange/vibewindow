//! 承接系统清理工具的执行阶段，按平台分派清理命令并汇总清理结果。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

#[cfg(not(target_arch = "wasm32"))]
use super::fs::{covers_target, expand_env_path, measure_cleanup_target};
#[cfg(not(target_arch = "wasm32"))]
use super::scan::ScanDetailKind;
use super::types::CleanupRequest;
#[cfg(not(target_arch = "wasm32"))]
use super::types::{CleanerPlatform, current_platform, format_bytes, unsupported_platform_message};

#[cfg(not(target_arch = "wasm32"))]
#[path = "run_macos.rs"]
mod macos;
#[cfg(not(target_arch = "wasm32"))]
#[path = "run_windows.rs"]
mod windows;

#[cfg(not(target_arch = "wasm32"))]
use self::{macos::run_macos_cleanup, windows::run_windows_cleanup};

#[cfg(target_arch = "wasm32")]
/// execute_cleanup 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn execute_cleanup(
    _request: CleanupRequest,
    _cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Option<Result<String, String>> {
    Some(Err("Web 平台暂不支持直接执行系统清理。".to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
/// execute_cleanup 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn execute_cleanup(
    request: CleanupRequest,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Option<Result<String, String>> {
    Some(match current_platform() {
        Some(CleanerPlatform::MacOs) => run_macos_cleanup(&request, &cancel_flag),
        Some(CleanerPlatform::Windows) => run_windows_cleanup(&request, &cancel_flag),
        None => Err(unsupported_platform_message()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Default)]
struct CleanupStats {
    targets: Vec<CleanupTarget>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CleanupStats {
    fn track_directory(&mut self, raw_path: &'static str) {
        self.track(raw_path, ScanDetailKind::Directory);
    }

    fn track_matching_files(
        &mut self,
        raw_path: &'static str,
        extensions: &'static [&'static str],
    ) {
        self.track(raw_path, ScanDetailKind::FileExtensions(extensions));
    }

    fn summary_line(&self) -> String {
        format!("本次预计清理垃圾数据：{}", format_bytes(self.expected_removed()))
    }

    fn actual_removed_line(&self) -> String {
        format!("本次实际删除垃圾数据：{}", format_bytes(self.actual_removed()))
    }

    fn expected_removed(&self) -> u64 {
        self.targets.iter().map(|target| target.before_bytes).sum()
    }

    fn actual_removed(&self) -> u64 {
        self.targets
            .iter()
            .map(|target| {
                let after_bytes = measure_cleanup_target(&target.path, target.kind);
                target.before_bytes.saturating_sub(after_bytes)
            })
            .sum()
    }

    fn track(&mut self, raw_path: &'static str, kind: ScanDetailKind) {
        let path = std::path::PathBuf::from(expand_env_path(raw_path));
        if self.targets.iter().any(|target| target.covers(&path, kind)) {
            return;
        }
        self.targets.retain(|target| !covers_target(&path, kind, &target.path, target.kind));
        let before_bytes = measure_cleanup_target(&path, kind);
        self.targets.push(CleanupTarget { path, kind, before_bytes });
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
struct CleanupTarget {
    path: std::path::PathBuf,
    kind: ScanDetailKind,
    before_bytes: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl CleanupTarget {
    fn covers(&self, other_path: &std::path::Path, other_kind: ScanDetailKind) -> bool {
        covers_target(&self.path, self.kind, other_path, other_kind)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn command_output(label: &str, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = if output.status.success() { "成功" } else { "失败" };
    let mut lines = vec![format!("{label}: {status}")];
    if !stdout.is_empty() {
        lines.push(format!("stdout:\n{stdout}"));
    }
    if !stderr.is_empty() {
        lines.push(format!("stderr:\n{stderr}"));
    }
    lines.join("\n")
}

#[cfg(not(target_arch = "wasm32"))]
fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(not(target_arch = "wasm32"))]
fn quote_for_single_argument(input: &str) -> String {
    format!("'{}'", input.replace('\'', "''"))
}

#[cfg(not(target_arch = "wasm32"))]
fn cancelled(
    cancel_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    log: &mut Vec<String>,
) -> bool {
    use std::sync::atomic::Ordering;

    if cancel_flag.load(Ordering::Relaxed) {
        log.push("清理任务已取消，已停止后续步骤。".to_string());
        true
    } else {
        false
    }
}
#[cfg(test)]
#[path = "run_tests.rs"]
mod run_tests;
