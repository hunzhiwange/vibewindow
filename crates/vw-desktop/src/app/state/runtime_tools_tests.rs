//! 维护运行时工具选择器状态及其回归测试。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::{
    AdvancedToolSurfaceState,
    SessionToolBucket,
    SessionToolGroup,
    SessionToolSelectorState,
    SessionToolSelectorTab,
    explicit_advanced_tool_surface_spec,
    tool_display_name,
    tool_bucket,
    tool_group,
};

#[test]
fn tool_selector_buckets_match_runtime_surfaces() {
    assert_eq!(tool_bucket("read"), SessionToolBucket::ReadOnly);
    assert_eq!(tool_bucket("file_edit"), SessionToolBucket::Edit);
    assert_eq!(tool_bucket("bash"), SessionToolBucket::Execution);
    assert_eq!(tool_bucket("browser"), SessionToolBucket::Browser);
    assert_eq!(tool_bucket("AgentTool"), SessionToolBucket::Agent);
    assert_eq!(tool_bucket("tool_search"), SessionToolBucket::Agent);
    assert_eq!(tool_bucket("plan_enter"), SessionToolBucket::Agent);
    assert_eq!(tool_bucket("verify_plan_execution"), SessionToolBucket::Agent);
}

#[test]
fn tool_selector_filters_tools_by_enabled_bucket() {
    let mut selector = SessionToolSelectorState::default();
    assert!(selector.toggle_bucket(SessionToolBucket::Browser));

    let filtered = selector.filter_tools(&[
        "read".to_string(),
        "browser".to_string(),
        "AgentTool".to_string(),
    ]);

    assert_eq!(filtered, vec!["read".to_string(), "AgentTool".to_string()]);
}

#[test]
fn tool_selector_keeps_last_bucket_enabled() {
    let mut selector = SessionToolSelectorState::default();
    assert!(selector.toggle_bucket(SessionToolBucket::ReadOnly));
    assert!(selector.toggle_bucket(SessionToolBucket::Edit));
    assert!(selector.toggle_bucket(SessionToolBucket::Execution));
    assert!(selector.toggle_bucket(SessionToolBucket::Browser));
    assert!(selector.toggle_bucket(SessionToolBucket::Agent));
    assert!(!selector.toggle_bucket(SessionToolBucket::Other));
    assert!(selector.is_bucket_enabled(SessionToolBucket::Other));
}

#[test]
fn tool_selector_tracks_tab_and_group_collapse_state() {
    let mut selector = SessionToolSelectorState::default();

    assert_eq!(selector.active_tab(), SessionToolSelectorTab::Agent);
    selector.select_tab(SessionToolSelectorTab::Tools);
    selector.toggle_group_collapsed(SessionToolGroup::Execute);

    assert_eq!(selector.active_tab(), SessionToolSelectorTab::Tools);
    assert!(selector.is_group_collapsed(SessionToolGroup::Execute));
}

#[test]
fn tool_selector_filters_tools_by_explicit_selection() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec![
        "read".to_string(),
        "apply_patch".to_string(),
        "bash".to_string(),
    ];

    assert!(selector.toggle_tool(&tools, "bash"));

    assert_eq!(selector.filter_tools(&tools), vec!["read".to_string(), "apply_patch".to_string()]);
    assert!(selector.has_custom_tool_selection());
}

#[test]
fn tool_selector_prevents_disabling_last_enabled_tool() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec!["read".to_string()];

    assert!(!selector.toggle_tool(&tools, "read"));
    assert_eq!(selector.filter_tools(&tools), tools);
}

#[test]
fn tool_selector_group_toggle_respects_last_enabled_tool() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec!["read".to_string(), "bash".to_string()];

    assert!(selector.toggle_group_tools(&tools, SessionToolGroup::Files));
    assert_eq!(selector.filter_tools(&tools), vec!["bash".to_string()]);

    let single_tool = vec!["read".to_string()];
    assert!(!selector.toggle_group_tools(&single_tool, SessionToolGroup::Files));
}

#[test]
fn tool_selector_select_all_clears_explicit_selection() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec![
        "read".to_string(),
        "apply_patch".to_string(),
        "bash".to_string(),
    ];

    assert!(selector.toggle_tool(&tools, "bash"));
    assert!(selector.has_custom_tool_selection());

    selector.select_all_tools(&tools);

    assert_eq!(selector.filter_tools(&tools), tools);
    assert!(!selector.has_custom_tool_selection());
}

#[test]
fn tool_selector_invert_uses_current_enabled_complement() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec![
        "read".to_string(),
        "apply_patch".to_string(),
        "bash".to_string(),
    ];

    assert!(selector.toggle_tool(&tools, "bash"));
    assert!(selector.invert_tools(&tools));

    assert_eq!(selector.filter_tools(&tools), vec!["bash".to_string()]);
}

#[test]
fn tool_selector_invert_rejects_empty_result() {
    let mut selector = SessionToolSelectorState::default();
    let tools = vec!["read".to_string(), "bash".to_string()];

    assert!(!selector.invert_tools(&tools));
}

#[test]
fn tool_selector_group_and_display_name_match_known_tools() {
    assert_eq!(tool_group("read"), SessionToolGroup::Files);
    assert_eq!(tool_group("grep"), SessionToolGroup::Search);
    assert_eq!(tool_group("bash"), SessionToolGroup::Execute);
    assert_eq!(tool_group("browser"), SessionToolGroup::Web);
    assert_eq!(tool_group("memory_store"), SessionToolGroup::Memory);
    assert_eq!(tool_display_name("apply_patch"), "补丁编辑".to_string());
}

#[test]
fn advanced_surface_specs_mark_available_and_planned() {
    assert_eq!(
        explicit_advanced_tool_surface_spec("enter_plan_mode").map(|spec| spec.state),
        Some(AdvancedToolSurfaceState::Available)
    );
    assert_eq!(
        explicit_advanced_tool_surface_spec("verify_plan_execution").map(|spec| spec.state),
        Some(AdvancedToolSurfaceState::Available)
    );
    assert_eq!(
        explicit_advanced_tool_surface_spec("mcp_auth").map(|spec| spec.state),
        Some(AdvancedToolSurfaceState::Planned)
    );
    assert_eq!(
        explicit_advanced_tool_surface_spec("tool_search").map(|spec| spec.state),
        Some(AdvancedToolSurfaceState::Available)
    );
}
