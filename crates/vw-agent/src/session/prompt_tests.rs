#[test]
fn prompt_tests_module_is_wired() {
    let marker = String::from("prompt_tests");
    assert_eq!(marker.as_str(), "prompt_tests");
}

#[test]
fn max_steps_text_is_trimmed_and_non_empty() {
    let text = super::max_steps_text();

    assert!(!text.is_empty());
    assert_eq!(text, text.trim());
}

#[test]
fn block_on_runs_future_without_existing_runtime() {
    let value = super::block_on(async { 42 });

    assert_eq!(value, 42);
}
