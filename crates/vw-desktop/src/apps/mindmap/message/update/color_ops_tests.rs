use super::color_ops::{
    cancel_theme_background, color_picker_changed, color_picker_format_changed,
    delete_custom_theme, open_color_picker, reset_color_target, rgba_u32_from_color,
    save_theme_to_custom, set_background, set_edge_style, set_node_border_style, set_theme_group,
    set_theme_variant,
};
use crate::app::App;
use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::canvas::theme::{CUSTOM_THEME_GROUP_ID, default_custom_themes};
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{EdgeStyle, MindMapColorPicker, MindMapColorTarget, MindMapTab};
use iced::Color;

fn app_with_tab() -> App {
    let mut app = App::new().0;
    app.mindmap_tabs.push(MindMapTab::new(
        "tab-1".to_string(),
        "Tab".to_string(),
        None,
        model::default_doc(),
    ));
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn rgba_u32_from_color_clamps_and_rounds_channels() {
    let color = Color { r: 1.2, g: 0.5, b: -0.2, a: 0.0 };

    assert_eq!(rgba_u32_from_color(color), 0xFF800000);
}

#[test]
fn open_color_picker_preserves_previous_format_and_closes_panels() {
    let mut app = app_with_tab();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.active_color_picker = Some(MindMapColorPicker {
        color: Color::BLACK,
        format: ColorFormat::Rgba,
        target: MindMapColorTarget::Background,
        picking: true,
    });
    tab.show_diagram_type_picker = true;
    tab.show_markdown_import = true;
    tab.show_zoom_menu = true;
    tab.show_priority_picker = true;
    tab.show_action_menu = true;
    tab.show_theme_panel = true;

    let _ =
        open_color_picker(&mut app, MindMapColorTarget::NodeFill, Color::from_rgb(0.25, 0.5, 0.75));

    let tab = app.active_mindmap_tab().unwrap();
    let picker = tab.active_color_picker.as_ref().unwrap();
    assert_eq!(picker.target, MindMapColorTarget::NodeFill);
    assert_eq!(picker.format, ColorFormat::Rgba);
    assert!(!picker.picking);
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_zoom_menu);
    assert!(!tab.show_priority_picker);
    assert!(!tab.show_action_menu);
    assert!(!tab.show_theme_panel);
}

#[test]
fn color_picker_changed_applies_selected_node_and_background_targets() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![0, 1]);

    for target in [
        MindMapColorTarget::NodeFill,
        MindMapColorTarget::NodeText,
        MindMapColorTarget::NodeBorder,
        MindMapColorTarget::EdgeStroke,
        MindMapColorTarget::Background,
    ] {
        let _ = open_color_picker(&mut app, target, Color::BLACK);
        let _ = color_picker_changed(&mut app, Color::from_rgba(0.1, 0.2, 0.3, 0.4));
    }

    let tab = app.active_mindmap_tab().unwrap();
    let rgba = rgba_u32_from_color(Color::from_rgba(0.1, 0.2, 0.3, 0.4));
    assert_eq!(tab.node_fills.get(&vec![0, 1]), Some(&rgba));
    assert_eq!(tab.node_text_colors.get(&vec![0, 1]), Some(&rgba));
    assert_eq!(tab.node_border_colors.get(&vec![0, 1]), Some(&rgba));
    assert_eq!(tab.edge_colors.get(&vec![0, 1]), Some(&rgba));
    assert_eq!(tab.background, Some(rgba));
}

#[test]
fn color_picker_changed_ignores_edge_color_for_root_path_and_missing_picker() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![]);

    let _ = color_picker_changed(&mut app, Color::WHITE);
    let _ = open_color_picker(&mut app, MindMapColorTarget::EdgeStroke, Color::BLACK);
    let _ = color_picker_changed(&mut app, Color::WHITE);

    assert!(app.active_mindmap_tab().unwrap().edge_colors.is_empty());
}

#[test]
fn color_picker_format_changed_updates_active_picker_only() {
    let mut app = app_with_tab();

    let _ = color_picker_format_changed(&mut app, ColorFormat::Hsl);
    assert!(app.active_mindmap_tab().unwrap().active_color_picker.is_none());

    let _ = open_color_picker(&mut app, MindMapColorTarget::Background, Color::BLACK);
    let _ = color_picker_format_changed(&mut app, ColorFormat::Hsl);

    assert_eq!(
        app.active_mindmap_tab().unwrap().active_color_picker.as_ref().unwrap().format,
        ColorFormat::Hsl
    );
}

#[test]
fn reset_color_target_removes_overrides_and_updates_matching_picker_color() {
    let mut app = app_with_tab();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.selected_path = Some(vec![0, 1]);
        tab.node_fills.insert(vec![0, 1], 1);
        tab.node_text_colors.insert(vec![0, 1], 2);
        tab.node_border_colors.insert(vec![0, 1], 3);
        tab.edge_colors.insert(vec![0, 1], 4);
        tab.background = Some(5);
    }

    for target in [
        MindMapColorTarget::NodeFill,
        MindMapColorTarget::NodeText,
        MindMapColorTarget::NodeBorder,
        MindMapColorTarget::EdgeStroke,
        MindMapColorTarget::Background,
    ] {
        let _ = open_color_picker(&mut app, target, Color::BLACK);
        let _ = reset_color_target(&mut app, target);
    }

    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.node_fills.is_empty());
    assert!(tab.node_text_colors.is_empty());
    assert!(tab.node_border_colors.is_empty());
    assert!(tab.edge_colors.is_empty());
    assert_eq!(tab.background, None);
    assert_eq!(
        tab.active_color_picker.as_ref().unwrap().color,
        Color::from_rgba8(255, 255, 255, 1.0)
    );
}

