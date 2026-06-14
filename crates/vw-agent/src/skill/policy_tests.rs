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

#[test]
fn open_skills_enabled_sources_parse_env_like_values() {
    assert_eq!(parse_open_skills_enabled(" yes "), Some(true));
    assert_eq!(parse_open_skills_enabled("OFF"), Some(false));
    assert_eq!(parse_open_skills_enabled("maybe"), None);

    assert!(open_skills_enabled_from_sources(Some(false), Some("true")));
    assert!(open_skills_enabled_from_sources(Some(true), Some("bad-value")));
    assert!(!open_skills_enabled_from_sources(None, None));
}

#[test]
fn load_or_init_policy_creates_normalizes_and_preserves_aliases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skills_path = dir.path().join("skills");
    std::fs::create_dir_all(&skills_path).unwrap();

    let policy = load_or_init_skill_download_policy(&skills_path).unwrap();
    assert!(policy.aliases.contains_key("find-skills"));
    assert!(download_policy_path(&skills_path).is_file());

    std::fs::write(
        download_policy_path(&skills_path),
        r#"
        version = 1
        trusted_domains = [" HTTPS://GitHub.COM/org/repo ", "*.example.com:443"]
        blocked_domains = [" BAD.example.com/path ", "bad.example.com"]

        [aliases]
        local = "/tmp/local"
        "#,
    )
    .unwrap();

    let policy = load_or_init_skill_download_policy(&skills_path).unwrap();

    assert_eq!(policy.trusted_domains, vec!["example.com", "github.com"]);
    assert_eq!(policy.blocked_domains, vec!["bad.example.com"]);
    assert_eq!(policy.aliases.get("local").map(String::as_str), Some("/tmp/local"));
    assert!(policy.aliases.contains_key("skill-creator"));
}

#[test]
fn save_policy_normalizes_without_mutating_input_and_alias_resolution_trims() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skills_path = dir.path();
    let mut policy = SkillDownloadPolicy::default();
    policy.trusted_domains = vec!["*.Example.com".to_string(), "example.com".to_string()];
    policy.aliases.insert("alias".to_string(), "https://github.com/acme/skill.git".to_string());

    save_skill_download_policy(skills_path, &policy).unwrap();
    assert_eq!(policy.trusted_domains.len(), 2);

    let loaded = load_or_init_skill_download_policy(skills_path).unwrap();
    assert_eq!(loaded.trusted_domains, vec!["example.com"]);
    assert_eq!(resolve_skill_source_alias(" alias ", &loaded), "https://github.com/acme/skill.git");
    assert_eq!(resolve_skill_source_alias("unknown", &loaded), "unknown");
}

#[test]
fn source_domain_trust_allows_trusted_and_rejects_blocked_domains() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut policy = SkillDownloadPolicy::default();
    policy.trusted_domains = vec!["github.com".to_string()];
    policy.blocked_domains = vec!["evil.example".to_string()];

    ensure_source_domain_trust("https://api.github.com/acme/skill.git", &mut policy, dir.path())
        .unwrap();

    let err =
        ensure_source_domain_trust("https://evil.example/acme/skill.git", &mut policy, dir.path())
            .expect_err("blocked domain must fail");
    assert!(err.to_string().contains("explicitly blocked"));
}
