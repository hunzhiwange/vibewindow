use super::*;

#[test]
fn prompt_for_permission_with_io_accepts_yes_and_writes_prompt_parts() {
    let mut reader = std::io::Cursor::new(b"  YeS  \n".to_vec());
    let mut output = Vec::new();

    let allowed = prompt_for_permission_with_io(
        &PermissionPromptOptions {
            prompt: "Allow terminal child?".to_string(),
            header: Some("Permission header".to_string()),
            details: Some("Permission details".to_string()),
        },
        &mut reader,
        &mut output,
    )
    .expect("prompt should read answer");

    let output = String::from_utf8(output).expect("prompt output should be utf8");
    assert!(allowed);
    assert!(output.contains("Permission header"));
    assert!(output.contains("Permission details"));
    assert!(output.contains("Allow terminal child?"));
}

#[test]
fn prompt_for_permission_with_io_rejects_non_yes_and_skips_blank_details() {
    let mut reader = std::io::Cursor::new(b"no\n".to_vec());
    let mut output = Vec::new();

    let allowed = prompt_for_permission_with_io(
        &PermissionPromptOptions {
            prompt: "Allow terminal child?".to_string(),
            header: None,
            details: Some("   \t  ".to_string()),
        },
        &mut reader,
        &mut output,
    )
    .expect("prompt should read answer");

    let output = String::from_utf8(output).expect("prompt output should be utf8");
    assert!(!allowed);
    assert!(output.contains("Allow terminal child?"));
    assert!(!output.contains("   \t  \n"));
}

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
