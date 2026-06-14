use super::tool_renderer::{SharedToolRenderKind, shared_tool_render_kind};

#[test]
fn shared_tool_render_kind_routes_known_tools() {
    assert_eq!(
        shared_tool_render_kind("tool bash\n{}"),
        Some(SharedToolRenderKind::Bash)
    );
    assert_eq!(
        shared_tool_render_kind("tool enter_plan_mode\n{}"),
        Some(SharedToolRenderKind::PlanMode)
    );
    assert_eq!(
        shared_tool_render_kind("tool skill\n{}"),
        Some(SharedToolRenderKind::Skill)
    );
    assert_eq!(
        shared_tool_render_kind("tool workflow_node\n{}"),
        Some(SharedToolRenderKind::Workflow)
    );
    assert_eq!(
        shared_tool_render_kind("tool custom_tool\n{}"),
        Some(SharedToolRenderKind::Text)
    );
}

#[test]
fn shared_tool_render_kind_routes_aliases_and_special_tools() {
    assert_eq!(
        shared_tool_render_kind("tool image_info\n{}"),
        Some(SharedToolRenderKind::Bash)
    );
    assert_eq!(
        shared_tool_render_kind("tool glob_search\n{}"),
        Some(SharedToolRenderKind::Files)
    );
    assert_eq!(
        shared_tool_render_kind("tool browser_open\n{}"),
        Some(SharedToolRenderKind::Advanced)
    );
    assert_eq!(
        shared_tool_render_kind("tool mcp_demo\n{}"),
        Some(SharedToolRenderKind::Advanced)
    );
    assert_eq!(shared_tool_render_kind("invalid"), None);
}
