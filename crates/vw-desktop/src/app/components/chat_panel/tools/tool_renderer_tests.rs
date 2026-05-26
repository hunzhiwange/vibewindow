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
        shared_tool_render_kind("tool custom_tool\n{}"),
        Some(SharedToolRenderKind::Text)
    );
}
