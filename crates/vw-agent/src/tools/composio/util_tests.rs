use super::*;

#[test]
fn ensure_https_rejects_non_https_urls() {
    assert!(ensure_https("https://api.example.com").is_ok());
    assert!(ensure_https("http://api.example.com").is_err());
}

#[test]
fn normalizers_trim_case_and_collapse_empty_values() {
    assert_eq!(normalize_entity_id("  user  "), "user");
    assert_eq!(normalize_entity_id("   "), "default");
    assert_eq!(normalize_tool_slug("GITHUB_GET_REPO"), "github-get-repo");
    assert_eq!(normalize_app_slug("--GitHub_App--"), "github-app");
}

#[test]
fn cache_key_helpers_are_stable() {
    assert_eq!(connected_account_cache_key("GitHub_App", " user "), "user:github-app");
    assert_eq!(normalize_action_cache_key("--GITHUB__GET--"), Some("github-get".into()));
    assert_eq!(normalize_action_cache_key("   "), None);
}

#[test]
fn tool_slug_candidates_keep_original_first_and_deduplicate() {
    let candidates = build_tool_slug_candidates("GITHUB_GET_REPO");
    assert_eq!(candidates.first().map(String::as_str), Some("GITHUB_GET_REPO"));
    assert!(candidates.contains(&"github-get-repo".to_string()));
    assert_eq!(candidates.iter().filter(|item| item.as_str() == "github-get-repo").count(), 1);
}
