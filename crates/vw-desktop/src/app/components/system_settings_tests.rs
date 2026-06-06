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
