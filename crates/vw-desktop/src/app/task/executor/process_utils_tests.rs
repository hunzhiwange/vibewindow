#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("process_utils_tests"));
}

#[test]
fn shell_escape_arg_leaves_plain_values_unquoted() {
    assert_eq!(super::shell_escape_arg("abc_DEF-1.2/path:ok=value"), "abc_DEF-1.2/path:ok=value");
}

#[test]
fn shell_escape_arg_quotes_spaces_and_embedded_quotes() {
    assert_eq!(super::shell_escape_arg("hello world"), "'hello world'");
    assert_eq!(super::shell_escape_arg("it's ok"), "'it'\\''s ok'");
}

#[test]
fn to_shell_command_escapes_program_and_args() {
    let args = vec!["status".to_string(), "two words".to_string(), "it's".to_string()];

    assert_eq!(super::to_shell_command("git", &args), "git status 'two words' 'it'\\''s'");
}

#[test]
fn tail_chars_returns_full_string_when_short_enough() {
    assert_eq!(super::tail_chars("abc", 3), "abc");
    assert_eq!(super::tail_chars("abc", 10), "abc");
}

#[test]
fn tail_chars_truncates_by_char_boundary() {
    assert_eq!(super::tail_chars("a你b好c", 3), "b好c");
}

#[test]
fn build_command_failure_detail_prefers_stderr_then_stdout() {
    assert_eq!(
        super::build_command_failure_detail(Some(2), None, "stdout text", "stderr text", false),
        "code=2 stderr=stderr text"
    );
    assert_eq!(
        super::build_command_failure_detail(Some(2), None, "stdout text", "   ", false),
        "code=2 stdout=stdout text"
    );
}

#[test]
fn build_command_failure_detail_reports_broken_pipe_and_empty_exit() {
    assert_eq!(
        super::build_command_failure_detail(None, None, "", "", true),
        "code=None stdin=BrokenPipe(对端提前关闭输入)"
    );
    assert_eq!(super::build_command_failure_detail(Some(0), None, "", "", false), "code=0");
}

#[test]
fn build_command_failure_detail_formats_known_and_unknown_signals() {
    assert_eq!(
        super::build_command_failure_detail(None, Some(15), "", "", false),
        "signal=15(SIGTERM)"
    );
    assert_eq!(super::build_command_failure_detail(None, Some(64), "", "", false), "signal=64");
}

#[test]
fn normalize_path_returns_canonical_path_when_it_exists() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let nested = temp.path().join("nested");
    std::fs::create_dir_all(&nested).expect("nested dir should be created");

    let normalized = super::normalize_path(nested.to_string_lossy().as_ref());

    assert!(std::path::Path::new(&normalized).is_absolute());
    assert!(normalized.ends_with("nested"));
}

#[test]
fn normalize_path_returns_original_when_missing() {
    let missing = "/definitely/missing/vibe-window-test-path";

    assert_eq!(super::normalize_path(missing), missing);
}

#[test]
fn emit_log_helpers_send_to_matching_streams() {
    let (tx, rx) = std::sync::mpsc::channel();

    super::emit_stdout_log(Some(&tx), "out");
    super::emit_stderr_log(Some(&tx), "err");
    super::emit_stdout_log(None, "ignored");

    match rx.recv().expect("stdout log should be sent") {
        super::TaskLogStream::Stdout(value) => assert_eq!(value, "out"),
        other => panic!("unexpected log: {other:?}"),
    }
    match rx.recv().expect("stderr log should be sent") {
        super::TaskLogStream::Stderr(value) => assert_eq!(value, "err"),
        other => panic!("unexpected log: {other:?}"),
    }
    assert!(rx.try_recv().is_err());
}

#[test]
fn truncate_for_log_preview_preserves_content_and_escapes_newlines() {
    assert_eq!(super::truncate_for_log_preview("a\nb\nc", 1), "a\\nb\\nc");
}
