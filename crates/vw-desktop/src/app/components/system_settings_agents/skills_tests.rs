const SOURCE: &str = include_str!("skills.rs");

#[test]
fn skills_view_declares_selection_helpers() {
    for name in [
        "skill_card_button_style",
        "skill_source_label",
        "scope_source_matches",
        "scope_description",
        "discovery_order_text",
        "scope_button",
        "enabled_skill_ids",
        "skill_card",
    ] {
        assert!(SOURCE.contains(name), "expected skills view to declare {name}");
    }
}
