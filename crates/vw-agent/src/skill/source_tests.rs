use super::*;

#[test]
fn extracts_hosts_from_supported_git_urls() {
    assert_eq!(
        extract_link_host("zip:ssh://git@github.com:22/org/repo.git").as_deref(),
        Some("github.com")
    );
    assert_eq!(
        extract_link_host("https://Example.COM/org/repo?x=1").as_deref(),
        Some("example.com")
    );
    assert_eq!(extract_link_host("not-a-url"), None);
}

#[test]
fn skills_sh_source_rejects_traversal() {
    let parsed = parse_skills_sh_source("https://skills.sh/openai/skills/rust");
    assert_eq!(parsed.unwrap().github_repo_url(), "https://github.com/openai/skills.git");
    assert!(parse_skills_sh_source("https://skills.sh/openai/../secret/rust").is_none());
    assert!(parse_skills_sh_source("https://example.com/openai/skills/rust").is_none());
}

#[test]
fn trust_check_urls_include_direct_and_skills_sh_github_repo_once() {
    assert_eq!(
        source_urls_for_trust_check("https://github.com/acme/skill.git"),
        vec!["https://github.com/acme/skill.git".to_string()]
    );
    assert_eq!(
        source_urls_for_trust_check("https://skills.sh/acme/repo/rust"),
        vec![
            "https://skills.sh/acme/repo/rust".to_string(),
            "https://github.com/acme/repo.git".to_string(),
        ]
    );
    assert!(source_urls_for_trust_check("local-alias").is_empty());
}

#[test]
fn skills_sh_parsing_ignores_query_fragment_and_normalizes_dir_names() {
    let parsed = parse_skills_sh_source("https://skills.sh/Owner/Repo/My_Skill?ref=main#readme")
        .expect("skills.sh source");

    assert_eq!(parsed.owner, "Owner");
    assert_eq!(parsed.repo, "Repo");
    assert_eq!(parsed.skill, "My_Skill");
    assert!(is_skills_sh_source("https://skills.sh/owner/repo/skill"));
    assert_eq!(normalize_skills_sh_dir_name("My Skill/@2026!"), "myskill2026");
}

#[test]
fn extract_host_rejects_empty_hosts_and_strips_ports() {
    assert_eq!(extract_link_host("https:///missing-host"), None);
    assert_eq!(extract_link_host("git://example.com:9418/repo").as_deref(), Some("example.com"));
}
