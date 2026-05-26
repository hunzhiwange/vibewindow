use crate::project::{
    ListProjectsRequest, ProjectGitStateDto, ProjectStatus, UpdateProjectRequest,
};
use serde_json::json;

#[test]
fn project_requests_default_optional_fields() {
    let list: ListProjectsRequest = serde_json::from_value(json!({})).expect("valid list");
    assert_eq!(list.cursor, None);
    assert_eq!(list.limit, None);
    assert_eq!(list.status, None);

    let update: UpdateProjectRequest = serde_json::from_value(json!({})).expect("valid update");
    assert_eq!(update.name, None);
    assert_eq!(update.commands, None);

    assert_eq!(
        serde_json::to_value(ProjectStatus::Indexing).expect("serialize"),
        json!("indexing")
    );
    let git = ProjectGitStateDto {
        is_repo: true,
        has_uncommitted_changes: false,
        ahead: None,
        behind: Some(1),
    };
    assert_eq!(serde_json::to_value(git).expect("serialize")["behind"], json!(1));
}
