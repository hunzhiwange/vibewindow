#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("draft_tests"));
}

use super::{build_draft_uri, draft_to_upsert_body};
use crate::app::state::RedisConnectionDraft;

fn draft() -> RedisConnectionDraft {
    RedisConnectionDraft {
        name: " Local Redis ".to_string(),
        host: " 127.0.0.1 ".to_string(),
        port: " 6379 ".to_string(),
        db: " 0 ".to_string(),
        username: String::new(),
        password: String::new(),
        use_tls: false,
        tls_cert: Default::default(),
        ssh_tunnel: Default::default(),
        sentinel: Default::default(),
        use_cluster: false,
        read_only: false,
        key_pattern: String::new(),
    }
}

#[test]
fn build_draft_uri_formats_plain_tls_and_encoded_auth_variants() {
    let mut draft = draft();

    assert_eq!(build_draft_uri(&draft), Ok("redis://127.0.0.1:6379/0".to_string()));

    draft.use_tls = true;
    draft.username = "user name".to_string();
    assert_eq!(
        build_draft_uri(&draft),
        Ok("rediss://user%20name@127.0.0.1:6379/0".to_string())
    );

    draft.username.clear();
    draft.password = "p@ ss".to_string();
    assert_eq!(
        build_draft_uri(&draft),
        Ok("rediss://:p%40%20ss@127.0.0.1:6379/0".to_string())
    );

    draft.username = "user/name".to_string();
    assert_eq!(
        build_draft_uri(&draft),
        Ok("rediss://user%2Fname:p%40%20ss@127.0.0.1:6379/0".to_string())
    );
}

#[test]
fn build_draft_uri_rejects_missing_and_invalid_core_fields() {
    let mut draft = draft();

    draft.host = "   ".to_string();
    assert_eq!(build_draft_uri(&draft), Err("请输入 Redis 主机地址".to_string()));

    draft = self::draft();
    draft.port = "0".to_string();
    assert_eq!(build_draft_uri(&draft), Ok("redis://127.0.0.1:0/0".to_string()));

    draft.port = "65536".to_string();
    assert_eq!(build_draft_uri(&draft), Err("端口必须是 1-65535 的整数".to_string()));

    draft = self::draft();
    draft.db = "main".to_string();
    assert_eq!(build_draft_uri(&draft), Err("数据库编号必须是整数".to_string()));
}

#[test]
fn build_draft_uri_rejects_modes_that_cannot_be_serialized_to_one_uri() {
    let mut draft = draft();

    draft.ssh_tunnel.enabled = true;
    assert_eq!(
        build_draft_uri(&draft),
        Err("当前版本暂不支持通过 SSH 隧道复制标准 Redis URI".to_string())
    );

    draft = self::draft();
    draft.sentinel.enabled = true;
    assert_eq!(
        build_draft_uri(&draft),
        Err("当前版本暂不支持为 Sentinel 模式生成单条 Redis URI".to_string())
    );

    draft = self::draft();
    draft.use_cluster = true;
    assert_eq!(
        build_draft_uri(&draft),
        Err("当前版本暂不支持为 Cluster 模式生成单条 Redis URI".to_string())
    );

    draft = self::draft();
    draft.use_tls = true;
    draft.tls_cert.ca_cert_path = "/ca.pem".to_string();
    assert_eq!(
        build_draft_uri(&draft),
        Err("当前版本暂不支持将自定义 SSL 证书路径编码为连接 URI".to_string())
    );
}

#[test]
fn draft_to_upsert_body_trims_fields_and_applies_defaults() {
    let mut draft = draft();
    draft.username = " alice ".to_string();
    draft.password = " secret ".to_string();
    draft.key_pattern = "   ".to_string();
    draft.read_only = true;

    let body = draft_to_upsert_body(&draft).expect("valid upsert body");

    assert_eq!(body.name, "Local Redis");
    assert_eq!(body.host, "127.0.0.1");
    assert_eq!(body.port, 6379);
    assert_eq!(body.db, 0);
    assert_eq!(body.username, "alice");
    assert_eq!(body.password, "secret");
    assert_eq!(body.key_pattern, "*");
    assert!(body.read_only);
    assert_eq!(body.ssh_tunnel.port, 22);
    assert_eq!(body.ssh_tunnel.timeout_secs, 30);
}

