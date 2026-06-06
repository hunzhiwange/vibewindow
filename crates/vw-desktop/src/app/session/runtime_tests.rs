#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("runtime_tests"));
}

#[test]
fn request_allowed_tools_from_inventory_uses_filtered_tools() {
    let mut runtime = crate::app::state::SessionRuntimeState::default();
    let inventory = crate::app::state::SessionToolInventory {
        base_tools: vec!["file_read".to_string(), "file_write".to_string()],
    };

    assert_eq!(
        super::request_allowed_tools_from_inventory(&runtime, &inventory),
        Some(vec!["file_read".to_string(), "file_write".to_string()])
    );

    runtime.tool_selector.toggle_tool(&inventory.base_tools, "file_write");

    assert_eq!(
        super::request_allowed_tools_from_inventory(&runtime, &inventory),
        Some(vec!["file_read".to_string()])
    );
}
