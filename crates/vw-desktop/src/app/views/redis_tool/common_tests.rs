use super::*;
use crate::app::assets::Icon;
use crate::app::state::{
    RedisConnectionConfig, RedisConnectionDraft, RedisHistoryRecord, RedisSentinelConfig,
    RedisSshTunnelConfig, RedisTlsCertConfig,
};
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

    assert!(module.ends_with("common_tests"));
}

#[test]
fn current_load_count_defaults_and_clamps_bounds() {
    let mut app = test_app();

    for (input, expected) in [
        ("", 500),
        ("abc", 500),
        ("0", 1),
        ("1", 1),
        ("9999", 9999),
        ("10001", 10_000),
        (" 42 ", 42),
    ] {
        app.redis_tool.default_load_count_input = input.to_string();

        assert_eq!(current_load_count(&app), expected);
    }
}

#[test]
fn history_page_label_handles_empty_partial_and_saturated_ranges() {
    let mut app = test_app();

    app.redis_tool.history_total = 0;
    assert_eq!(history_page_label(&app), "暂无历史记录");

    app.redis_tool.history = vec![history_record("PING", 3), history_record("GET key", 4)];
    app.redis_tool.history_page_offset = 20;
    app.redis_tool.history_total = 25;
    assert_eq!(history_page_label(&app), "第 21-22 条 / 共 25 条");

    app.redis_tool.history_page_offset = usize::MAX;
    app.redis_tool.history_total = usize::MAX;
    assert_eq!(
        history_page_label(&app),
        format!("第 {}-{} 条 / 共 {} 条", usize::MAX, usize::MAX, usize::MAX)
    );
}

#[test]
fn masked_connection_preview_masks_auth_and_uses_fallbacks() {
    let mut app = test_app();
    app.redis_tool.draft = RedisConnectionDraft::default();
    app.redis_tool.draft.host = " redis.example.com ".to_string();
    app.redis_tool.draft.port = " 6380 ".to_string();
    app.redis_tool.draft.db = " 2 ".to_string();

    assert_eq!(masked_connection_preview(&app), "redis://redis.example.com:6380/2");

    app.redis_tool.draft.username = " app ".to_string();
    assert_eq!(masked_connection_preview(&app), "redis://app@redis.example.com:6380/2");

    app.redis_tool.draft.password = " secret ".to_string();
    assert_eq!(masked_connection_preview(&app), "redis://app:******@redis.example.com:6380/2");

    app.redis_tool.draft.username.clear();
    assert_eq!(masked_connection_preview(&app), "redis://:******@redis.example.com:6380/2");

    app.redis_tool.draft.use_tls = true;
    app.redis_tool.draft.host = " ".to_string();
    app.redis_tool.draft.port = " ".to_string();
    app.redis_tool.draft.db = " ".to_string();
    assert_eq!(masked_connection_preview(&app), "rediss://:******@<host>:6379/0");
}

#[test]
fn masked_connection_preview_prioritizes_advanced_modes() {
    let mut app = test_app();
    app.redis_tool.draft.host = "redis.example.com".to_string();
    app.redis_tool.draft.port = "6380".to_string();
    app.redis_tool.draft.username = "app".to_string();
    app.redis_tool.draft.password = "secret".to_string();

    app.redis_tool.draft.ssh_tunnel.enabled = true;
    app.redis_tool.draft.ssh_tunnel.host = " bastion.example.com ".to_string();
    app.redis_tool.draft.ssh_tunnel.port = " 2200 ".to_string();
    app.redis_tool.draft.ssh_tunnel.username = " deploy ".to_string();
    assert_eq!(
        masked_connection_preview(&app),
        "ssh://deploy@bastion.example.com:2200 -> redis://app:******@redis.example.com:6380/0"
    );

    app.redis_tool.draft.ssh_tunnel.host.clear();
    app.redis_tool.draft.ssh_tunnel.port.clear();
    app.redis_tool.draft.ssh_tunnel.username.clear();
    assert_eq!(
        masked_connection_preview(&app),
        "ssh://<user>@<ssh-host>:22 -> redis://app:******@redis.example.com:6380/0"
    );

    app.redis_tool.draft.ssh_tunnel.enabled = false;
    app.redis_tool.draft.sentinel.enabled = true;
    app.redis_tool.draft.sentinel.master_name = " primary ".to_string();
    assert_eq!(masked_connection_preview(&app), "sentinel://redis.example.com:6380?master=primary");

    app.redis_tool.draft.sentinel.master_name.clear();
    assert_eq!(
        masked_connection_preview(&app),
        "sentinel://redis.example.com:6380?master=<master>"
    );

    app.redis_tool.draft.sentinel.enabled = false;
    app.redis_tool.draft.use_cluster = true;
    assert_eq!(
        masked_connection_preview(&app),
        "redis-cluster://redis.example.com:6380?mode=readwrite"
    );

    app.redis_tool.draft.read_only = true;
    assert_eq!(
        masked_connection_preview(&app),
        "redis-cluster://redis.example.com:6380?mode=readonly"
    );
}

