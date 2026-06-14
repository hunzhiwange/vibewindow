use super::canvas_ops::{
    doodle_commit, doodle_erase, node_drag_start, node_dragged, pan_by, set_canvas_tool,
    set_doodle_color, set_doodle_width, toggle_zoom_menu, zoom, zoom_fit, zoom_set,
};
use crate::app::App;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapDoodleStroke, MindMapTab};
use iced::{Point, Vector};

fn app_with_tab() -> App {
    let mut app = App::new().0;
    app.window_size = (800.0, 600.0);
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
fn pan_by_moves_active_tab_pan() {
    let mut app = app_with_tab();

    let _ = pan_by(&mut app, Vector::new(10.0, -15.0));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.pan.x, 310.0);
    assert_eq!(tab.pan.y, 185.0);
}

#[test]
fn zoom_clamps_and_keeps_center_point_stable() {
    let mut app = app_with_tab();

    let _ = zoom(&mut app, 20.0, Some(Point::new(100.0, 50.0)));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.zoom, 10.0);
    assert_eq!(tab.pan.x, 2100.0);
    assert_eq!(tab.pan.y, 1550.0);
}

#[test]
fn zoom_uses_window_center_when_center_is_missing() {
    let mut app = app_with_tab();

    let _ = zoom(&mut app, 0.01, None);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.zoom, 0.1);
    assert_eq!(tab.pan.x, 390.0);
    assert_eq!(tab.pan.y, 290.0);
}

#[test]
fn zoom_set_closes_zoom_menu_and_clamps_value() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().show_zoom_menu = true;

    let _ = zoom_set(&mut app, 0.01);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.zoom, 0.1);
    assert!(!tab.show_zoom_menu);
}

#[test]
fn zoom_fit_sets_view_to_layout_bounds_and_closes_menu() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().show_zoom_menu = true;

    let _ = zoom_fit(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert!((0.1..=10.0).contains(&tab.zoom));
    assert!(!tab.show_zoom_menu);
}

#[test]
fn toggle_zoom_menu_opens_menu_and_closes_competing_panels() {
    let mut app = app_with_tab();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_action_menu = true;
    tab.show_context_menu = true;
    tab.context_menu_anchor = Some(Point::new(1.0, 2.0));
    tab.show_theme_panel = true;
    tab.show_markdown_import = true;
    tab.show_priority_picker = true;

    let _ = toggle_zoom_menu(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.show_zoom_menu);
    assert!(!tab.show_action_menu);
    assert!(!tab.show_context_menu);
    assert_eq!(tab.context_menu_anchor, None);
    assert!(!tab.show_theme_panel);
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_priority_picker);
}

#[test]
fn node_drag_start_selects_node_and_records_initial_position() {
    let mut app = app_with_tab();

    let _ = node_drag_start(&mut app, vec![0, 1], Point::new(10.0, 20.0), Point::new(30.0, 40.0));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.selected_path.as_deref(), Some(&[0, 1][..]));
    assert_eq!(tab.node_positions.get(&vec![0, 1]), Some(&Point::new(10.0, 20.0)));
    assert_eq!(tab.last_click_screen, Some(Point::new(30.0, 40.0)));
    assert!(!tab.show_context_menu);
}

#[test]
fn node_dragged_updates_existing_position_only() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().node_positions.insert(vec![0], Point::new(5.0, 6.0));

    let _ = node_dragged(&mut app, vec![0], Vector::new(2.0, -3.0));
    let _ = node_dragged(&mut app, vec![1], Vector::new(99.0, 99.0));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.node_positions.get(&vec![0]), Some(&Point::new(7.0, 3.0)));
    assert!(!tab.node_positions.contains_key(&vec![1]));
}

#[test]
fn set_canvas_tool_updates_tool_and_closes_context_ui() {
    let mut app = app_with_tab();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_context_menu = true;
    tab.context_menu_anchor = Some(Point::new(1.0, 2.0));
    tab.show_theme_panel = true;
    tab.show_text_editor = true;

    let _ = set_canvas_tool(&mut app, MindMapCanvasTool::Pen);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.canvas_tool, MindMapCanvasTool::Pen);
    assert!(!tab.show_context_menu);
    assert_eq!(tab.context_menu_anchor, None);
    assert!(!tab.show_theme_panel);
    assert!(!tab.show_text_editor);
}

#[test]
fn doodle_settings_are_applied_with_width_clamp() {
    let mut app = app_with_tab();

    let _ = set_doodle_color(&mut app, 0xABCDEF12);
    let _ = set_doodle_width(&mut app, 99.0);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doodle_rgba, 0xABCDEF12);
    assert_eq!(tab.doodle_width_px, 18.0);
}

#[test]
fn doodle_commit_ignores_short_strokes_and_keeps_valid_ones() {
    let mut app = app_with_tab();

    let _ = doodle_commit(
        &mut app,
        MindMapDoodleStroke { points_world: vec![Point::new(1.0, 1.0)], rgba: 1, width_px: 1.0 },
    );
    let _ = doodle_commit(
        &mut app,
        MindMapDoodleStroke {
            points_world: vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
            rgba: 2,
            width_px: 3.0,
        },
    );

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doodles.len(), 1);
    assert_eq!(tab.doodles[0].rgba, 2);
}

#[test]
fn doodle_erase_splits_strokes_and_ignores_non_positive_radius() {
    let mut app = app_with_tab();
    app.active_mindmap_tab_mut().unwrap().doodles.push(MindMapDoodleStroke {
        points_world: vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(15.0, 0.0),
        ],
        rgba: 7,
        width_px: 4.0,
    });

    let _ = doodle_erase(&mut app, Point::new(100.0, 100.0), 0.0);
    assert_eq!(app.active_mindmap_tab().unwrap().doodles.len(), 1);

    let _ = doodle_erase(&mut app, Point::new(10.0, 0.0), 0.1);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doodles.len(), 1);
    assert_eq!(tab.doodles[0].points_world, vec![Point::new(0.0, 0.0), Point::new(5.0, 0.0)]);
}
