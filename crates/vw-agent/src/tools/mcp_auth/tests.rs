#[test]
fn module_is_linked_for_plan6_task() {
    let marker = module_path!();
    assert!(marker.contains("mcp_auth::tests"));
}
