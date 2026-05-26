#[test]
fn safe_scope_is_used_as_path_segment() {
    assert_eq!(super::session_scope_key("project_1.alpha"), "project_1.alpha");
}

#[test]
fn unsafe_scope_is_hashed_to_stable_segment() {
    let first = super::session_scope_key("../project");
    let second = super::session_scope_key("../project");

    assert_eq!(first, second);
    assert_eq!(first.len(), 16);
    assert!(first.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn session_paths_include_scope_and_snapshot_kind() {
    let data_dir = std::path::Path::new("/tmp/vw-data");

    let scoped_dir = super::sessions_dir_for_scope(data_dir, Some("repo")).unwrap();
    let db_path = super::session_file_path(data_dir, "abc", Some("repo")).unwrap();
    let snapshot =
        super::session_step_snapshot_file_path(data_dir, "abc", 3, "finish", Some("repo"))
            .unwrap();
    let raw = super::session_step_llm_raw_file_path(data_dir, "abc", 3, Some("repo")).unwrap();

    assert_eq!(scoped_dir, data_dir.join("storage/session/scoped/repo"));
    assert_eq!(db_path, scoped_dir.join("index.sqlite3"));
    assert_eq!(
        snapshot,
        scoped_dir.join("snapshots/session-abc-step-3-finish.json")
    );
    assert_eq!(raw, scoped_dir.join("snapshots/session-abc-step-3-llm_raw.json"));
}
