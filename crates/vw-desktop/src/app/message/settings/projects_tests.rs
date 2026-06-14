use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn renames_deletes_and_toggles_worktree() {
    let mut app = app();
    app.recent_projects = vec!["/tmp/project-a".to_string(), "/tmp/project-b".to_string()];
    app.recent_projects_edits = vec!["A".to_string(), "B".to_string()];
    let _ =
        update(&mut app, SettingsMessage::RecentProjectRenameChanged(0, "Project A".to_string()));
    assert_eq!(app.recent_projects_edits[0], "Project A");
    let _ = update(&mut app, SettingsMessage::RecentProjectRenameSave(0));
    assert_eq!(
        app.recent_projects_meta.iter().find(|m| m.path == "/tmp/project-a").unwrap().name,
        "Project A"
    );
    let _ = update(&mut app, SettingsMessage::RecentProjectDeleteRequested(0));
    assert_eq!(app.recent_project_delete_confirm_idx, Some(0));
    let _ = update(&mut app, SettingsMessage::RecentProjectDeleteCanceled);
    assert_eq!(app.recent_project_delete_confirm_idx, None);
    app.project_worktree_enabled.insert("/tmp/project-a".to_string(), true);
    let _ = update(&mut app, SettingsMessage::RecentProjectDeleteConfirmed(0));
    assert_eq!(app.recent_projects, vec!["/tmp/project-b"]);
    assert!(!app.project_worktree_enabled.contains_key("/tmp/project-a"));
    let _ = update(
        &mut app,
        SettingsMessage::ProjectEnableWorktreeToggled("/tmp/project-b".to_string(), true),
    );
    assert_eq!(app.project_worktree_enabled.get("/tmp/project-b"), Some(&true));
    let _ = update(
        &mut app,
        SettingsMessage::ProjectEnableWorktreeToggled("/tmp/project-b".to_string(), false),
    );
    assert!(!app.project_worktree_enabled.contains_key("/tmp/project-b"));
}
