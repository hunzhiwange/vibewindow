use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn file_write_body_defaults_create_if_missing_to_false() {
    let body: FileWriteBody = serde_json::from_value(serde_json::json!({
        "path": "notes/todo.md",
        "content": "hello"
    }))
    .expect("valid body");

    assert_eq!(body.path, "notes/todo.md");
    assert_eq!(body.content, "hello");
    assert!(!body.create_if_missing);
}

#[test]
fn resolve_workspace_path_rejects_absolute_path_outside_root() {
    let root = std::path::PathBuf::from("/tmp/vibewindow-root");
    let result = resolve_workspace_path(&root, "/etc/passwd");

    assert!(result.is_err());
}
