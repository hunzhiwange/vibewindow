use super::*;

#[test]
fn default_policy_contains_stable_builtin_aliases() {
    let policy = SkillDownloadPolicy::default();
    assert_eq!(policy.version, default_policy_version());
    assert!(policy.aliases.contains_key("find-skills"));
    assert!(policy.aliases.contains_key("skill-creator"));
    assert!(policy.trusted_domains.is_empty());
    assert!(policy.blocked_domains.is_empty());
}

#[test]
fn skill_meta_deserialization_uses_default_version() {
    let manifest: SkillMetadataManifest = toml::from_str(
        r#"
        [skill]
        name = "demo"
        description = "Demo skill"
        "#,
    )
    .unwrap();
    assert_eq!(manifest.skill.version, default_version());
    assert_eq!(manifest.skill.tags, Vec::<String>::new());
}
