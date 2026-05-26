#[test]
fn module_is_linked_for_plan6_task() {
    let marker = module_path!();
    assert!(marker.contains("read_mcp_resource::tests"));
}
