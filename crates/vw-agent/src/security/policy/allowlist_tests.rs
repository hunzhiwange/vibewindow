use super::*;

#[test]
fn allowlist_matches_exact_base_and_wildcard() {
    assert!(is_allowlist_entry_match("git", "/usr/bin/git", "git"));
    assert!(is_allowlist_entry_match("*", "/usr/bin/rm", "rm"));
    assert!(!is_allowlist_entry_match("cargo", "/usr/bin/git", "git"));
}
