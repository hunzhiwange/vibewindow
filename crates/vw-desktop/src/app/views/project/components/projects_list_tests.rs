use std::fs;

use iced::{Color, Element};
use tempfile::tempdir;
use vw_shared::session::info::{Info, TimeInfo};

use super::*;
use crate::app::state::SessionRuntimeState;
use crate::app::{App, Message, RecentProjectMeta};

fn assert_color_eq(actual: Color, expected: Color) {
    assert_eq!(actual.r, expected.r);
    assert_eq!(actual.g, expected.g);
    assert_eq!(actual.b, expected.b);
    assert_eq!(actual.a, expected.a);
}

fn session_info(id: &str, directory: &str) -> Info {
    Info {
        id: id.to_string(),
        slug: id.to_string(),
        project_id: "project".to_string(),
        directory: directory.to_string(),
        parent_id: None,
        summary: None,
        share: None,
        title: "Session".to_string(),
        version: "1".to_string(),
        time: TimeInfo { created: 1, updated: 2, compacting: None, archived: None },
        permission: None,
        revert: None,
    }
}

fn recent_meta(path: &str, icon: Option<&str>, icon_color: Option<&str>) -> RecentProjectMeta {
    RecentProjectMeta {
        path: path.to_string(),
        name: "Project".to_string(),
        task_board_settings: None,
        session_auto_refresh: true,
        session_refresh_interval_seconds: 60,
        icon: icon.map(str::to_string),
        icon_color: icon_color.map(str::to_string),
        worktree_start_command: None,
    }
}

#[test]
fn parse_hex_color_accepts_hash_and_trimmed_plain_values() {
    assert_color_eq(parse_hex_color("#0A1b2C").unwrap(), Color::from_rgb8(10, 27, 44));
    assert_color_eq(parse_hex_color("  ff8040  ").unwrap(), Color::from_rgb8(255, 128, 64));
}

#[test]
fn parse_hex_color_rejects_invalid_length_or_digits() {
    assert!(parse_hex_color("").is_none());
    assert!(parse_hex_color("#12345").is_none());
    assert!(parse_hex_color("#1234567").is_none());
    assert!(parse_hex_color("#12xx56").is_none());
}

#[test]
fn icon_image_handle_rejects_empty_or_missing_paths() {
    assert!(icon_image_handle("").is_none());
    assert!(icon_image_handle("   ").is_none());
    assert!(icon_image_handle("/path/that/does/not/exist.png").is_none());
}

#[test]
fn icon_image_handle_accepts_file_urls_and_plain_paths() {
    let dir = tempdir().unwrap();
    let icon_path = dir.path().join("icon.png");
    fs::write(&icon_path, b"not decoded until render").unwrap();

    assert!(icon_image_handle(icon_path.to_str().unwrap()).is_some());
    assert!(icon_image_handle(&format!("file://{}", icon_path.display())).is_some());
    assert!(icon_image_handle(&format!("file:///{}", icon_path.display())).is_some());
}

#[test]
fn project_badge_button_builds_default_text_icon() {
    let element: Element<'_, Message> = project_badge_button(
        "/tmp/project-a".to_string(),
        "Alpha Project".to_string(),
        false,
        false,
        false,
        None,
        None,
    );

    std::hint::black_box(element);
}

#[test]
fn project_badge_button_builds_selected_attention_custom_color_icon() {
    let element: Element<'_, Message> = project_badge_button(
        "/tmp/project-b".to_string(),
        "项目乙".to_string(),
        true,
        true,
        true,
        Some("字".to_string()),
        Some(Color::from_rgb8(24, 144, 255)),
    );

    std::hint::black_box(element);
}

#[test]
fn project_badge_button_builds_image_icon_branch() {
    let dir = tempdir().unwrap();
    let icon_path = dir.path().join("icon.png");
    fs::write(&icon_path, b"not decoded until render").unwrap();

    let element: Element<'_, Message> = project_badge_button(
        "/tmp/project-c".to_string(),
        "Gamma".to_string(),
        false,
        true,
        false,
        Some(icon_path.to_string_lossy().to_string()),
        Some(Color::from_rgb8(180, 40, 80)),
    );

    std::hint::black_box(element);
}

#[test]
fn open_project_badge_button_builds() {
    let element: Element<'_, Message> = open_project_badge_button();

    std::hint::black_box(element);
}

#[test]
fn projects_list_builds_empty_recent_projects() {
    let (mut app, _task) = App::new();
    app.recent_projects.clear();
    app.recent_projects_edits.clear();
    app.recent_projects_meta.clear();

    let element: Element<'_, Message> = projects_list(&app, false);

    std::hint::black_box(element);
}

#[test]
fn projects_list_builds_with_hover_meta_and_project_attention() {
    let (mut app, _task) = App::new();
    let path = "/tmp/vibe-window-alpha".to_string();
    let session_id = "session-alpha".to_string();

    app.recent_projects = vec![path.clone()];
    app.recent_projects_edits = vec!["Alpha".to_string()];
    app.recent_projects_meta = vec![recent_meta(&path, Some("A"), Some("#336699"))];
    app.project_path = Some(path.clone());
    app.hovered_recent_project = Some(path.clone());
    app.project_sessions.insert(path.clone(), vec![session_info(&session_id, &path)]);

    let mut runtime = SessionRuntimeState::new();
    runtime.is_requesting = true;
    app.session_runtime_states.insert(session_id, runtime);

    let element: Element<'_, Message> = projects_list(&app, true);

    std::hint::black_box(element);
}

#[test]
fn projects_list_falls_back_to_current_sessions_for_attention() {
    let (mut app, _task) = App::new();
    let path = "/tmp/vibe-window-beta".to_string();
    let session_id = "session-beta".to_string();

    app.recent_projects = vec![path.clone()];
    app.recent_projects_edits = vec![String::new()];
    app.recent_projects_meta = vec![recent_meta(&path, None, Some("not-a-color"))];
    app.project_path = Some(path.clone());
    app.sessions = vec![session_info(&session_id, &path)];

    let mut runtime = SessionRuntimeState::new();
    runtime.has_unseen_success = true;
    app.session_runtime_states.insert(session_id, runtime);

    let element: Element<'_, Message> = projects_list(&app, false);

    std::hint::black_box(element);
}

#[test]
fn projects_list_uses_empty_path_when_edit_has_no_matching_recent_project() {
    let (mut app, _task) = App::new();
    app.recent_projects.clear();
    app.recent_projects_edits = vec!["Loose edit".to_string()];
    app.recent_projects_meta.clear();

    let element: Element<'_, Message> = projects_list(&app, false);

    std::hint::black_box(element);
}
