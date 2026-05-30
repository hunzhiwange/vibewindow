#[test]
fn drag_preview_supported_for_shape_tools_only() {
    assert!(super::tool_supports_drag_preview(
        crate::app::views::design::models::DesignTool::Rectangle
    ));
    assert!(!super::tool_supports_drag_preview(
        crate::app::views::design::models::DesignTool::Move
    ));
}
