use super::*;

#[test]
fn parse_skills_sh_source_accepts_query_and_fragment() {
    let source = parse_skills_sh_source("https://skills.sh/acme/tools/review?version=1#readme").unwrap();
    assert_eq!(source.owner, "acme");
    assert_eq!(source.repo, "tools");
    assert_eq!(source.skill, "review");
    assert_eq!(source.github_repo_url(), "https://github.com/acme/tools.git");
}

#[test]
fn parse_skills_sh_source_rejects_unsafe_or_wrong_hosts() {
    assert!(parse_skills_sh_source("https://example.com/acme/tools/review").is_none());
    assert!(parse_skills_sh_source("https://skills.sh/acme/../review").is_none());
    assert!(parse_skills_sh_source("https://skills.sh/acme/tools/re\\view").is_none());
}

#[test]
fn normalize_skills_sh_dir_name_keeps_only_safe_chars() {
    assert_eq!(normalize_skills_sh_dir_name("My Skill_01!"), "myskill_01");
    assert!(is_skills_sh_source("https://skills.sh/acme/tools/review"));
}
