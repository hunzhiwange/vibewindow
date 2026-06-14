use super::*;
use iced::Theme;

fn test_app() -> App {
    App::new().0
}

fn test_analysis() -> RedisKeyAnalysis {
    RedisKeyAnalysis {
        connection_id: "redis-local".to_string(),
        key: "cache:user:1".to_string(),
        key_type: "Hash".to_string(),
        ttl_secs: 3600,
        memory_usage_bytes: Some(1536),
        preview_command: "HGETALL cache:user:1".to_string(),
        preview_output: "name\nAda".to_string(),
    }
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("analysis_tests"));
}

#[test]
fn ttl_label_formats_known_redis_states_and_fallbacks() {
    assert_eq!(ttl_label(-2), "不存在");
    assert_eq!(ttl_label(-1), "永久");
    assert_eq!(ttl_label(0), "TTL 0s");
    assert_eq!(ttl_label(42), "TTL 42s");
    assert_eq!(ttl_label(-30), "TTL -30");
}

#[test]
fn memory_usage_label_formats_missing_bytes_kilobytes_and_megabytes() {
    assert_eq!(memory_usage_label(None), "内存未知");
    assert_eq!(memory_usage_label(Some(0)), "0 B");
    assert_eq!(memory_usage_label(Some(1023)), "1023 B");
    assert_eq!(memory_usage_label(Some(1024)), "1.0 KB");
    assert_eq!(memory_usage_label(Some(1536)), "1.5 KB");
    assert_eq!(memory_usage_label(Some(1024 * 1024 - 1)), "1024.0 KB");
    assert_eq!(memory_usage_label(Some(1024 * 1024)), "1.00 MB");
    assert_eq!(memory_usage_label(Some(3 * 1024 * 1024 + 512 * 1024)), "3.50 MB");
}

#[test]
fn non_empty_preview_replaces_blank_values_only() {
    assert_eq!(non_empty_preview(""), "(empty)");
    assert_eq!(non_empty_preview("   \n\t"), "(empty)");
    assert_eq!(non_empty_preview("  value  "), "  value  ");
}

#[test]
fn key_analysis_panel_builds_with_enabled_and_busy_refresh_states() {
    let mut app = test_app();
    app.redis_tool.selected_key = Some("cache:user:1".to_string());
    let analysis = test_analysis();

    keep_element(build_key_analysis_panel(&app, &analysis, false));
    keep_element(build_key_analysis_panel(&app, &analysis, true));
}

#[test]
fn key_analysis_panel_builds_missing_ttl_memory_and_empty_preview() {
    let mut app = test_app();
    app.redis_tool.selected_key = Some("missing".to_string());
    let analysis = RedisKeyAnalysis {
        ttl_secs: -2,
        memory_usage_bytes: None,
        preview_output: " \n ".to_string(),
        ..test_analysis()
    };

    keep_element(build_key_analysis_panel(&app, &analysis, false));
}

#[test]
fn key_analysis_empty_state_builds_selected_and_unselected_variants() {
    let mut app = test_app();

    keep_element(build_key_analysis_empty_state(
        &app,
        "未选择 Key",
        "请选择左侧键树中的 Key。",
        false,
    ));

    app.redis_tool.selected_key = Some("cache:user:1".to_string());
    keep_element(build_key_analysis_empty_state(
        &app,
        "暂无分析",
        "点击刷新读取当前 Key 内容。",
        false,
    ));
    keep_element(build_key_analysis_empty_state(&app, "正在分析", "网关请求仍在进行。", true));
}

#[test]
fn preview_panel_style_keeps_dark_theme_readable() {
    let style = preview_panel_style(&Theme::Dark);

    assert!(style.background.is_some());
    assert_eq!(style.border.width, 1.0);
    assert!(style.text_color.is_some());
    assert!(style.shadow.blur_radius > 0.0);
}
