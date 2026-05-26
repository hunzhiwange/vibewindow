use super::builtins::builtin_formatters;

#[test]
fn builtin_formatters_include_core_languages() {
    let formatters = builtin_formatters();

    assert!(formatters.contains_key("prettier"));
    assert!(formatters.contains_key("gofmt"));
    assert!(formatters["prettier"].extensions.contains(&".ts".to_string()));
}
