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

#[test]
fn runtime_and_skill_config_defaults_are_conservative() {
    let config = SkillsConfig::default();
    assert!(!config.open_skills_enabled);
    assert!(config.open_skills_dir.is_none());
    assert_eq!(config.prompt_injection_mode, SkillsPromptInjectionMode::Full);

    let runtime = SkillRuntimeConfig::default();
    assert_eq!(runtime.workspace_dir, PathBuf::from("."));
    assert!(!runtime.skills.open_skills_enabled);
}

#[test]
fn load_mode_tracks_prompt_injection_mode() {
    assert_eq!(
        SkillLoadMode::from_prompt_mode(SkillsPromptInjectionMode::Full),
        SkillLoadMode::Full
    );
    assert_eq!(
        SkillLoadMode::from_prompt_mode(SkillsPromptInjectionMode::Compact),
        SkillLoadMode::MetadataOnly
    );
}

#[test]
fn skill_manifest_deserializes_defaults_prompts_and_tools() {
    let manifest: SkillManifest = toml::from_str(
        r#"
        prompts = ["Check carefully"]

        [skill]
        name = "lint"
        description = "Lint code"

        [[tools]]
        name = "cargo_check"
        description = "Run cargo check"
        kind = "shell"
        command = "cargo check"
        [tools.args]
        package = "vw-agent"
        "#,
    )
    .unwrap();

    assert_eq!(manifest.skill.version, "0.1.0");
    assert_eq!(manifest.prompts, vec!["Check carefully"]);
    assert_eq!(manifest.tools.len(), 1);
    assert_eq!(manifest.tools[0].args.get("package").map(String::as_str), Some("vw-agent"));
}

#[test]
fn public_skill_serializes_without_runtime_location() {
    let skill = Skill {
        name: "demo".to_string(),
        description: "Demo".to_string(),
        version: "1.2.3".to_string(),
        author: Some("me".to_string()),
        tags: vec!["test".to_string()],
        tools: Vec::new(),
        prompts: vec!["Use it".to_string()],
        location: Some(PathBuf::from("/tmp/secret/SKILL.md")),
    };

    let value = serde_json::to_value(&skill).unwrap();

    assert_eq!(value["name"], "demo");
    assert!(value.get("location").is_none());
}
