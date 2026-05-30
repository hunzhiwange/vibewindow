use crate::app::agent::shell::git_std_command;

use super::{Info, Status};
use std::path::Path;

/// 获取 Git 工作区的文件状态。
pub fn status_git(worktree: impl AsRef<Path>) -> Vec<Info> {
    let worktree = worktree.as_ref();

    let output = git_std_command()
        .args(["-c", "core.quotepath=false", "diff", "--numstat", "HEAD"])
        .current_dir(worktree)
        .output();

    let mut changed = Vec::new();
    if let Ok(out) = output {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout).to_string();
            for line in txt.lines().filter(|line| !line.trim().is_empty()) {
                let parts = line.split('\t').collect::<Vec<_>>();
                if parts.len() < 3 {
                    continue;
                }
                let added = parts[0].parse::<i64>().unwrap_or(0);
                let removed = parts[1].parse::<i64>().unwrap_or(0);
                let path = parts[2].to_string();
                changed.push(Info { path, added, removed, status: Status::Modified });
            }
        }
    }

    let output = git_std_command()
        .args(["-c", "core.quotepath=false", "ls-files", "--others", "--exclude-standard"])
        .current_dir(worktree)
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout).to_string();
            for path in txt.lines().filter(|line| !line.trim().is_empty()) {
                let full = worktree.join(path);
                let lines = std::fs::read_to_string(&full)
                    .map(|content| content.lines().count() as i64)
                    .unwrap_or(0);
                changed.push(Info {
                    path: path.to_string(),
                    added: lines,
                    removed: 0,
                    status: Status::Added,
                });
            }
        }
    }

    let output = git_std_command()
        .args(["-c", "core.quotepath=false", "diff", "--name-only", "--diff-filter=D", "HEAD"])
        .current_dir(worktree)
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout).to_string();
            for path in txt.lines().filter(|line| !line.trim().is_empty()) {
                changed.push(Info {
                    path: path.to_string(),
                    added: 0,
                    removed: 0,
                    status: Status::Deleted,
                });
            }
        }
    }

    changed
}
