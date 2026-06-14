use super::*;
use crate::app::agent::skills::types::SkillDownloadPolicy;

#[test]
fn normalize_domain_list_sorts_deduplicates_and_strips_url_parts() {
    let mut entries = vec![
        " HTTPS://Example.COM/path ".to_string(),
        "*.example.com".to_string(),
        "docs.example.com:443".to_string(),
        "".to_string(),
    ];
    normalize_domain_list(&mut entries);
    assert_eq!(entries, vec!["docs.example.com", "example.com"]);
}

#[test]
fn trusted_domain_matches_exact_and_subdomains_only() {
    assert!(host_matches_trusted_domain("example.com", "example.com"));
    assert!(host_matches_trusted_domain("docs.example.com", "example.com"));
    assert!(!host_matches_trusted_domain("badexample.com", "example.com"));
}

#[test]
fn source_urls_for_trust_check_expands_skills_sh_source() {
    let urls = source_urls_for_trust_check("https://skills.sh/acme/tools/review");
    assert_eq!(
        urls,
        vec![
            "https://skills.sh/acme/tools/review".to_string(),
            "https://github.com/acme/tools.git".to_string(),
        ]
    );
}

#[test]
fn resolve_skill_source_alias_trims_lookup_key() {
    let mut policy = SkillDownloadPolicy::default();
    policy.aliases.insert("demo".into(), "https://example.com/demo.git".into());
    assert_eq!(resolve_skill_source_alias(" demo ", &policy), "https://example.com/demo.git");
    assert_eq!(resolve_skill_source_alias("raw", &policy), "raw");
}

#[test]
fn policy_load_save_initializes_defaults_and_normalizes_domains() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = SkillDownloadPolicy {
        version: 7,
        aliases: Default::default(),
        trusted_domains: vec!["HTTPS://Example.COM/path".into()],
        blocked_domains: vec!["*.Bad.COM:443".into(), "bad.com".into()],
    };

    save_skill_download_policy(dir.path(), &policy).unwrap();
    policy.trusted_domains.clear();
    let loaded = load_or_init_skill_download_policy(dir.path()).unwrap();

    assert_eq!(loaded.version, 7);
    assert_eq!(loaded.trusted_domains, vec!["example.com"]);
    assert_eq!(loaded.blocked_domains, vec!["bad.com"]);
    assert!(loaded.aliases.contains_key("find-skills"));
}

#[test]
fn malformed_policy_falls_back_to_default_aliases() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".download-policy.toml"), "not valid = [").unwrap();

    let policy = load_or_init_skill_download_policy(dir.path()).unwrap();

    assert!(policy.aliases.contains_key("find-skills"));
    assert!(policy.trusted_domains.is_empty());
}

#[test]
fn extract_link_host_supports_common_remote_source_shapes() {
    assert_eq!(
        extract_link_host("zip:https://User@Example.COM:443/path?x=1").as_deref(),
        Some("example.com")
    );
    assert_eq!(
        extract_link_host("ssh://git@github.com/acme/repo.git").as_deref(),
        Some("github.com")
    );
    assert_eq!(extract_link_host("git://github.com/acme/repo.git").as_deref(), Some("github.com"));
    assert_eq!(extract_link_host("file:///tmp/skill"), None);
}

#[test]
fn ensure_source_domain_trust_allows_local_and_trusted_sources() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = SkillDownloadPolicy::default();

    ensure_source_domain_trust("/tmp/local-skill", &mut policy, dir.path()).unwrap();

    policy.trusted_domains = vec!["github.com".into(), "skills.sh".into()];
    ensure_source_domain_trust("https://skills.sh/acme/tools/review", &mut policy, dir.path())
        .unwrap();
}

#[test]
fn ensure_source_domain_trust_rejects_blocked_and_unknown_noninteractive_sources() {
    let dir = tempfile::tempdir().unwrap();
    let mut blocked = SkillDownloadPolicy::default();
    blocked.blocked_domains = vec!["example.com".into()];

    let err = ensure_source_domain_trust("https://example.com/skill.git", &mut blocked, dir.path())
        .unwrap_err();
    assert!(err.to_string().contains("explicitly blocked"));

    let mut unknown = SkillDownloadPolicy::default();
    let err = ensure_source_domain_trust_with_interactivity(
        "https://unknown.invalid/skill.git",
        &mut unknown,
        dir.path(),
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("untrusted domain"));
}
