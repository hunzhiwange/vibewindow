use super::*;

#[test]
fn exported_handlers_are_available() {
    let _ = handle_api_memory_list;
    let _ = handle_api_memory_store;
    let _ = handle_api_memory_delete;
}
