use super::*;

#[test]
fn parse_command_info_handles_pipeline_and_redirect() {
    let info = parse_command_info("cat file.txt | grep needle > out.txt", 0).unwrap();
    assert_eq!(info.name, "cat");
    assert_eq!(info.pipes.len(), 2);
    assert_eq!(info.redirects.len(), 1);
}

#[test]
fn tokenize_segment_preserves_quoted_argument() {
    let info = parse_command_info("echo 'hello world'", 0).unwrap();
    assert_eq!(info.name, "echo");
    assert_eq!(info.args, vec!["hello world".to_string()]);
}
