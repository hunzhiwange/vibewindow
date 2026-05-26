use super::*;
use crate::app::agent::skill::constants::SKILL_DOWNLOAD_POLICY_FILE;
use std::path::Path;

#[test]
fn normalizes_domain_lists_and_matches_subdomains() {
    let mut domains = vec![
        " HTTPS://GitHub.COM/org/repo ".to_string(),
        "*.example.com:443".to_string(),
        "github.com".to_string(),
        " ".to_string(),
    ];
    normalize_domain_list(&mut domains);

    assert_eq!(domains, vec!["example.com", "github.com"]);
    assert!(host_matches_trusted_domain("api.github.com", "github.com"));
    assert!(!host_matches_trusted_domain("github.io", "github.com"));
}

#[test]
fn policy_path_uses_stable_file_name() {
    assert_eq!(
        download_policy_path(Path::new("/tmp/skills")),
        Path::new("/tmp/skills").join(SKILL_DOWNLOAD_POLICY_FILE)
    );
}
