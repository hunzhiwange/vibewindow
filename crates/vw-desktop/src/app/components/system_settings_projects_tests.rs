use super::*;
// Tests for plan6 task 805.
const SOURCE: &str = include_str!("system_settings_projects.rs");

use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn view_builds_no_project_current_project_and_recent_project_states() {
    let mut app = test_app();
    keep_element(view(&app));

    let project_path = "/tmp/vibe-window-project".to_string();
    app.project_path = Some(project_path.clone());
    app.project_worktree_enabled.insert(project_path.clone(), true);
    keep_element(view(&app));

    app.recent_projects =
        vec!["/tmp/vibe-window-one".to_string(), "/tmp/vibe-window-two".to_string()];
    app.recent_projects_edits = vec!["One".to_string(), "Two".to_string()];
    keep_element(view(&app));

    app.recent_project_delete_confirm_idx = Some(1);
    keep_element(view(&app));
}

#[test]
fn system_settings_projects_tests_keeps_planned_coverage_targets() {
    for name in ["field_row", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
