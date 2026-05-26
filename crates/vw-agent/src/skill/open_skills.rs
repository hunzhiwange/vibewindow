//! Open Skills 仓库管理模块
//!
//! 本模块负责 open-skills 仓库的生命周期管理，包括：
//! - 自动克隆远程仓库到本地
//! - 定期同步更新仓库内容
//! - 解析和验证仓库路径
//!
//! # 同步策略
//!
//! 使用基于时间戳的同步策略，通过标记文件记录上次同步时间，
//! 避免频繁的网络请求，同时保持内容相对最新。
//!
//! # 配置优先级
//!
//! 仓库路径按以下优先级解析：
//! 1. 环境变量 `VIBEWINDOW_OPEN_SKILLS_DIR`
//! 2. 配置文件中的 `open_skills_dir` 设置
//! 3. 用户主目录下的 `open-skills` 子目录

use crate::app::agent::shell::git_std_command;
use crate::app::agent::skill::constants::{
    OPEN_SKILLS_REPO_URL, OPEN_SKILLS_SYNC_INTERVAL_SECS, OPEN_SKILLS_SYNC_MARKER,
};
use crate::app::agent::skill::policy::open_skills_enabled;
use anyhow::Result;
use directories::UserDirs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// 确保 open-skills 仓库可用并返回其路径
///
/// 此函数是 open-skills 仓库管理的主入口点，负责：
/// 1. 检查功能是否启用
/// 2. 如果仓库不存在则克隆
/// 3. 如果超过同步间隔则拉取更新
///
/// # 参数
///
/// * `config_open_skills_enabled` - 配置中的功能开关，None 表示使用默认策略
/// * `config_open_skills_dir` - 配置中指定的仓库目录路径
///
/// # 返回值
///
/// * `Some(PathBuf)` - 仓库路径，表示功能已启用且仓库可用
/// * `None` - 功能未启用或仓库初始化失败
///
/// # 错误处理
///
/// - 克隆失败时返回 None
/// - 拉取更新失败时发出警告但继续使用本地副本
/// - 标记同步时间的失败会被静默忽略（使用 `let _ = ...`）
///
/// # 示例
///
/// ```no_run
/// use vibe_agent::app::agent::skill::open_skills::ensure_open_skills_repo;
///
/// // 使用默认配置
/// if let Some(path) = ensure_open_skills_repo(None, None) {
///     println!("Open skills repo ready at: {:?}", path);
/// }
///
/// // 使用自定义目录
/// if let Some(path) = ensure_open_skills_repo(Some(true), Some("/custom/path")) {
///     println!("Using custom location: {:?}", path);
/// }
/// ```
pub(crate) fn ensure_open_skills_repo(
    config_open_skills_enabled: Option<bool>,
    config_open_skills_dir: Option<&str>,
) -> Option<PathBuf> {
    // 检查功能是否启用，未启用则直接返回 None
    if !open_skills_enabled(config_open_skills_enabled) {
        return None;
    }

    // 解析仓库目录路径
    let repo_dir = resolve_open_skills_dir(config_open_skills_dir)?;

    // 如果仓库目录不存在，执行克隆操作
    if !repo_dir.exists() {
        if !clone_open_skills_repo(&repo_dir) {
            return None;
        }
        // 标记同步完成时间
        let _ = mark_open_skills_synced(&repo_dir);
        return Some(repo_dir);
    }

    // 仓库已存在，检查是否需要同步更新
    if should_sync_open_skills(&repo_dir) {
        if pull_open_skills_repo(&repo_dir) {
            // 更新同步标记
            let _ = mark_open_skills_synced(&repo_dir);
        } else {
            // 拉取失败，使用现有本地副本继续工作
            tracing::warn!(
                "open-skills update failed; using local copy from {}",
                repo_dir.display()
            );
        }
    }

    Some(repo_dir)
}

/// 从多个来源解析 open-skills 目录路径
///
/// 按优先级顺序尝试从环境变量、配置文件和用户主目录获取路径
///
/// # 参数
///
/// * `env_dir` - 环境变量中指定的路径（最高优先级）
/// * `config_dir` - 配置文件中指定的路径（次优先级）
/// * `home_dir` - 用户主目录路径（最低优先级，作为基准构建默认路径）
///
/// # 返回值
///
/// 返回解析后的有效路径，如果所有来源都无效则返回 None
fn resolve_open_skills_dir_from_sources(
    env_dir: Option<&str>,
    config_dir: Option<&str>,
    home_dir: Option<&Path>,
) -> Option<PathBuf> {
    // 辅助闭包：解析并验证路径字符串
    let parse_dir = |raw: &str| {
        let trimmed = raw.trim();
        if trimmed.is_empty() { None } else { Some(PathBuf::from(trimmed)) }
    };

    // 优先级 1：使用环境变量路径
    if let Some(env_dir) = env_dir.and_then(parse_dir) {
        return Some(env_dir);
    }
    // 优先级 2：使用配置文件路径
    if let Some(config_dir) = config_dir.and_then(parse_dir) {
        return Some(config_dir);
    }
    // 优先级 3：使用主目录下的默认路径
    home_dir.map(|home| home.join("open-skills"))
}

