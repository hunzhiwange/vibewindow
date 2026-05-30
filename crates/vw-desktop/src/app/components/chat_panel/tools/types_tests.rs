use super::types::{ChangeFile, ChangeFileSummary, EXPLORE_GROUP_TOOL_IDX, ExploreItem};

#[test]
fn tool_types_preserve_explicit_fields() {
    let change = ChangeFile {
        path: "src/main.rs".to_string(),
        additions: 2,
        deletions: 1,
        before: "old".to_string(),
        after: "new".to_string(),
    };
    let summary =
        ChangeFileSummary { kind: 'M', path: change.path.clone(), additions: 2, deletions: 1 };
    let item = ExploreItem { tool_idx: EXPLORE_GROUP_TOOL_IDX, raw: "raw".to_string() };

    assert_eq!(summary.path, "src/main.rs");
    assert_eq!(summary.additions, change.additions);
    assert_eq!(item.tool_idx, EXPLORE_GROUP_TOOL_IDX);
}
