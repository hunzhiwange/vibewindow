use vw_api_types::cleaner::CleanerCleanupRequest;

#[test]
fn cleaner_cleanup_request_is_available_to_client_api() {
    let request = CleanerCleanupRequest { clear_system_temp: true, ..Default::default() };

    assert!(request.clear_system_temp);
}
