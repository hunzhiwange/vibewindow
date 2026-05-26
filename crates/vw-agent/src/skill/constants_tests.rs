use super::*;

#[test]
fn defaults_are_stable_and_include_builtin_aliases() {
    let aliases = default_preloaded_skill_aliases();
    assert_eq!(default_policy_version(), 1);
    assert_eq!(default_version(), "0.1.0");
    assert_eq!(
        aliases.get("find-skills").map(String::as_str),
        Some("https://skills.sh/vercel-labs/skills/find-skills")
    );
    assert_eq!(BUILTIN_PRELOADED_SKILLS.len(), DEFAULT_PRELOADED_SKILL_SOURCES.len());
}
