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
