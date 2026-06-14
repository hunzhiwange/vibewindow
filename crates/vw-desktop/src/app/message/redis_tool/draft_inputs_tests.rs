#![allow(unused_must_use)]
use super::*;
use crate::app::state::RedisConnectionTab;

fn app() -> App {
    App::new().0
}

#[test]
fn draft_basic_fields_are_updated() {
    let mut app = app();

    draft_name_changed(&mut app, "local".to_string());
    draft_host_changed(&mut app, "redis.local".to_string());
    draft_port_changed(&mut app, "6380".to_string());
    draft_db_changed(&mut app, "2".to_string());
    draft_username_changed(&mut app, "user".to_string());
    draft_password_changed(&mut app, "secret".to_string());
    draft_tab_changed(&mut app, RedisConnectionTab::Ssh);

    assert_eq!(app.redis_tool.draft.name, "local");
    assert_eq!(app.redis_tool.draft.host, "redis.local");
    assert_eq!(app.redis_tool.draft.port, "6380");
    assert_eq!(app.redis_tool.draft.db, "2");
    assert_eq!(app.redis_tool.draft.username, "user");
    assert_eq!(app.redis_tool.draft.password, "secret");
    assert_eq!(app.redis_tool.draft_tab, RedisConnectionTab::Ssh);
}

#[test]
fn draft_tls_fields_and_picked_files_are_updated_only_when_some() {
    let mut app = app();
    app.redis_tool.draft.tls_cert.private_key_path = "old-key".to_string();
    app.redis_tool.draft.tls_cert.public_cert_path = "old-cert".to_string();
    app.redis_tool.draft.tls_cert.ca_cert_path = "old-ca".to_string();

    draft_tls_toggled(&mut app, true);
    draft_tls_private_key_path_changed(&mut app, "typed-key".to_string());
    draft_tls_public_cert_path_changed(&mut app, "typed-cert".to_string());
    draft_tls_ca_cert_path_changed(&mut app, "typed-ca".to_string());
    tls_private_key_file_picked(&mut app, Some("picked-key".to_string()));
    tls_public_cert_file_picked(&mut app, Some("picked-cert".to_string()));
    tls_ca_cert_file_picked(&mut app, Some("picked-ca".to_string()));

    assert!(app.redis_tool.draft.use_tls);
    assert_eq!(app.redis_tool.draft.tls_cert.private_key_path, "picked-key");
    assert_eq!(app.redis_tool.draft.tls_cert.public_cert_path, "picked-cert");
    assert_eq!(app.redis_tool.draft.tls_cert.ca_cert_path, "picked-ca");

    tls_private_key_file_picked(&mut app, None);
    tls_public_cert_file_picked(&mut app, None);
    tls_ca_cert_file_picked(&mut app, None);

    assert_eq!(app.redis_tool.draft.tls_cert.private_key_path, "picked-key");
    assert_eq!(app.redis_tool.draft.tls_cert.public_cert_path, "picked-cert");
    assert_eq!(app.redis_tool.draft.tls_cert.ca_cert_path, "picked-ca");
}

#[test]
fn draft_ssh_fields_and_picked_key_are_updated_only_when_some() {
    let mut app = app();

    draft_ssh_enabled_toggled(&mut app, true);
    draft_ssh_host_changed(&mut app, "bastion".to_string());
    draft_ssh_port_changed(&mut app, "2222".to_string());
    draft_ssh_username_changed(&mut app, "ssh-user".to_string());
    draft_ssh_password_changed(&mut app, "ssh-secret".to_string());
    draft_ssh_private_key_path_changed(&mut app, "typed-key".to_string());
    draft_ssh_passphrase_changed(&mut app, "phrase".to_string());
    draft_ssh_timeout_secs_changed(&mut app, "45".to_string());
    ssh_private_key_file_picked(&mut app, Some("picked-key".to_string()));

    assert!(app.redis_tool.draft.ssh_tunnel.enabled);
    assert_eq!(app.redis_tool.draft.ssh_tunnel.host, "bastion");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.port, "2222");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.username, "ssh-user");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.password, "ssh-secret");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.private_key_path, "picked-key");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.passphrase, "phrase");
    assert_eq!(app.redis_tool.draft.ssh_tunnel.timeout_secs, "45");

    ssh_private_key_file_picked(&mut app, None);

    assert_eq!(app.redis_tool.draft.ssh_tunnel.private_key_path, "picked-key");
}

#[test]
fn draft_sentinel_cluster_and_pattern_fields_are_updated() {
    let mut app = app();

    draft_sentinel_enabled_toggled(&mut app, true);
    draft_sentinel_master_name_changed(&mut app, "mymaster".to_string());
    draft_sentinel_node_password_changed(&mut app, "node-secret".to_string());
    draft_cluster_toggled(&mut app, true);
    draft_read_only_toggled(&mut app, true);
    draft_key_pattern_changed(&mut app, "app:*".to_string());

    assert!(app.redis_tool.draft.sentinel.enabled);
    assert_eq!(app.redis_tool.draft.sentinel.master_name, "mymaster");
    assert_eq!(app.redis_tool.draft.sentinel.node_password, "node-secret");
    assert!(app.redis_tool.draft.use_cluster);
    assert!(app.redis_tool.draft.read_only);
    assert_eq!(app.redis_tool.draft.key_pattern, "app:*");
}
