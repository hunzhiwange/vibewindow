use super::*;
use crate::app::agent::config::SkillsPromptInjectionMode;
use crate::app::agent::skills::types::Skill;

#[test]
fn compact_prompt_escapes_xml_and_omits_full_sections() {
    let skill = Skill {
        name: "docs&review".to_string(),
        description: "Read <docs>".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        tags: Vec::new(),
        tools: Vec::new(),
        prompts: vec!["full instructions".to_string()],
        location: None,
    };
    let prompt = skills_to_prompt_with_mode(
        &[skill],
        std::path::Path::new("/workspace"),
        SkillsPromptInjectionMode::Compact,
    );

    assert!(prompt.contains("docs&amp;review"));
    assert!(prompt.contains("Read &lt;docs&gt;"));
    assert!(prompt.contains("skills/docs&amp;review/SKILL.md"));
    assert!(!prompt.contains("<instructions>"));
}
