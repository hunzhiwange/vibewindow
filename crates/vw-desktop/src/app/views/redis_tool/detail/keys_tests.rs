use crate::app::{App, Message};
use iced::{Element, Theme};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("keys_tests"));
}

#[test]
fn key_tree_root_sorts_segments_and_keeps_terminal_keys() {
    let keys = vec![
        "user:2:name".to_string(),
        "user:1".to_string(),
        "order:9".to_string(),
        "user:2".to_string(),
    ];

    let root = super::build_key_tree_root(&keys);
    let labels = root.children.keys().cloned().collect::<Vec<_>>();

    assert_eq!(labels, vec!["order", "user"]);
    assert_eq!(super::count_terminal_keys(&root), 4);
    assert_eq!(root.children["user"].children["2"].full_key.as_deref(), Some("user:2"));
    assert_eq!(
        root.children["user"].children["2"].children["name"].full_key.as_deref(),
        Some("user:2:name")
    );
}

#[test]
fn insert_key_tree_ignores_blank_segments_but_preserves_original_key() {
    let mut root = super::RedisKeyTreeNode::default();

    super::insert_key_tree(&mut root, "cache::user:  :1");

    assert_eq!(super::count_terminal_keys(&root), 1);
    assert!(root.children["cache"].children.contains_key("user"));
    assert_eq!(
        root.children["cache"].children["user"].children["1"].full_key.as_deref(),
        Some("cache::user:  :1")
    );
}

#[test]
fn insert_key_tree_empty_key_marks_root_terminal() {
    let mut root = super::RedisKeyTreeNode::default();

    super::insert_key_tree(&mut root, "");
    super::insert_key_tree(&mut root, "   ");

    assert_eq!(root.full_key.as_deref(), Some("   "));
    assert_eq!(super::count_terminal_keys(&root), 1);
    assert!(root.children.is_empty());
}

#[test]
fn truncate_key_label_preserves_short_keys_and_truncates_long_keys() {
    assert_eq!(super::truncate_key_label("short:key"), "选中 short:key");
    assert_eq!(super::truncate_key_label("12345678901234567890"), "选中 12345678901234567890");
    assert_eq!(super::truncate_key_label("123456789012345678901"), "选中 12345678901234567890...");
    assert_eq!(
        super::truncate_key_label("用户:账户:余额:更新:队列:2026"),
        "选中 用户:账户:余额:更新:队列:2026"
    );
}

#[test]
fn key_tree_rows_build_branch_leaf_and_expanded_terminal_variants() {
    let keys = vec![
        "cache:user".to_string(),
        "cache:user:name".to_string(),
        "cache:session".to_string(),
        "plain".to_string(),
    ];
    let tree = super::build_key_tree_root(&keys);
    let mut expanded_paths = std::collections::HashSet::new();

    let cache = tree.children.get("cache").expect("cache branch").clone();
    keep_element(super::render_key_tree_node(
        cache.clone(),
        "cache".to_string(),
        0,
        &expanded_paths,
        Some("cache:user"),
    ));

    expanded_paths.insert("cache".to_string());
    expanded_paths.insert("cache:user".to_string());
    keep_element(super::render_key_tree_node(
        cache,
        "cache".to_string(),
        0,
        &expanded_paths,
        Some("cache:user"),
    ));

    let plain = tree.children.get("plain").expect("plain leaf").clone();
    keep_element(super::render_key_tree_node(
        plain,
        "plain".to_string(),
        1,
        &expanded_paths,
        Some("plain"),
    ));
}

#[test]
fn key_tree_row_builders_cover_enabled_selected_and_child_states() {
    keep_element(super::build_key_tree_branch_row(
        "cache".to_string(),
        "cache".to_string(),
        2,
        false,
        3,
    ));
    keep_element(super::build_key_tree_branch_row(
        "cache".to_string(),
        "cache".to_string(),
        1,
        true,
        1,
    ));
    keep_element(super::build_key_tree_leaf_row("cache:user".to_string(), 0, false, false));
    keep_element(super::build_key_tree_leaf_row("cache:user:name".to_string(), 2, true, true));
}

#[test]
fn key_tree_panel_builds_empty_loaded_and_load_more_states() {
    let mut app = test_app();

    keep_element(super::build_key_tree_panel(&app, false));

    app.redis_tool.selected_connection_id = Some("redis-local".to_string());
    app.redis_tool.key_browser_pattern = "cache:*".to_string();
    app.redis_tool.key_browser_items =
        vec!["cache:user".to_string(), "cache:user:name".to_string(), "cache:session".to_string()];
    app.redis_tool.key_tree_expanded_paths.insert("cache".to_string());
    app.redis_tool.key_tree_expanded_paths.insert("cache:user".to_string());
    app.redis_tool.selected_key = Some("cache:user:name".to_string());

    keep_element(super::build_key_tree_panel(&app, false));

    app.redis_tool.key_browser_has_more = true;
    keep_element(super::build_key_tree_panel(&app, true));
}

#[test]
fn key_tree_styles_keep_dark_theme_readable() {
    let branch = super::branch_row_container_style(&Theme::Dark);
    let selected_leaf = super::leaf_row_container_style(&Theme::Dark, true);
    let plain_leaf = super::leaf_row_container_style(&Theme::Dark, false);
    let selected_text = super::leaf_row_text_style(&Theme::Dark, true, false);
    let child_text = super::leaf_row_text_style(&Theme::Dark, false, true);
    let plain_text = super::leaf_row_text_style(&Theme::Dark, false, false);

    assert!(branch.background.is_some());
    assert_eq!(branch.border.width, 1.0);
    assert!(selected_leaf.background.is_some());
    assert!(plain_leaf.background.is_some());
    assert_ne!(selected_leaf.border.color, plain_leaf.border.color);
    assert_ne!(selected_text.color, child_text.color);
    assert_ne!(child_text.color, plain_text.color);
}
