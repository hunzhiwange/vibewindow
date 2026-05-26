use super::*;
use crate::app::agent::skill::types::{Skill, SkillsPromptInjectionMode};

#[test]
fn prompt_escapes_xml_and_compact_uses_relative_location() {
    let skill = Skill {
        name: "review&fix".to_string(),
        description: "Use <care>".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        tags: Vec::new(),
        tools: Vec::new(),
        prompts: vec!["Keep \"scope\" tight".to_string()],
        location: None,
    };
    let prompt = skills_to_prompt_with_mode(
        &[skill],
        std::path::Path::new("/workspace"),
        SkillsPromptInjectionMode::Compact,
    );

    assert!(prompt.contains("review&amp;fix"));
    assert!(prompt.contains("Use &lt;care&gt;"));
    assert!(prompt.contains("skills/review&amp;fix/SKILL.md"));
    assert!(!prompt.contains("<tools>"));
}
