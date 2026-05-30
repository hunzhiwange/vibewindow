//! 刷新 Git 仓库状态，并把分支、变更和错误信息写回应用状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::GitMessage;
use super::shared::{
    load_missing_diff_contents, persist_file_tree_expanded, refresh_git_panel_data_task,
};
use crate::app::{App, Message};
use iced::Task;

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn update(app: &mut App, message: GitMessage) -> Task<Message> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        GitMessage::RefreshGitPanelData => refresh_git_panel_data_task(),
        GitMessage::RefreshChangedFiles => {
            app.git_diff_context_menu = None;
            if app.git_changed_files_loading {
                return Task::none();
            }
            let Some(path) = app.project_path.clone() else {
                app.git_changed_files.clear();
                app.git_changed_files_loading = false;
                return Task::none();
            };

            app.git_changed_files_loading = true;
            // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        Some(crate::app::components::git_panel::changed_files_in_repo(&path))
                    })
                    .await
                    .unwrap_or_default()
                },
                |files| Message::Git(GitMessage::ChangedFilesReady(files)),
            )
        }
        GitMessage::RefreshDiffFileMetas => {
            app.git_diff_context_menu = None;
            let repo_path = crate::app::components::git_panel::git_repo_path_for_app(app);
            let Some(path) = repo_path.clone() else {
                app.git_diff_file_metas.clear();
                app.git_diff_file_metas_loading = false;
                app.git_diff_file_metas_repo_path = None;
                app.git_diff_contents.clear();
                app.git_diff_contents_loading.clear();
                return Task::none();
            };

            if app.git_diff_file_metas_loading
                && app.git_diff_file_metas_repo_path.as_deref() == Some(path.as_str())
            {
                return Task::none();
            }

            app.git_diff_file_metas_loading = true;
            app.git_diff_file_metas_repo_path = Some(path.clone());

            Task::perform(
                async move {
                    let repo_path = path.clone();
                    let metas = crate::app::message::spawn_blocking_opt(move || {
                        Some(crate::app::components::git_panel::get_diff_file_metas_for_repo_path(
                            &repo_path,
                        ))
                    })
                    .await
                    .unwrap_or_default();
                    (Some(path), metas)
                },
                |(repo_path, metas)| {
                    Message::Git(GitMessage::DiffFileMetasReady { repo_path, metas })
                },
            )
        }
        GitMessage::ChangedFilesReady(files) => {
            app.git_changed_files = files;
            app.git_changed_files_loading = false;
            let was_manual_refresh = app.file_manager_changes_refreshing;
            app.file_manager_changes_refreshing = false;
            app.git_diff_selected_lines.clear();
            app.git_diff_selected_range = None;
            app.git_diff_comment_draft = None;

            if app.file_manager_show_changes {
                let mut keys = std::collections::BTreeSet::<String>::new();
                for file in &app.git_changed_files {
                    let parts = file.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
                    if parts.len() <= 1 {
                        continue;
                    }
                    for idx in 0..(parts.len() - 1) {
                        keys.insert(format!("chg:{}", parts[..=idx].join("/")));
                    }
                }
                for key in keys {
                    app.ensure_file_tree_dir_expanded(key);
                }
                persist_file_tree_expanded(app);
            }

            if was_manual_refresh {
                app.show_success_toast("Git 更改已刷新")
            } else {
                Task::none()
            }
        }
        GitMessage::DiffFileMetasReady { repo_path, metas } => {
            if app.git_diff_file_metas_repo_path != repo_path {
                return Task::none();
            }
            app.git_diff_file_metas = metas;
            app.git_diff_file_metas_loading = false;
            app.git_diff_contents.clear();
            app.git_diff_contents_loading.clear();
            load_missing_diff_contents(app, app.expanded_files.clone())
        }
        GitMessage::LoadDiffContent(file) => load_missing_diff_contents(app, std::iter::once(file)),
        GitMessage::DiffContentReady { repo_path, file, old_content, new_content } => {
            app.git_diff_contents_loading.remove(&file);
            if app.git_diff_file_metas_repo_path != repo_path {
                return Task::none();
            }
            if !app.git_diff_file_metas.iter().any(|meta| meta.path == file) {
                return Task::none();
            }
            app.git_diff_contents.insert(file, (old_content, new_content));
            Task::none()
        }
        _ => unreachable!("unexpected refresh git message"),
    }
}
