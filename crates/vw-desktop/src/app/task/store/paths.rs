//! 任务存储层的 paths.rs 子模块。
//!
//! 该模块负责任务索引、持久化或产物写入中的一部分能力。实现保持文件系统与 SQLite 路径清晰分离，让上层任务流程只依赖稳定的存储函数。

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;

static TASK_INDEX_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 模块内部可见的 get_task_dir 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn get_task_dir(project_path: &str) -> PathBuf {
    let mut path = PathBuf::from(project_path);
    path.push(".vibewindow");
    path.push("tasks");
    path
}

/// 模块内部可见的 get_task_file_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub(super) fn get_task_file_path(project_path: &str, task_id: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push(format!("{}.json", task_id));
    path
}

/// 模块内部可见的 get_legacy_index_file_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub(super) fn get_legacy_index_file_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.json");
    path
}

/// 模块内部可见的 get_index_db_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn get_index_db_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.sqlite3");
    path
}

fn get_index_lock_file_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.lock");
    path
}

fn get_project_index_lock(project_path: &str) -> Arc<Mutex<()>> {
    let mut locks = TASK_INDEX_LOCKS.lock();
    locks.entry(project_path.to_string()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
}

/// 模块内部可见的 with_index_lock 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn with_index_lock<T, F>(project_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    ensure_task_dir(project_path).expect("failed to create task directory before locking index");
    let lock = get_project_index_lock(project_path);
    let _memory_guard = lock.lock();
    let _file_guard =
        IndexFileLockGuard::acquire(project_path).expect("failed to acquire task index file lock");
    f()
}

struct IndexFileLockGuard {
    _file: File,
}

impl IndexFileLockGuard {
    fn acquire(project_path: &str) -> io::Result<Self> {
        let lock_path = get_index_lock_file_path(project_path);
        let file = OpenOptions::new().read(true).write(true).create(true).open(lock_path)?;
        acquire_exclusive_file_lock(&file)?;
        Ok(Self { _file: file })
    }
}

#[cfg(unix)]
fn acquire_exclusive_file_lock(file: &File) -> io::Result<()> {
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
    if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

#[cfg(not(unix))]
fn acquire_exclusive_file_lock(_file: &File) -> io::Result<()> {
    Ok(())
}

/// 模块内部可见的 get_task_log_dir 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn get_task_log_dir(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("logs");
    path
}

/// 模块内部可见的 ensure_task_dir 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn ensure_task_dir(project_path: &str) -> io::Result<()> {
    let dir = get_task_dir(project_path);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod paths_tests;