#[test]
fn connection_labels_and_summaries_cover_empty_and_combined_modes() {
    let mut draft = RedisConnectionDraft::default();
    assert_eq!(connection_mode_label(&draft), "直连");
    assert_eq!(enabled_feature_summary(&draft), "基础直连");
    assert_eq!(advanced_execution_note(&draft), None);

    draft.use_tls = true;
    draft.tls_cert.ca_cert_path = "/tmp/ca.crt".to_string();
    draft.ssh_tunnel.enabled = true;
    draft.sentinel.enabled = true;
    draft.use_cluster = true;
    draft.read_only = true;

    assert_eq!(connection_mode_label(&draft), "SSH 隧道 / SSL/TLS / Sentinel / Cluster / Readonly");
    assert_eq!(
        enabled_feature_summary(&draft),
        "TLS / 证书路径 / SSH / Sentinel / Cluster / Readonly"
    );
    assert_eq!(
        advanced_execution_note(&draft),
        Some("当前版本会保存 SSH 隧道配置，但测试连接、运行态读取、命令执行和复制 URI 暂不支持 SSH。".to_string())
    );

    draft.ssh_tunnel.enabled = false;
    assert_eq!(
        advanced_execution_note(&draft),
        Some(
            "当前版本已支持测试连接、运行态读取与命令执行，但复制 URI 仍仅支持直连 URI。"
                .to_string()
        )
    );

    draft.sentinel.enabled = false;
    draft.use_cluster = false;
    assert_eq!(
        advanced_execution_note(&draft),
        Some("当前版本已支持自定义 SSL 证书测试、运行态读取与命令执行，但复制 URI 不会包含证书路径。".to_string())
    );
}

#[test]
fn connection_mode_summary_uses_saved_connection_flags() {
    let mut connection = connection_config();
    assert_eq!(connection_mode_summary(&connection), "基础直连");

    connection.ssh_tunnel.enabled = true;
    connection.use_tls = true;
    connection.sentinel.enabled = true;
    connection.use_cluster = true;
    connection.read_only = true;

    assert_eq!(connection_mode_summary(&connection), "SSH / TLS / Sentinel / Cluster / Readonly");
}

#[test]
fn timestamp_formatting_handles_valid_and_invalid_millis() {
    let formatted = format_timestamp(1_725_000_000_000);

    assert_ne!(formatted, "--");
    assert_eq!(formatted.len(), "06-28 08:00:00".len());
    assert_eq!(format_timestamp(i64::MAX as u64), "--");
}

#[test]
fn table_rows_and_common_widgets_build_all_variants() {
    let mut app = test_app();

    keep_element(build_status_badge(&app));
    app.redis_tool.gateway_loading_label = Some("加载中".to_string());
    keep_element(build_status_badge(&app));
    app.redis_tool.gateway_loading_label = None;
    app.redis_tool.gateway_error = Some("连接失败".to_string());
    keep_element(build_status_badge(&app));
    app.redis_tool.gateway_error = None;
    app.redis_tool.notification = Some("已保存".to_string());
    keep_element(build_status_badge(&app));

    keep_element(build_error_banner("连接失败"));
    keep_element(build_round_icon_action(
        Icon::X,
        Message::RedisTool(RedisToolMessage::ClearNotification),
        true,
    ));
    keep_element(build_round_icon_action(
        Icon::X,
        Message::RedisTool(RedisToolMessage::ClearNotification),
        false,
    ));
    keep_element(build_detail_action_button(
        "运行",
        Message::RedisTool(RedisToolMessage::RunCommand),
        true,
        true,
    ));
    keep_element(build_detail_action_button(
        "运行",
        Message::RedisTool(RedisToolMessage::RunCommand),
        false,
        false,
    ));
    keep_element(build_input("Host", "127.0.0.1", RedisToolMessage::DraftHostChanged));
    keep_element(build_path_picker_input(
        "证书",
        "/tmp/ca.crt",
        RedisToolMessage::DraftTlsCaCertPathChanged,
        Message::RedisTool(RedisToolMessage::PickTlsCaCertFile),
        true,
    ));
    keep_element(form_row(
        "Host",
        "Redis host",
        build_input("Host", "127.0.0.1", RedisToolMessage::DraftHostChanged),
        false,
    ));
    keep_element(form_row(
        "Host",
        "Redis host",
        build_input("Host", "127.0.0.1", RedisToolMessage::DraftHostChanged),
        true,
    ));
    keep_element(overview_row("DB", 2));
    keep_element(empty_sidebar_hint("暂无连接", "创建连接后开始使用"));
    keep_element(modal_shell(text("content").into()).into());
    keep_element(modal_header("标题", Message::RedisTool(RedisToolMessage::CloseConnectionModal)));
    keep_element(history_table_header());
    keep_element(history_table_row(&history_record("SET key value", 9)));

    std::hint::black_box(redis_scroll_direction());
    std::hint::black_box(themed_icon_svg(Icon::X, 12.0));
    std::hint::black_box(primary_icon_svg(Icon::X, 12.0));
}

#[test]
fn icon_styles_return_expected_theme_colors() {
    let themed = themed_icon_svg(Icon::X, 12.0);
    let primary = primary_icon_svg(Icon::X, 12.0);

    std::hint::black_box(themed);
    std::hint::black_box(primary);

    let text_style = iced::widget::text::Style { color: Some(Theme::Dark.palette().text) };
    assert!(text_style.color.is_some());
}

fn history_record(command: &str, cost_ms: u64) -> RedisHistoryRecord {
    RedisHistoryRecord {
        time_ms: 1_725_000_000_000,
        connection_id: Some("local".to_string()),
        connection_label: "local".to_string(),
        command: command.to_string(),
        args: "key".to_string(),
        cost_ms,
        is_write: command.starts_with("SET"),
    }
}

fn connection_config() -> RedisConnectionConfig {
    RedisConnectionConfig {
        id: "local".to_string(),
        name: "local".to_string(),
        host: "127.0.0.1".to_string(),
        port: 6379,
        db: 0,
        username: String::new(),
        password: String::new(),
        use_tls: false,
        tls_cert: RedisTlsCertConfig::default(),
        ssh_tunnel: RedisSshTunnelConfig::default(),
        sentinel: RedisSentinelConfig::default(),
        use_cluster: false,
        read_only: false,
        key_pattern: "*".to_string(),
        last_used_ms: None,
        updated_at_ms: 0,
    }
}
