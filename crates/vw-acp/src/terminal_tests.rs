use super::*;

#[test]
fn trim_to_utf8_boundary_keeps_valid_suffix() {
    let text = "aé日";
    let trimmed = trim_to_utf8_boundary(text.as_bytes(), 4);

    assert_eq!(std::str::from_utf8(&trimmed).unwrap(), "日");
}

#[test]
fn to_command_line_renders_quoted_arguments() {
    let args = vec!["hello world".to_string(), "plain".to_string()];

    assert_eq!(to_command_line("echo", &args), r#"echo "hello world" "plain""#);
}
