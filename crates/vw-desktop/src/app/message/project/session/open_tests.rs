#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("open_tests"));
}

fn session_info(id: &str, additions: i64, deletions: i64) -> vw_shared::session::info::Info {
    vw_shared::session::info::Info {
        id: id.to_string(),
        slug: id.to_string(),
        project_id: "project".to_string(),
        directory: "/tmp/project".to_string(),
        parent_id: None,
        summary: Some(vw_shared::session::info::Summary {
            additions,
            deletions,
            files: 1,
            diffs: None,
        }),
        share: None,
        title: id.to_string(),
        version: "1".to_string(),
        time: vw_shared::session::info::TimeInfo {
            created: 1,
            updated: 1,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

#[test]
fn merge_project_sessions_replaces_existing_summary() {
    let existing = vec![session_info("task-board-1", 0, 0), session_info("local-only", 2, 1)];
    let incoming = vec![session_info("task-board-1", 7, 3)];

    let merged = super::merge_project_sessions(Some(existing), incoming);

    assert_eq!(merged.len(), 2);
    let summary = merged[0].summary.as_ref().expect("incoming summary should be kept");
    assert_eq!(merged[0].id, "task-board-1");
    assert_eq!(summary.additions, 7);
    assert_eq!(summary.deletions, 3);
    assert_eq!(merged[1].id, "local-only");
}
