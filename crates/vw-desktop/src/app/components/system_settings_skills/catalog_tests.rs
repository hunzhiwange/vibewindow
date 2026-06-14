// Tests for plan6 task 820.
const SOURCE: &str = include_str!("catalog.rs");

use super::catalog::{
    catalog_group_section, catalog_item, catalog_matches_query, catalog_skill_initials,
    section_card_style, section_copy, skill_badge, source_label,
};
use crate::app::state::{
    SkillsCatalogItem as CatalogSkillMeta, SkillsCatalogKind as CatalogSkillKind,
};

fn skill(
    id: &str,
    title: &str,
    description: &str,
    source: &str,
    source_path: Option<&str>,
) -> CatalogSkillMeta {
    CatalogSkillMeta {
        id: id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        kind: CatalogSkillKind::Personal,
        resource_count: 0,
        installed: true,
        enabled: true,
        source: source.to_string(),
        source_path: source_path.map(ToOwned::to_owned),
    }
}

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn catalog_matches_query_searches_all_visible_fields_case_insensitively() {
    let item = skill(
        "rust-helper",
        "Code Review",
        "Finds risky changes",
        "workspace",
        Some("/repo/.vibewindow/skills/review"),
    );

    for query in ["", "RUST", "review", "RISKY", "WORKSPACE", "VIBEWINDOW"] {
        assert!(catalog_matches_query(&item, query), "query should match: {query}");
    }

    assert!(!catalog_matches_query(&item, "browser"));
}

#[test]
fn catalog_matches_query_handles_missing_source_path() {
    let item = skill("id", "Title", "Description", "bundled", None);

    assert!(catalog_matches_query(&item, "bundle"));
    assert!(!catalog_matches_query(&item, "missing/path"));
}

#[test]
fn catalog_skill_initials_prefers_title_words_and_falls_back_to_id() {
    let titled = skill("fallback-id", "agent_builder daily", "", "global", None);
    let untitled = skill("xy-tool", "", "", "global", None);

    assert_eq!(catalog_skill_initials(&titled), "AD");
    assert_eq!(catalog_skill_initials(&untitled), "XY");
}

#[test]
fn source_labels_and_section_copy_cover_known_and_unknown_sources() {
    let cases = [
        ("workspace", "本目录", "项目目录"),
        ("ancestor", "父级目录", "父级目录"),
        ("global", "全局目录", "全局目录"),
        ("bundled", "内置技能", "内置技能"),
        ("other", "来源", "其他来源"),
    ];

    for (source, label, title) in cases {
        let (_, section_title, subtitle) = section_copy(source);
        assert_eq!(source_label(source), label);
        assert_eq!(section_title, title);
        assert!(!subtitle.is_empty());
    }
}

#[test]
fn catalog_widgets_build_for_badges_items_groups_and_styles() {
    let _ = section_card_style(&iced::Theme::Dark);
    let _ = skill_badge("Enabled", true);
    let item = skill("local", "Local Skill", "Description", "workspace", Some("/tmp/skill"));
    let _ = catalog_item(item.clone(), true);
    let _ = catalog_group_section("workspace", vec![item], Some("local"));
}

#[test]
fn catalog_tests_keeps_planned_coverage_targets() {
    for name in [
        "catalog_matches_query",
        "section_card_style",
        "skill_badge",
        "catalog_skill_initials",
        "section_copy",
        "source_label",
        "catalog_item",
        "catalog_group_section",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
