//! 维护 fs.rs 的桌面应用职责。
//! 本模块保持领域边界清晰，避免调用侧依赖内部实现细节。

#[cfg(not(target_arch = "wasm32"))]
use super::scan::ScanDetailKind;

/// 模块内可见函数，执行 directory_size 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn directory_size(raw_path: &str) -> u64 {
    let path = expand_env_path(raw_path);
    directory_size_path(std::path::Path::new(&path))
}

/// 模块内可见函数，执行 matching_file_size 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn matching_file_size(raw_path: &str, extensions: &[&str]) -> u64 {
    let path = expand_env_path(raw_path);
    matching_file_size_path(std::path::Path::new(&path), extensions)
}

/// 模块内可见函数，执行 measure_cleanup_target 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn measure_cleanup_target(path: &std::path::Path, kind: ScanDetailKind) -> u64 {
    match kind {
        ScanDetailKind::Directory => directory_size_path(path),
        ScanDetailKind::FileExtensions(extensions) => matching_file_size_path(path, extensions),
    }
}

/// 模块内可见函数，执行 covers_target 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn covers_target(
    base_path: &std::path::Path,
    base_kind: ScanDetailKind,
    other_path: &std::path::Path,
    other_kind: ScanDetailKind,
) -> bool {
    match base_kind {
        ScanDetailKind::Directory => base_path == other_path || other_path.starts_with(base_path),
        ScanDetailKind::FileExtensions(extensions) => {
            matches!(other_kind, ScanDetailKind::FileExtensions(other_extensions) if base_path == other_path && other_extensions == extensions)
        }
    }
}

/// 模块内可见函数，执行 expand_env_path 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn expand_env_path(raw_path: &str) -> String {
    let mut result = raw_path.to_string();

    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy();
        result = result.replace("$HOME", &home);
    }
    if let Some(tmpdir) = std::env::var_os("TMPDIR") {
        let tmpdir = tmpdir.to_string_lossy();
        result = result.replace("$TMPDIR", &tmpdir);
    }
    for (name, value) in std::env::vars() {
        let token_dollar = format!("${name}");
        if result.contains(&token_dollar) {
            result = result.replace(&token_dollar, &value);
        }
        let token_percent = format!("%{name}%");
        if result.contains(&token_percent) {
            result = result.replace(&token_percent, &value);
        }
    }

    result
}

#[cfg(not(target_arch = "wasm32"))]
fn directory_size_path(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            total = total.saturating_add(directory_size_path(&entry.path()));
        }
    }
    total
}

#[cfg(not(target_arch = "wasm32"))]
fn matching_file_size_path(path: &std::path::Path, extensions: &[&str]) -> u64 {
    if !path.exists() {
        return 0;
    }

    if path.is_file() {
        let matches_extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| extensions.iter().any(|ext| value.eq_ignore_ascii_case(ext)))
            .unwrap_or(false);
        return if matches_extension { path.metadata().map(|m| m.len()).unwrap_or(0) } else { 0 };
    }

    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            total = total.saturating_add(matching_file_size_path(&entry.path(), extensions));
        }
    }
    total
}
#[cfg(test)]
#[path = "fs_tests.rs"]
mod fs_tests;
