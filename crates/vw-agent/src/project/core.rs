//! 项目元数据的发现、持久化与更新入口。
//!
//! 本模块把目录映射为稳定的项目 `Info`：优先识别 Git 仓库根与初始提交，
//! 在 Git 不可用或非仓库目录时退回本地路径哈希。它同时维护 sandbox 列表、
//! 初始化时间、用户可编辑的项目名称/图标/命令，并通过事件总线通知调用方。

use super::{Error, Info, LOGGER, TimeInfo, UpdateInput, Vcs, discover, event, extra};
use crate::app::agent::bus;
use crate::app::agent::flag;
use crate::app::agent::shell::git_std_command;
use crate::app::agent::storage;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn local_project_id_from_path(path: &Path) -> String {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let normalized = canonical.to_string_lossy().replace('\\', "/");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    // 非 Git 目录没有天然仓库 ID，用路径的稳定哈希避免把绝对路径直接暴露成 ID。
    for b in normalized.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("local-{hash:016x}")
}

fn find_git_entry(start: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut cur = start.to_path_buf();
    loop {
        let git = cur.join(".git");
        if git.exists() {
            return Some((cur.clone(), git));
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn git_available() -> bool {
    git_std_command()
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn read_cached_id(git_entry: &Path) -> Option<String> {
    std::fs::read_to_string(git_entry.join("vibewindow"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn write_cached_id(git_entry: &Path, id: &str) {
    let _ = std::fs::write(git_entry.join("vibewindow"), id);
}

fn run_git(cwd: &Path, args: &[&str]) -> Option<String> {
    let out = git_std_command()
        .current_dir(cwd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn roots_from_git(cwd: &Path) -> Option<Vec<String>> {
    let out = run_git(cwd, &["rev-list", "--max-parents=0", "--all"])?;
    let mut roots =
        out.split('\n').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
    // 多根历史的仓库按字典序取稳定首项，避免项目 ID 随 git 输出顺序漂移。
    roots.sort();
    if roots.is_empty() { None } else { Some(roots) }
}

fn show_toplevel(cwd: &Path) -> Option<PathBuf> {
    let out = run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    let p = PathBuf::from(out);
    if p.is_absolute() { Some(p) } else { Some(cwd.join(p)) }
}

fn git_common_dir_root(cwd: &Path) -> Option<PathBuf> {
    let out = run_git(cwd, &["rev-parse", "--git-common-dir"])?;
    let dir = Path::new(&out).parent().unwrap_or(Path::new("."));
    if dir == Path::new(".") {
        return Some(cwd.to_path_buf());
    }
    if dir.is_absolute() { Some(dir.to_path_buf()) } else { Some(cwd.join(dir)) }
}

fn fake_vcs() -> Option<Vcs> {
    if flag::VIBEWINDOW_FAKE_VCS.as_deref() == Some("git") {
        return Some(Vcs::Git);
    }
    None
}

#[cfg(target_arch = "wasm32")]
/// 从目录创建 wasm 目标下的项目信息。
///
/// wasm 环境不能可靠访问本机 Git 与文件系统元数据，因此返回一个最小可用的
/// 项目描述。
///
/// # 返回值
///
/// 成功时返回项目信息和 sandbox 路径字符串。
///
/// # 错误
///
/// 当前实现不产生错误，保留 `Result` 是为了与非 wasm 入口保持一致。
pub async fn from_directory(directory: impl AsRef<Path>) -> Result<(Info, String), Error> {
    let directory = directory.as_ref().to_path_buf();
    let id = "wasm-project".to_string();
    let info = Info {
        id: id.clone(),
        worktree: directory.to_string_lossy().to_string(),
        vcs: None,
        name: Some("Wasm Project".to_string()),
        icon: None,
        commands: None,
        time: TimeInfo { created: 0, updated: 0, initialized: None },
        sandboxes: vec![],
    };
    Ok((info, directory.to_string_lossy().to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
/// 从目录发现或创建项目信息。
///
/// 函数会识别所在 Git 仓库、复用 `.git/vibewindow` 缓存的项目 ID，更新项目
/// worktree/sandbox 元数据，并在需要时触发图标发现。
///
/// # 返回值
///
/// 成功时返回最新项目信息，以及本次输入目录对应的 sandbox 路径。
///
/// # 错误
///
/// 存储读写失败、后台任务 join 失败，或图标发现/更新中传播的错误会返回。
pub async fn from_directory(directory: impl AsRef<Path>) -> Result<(Info, String), Error> {
    let directory = directory.as_ref().to_path_buf();
    let local_project_id = local_project_id_from_path(&directory);
    LOGGER.info(
        "from_directory",
        Some(extra([("directory", Value::String(directory.to_string_lossy().to_string()))])),
    );

    let (id, sandbox, worktree, vcs) = tokio::task::spawn_blocking(move || {
        let mut id = local_project_id;
        let mut sandbox = directory.clone();
        let mut worktree = directory.clone();
        let mut vcs = fake_vcs();

        if let Some((sandbox0, git_entry)) = find_git_entry(&directory) {
            sandbox = sandbox0;
            worktree = sandbox.clone();

            let cached = read_cached_id(&git_entry);
            let cached_non_global =
                cached.as_deref().filter(|c| *c != "global").map(str::to_string);
            let has_git = git_available();
            let roots = if has_git { roots_from_git(&sandbox) } else { None };

            if !has_git {
                // Git 不可用时只信任已有缓存；不猜测仓库状态，避免生成误导性的 VCS 信息。
                if let Some(cached) = cached_non_global.clone() {
                    id = cached;
                }
                vcs = fake_vcs();
            } else {
                if let Some(cached) = cached_non_global.clone() {
                    id = cached;
                } else if let Some(roots) = roots.clone() {
                    id = roots[0].clone();
                    write_cached_id(&git_entry, &id);
                } else {
                    vcs = fake_vcs();
                }

                if roots.is_some() || cached_non_global.is_some() {
                    if let Some(top) = show_toplevel(&sandbox) {
                        sandbox = top;
                    }
                    if let Some(common) = git_common_dir_root(&sandbox) {
                        worktree = common;
                    } else {
                        worktree = sandbox.clone();
                    }
                    vcs = Some(Vcs::Git);
                }
            }
        }
        (id, sandbox, worktree, vcs)
    })
    .await
    .map_err(Error::Join)?;

    let worktree_s = worktree.to_string_lossy().to_string();
    let sandbox_s = sandbox.to_string_lossy().to_string();

    let mut existing: Option<Info> = match storage::read::<Info>(&["project", &id]).await {
        Ok(v) => Some(v),
        Err(storage::Error::NotFound(_)) => None,
        Err(e) => return Err(e.into()),
    };

    if existing.is_none() {
        let now = now_ms();
        existing = Some(Info {
            id: id.clone(),
            worktree: worktree_s.clone(),
            vcs: vcs.clone(),
            name: None,
            icon: None,
            commands: None,
            time: TimeInfo { created: now, updated: now, initialized: None },
            sandboxes: Vec::new(),
        });
    }

    let mut result = existing.unwrap_or_else(|| {
        let now = now_ms();
        Info {
            id: id.clone(),
            worktree: worktree_s.clone(),
            vcs: vcs.clone(),
            name: None,
            icon: None,
            commands: None,
            time: TimeInfo { created: now, updated: now, initialized: None },
            sandboxes: Vec::new(),
        }
    });

    if result.sandboxes.iter().all(|s| s != &sandbox_s) && sandbox_s != result.worktree {
        result.sandboxes.push(sandbox_s.clone());
    }
    // 清理已不存在的 sandbox，避免 UI 展示陈旧目录或后续操作落到无效路径。
    result.sandboxes = result.sandboxes.into_iter().filter(|p| Path::new(p).is_dir()).collect();
    result.worktree = worktree_s.clone();
    result.vcs = vcs.clone();
    result.time.updated = now_ms();

    if *flag::VIBEWINDOW_EXPERIMENTAL_ICON_DISCOVERY {
        let _ = discover(&result).await;
        if let Ok(updated) = storage::read::<Info>(&["project", &id]).await {
            result = updated;
        }
    }

    storage::write::<Info>(&["project", &id], &result).await?;
    let _ = bus::publish(event::UPDATED, &result, None);

    Ok((result, sandbox_s))
}

/// 标记项目已完成初始化。
///
/// # 错误
///
/// 当项目记录不存在或存储更新失败时返回错误。
pub async fn set_initialized(project_id: &str) -> Result<(), Error> {
    let _ = storage::update::<Info>(&["project", project_id], |draft| {
        draft.time.initialized = Some(now_ms());
        draft.time.updated = now_ms();
    })
    .await?;
    Ok(())
}

/// 列出所有已保存的项目。
///
/// 返回前会过滤掉已经不存在的 sandbox 路径，但不会修改存储中的原始记录。
///
/// # 错误
///
/// 当项目 key 列表读取失败时返回错误；单个项目读取失败会被跳过。
pub async fn list() -> Result<Vec<Info>, Error> {
    let keys = storage::list(&["project"]).await?;
    let mut out = Vec::new();
    for key in keys {
        if key.len() < 2 {
            continue;
        }
        let id = key[1].clone();
        if let Ok(mut project) = storage::read::<Info>(&["project", &id]).await {
            project.sandboxes =
                project.sandboxes.into_iter().filter(|p| Path::new(p).is_dir()).collect();
            out.push(project);
        }
    }
    Ok(out)
}

/// 更新项目的用户可编辑字段。
///
/// 支持名称、图标和启动命令的局部更新；空字符串形式的图标覆盖与启动命令会被
/// 归一化为 `None`。
///
/// # 错误
///
/// 当目标项目不存在或存储更新失败时返回错误。
pub async fn update(input: UpdateInput) -> Result<Info, Error> {
    let updated = storage::update::<Info>(&["project", &input.project_id], |draft| {
        if let Some(name) = input.name {
            draft.name = name;
        }

        if let Some(icon) = input.icon.clone() {
            let mut cur = draft.icon.clone().unwrap_or_default();
            if let Some(url) = icon.url {
                cur.url = url;
            }
            if let Some(ov) = icon.override_icon {
                cur.override_icon = ov.and_then(|s| if s.is_empty() { None } else { Some(s) });
            }
            if let Some(color) = icon.color {
                cur.color = color;
            }
            draft.icon = Some(cur);
        }

        if let Some(cmds) = input.commands.clone() {
            let mut cur = draft.commands.clone().unwrap_or_default();
            if let Some(start) = cmds.start {
                cur.start = start.and_then(|s| if s.is_empty() { None } else { Some(s) });
            }
            if cur.start.is_none() {
                draft.commands = None;
            } else {
                draft.commands = Some(cur);
            }
        }

        draft.time.updated = now_ms();
    })
    .await?;

    let _ = bus::publish(event::UPDATED, &updated, None);
    Ok(updated)
}

/// 查询项目仍然存在的 sandbox 列表。
///
/// # 错误
///
/// 存储层出现非 NotFound 错误时返回错误；项目不存在时返回空列表。
pub async fn sandboxes(project_id: &str) -> Result<Vec<String>, Error> {
    let project = match storage::read::<Info>(&["project", project_id]).await {
        Ok(v) => v,
        Err(storage::Error::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    Ok(project.sandboxes.into_iter().filter(|p| Path::new(p).is_dir()).collect())
}

/// 为项目追加一个 sandbox 目录。
///
/// 已存在的目录不会重复追加。
///
/// # 错误
///
/// 当项目不存在或存储更新失败时返回错误。
pub async fn add_sandbox(project_id: &str, directory: &str) -> Result<Info, Error> {
    let updated = storage::update::<Info>(&["project", project_id], |draft| {
        if !draft.sandboxes.iter().any(|p| p == directory) {
            draft.sandboxes.push(directory.to_string());
        }
        draft.time.updated = now_ms();
    })
    .await?;
    let _ = bus::publish(event::UPDATED, &updated, None);
    Ok(updated)
}

/// 从项目中移除一个 sandbox 目录。
///
/// # 错误
///
/// 当项目不存在或存储更新失败时返回错误。
pub async fn remove_sandbox(project_id: &str, directory: &str) -> Result<Info, Error> {
    let updated = storage::update::<Info>(&["project", project_id], |draft| {
        draft.sandboxes.retain(|p| p != directory);
        draft.time.updated = now_ms();
    })
    .await?;
    let _ = bus::publish(event::UPDATED, &updated, None);
    Ok(updated)
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod core_tests;