#[test]
fn set_background_sets_exact_background_override() {
    let mut app = app_with_tab();

    let _ = set_background(&mut app, Some(0x12345678));

    assert_eq!(app.active_mindmap_tab().unwrap().background, Some(0x12345678));
}

#[test]
fn set_theme_group_resets_variant_background_and_edge_overrides() {
    let mut app = app_with_tab();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.theme_variant = 4;
        tab.follow_theme_background = false;
        tab.background = Some(1);
        tab.edge_colors.insert(vec![0], 2);
        tab.edge_styles.insert(vec![0], EdgeStyle::Dotted);
    }

    let _ = set_theme_group(&mut app, "retro".to_string());

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.theme_group, "retro");
    assert_eq!(tab.theme_variant, 0);
    assert!(tab.follow_theme_background);
    assert_eq!(tab.background, None);
    assert!(tab.edge_colors.is_empty());
    assert!(tab.edge_styles.is_empty());
}

#[test]
fn set_theme_variant_wraps_preset_and_custom_indices() {
    let mut app = app_with_tab();

    let _ = set_theme_variant(&mut app, "classic".to_string(), 99);
    let preset_variant = app.active_mindmap_tab().unwrap().theme_variant;

    let custom_len = app.active_mindmap_tab().unwrap().custom_themes.len();
    let _ = set_theme_variant(&mut app, CUSTOM_THEME_GROUP_ID.to_string(), custom_len + 1);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(preset_variant < 8);
    assert_eq!(tab.theme_group, CUSTOM_THEME_GROUP_ID);
    assert_eq!(tab.theme_variant, 1 % custom_len.max(1));
}

#[test]
fn save_theme_to_custom_selects_new_custom_theme() {
    let mut app = app_with_tab();
    let original_len = app.active_mindmap_tab().unwrap().custom_themes.len();

    let _ = save_theme_to_custom(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.custom_themes.len(), original_len + 1);
    assert_eq!(tab.theme_group, CUSTOM_THEME_GROUP_ID);
    assert_eq!(tab.theme_variant, original_len);
    assert_eq!(tab.background, None);
    assert!(tab.edge_colors.is_empty());
}

#[test]
fn delete_custom_theme_handles_empty_out_of_range_and_last_theme() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().custom_themes.clear();

    let _ = delete_custom_theme(&mut app, 0);
    assert!(app.active_mindmap_tab().unwrap().custom_themes.is_empty());

    app.active_mindmap_tab_mut().unwrap().custom_themes = default_custom_themes();
    let len = app.active_mindmap_tab().unwrap().custom_themes.len();
    let _ = delete_custom_theme(&mut app, usize::MAX);
    assert_eq!(app.active_mindmap_tab().unwrap().custom_themes.len(), len - 1);

    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.custom_themes.truncate(1);
    tab.theme_group = CUSTOM_THEME_GROUP_ID.to_string();
    tab.theme_variant = 7;

    let _ = delete_custom_theme(&mut app, 0);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.custom_themes.is_empty());
    assert_eq!(tab.theme_group, "classic");
    assert_eq!(tab.theme_variant, 0);
}

#[test]
fn cancel_theme_background_disables_follow_theme_background() {
    let mut app = app_with_tab();

    let _ = cancel_theme_background(&mut app);

    assert!(!app.active_mindmap_tab().unwrap().follow_theme_background);
}

#[test]
fn set_edge_style_applies_global_root_or_selected_edge_style() {
    let mut app = app_with_tab();

    let _ = set_edge_style(&mut app, EdgeStyle::Dashed);
    assert_eq!(app.active_mindmap_tab().unwrap().edge_style, EdgeStyle::Dashed);

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![]);
    let _ = set_edge_style(&mut app, EdgeStyle::Dotted);
    assert_eq!(app.active_mindmap_tab().unwrap().edge_style, EdgeStyle::Dotted);

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![0, 1]);
    let _ = set_edge_style(&mut app, EdgeStyle::Dashed);
    assert_eq!(
        app.active_mindmap_tab().unwrap().edge_styles.get(&vec![0, 1]),
        Some(&EdgeStyle::Dashed)
    );
}

#[test]
fn set_node_border_style_applies_global_root_or_selected_node_style() {
    let mut app = app_with_tab();

    let _ = set_node_border_style(&mut app, EdgeStyle::Dashed);
    assert_eq!(app.active_mindmap_tab().unwrap().node_border_style, EdgeStyle::Dashed);

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![]);
    let _ = set_node_border_style(&mut app, EdgeStyle::Dotted);
    assert_eq!(app.active_mindmap_tab().unwrap().node_border_style, EdgeStyle::Dotted);

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![0, 1]);
    let _ = set_node_border_style(&mut app, EdgeStyle::Dashed);
    assert_eq!(
        app.active_mindmap_tab().unwrap().node_border_styles.get(&vec![0, 1]),
        Some(&EdgeStyle::Dashed)
    );
}
