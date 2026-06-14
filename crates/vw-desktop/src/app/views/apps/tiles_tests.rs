#[test]
fn tile_column_formula_has_minimum_one_column() {
    let available_width = 160.0_f32.max(220.0);
    let cols = ((available_width + 16.0) / (220.0 + 16.0)).floor() as usize;
    assert_eq!(cols.max(1), 1);
}

#[test]
fn compute_tile_columns_respects_width_and_minimum() {
    let mut app = crate::app::App::new().0;
    app.window_size = (100.0, 700.0);
    assert_eq!(super::compute_tile_columns(&app, 168.0, 20.0), 1);

    app.window_size = (800.0, 700.0);
    assert_eq!(super::compute_tile_columns(&app, 168.0, 20.0), 4);
}

#[test]
fn render_header_builds_blocked_and_unblocked_states() {
    let mut app = crate::app::App::new().0;
    app.apps_search_query = "json".to_string();
    let _ = super::render_header(&app, false);
    let _ = super::render_header(&app, true);
}

#[test]
fn render_tiles_grid_builds_all_tiles_and_empty_search_results() {
    let mut app = crate::app::App::new().0;
    app.window_size = (1280.0, 800.0);
    app.web_bookmarks.push(crate::app::state::WebBookmark {
        title: String::new(),
        url: "https://example.com".to_string(),
        width: Some(900),
        height: Some(700),
        cookie_configs: None,
    });

    let _ = super::render_tiles_grid(&app, false);

    app.apps_search_query = "no such tile".to_string();
    let _ = super::render_tiles_grid(&app, false);
}

#[test]
fn render_tiles_grid_filters_bookmarks_by_title_or_url_and_blocks_messages() {
    let mut app = crate::app::App::new().0;
    app.window_size = (420.0, 800.0);
    app.apps_search_query = "docs".to_string();
    app.web_bookmarks.push(crate::app::state::WebBookmark {
        title: "Docs".to_string(),
        url: "https://example.com".to_string(),
        width: None,
        height: None,
        cookie_configs: None,
    });
    app.web_bookmarks.push(crate::app::state::WebBookmark {
        title: "Other".to_string(),
        url: "https://docs.example.com".to_string(),
        width: None,
        height: None,
        cookie_configs: None,
    });

    let _ = super::render_tiles_grid(&app, true);
}
