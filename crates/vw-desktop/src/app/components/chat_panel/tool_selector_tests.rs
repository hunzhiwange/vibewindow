use super::tool_selector::{
    SESSION_SELECTOR_LIST_MAX_HEIGHT, SESSION_SELECTOR_SCROLLBAR_WIDTH,
    SESSION_SELECTOR_SKILL_DESCRIPTION_CHARS, ellipsize_text, session_tool_selector_popover,
};
use crate::app::App;
use crate::app::state::{SessionToolGroup, SessionToolSelectorTab, SkillsDirectoryScope};

#[test]
fn selector_scrollbar_width_matches_chat_design_contract() {
    assert_eq!(SESSION_SELECTOR_SCROLLBAR_WIDTH, 4.0);
}

#[test]
fn selector_list_has_positive_height_limit() {
    assert!(SESSION_SELECTOR_LIST_MAX_HEIGHT > 0.0);
}

#[test]
fn skill_description_preview_stays_compact() {
    assert!(SESSION_SELECTOR_SKILL_DESCRIPTION_CHARS <= 28);
}

#[test]
fn ellipsize_text_compacts_whitespace_and_uses_ascii_dots() {
    assert_eq!(ellipsize_text("alpha\n beta   gamma", 12), "alpha bet...");
    assert_eq!(ellipsize_text("short text", 20), "short text");
    assert_eq!(ellipsize_text("abcdef", 3), "...");
}

#[test]
fn selector_popover_renders_default_agent_tab() {
    let app = App::new().0;

    let _ = session_tool_selector_popover(&app);
}

#[test]
fn selector_popover_renders_tools_tab_with_empty_filtered_and_grouped_tools() {
    let mut app = App::new().0;
    app.current_session_runtime_mut().tool_selector.select_tab(SessionToolSelectorTab::Tools);

    let _ = session_tool_selector_popover(&app);

    app.agents_settings.available_tools =
        vec!["bash".to_string(), "read".to_string(), "web_search".to_string()];
    app.current_session_runtime_mut().tool_selector.toggle_group_collapsed(SessionToolGroup::Files);
    app.current_session_runtime_mut().tool_selector.set_query("bash".to_string());

    let _ = session_tool_selector_popover(&app);
}

#[test]
fn selector_popover_renders_skills_tab_loading_and_empty_scope_states() {
    let mut app = App::new().0;
    {
        let runtime = app.current_session_runtime_mut();
        runtime.tool_selector.select_tab(SessionToolSelectorTab::Skills);
        runtime.tool_selector.select_skill_directory_scope(SkillsDirectoryScope::Global);
    }
    app.skills_settings.loading = true;

    let _ = session_tool_selector_popover(&app);

    app.skills_settings.loading = false;
    let _ = session_tool_selector_popover(&app);
}
