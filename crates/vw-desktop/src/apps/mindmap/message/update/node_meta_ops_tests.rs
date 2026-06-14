use super::node_meta_ops::{
    clear_node_priority, clear_node_url, clear_selection, close_pickers,
    commit_url_editor_if_needed, node_url_changed, open_node_url, open_node_url_at, save_node_url,
    select_diagram_type, select_node, set_bracket_layout_format, set_diagram_type,
    set_fishbone_layout_format, set_layout_format, set_node_priority, set_org_chart_layout_format,
    set_timeline_layout_format, set_tree_layout_format, toggle_action_menu,
    toggle_diagram_type_picker, toggle_export_menu, toggle_node_url_editor, toggle_priority_picker,
    toggle_theme_panel,
};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat, MindMapTab,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use iced::Point;
use iced::widget::text_editor;

fn test_app() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n- B\n"),
    );
    tab.selected_path = Some(vec![0]);
    tab.node_positions.insert(vec![0], Point::new(1.0, 2.0));
    tab.show_context_menu = true;
    tab.context_menu_anchor = Some(Point::new(7.0, 8.0));
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn commit_url_editor_trims_inserts_removes_and_commits_text_first() {
    let mut app = test_app();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_text_editor = true;
    tab.node_text_editor = text_editor::Content::with_text("Edited A");
    tab.show_url_editor = true;
    tab.url_editor_value = " ` https://example.test ` ".to_string();

    commit_url_editor_if_needed(tab);
    assert_eq!(tab.doc.children[0].text, "Edited A");
    assert_eq!(tab.undo_stack.len(), 1);
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://example.test"));

    tab.url_editor_value = " `` ".to_string();
    commit_url_editor_if_needed(tab);
    assert!(!tab.node_urls.contains_key(&vec![0]));

    tab.show_url_editor = false;
    tab.url_editor_value = "https://ignored.test".to_string();
    commit_url_editor_if_needed(tab);
    assert!(!tab.node_urls.contains_key(&vec![0]));
}

#[test]
fn selection_and_close_pickers_reset_open_ui_state() {
    let mut app = test_app();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.show_url_editor = true;
        tab.url_editor_value = "https://a.test".to_string();
        tab.show_zoom_menu = true;
        tab.show_diagram_type_picker = true;
        tab.show_priority_picker = true;
        tab.show_export_menu = true;
        tab.show_theme_panel = true;
    }

    let _ = select_node(&mut app, vec![1]);
    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://a.test"));
    assert_eq!(tab.selected_path.as_deref(), Some(&[1][..]));
    assert!(!tab.show_context_menu);
    assert!(!tab.show_zoom_menu);
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_url_editor);

    let _ = clear_selection(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.selected_path.is_none());
    assert!(!tab.show_export_menu);
    assert!(!tab.show_theme_panel);

    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_markdown_import = true;
    tab.show_action_menu = true;
    tab.show_theme_panel = true;
    let _ = close_pickers(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_action_menu);
    assert!(!tab.show_theme_panel);
}

#[test]
fn toggles_open_one_panel_and_close_related_panels() {
    let mut app = test_app();

    let _ = toggle_diagram_type_picker(&mut app);
    assert!(app.active_mindmap_tab().unwrap().show_diagram_type_picker);
    let _ = toggle_diagram_type_picker(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().show_diagram_type_picker);

    let _ = toggle_export_menu(&mut app);
    {
        let tab = app.active_mindmap_tab().unwrap();
        assert!(tab.show_export_menu);
        assert!(!tab.show_diagram_type_picker);
    }

    let _ = toggle_theme_panel(&mut app);
    {
        let tab = app.active_mindmap_tab().unwrap();
        assert!(tab.show_theme_panel);
        assert!(tab.show_export_menu);
    }

    let _ = toggle_action_menu(&mut app);
    {
        let tab = app.active_mindmap_tab().unwrap();
        assert!(tab.show_action_menu);
        assert!(!tab.show_theme_panel);
    }

    let _ = toggle_priority_picker(&mut app);
    {
        let tab = app.active_mindmap_tab().unwrap();
        assert!(tab.show_priority_picker);
        assert!(!tab.show_action_menu);
    }
}

