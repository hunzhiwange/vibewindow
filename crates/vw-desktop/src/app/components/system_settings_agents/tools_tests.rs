use super::tools::{
    ToolGroup, preset_tool_count, tool_card_button_style, tool_english_name, tool_group_meta,
    tool_group_order, tool_in_preset, tool_matches_any, tool_meta, tool_preset_meta,
};
use iced::widget::button;
use iced::{Background, Theme};

#[test]
fn tool_in_preset_rejects_unknown_preset() {
    assert!(!tool_in_preset("shell", "missing"));
}

#[test]
fn tool_in_preset_covers_all_presets() {
    assert!(tool_in_preset("read", "minimal"));
    assert!(tool_in_preset("apply_patch", "coding"));
    assert!(tool_in_preset("web_search", "research"));
    assert!(tool_in_preset("memory_store", "collab"));
    assert!(tool_in_preset("totally_custom", "full"));

    assert!(!tool_in_preset("bash", "minimal"));
    assert!(!tool_in_preset("http_request", "coding"));
    assert!(!tool_in_preset("memory_store", "research"));
}

#[test]
fn preset_tool_count_counts_available_matches() {
    let tools = vec![
        "read".to_string(),
        "bash".to_string(),
        "web_search".to_string(),
        "unknown".to_string(),
    ];

    assert_eq!(preset_tool_count(&tools, "minimal"), 1);
    assert_eq!(preset_tool_count(&tools, "coding"), 2);
    assert_eq!(preset_tool_count(&tools, "research"), 2);
    assert_eq!(preset_tool_count(&tools, "full"), 4);
    assert_eq!(preset_tool_count(&tools, "missing"), 0);
}

#[test]
fn tool_group_metadata_and_order_cover_all_groups() {
    assert_eq!(
        tool_group_order(),
        [
            ToolGroup::Files,
            ToolGroup::Search,
            ToolGroup::Execute,
            ToolGroup::Web,
            ToolGroup::Collaboration,
            ToolGroup::Memory,
            ToolGroup::Integration,
            ToolGroup::Other,
        ]
    );

    for group in tool_group_order() {
        let meta = tool_group_meta(group);
        assert_eq!(meta.group, group);
        assert!(!meta.label.is_empty());
        assert!(!meta.description.is_empty());
    }
}

#[test]
fn tool_preset_metadata_is_stable() {
    let presets = tool_preset_meta();

    assert_eq!(presets.len(), 5);
    assert_eq!(presets[0].key, "minimal");
    assert_eq!(presets[4].key, "full");
    assert!(presets.iter().all(|preset| !preset.label.is_empty()));
    assert!(presets.iter().all(|preset| !preset.description.is_empty()));
}

#[test]
fn tool_meta_classifies_known_aliases_and_unknown_tools() {
    assert_eq!(tool_meta("read").group, ToolGroup::Files);
    assert_eq!(tool_meta("file_read").name, "读取文件");
    assert_eq!(tool_meta("grep").group, ToolGroup::Search);
    assert_eq!(tool_meta("shell").group, ToolGroup::Execute);
    assert_eq!(tool_meta("websearch").group, ToolGroup::Web);
    assert_eq!(tool_meta("AgentTool").group, ToolGroup::Collaboration);
    assert_eq!(tool_meta("memory_recall").group, ToolGroup::Memory);
    assert_eq!(tool_meta("model_routing_config").group, ToolGroup::Integration);
    assert_eq!(tool_meta("some_new_tool").group, ToolGroup::Other);
    assert_eq!(tool_meta("some_new_tool").name, "未分类工具");
}

#[test]
fn tool_matching_and_english_name_handle_edge_cases() {
    assert!(tool_matches_any("read", &["write", "read"]));
    assert!(!tool_matches_any("read", &["write"]));
    assert_eq!(tool_english_name("web_search-tool"), "Web Search Tool");
    assert_eq!(tool_english_name("__read--file__"), "Read File");
}

#[test]
fn tool_card_button_style_covers_selected_and_unselected_states() {
    let selected = tool_card_button_style(&Theme::Light, button::Status::Active, true);
    let hovered = tool_card_button_style(&Theme::Light, button::Status::Hovered, false);
    let pressed_dark = tool_card_button_style(&Theme::Dark, button::Status::Pressed, false);

    assert!(matches!(selected.background, Some(Background::Color(_))));
    assert!(matches!(hovered.background, Some(Background::Color(_))));
    assert!(matches!(pressed_dark.background, Some(Background::Color(_))));
    assert_eq!(selected.border.radius.top_left, 16.0);
}
