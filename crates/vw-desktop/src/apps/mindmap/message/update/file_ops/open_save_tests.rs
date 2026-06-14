use super::open_save::is_json_path;
use super::open_save::{file_opened, file_saved, new_tab, save_finished};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::MindMapTab;

fn test_app_with_tab() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let tab = MindMapTab::new(
        "tab-1".to_string(),
        "Original".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n"),
    );
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app.error_message = None;
    app
}

#[test]
fn is_json_path_matches_extension_case_insensitively() {
    assert!(is_json_path("/tmp/map.JSON"));
    assert!(!is_json_path("/tmp/map.md"));
    assert!(!is_json_path("/tmp/json"));
}

#[test]
fn new_tab_closes_action_menu_and_adds_blank_tab() {
    let mut app = test_app_with_tab();
    app.mindmap_tabs[0].show_action_menu = true;

    let _ = new_tab(&mut app);

    assert!(!app.mindmap_tabs[0].show_action_menu);
    assert_eq!(app.mindmap_tabs.len(), 2);
    let active = app.active_mindmap_tab().unwrap();
    assert_eq!(active.title, "思维导图 2");
    assert_eq!(active.doc.text, "中心主题");
}

#[test]
fn file_opened_loads_markdown_json_and_errors() {
    let mut app = test_app_with_tab();

    let _ = file_opened(
        &mut app,
        Ok((Some("/tmp/map.md".to_string()), "# Imported\n\n- Child\n".to_string())),
    );
    let active = app.active_mindmap_tab().unwrap();
    assert_eq!(active.title, "map.md");
    assert_eq!(active.file_path.as_deref(), Some("/tmp/map.md"));
    assert_eq!(active.doc.text, "Imported");
    assert_eq!(active.doc.children[0].text, "Child");

    let json = serde_json::json!({
        "format": "vibe-window-mindmap",
        "version": 1,
        "data": {
            "title": "Json Title",
            "markdown": "# Json Root\n\n- Json Child\n",
            "pan_x": 1.0,
            "pan_y": 2.0,
            "zoom": 20.0,
            "node_priorities": [{"path": [0], "priority": 99}, {"path": [1], "priority": 4}],
            "node_urls": [{"path": [0], "url": " ` https://example.test ` "}, {"path": [1], "url": "   "}],
            "doodle_rgba": 0,
            "doodle_width_px": -1.0,
            "doodles": [
                {"rgba": 1, "width_px": 2.0, "points": [{"x": 0.0, "y": 0.0}]},
                {"rgba": 2, "width_px": 3.0, "points": [{"x": 1.0, "y": 1.0}, {"x": 2.0, "y": 2.0}]}
            ]
        }
    })
    .to_string();
    let _ = file_opened(&mut app, Ok((Some("/tmp/map.json".to_string()), json)));
    let active = app.active_mindmap_tab().unwrap();
    assert_eq!(active.title, "map.json");
    assert_eq!(active.doc.text, "Json Root");
    assert_eq!(active.zoom, 10.0);
    assert_eq!(active.node_priorities.get(&vec![1]), Some(&4));
    assert!(!active.node_priorities.contains_key(&vec![0]));
    assert_eq!(active.node_urls.get(&vec![0]).map(String::as_str), Some("https://example.test"));
    assert!(!active.node_urls.contains_key(&vec![1]));
    assert_eq!(active.doodle_rgba, 0x111827FF);
    assert_eq!(active.doodle_width_px, 3.0);
    assert_eq!(active.doodles.len(), 1);

    let before_error = app.error_message.clone();
    let _ = file_opened(&mut app, Err("Cancelled".to_string()));
    assert_eq!(app.error_message, before_error);

    let _ = file_opened(&mut app, Err("boom".to_string()));
    assert_eq!(app.error_message.as_deref(), Some("boom"));

    app.error_message = None;
    let bad_json = serde_json::json!({
        "format": "other",
        "version": 1,
        "data": {"markdown": "", "pan_x": 0.0, "pan_y": 0.0, "zoom": 1.0}
    })
    .to_string();
    let _ = file_opened(&mut app, Ok((Some("/tmp/bad.json".to_string()), bad_json)));
    assert_eq!(app.error_message.as_deref(), Some("不支持的思维导图 JSON 格式"));
}

#[test]
fn save_finished_and_file_saved_update_visible_state() {
    let mut app = test_app_with_tab();

    let _ = save_finished(&mut app, Ok(()));
    assert!(app.error_message.is_none());

    let _ = save_finished(&mut app, Err("write failed".to_string()));
    assert_eq!(app.error_message.as_deref(), Some("write failed"));

    let _ = file_saved(&mut app, Some("/tmp/new-name.md".to_string()));
    let active = app.active_mindmap_tab().unwrap();
    assert_eq!(active.file_path.as_deref(), Some("/tmp/new-name.md"));
    assert_eq!(active.title, "new-name.md");

    let _ = file_saved(&mut app, None);
    let active = app.active_mindmap_tab().unwrap();
    assert_eq!(active.title, "new-name.md");
}