#[test]
fn priority_and_url_mutators_update_selected_path() {
    let mut app = test_app();

    let _ = set_node_priority(&mut app, 0);
    assert_eq!(app.active_mindmap_tab().unwrap().node_priorities.get(&vec![0]), Some(&1));
    let _ = set_node_priority(&mut app, 10);
    assert_eq!(app.active_mindmap_tab().unwrap().node_priorities.get(&vec![0]), Some(&10));
    let _ = clear_node_priority(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().node_priorities.contains_key(&vec![0]));

    let _ = toggle_node_url_editor(&mut app);
    assert!(app.active_mindmap_tab().unwrap().show_url_editor);
    let _ = node_url_changed(&mut app, " ` https://example.test/path ` ".to_string());
    let _ = save_node_url(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_url_editor);
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://example.test/path"));

    let _ = toggle_node_url_editor(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().url_editor_value, "https://example.test/path");
    let _ = toggle_node_url_editor(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().show_url_editor);

    let _ = clear_node_url(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().node_urls.contains_key(&vec![0]));

    let _ = open_node_url(&mut app);
    let _ = open_node_url_at(&mut app, vec![1]);
    assert_eq!(app.active_mindmap_tab().unwrap().selected_path.as_deref(), Some(&[1][..]));
}

#[test]
fn diagram_and_layout_setters_clear_positions_and_close_picker() {
    let mut app = test_app();

    let _ = set_diagram_type(&mut app, MindMapDiagramType::MindMap);
    assert!(app.active_mindmap_tab().unwrap().node_positions.contains_key(&vec![0]));
    let _ = set_diagram_type(&mut app, MindMapDiagramType::Tree);
    assert_eq!(app.active_mindmap_tab().unwrap().diagram_type, MindMapDiagramType::Tree);
    assert!(app.active_mindmap_tab().unwrap().node_positions.is_empty());

    app.active_mindmap_tab_mut().unwrap().node_positions.insert(vec![0], Point::new(1.0, 2.0));
    let _ = select_diagram_type(&mut app, MindMapDiagramType::OrgChart);
    assert_eq!(app.active_mindmap_tab().unwrap().diagram_type, MindMapDiagramType::OrgChart);
    assert!(app.active_mindmap_tab().unwrap().node_positions.is_empty());

    app.active_mindmap_tab_mut().unwrap().show_diagram_type_picker = true;
    let _ = set_layout_format(&mut app, MindMapLayoutFormat::Bidirectional);
    let _ = set_org_chart_layout_format(&mut app, OrgChartLayoutFormat::LeftRight);
    let _ = set_fishbone_layout_format(&mut app, FishboneLayoutFormat::HeadLeft);
    let _ = set_timeline_layout_format(&mut app, TimelineLayoutFormat::AllDown);
    let _ = set_bracket_layout_format(&mut app, BracketLayoutFormat::BraceLeft);
    let _ = set_tree_layout_format(&mut app, TreeLayoutFormat::SymmetricSplit);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.layout_format, MindMapLayoutFormat::Bidirectional);
    assert_eq!(tab.org_chart_layout_format, OrgChartLayoutFormat::LeftRight);
    assert_eq!(tab.fishbone_layout_format, FishboneLayoutFormat::HeadLeft);
    assert_eq!(tab.timeline_layout_format, TimelineLayoutFormat::AllDown);
    assert_eq!(tab.bracket_layout_format, BracketLayoutFormat::BraceLeft);
    assert_eq!(tab.tree_layout_format, TreeLayoutFormat::SymmetricSplit);
    assert!(!tab.show_diagram_type_picker);
    assert!(tab.node_positions.is_empty());
}

#[test]
fn meta_ops_do_nothing_without_active_tab() {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _ = select_node(&mut app, vec![0]);
    let _ = clear_selection(&mut app);
    let _ = close_pickers(&mut app);
    let _ = toggle_action_menu(&mut app);
    let _ = set_node_priority(&mut app, 1);
    let _ = toggle_node_url_editor(&mut app);
    let _ = save_node_url(&mut app);
    let _ = set_diagram_type(&mut app, MindMapDiagramType::Tree);
    assert!(app.mindmap_tabs.is_empty());
}