#[test]
fn draft_to_upsert_body_keeps_advanced_options_when_valid() {
    let mut draft = draft();
    draft.use_tls = true;
    draft.tls_cert.private_key_path = " /key.pem ".to_string();
    draft.tls_cert.public_cert_path = " /cert.pem ".to_string();
    draft.tls_cert.ca_cert_path = " /ca.pem ".to_string();
    draft.ssh_tunnel.enabled = true;
    draft.ssh_tunnel.host = " ssh.example.test ".to_string();
    draft.ssh_tunnel.port = " 2222 ".to_string();
    draft.ssh_tunnel.username = " deploy ".to_string();
    draft.ssh_tunnel.password = " pass ".to_string();
    draft.ssh_tunnel.private_key_path = " /id_rsa ".to_string();
    draft.ssh_tunnel.passphrase = " phrase ".to_string();
    draft.ssh_tunnel.timeout_secs = "0".to_string();
    draft.sentinel.enabled = true;
    draft.sentinel.master_name = " mymaster ".to_string();
    draft.sentinel.node_password = " node-pass ".to_string();
    draft.key_pattern = " app:* ".to_string();

    let body = draft_to_upsert_body(&draft).expect("advanced upsert body");

    assert!(body.use_tls);
    assert_eq!(body.tls_cert.private_key_path, "/key.pem");
    assert!(body.ssh_tunnel.enabled);
    assert_eq!(body.ssh_tunnel.host, "ssh.example.test");
    assert_eq!(body.ssh_tunnel.port, 2222);
    assert_eq!(body.ssh_tunnel.timeout_secs, 1);
    assert!(body.sentinel.enabled);
    assert_eq!(body.sentinel.master_name, "mymaster");
    assert_eq!(body.key_pattern, "app:*");
}

#[test]
fn draft_to_upsert_body_validates_required_and_conflicting_fields() {
    let mut draft = draft();
    draft.name.clear();
    assert_eq!(draft_to_upsert_body(&draft), Err("请输入连接名称".to_string()));

    draft = self::draft();
    draft.host.clear();
    assert_eq!(draft_to_upsert_body(&draft), Err("请输入 Redis 主机地址".to_string()));

    draft = self::draft();
    draft.port = "bad".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("端口必须是 1-65535 的整数".to_string()));

    draft = self::draft();
    draft.db = "bad".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("数据库编号必须是整数".to_string()));

    draft = self::draft();
    draft.sentinel.enabled = true;
    draft.use_cluster = true;
    assert_eq!(draft_to_upsert_body(&draft), Err("Sentinel 与 Cluster 不能同时启用".to_string()));

    draft = self::draft();
    draft.use_cluster = true;
    draft.db = "1".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("Cluster 模式仅支持 DB 0".to_string()));

    draft = self::draft();
    draft.tls_cert.ca_cert_path = "/ca.pem".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("使用证书文件前必须启用 SSL/TLS".to_string()));
}

#[test]
fn draft_to_upsert_body_validates_ssh_sentinel_and_numeric_defaults() {
    let mut draft = draft();
    draft.ssh_tunnel.enabled = true;
    assert_eq!(draft_to_upsert_body(&draft), Err("启用 SSH 时必须填写 SSH 地址".to_string()));

    draft.ssh_tunnel.host = "ssh.example.test".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("启用 SSH 时必须填写 SSH 用户名".to_string()));

    draft.ssh_tunnel.username = "deploy".to_string();
    draft.ssh_tunnel.port = "bad".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("SSH 端口必须是 1-65535 的整数".to_string()));

    draft.ssh_tunnel.port = "22".to_string();
    draft.ssh_tunnel.timeout_secs = "bad".to_string();
    assert_eq!(draft_to_upsert_body(&draft), Err("SSH 超时必须是正整数".to_string()));

    draft = self::draft();
    draft.sentinel.enabled = true;
    assert_eq!(
        draft_to_upsert_body(&draft),
        Err("启用 Sentinel 时必须填写 Master 组名称".to_string())
    );
}
