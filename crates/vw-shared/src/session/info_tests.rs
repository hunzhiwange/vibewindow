#[test]
fn info_serialization_uses_frontend_field_names_and_skips_empty_options() {
    let info = super::Info {
        id: "session-1".to_string(),
        slug: "slug".to_string(),
        project_id: "project-1".to_string(),
        directory: "/work".to_string(),
        parent_id: None,
        summary: None,
        share: None,
        title: "Title".to_string(),
        version: "1".to_string(),
        time: super::TimeInfo {
            created: 1,
            updated: 2,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: Some(super::RevertInfo {
            message_id: "msg-1".to_string(),
            part_id: None,
            snapshot: None,
            diff: None,
        }),
    };

    let value = serde_json::to_value(info).unwrap();

    assert_eq!(value["projectID"], "project-1");
    assert_eq!(value["revert"]["messageID"], "msg-1");
    assert!(value.get("parentID").is_none());
    assert!(value["revert"].get("partID").is_none());
}
