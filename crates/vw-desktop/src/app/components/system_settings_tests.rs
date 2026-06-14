// Tests for plan6 task 827.
const SOURCE: &str = include_str!("system_settings.rs");

use super::system_settings::{SystemTab, system_tab_matches_query};

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
fn system_settings_tests_keeps_planned_coverage_targets() {
    for name in ["SystemTab", "all", "fmt", "active_tab_help_modal_open", "view"] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn system_settings_search_matches_child_items() {
    assert!(system_tab_matches_query(SystemTab::General, "语言"));
    assert!(system_tab_matches_query(SystemTab::Agents, "API 密钥"));
    assert!(system_tab_matches_query(SystemTab::Scheduler, "最大并发"));
    assert!(system_tab_matches_query(SystemTab::Acp, "初始化"));
    assert!(system_tab_matches_query(SystemTab::WebSearch, "Brave 密钥"));
    assert!(system_tab_matches_query(SystemTab::HttpRequest, "允许域名"));
}

#[test]
fn system_tab_all_contains_every_display_label_once_in_order() {
    let tabs = SystemTab::all();

    assert_eq!(tabs.len(), 39);
    assert_eq!(tabs.first(), Some(&SystemTab::General));
    assert_eq!(tabs.last(), Some(&SystemTab::Transcription));
    assert_eq!(tabs.iter().filter(|tab| **tab == SystemTab::HttpRequest).count(), 1);

    let labels = tabs.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert!(labels.contains(&"常规设置".to_string()));
    assert!(labels.contains(&"ACP 配置".to_string()));
    assert!(labels.contains(&"委托代理配置".to_string()));
    assert!(labels.contains(&"转录配置".to_string()));
}

#[test]
fn system_tab_search_is_trimmed_case_insensitive_and_empty_matches_all() {
    assert!(system_tab_matches_query(SystemTab::Providers, "  API KEY "));
    assert!(system_tab_matches_query(SystemTab::Acp, "CODEX"));
    assert!(system_tab_matches_query(SystemTab::General, "   "));
    assert!(!system_tab_matches_query(SystemTab::General, "definitely-not-a-tab"));
}
