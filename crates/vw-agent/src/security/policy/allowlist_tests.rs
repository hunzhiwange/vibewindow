use super::*;

#[test]
fn allowlist_matches_exact_base_and_wildcard() {
    assert!(is_allowlist_entry_match("git", "/usr/bin/git", "git"));
    assert!(is_allowlist_entry_match("*", "/usr/bin/rm", "rm"));
    assert!(!is_allowlist_entry_match("cargo", "/usr/bin/git", "git"));
}

#[test]
fn allowlist_path_entries_expand_home_and_quotes() {
    let home = std::env::var("HOME").expect("HOME should be set for path expansion tests");
    let executable = format!("{home}/bin/tool");

    assert!(is_allowlist_entry_match("\"~/bin/tool\"", &executable, "tool"));
    assert!(!is_allowlist_entry_match("", &executable, "tool"));
    assert!(!is_allowlist_entry_match("~/bin/other", &executable, "tool"));
}

#[test]
fn forbidden_path_arguments_include_flags_and_redirects() {
    let denied = |path: &str| !path.contains("secret") && !path.contains("outside");

    assert_eq!(
        find_forbidden_path_argument("cat ./ok.txt ./secret.txt", denied),
        Some("./secret.txt".to_string())
    );
    assert_eq!(
        find_forbidden_path_argument("grep needle --path=../outside/file", denied),
        Some("../outside/file".to_string())
    );
    assert_eq!(
        find_forbidden_path_argument("cmd -o./secret-output", denied),
        Some("./secret-output".to_string())
    );
    assert_eq!(
        find_forbidden_path_argument("echo hi >./secret.log", denied),
        Some("./secret.log".to_string())
    );
}

#[test]
fn forbidden_path_arguments_ignore_urls_empty_values_and_safe_paths() {
    fn always_deny(_: &str) -> bool {
        false
    }

    assert_eq!(find_forbidden_path_argument("curl https://example.com/a/b", always_deny), None);
    assert_eq!(find_forbidden_path_argument("echo plain words", always_deny), None);
    assert_eq!(
        find_forbidden_path_argument("cat ./allowed.txt", |path| path == "./allowed.txt"),
        None
    );
}
