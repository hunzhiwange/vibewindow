// Tests for plan6 task 819.
const SOURCE: &str = include_str!("browser.rs");

use super::browser::DetailActionStyle;
use super::browser::{
    active_scope_badge, catalog_panel_style, detail_action_button, detail_source_note, empty_state,
    header_panel_style, loading_banner, provider_global_paths, provider_project_paths,
    refresh_button, scope_button, scope_description, scope_source_matches, scope_title,
    search_bar_style, search_text_input_style, status_banner,
};
use crate::app::assets::Icon;
use crate::app::message;
use crate::app::state::SkillsDirectoryScope;
use crate::app::{Message, message::SettingsMessage};
use iced::widget::text_input;
use vw_config_types::skills::SkillsDirectoryProvider;

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
fn scope_source_matching_is_explicit_for_each_scope() {
    assert!(scope_source_matches(SkillsDirectoryScope::Project, "workspace"));
    assert!(scope_source_matches(SkillsDirectoryScope::Ancestor, "ancestor"));
    assert!(scope_source_matches(SkillsDirectoryScope::Global, "global"));
    assert!(scope_source_matches(SkillsDirectoryScope::Bundled, "bundled"));
    assert!(scope_source_matches(SkillsDirectoryScope::All, "anything"));

    assert!(!scope_source_matches(SkillsDirectoryScope::Project, "global"));
    assert!(!scope_source_matches(SkillsDirectoryScope::Bundled, "workspace"));
}

#[test]
fn scope_titles_and_badges_cover_all_scopes() {
    let cases = [
        (SkillsDirectoryScope::Project, "项目目录技能", "当前筛选: 项目目录"),
        (SkillsDirectoryScope::Ancestor, "父级目录技能", "当前筛选: 父级目录"),
        (SkillsDirectoryScope::Global, "全局目录技能", "当前筛选: 全局目录"),
        (SkillsDirectoryScope::Bundled, "内置技能", "当前筛选: 内置技能"),
        (SkillsDirectoryScope::All, "全部技能目录", "当前筛选: 全部目录"),
    ];

    for (scope, title, badge) in cases {
        assert_eq!(scope_title(scope), title);
        assert_eq!(active_scope_badge(scope), badge);
    }
}

#[test]
fn provider_paths_match_directory_provider_contract() {
    let vibewindow_global =
        format!("~/{}/skills 与 ~/.skills", vw_config_types::paths::HOME_CONFIG_DIR_NAME);
    let cases = [
        (
            SkillsDirectoryProvider::Vibewindow,
            ".vibewindow/skills 与 skills",
            vibewindow_global.as_str(),
        ),
        (
            SkillsDirectoryProvider::Codex,
            ".codex/skills 与 .agents/skills",
            "~/.codex/skills 与 ~/.agents/skills",
        ),
        (SkillsDirectoryProvider::Claude, ".claude/skills", "~/.claude/skills"),
        (SkillsDirectoryProvider::Cursor, ".cursor/skills", "~/.cursor/skills"),
    ];

    for (provider, project, global) in cases {
        assert_eq!(provider_project_paths(provider), project);
        assert_eq!(provider_global_paths(provider), global);
    }
}

#[test]
fn scope_descriptions_include_provider_specific_paths() {
    assert!(
        scope_description(SkillsDirectoryScope::Project, SkillsDirectoryProvider::Codex)
            .contains(".codex/skills")
    );
    assert!(
        scope_description(SkillsDirectoryScope::Ancestor, SkillsDirectoryProvider::Claude)
            .contains(".claude/skills")
    );
    assert!(
        scope_description(SkillsDirectoryScope::Global, SkillsDirectoryProvider::Cursor)
            .contains("~/.cursor/skills")
    );
    assert!(
        scope_description(SkillsDirectoryScope::Bundled, SkillsDirectoryProvider::Vibewindow)
            .contains("内置技能")
    );
    assert!(
        scope_description(SkillsDirectoryScope::All, SkillsDirectoryProvider::Vibewindow)
            .contains("全部")
    );
}

#[test]
fn browser_widgets_build_for_status_actions_and_common_states() {
    let _ = search_bar_style(&iced::Theme::Dark);
    let _ = header_panel_style(&iced::Theme::Dark);
    let _ = catalog_panel_style(&iced::Theme::Dark);
    let _ = search_text_input_style(&iced::Theme::Dark, text_input::Status::Active);
    let _ = search_text_input_style(&iced::Theme::Dark, text_input::Status::Disabled);

    let _ = status_banner("同步完成", false);
    let _ = status_banner("同步失败", true);
    let _ =
        scope_button("全局目录", SkillsDirectoryScope::Global, SkillsDirectoryScope::Project, true);
    let _ = scope_button(
        "项目目录",
        SkillsDirectoryScope::Project,
        SkillsDirectoryScope::Project,
        false,
    );
    let _ = refresh_button(false);
    let _ = refresh_button(true);
    let _ = loading_banner();
    let _ = empty_state("空", "没有结果");
    let _ = detail_source_note(Some("/tmp/skill"));
    let _ = detail_source_note(None);
    let _ = detail_action_button(
        "执行",
        Icon::Plus,
        DetailActionStyle::Primary,
        Some(Message::Settings(SettingsMessage::SkillsRefresh)),
    );
    let _ = detail_action_button("删除", Icon::Trash, DetailActionStyle::Danger, None);
    let _ = detail_action_button(
        "次要",
        Icon::ArrowRepeat,
        DetailActionStyle::Secondary,
        Some(Message::Settings(message::SettingsMessage::SkillsRefresh)),
    );
}

#[test]
fn browser_tests_keeps_planned_coverage_targets() {
    for name in [
        "DetailActionStyle",
        "search_bar_style",
        "header_panel_style",
        "catalog_panel_style",
        "status_banner",
        "scope_button",
        "refresh_button",
        "scope_source_matches",
        "scope_title",
        "scope_description",
        "active_scope_badge",
        "discovery_order_text",
        "loading_banner",
        "empty_state",
        "detail_source_note",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
