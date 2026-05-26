use super::*;

#[test]
fn tool_handlers_are_available() {
    let _ = handle_api_tools;
    let _ = handle_api_cli_tools;
    let _ = handle_api_doctor;
}