/// 解析 open-skills 仓库目录路径
///
/// 从实际配置源收集路径信息并委托给 `resolve_open_skills_dir_from_sources` 进行解析
///
/// # 参数
///
/// * `config_open_skills_dir` - 配置文件中指定的仓库目录
///
/// # 返回值
///
/// 返回解析后的仓库路径
fn resolve_open_skills_dir(config_open_skills_dir: Option<&str>) -> Option<PathBuf> {
    // 从环境变量获取路径
    let env_dir = std::env::var("VIBEWINDOW_OPEN_SKILLS_DIR").ok();
    // 获取用户主目录
    let home_dir = UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    // 委托给统一解析函数
    resolve_open_skills_dir_from_sources(
        env_dir.as_deref(),
        config_open_skills_dir,
        home_dir.as_deref(),
    )
}

/// 克隆 open-skills 仓库到指定目录
///
/// 使用 git clone 命令克隆远程仓库，使用浅克隆（depth=1）以节省时间和空间
///
/// # 参数
///
/// * `repo_dir` - 目标仓库目录路径
///
/// # 返回值
///
/// * `true` - 克隆成功
/// * `false` - 克隆失败（父目录创建失败或 git 命令失败）
///
/// # 错误处理
///
/// 所有错误都会记录警告日志，不会 panic
fn clone_open_skills_repo(repo_dir: &Path) -> bool {
    // 确保父目录存在
    if let Some(parent) = repo_dir.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "failed to create open-skills parent directory {}: {err}",
                parent.display()
            );
            return false;
        }
    }

    // 执行 git clone 命令（浅克隆，仅获取最新提交）
    let output = git_std_command()
        .args(["clone", "--depth", "1", OPEN_SKILLS_REPO_URL])
        .arg(repo_dir)
        .output();

    // 处理命令执行结果
    match output {
        Ok(result) if result.status.success() => {
            tracing::info!("initialized open-skills at {}", repo_dir.display());
            true
        }
        Ok(result) => {
            // 命令执行但返回非零状态码
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!("failed to clone open-skills: {stderr}");
            false
        }
        Err(err) => {
            // 无法启动 git 命令
            tracing::warn!("failed to run git clone for open-skills: {err}");
            false
        }
    }
}

/// 拉取 open-skills 仓库的更新
///
/// 使用 git pull 命令更新本地仓库，采用 fast-forward 模式避免产生合并提交
///
/// # 参数
///
/// * `repo_dir` - 仓库目录路径
///
/// # 返回值
///
/// * `true` - 拉取成功或无需拉取（非 git 目录）
/// * `false` - 拉取失败
///
/// # 特殊行为
///
/// 如果用户通过环境变量指向一个非 git 目录，函数会直接返回 true，
/// 允许继续使用该目录而不执行拉取操作。
fn pull_open_skills_repo(repo_dir: &Path) -> bool {
    // 如果用户通过环境变量指向非 git 目录，直接使用而不拉取
    if !repo_dir.join(".git").exists() {
        return true;
    }

    // 执行 git pull 命令（仅允许 fast-forward 合并）
    let output = git_std_command().arg("-C").arg(repo_dir).args(["pull", "--ff-only"]).output();

    // 处理命令执行结果
    match output {
        Ok(result) if result.status.success() => true,
        Ok(result) => {
            // 命令执行但返回非零状态码
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!("failed to pull open-skills updates: {stderr}");
            false
        }
        Err(err) => {
            // 无法启动 git 命令
            tracing::warn!("failed to run git pull for open-skills: {err}");
            false
        }
    }
}

/// 判断是否应该同步 open-skills 仓库
///
/// 通过检查同步标记文件的修改时间来判断是否超过了同步间隔
///
/// # 参数
///
/// * `repo_dir` - 仓库目录路径
///
/// # 返回值
///
/// * `true` - 需要同步（标记文件不存在或已过期）
/// * `false` - 无需同步（尚未超过同步间隔）
///
/// # 同步策略
///
/// 基于 `OPEN_SKILLS_SYNC_INTERVAL_SECS` 常量定义的间隔时间，
/// 避免频繁的网络请求。
fn should_sync_open_skills(repo_dir: &Path) -> bool {
    // 获取同步标记文件路径
    let marker = repo_dir.join(OPEN_SKILLS_SYNC_MARKER);

    // 尝试读取标记文件元数据
    let Ok(metadata) = std::fs::metadata(marker) else {
        // 标记文件不存在，需要同步
        return true;
    };

    // 获取文件最后修改时间
    let Ok(modified_at) = metadata.modified() else {
        // 无法获取修改时间，需要同步
        return true;
    };

    // 计算距离上次同步的时间间隔
    let Ok(age) = SystemTime::now().duration_since(modified_at) else {
        // 时间计算失败（系统时间异常），需要同步
        return true;
    };

    // 判断是否超过同步间隔
    age >= Duration::from_secs(OPEN_SKILLS_SYNC_INTERVAL_SECS)
}

/// 标记 open-skills 仓库已完成同步
///
/// 创建或更新同步标记文件，用于记录最后同步时间
///
/// # 参数
///
/// * `repo_dir` - 仓库目录路径
///
/// # 返回值
///
/// * `Ok(())` - 标记成功
/// * `Err` - 标记失败（文件写入错误）
///
/// # 标记文件
///
/// 文件名为 `OPEN_SKILLS_SYNC_MARKER` 定义的常量，
/// 文件内容为简单的 "synced" 文本，实际的同步时间由文件修改时间决定。
fn mark_open_skills_synced(repo_dir: &Path) -> Result<()> {
    // 写入标记文件，内容为 "synced"
    std::fs::write(repo_dir.join(OPEN_SKILLS_SYNC_MARKER), b"synced")?;
    Ok(())
}
#[cfg(test)]
#[path = "open_skills_tests.rs"]
mod open_skills_tests;
