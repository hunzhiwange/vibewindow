#[test]
fn top_level_reexports_worktree_types() {
    let info = super::Info {
        name: "demo".to_string(),
        branch: "vibewindow/demo".to_string(),
        directory: "/tmp/demo".to_string(),
    };

    assert_eq!(info.name, "demo");
}
