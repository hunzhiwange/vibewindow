use super::*;

#[test]
fn permission_prompt_options_preserve_prompt_header_and_details() {
    let options = PermissionPromptOptions {
        prompt: "Allow?".to_string(),
        header: Some("Header".to_string()),
        details: Some("Details".to_string()),
    };

    assert_eq!(options.prompt, "Allow?");
    assert_eq!(options.header.as_deref(), Some("Header"));
    assert_eq!(options.details.as_deref(), Some("Details"));
}

#[test]
fn prompt_for_permission_defaults_to_false_when_terminal_is_unavailable() {
    if can_prompt_for_permission() {
        return;
    }

    let allowed = prompt_for_permission(&PermissionPromptOptions {
        prompt: "Allow?".to_string(),
        header: None,
        details: None,
    })
    .expect("non-terminal prompt should be handled");
    assert!(!allowed);
}
