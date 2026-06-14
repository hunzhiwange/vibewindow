use super::*;
use crate::app::state::RedisConnectionTab;

fn test_app() -> App {
    let (mut app, _task) = App::new();
    app.redis_tool.draft.name = "production-cache".to_string();
    app.redis_tool.draft.host = "redis.example.com".to_string();
    app.redis_tool.draft.port = "6380".to_string();
    app.redis_tool.draft.db = "2".to_string();
    app.redis_tool.draft.username = "app".to_string();
    app.redis_tool.draft.password = "secret".to_string();
    app.redis_tool.draft.key_pattern = "svc:*".to_string();
    app.redis_tool.draft.ssh_tunnel.enabled = true;
    app.redis_tool.draft.ssh_tunnel.host = "bastion.example.com".to_string();
    app.redis_tool.draft.ssh_tunnel.port = "22".to_string();
    app.redis_tool.draft.ssh_tunnel.username = "deploy".to_string();
    app.redis_tool.draft.ssh_tunnel.password = "ssh-secret".to_string();
    app.redis_tool.draft.ssh_tunnel.private_key_path = "/tmp/id_rsa".to_string();
    app.redis_tool.draft.ssh_tunnel.passphrase = "phrase".to_string();
    app.redis_tool.draft.ssh_tunnel.timeout_secs = "45".to_string();
    app.redis_tool.draft.use_tls = true;
    app.redis_tool.draft.tls_cert.private_key_path = "/tmp/client.key".to_string();
    app.redis_tool.draft.tls_cert.public_cert_path = "/tmp/client.crt".to_string();
    app.redis_tool.draft.tls_cert.ca_cert_path = "/tmp/ca.crt".to_string();
    app.redis_tool.draft.sentinel.enabled = true;
    app.redis_tool.draft.sentinel.master_name = "mymaster".to_string();
    app.redis_tool.draft.sentinel.node_password = "node-secret".to_string();
    app.redis_tool.draft.use_cluster = true;
    app.redis_tool.draft.read_only = true;
    app
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("connection_tests"));
}

#[test]
fn connection_tab_titles_are_stable() {
    assert_eq!(RedisConnectionTab::Basic.title(), "基础");
    assert_eq!(RedisConnectionTab::Ssh.title(), "SSH");
    assert_eq!(RedisConnectionTab::Tls.title(), "SSL/TLS");
    assert_eq!(RedisConnectionTab::Sentinel.title(), "Sentinel");
    assert_eq!(RedisConnectionTab::Cluster.title(), "Cluster");
}

#[test]
fn tab_bar_builds_active_inactive_and_busy_states() {
    for active_tab in [
        RedisConnectionTab::Basic,
        RedisConnectionTab::Ssh,
        RedisConnectionTab::Tls,
        RedisConnectionTab::Sentinel,
        RedisConnectionTab::Cluster,
    ] {
        let mut app = test_app();
        app.redis_tool.draft_tab = active_tab;

        keep_element(build_tab_bar(&app, false));
        keep_element(build_tab_bar(&app, true));
    }
}

#[test]
fn active_tab_builds_every_connection_section() {
    for active_tab in [
        RedisConnectionTab::Basic,
        RedisConnectionTab::Ssh,
        RedisConnectionTab::Tls,
        RedisConnectionTab::Sentinel,
        RedisConnectionTab::Cluster,
    ] {
        let mut app = test_app();
        app.redis_tool.draft_tab = active_tab;

        keep_element(build_active_tab(&app, false));
        keep_element(build_active_tab(&app, true));
    }
}

#[test]
fn ssh_and_tls_tabs_build_when_gateway_loading_disables_file_pickers() {
    for active_tab in [RedisConnectionTab::Ssh, RedisConnectionTab::Tls] {
        let mut app = test_app();
        app.redis_tool.draft_tab = active_tab;
        app.redis_tool.begin_gateway_request("测试连接");

        keep_element(build_active_tab(&app, false));
        keep_element(build_active_tab(&app, true));
    }
}

#[test]
fn private_tab_builders_cover_compact_and_regular_layouts() {
    let mut app = test_app();
    keep_element(build_basic_tab(&app, false));
    keep_element(build_basic_tab(&app, true));

    app.redis_tool.gateway_loading_label = None;
    keep_element(build_ssh_tab(&app, false));
    keep_element(build_ssh_tab(&app, true));
    keep_element(build_tls_tab(&app, false));
    keep_element(build_tls_tab(&app, true));

    app.redis_tool.begin_gateway_request("加载中");
    keep_element(build_ssh_tab(&app, false));
    keep_element(build_tls_tab(&app, true));

    keep_element(build_sentinel_tab(&app, false));
    keep_element(build_sentinel_tab(&app, true));
    keep_element(build_cluster_tab(&app, false));
    keep_element(build_cluster_tab(&app, true));
}
