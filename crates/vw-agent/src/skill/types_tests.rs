use super::*;

#[test]
fn default_download_policy_is_deny_by_default_for_domains() {
    let policy = SkillDownloadPolicy::default();
    assert_eq!(policy.version, 1);
    assert!(policy.trusted_domains.is_empty());
    assert!(policy.blocked_domains.is_empty());
    assert!(policy.aliases.contains_key("skill-creator"));
}

#[test]
fn skills_sh_source_formats_github_clone_url() {
    let source = SkillsShSource {
        owner: "owner".to_string(),
        repo: "repo".to_string(),
        skill: "skill".to_string(),
    };
    assert_eq!(source.github_repo_url(), "https://github.com/owner/repo.git");
}
