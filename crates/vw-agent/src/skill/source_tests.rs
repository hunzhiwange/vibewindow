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
