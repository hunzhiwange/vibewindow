#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("projects_tests"));
}

fn recent_meta(path: &str) -> super::RecentProjectMeta {
    super::RecentProjectMeta {
        path: path.to_string(),
        name: "Project".to_string(),
        task_board_settings: None,
        session_auto_refresh: crate::app::state::default_recent_project_session_auto_refresh(),
        session_refresh_interval_seconds:
            crate::app::state::default_recent_project_session_refresh_interval_seconds(),
        icon: None,
        icon_color: None,
        worktree_start_command: None,
    }
}

#[test]
fn is_visible_recent_project_path_rejects_hidden_last_segment() {
    assert!(!super::is_visible_recent_project_path(
        "/var/folders/dt/hvzzy7ds3756n3cpg14j17200000gn/T/.tmp0fBHwT"
    ));
}

#[test]
fn is_visible_recent_project_path_rejects_windows_hidden_last_segment() {
    assert!(!super::is_visible_recent_project_path("C:\\Users\\me\\.tmp0fBHwT\\"));
}

#[test]
fn is_visible_recent_project_path_accepts_visible_project_under_hidden_parent() {
    assert!(super::is_visible_recent_project_path("/Users/me/.local/vibe-window"));
}

#[test]
fn parse_recent_projects_filters_hidden_projects() {
    let content = r#"[
        "/Users/me/vibe-window",
        "/var/folders/dt/hvzzy7ds3756n3cpg14j17200000gn/T/.tmp0fBHwT",
        "/Users/me/vibe-window"
    ]"#;

    assert_eq!(
        super::parse_recent_projects(content),
        Some(vec!["/Users/me/vibe-window".to_string()])
    );
}

#[test]
fn normalize_recent_projects_meta_filters_hidden_projects() {
    let metas = vec![
        recent_meta("/Users/me/vibe-window"),
        recent_meta("/var/folders/dt/hvzzy7ds3756n3cpg14j17200000gn/T/.tmp0fBHwT"),
        recent_meta(" /Users/me/vibe-window "),
    ];

    let normalized = super::normalize_recent_projects_meta(metas);

    assert_eq!(normalized.len(), 1);
    assert_eq!(normalized[0].path, "/Users/me/vibe-window");
}
