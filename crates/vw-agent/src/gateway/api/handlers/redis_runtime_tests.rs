use super::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use vw_api_types::tool::{
    GatewayRedisConnectionConfig, GatewayRedisSentinelConfig, GatewayRedisSshTunnelConfig,
    GatewayRedisTlsCertConfig,
};

fn test_connection() -> GatewayRedisConnectionConfig {
    GatewayRedisConnectionConfig {
        id: "redis-local".to_string(),
        name: "Local Redis".to_string(),
        host: "127.0.0.1".to_string(),
        port: 6379,
        db: 0,
        username: String::new(),
        password: String::new(),
        use_tls: false,
        tls_cert: GatewayRedisTlsCertConfig::default(),
        ssh_tunnel: GatewayRedisSshTunnelConfig::default(),
        sentinel: GatewayRedisSentinelConfig::default(),
        use_cluster: false,
        read_only: false,
        key_pattern: "*".to_string(),
        last_used_ms: None,
        updated_at_ms: 0,
    }
}

struct FakeRedis {
    addr: std::net::SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeRedis {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake redis");
        let addr = listener.local_addr().expect("local addr");
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        let thread_stop = Arc::clone(&stop);
        let thread_state = Arc::clone(&state);
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                if thread_stop.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else {
                    break;
                };
                handle_fake_redis_client(stream, Arc::clone(&thread_state));
            }
        });

        Self { addr, stop, handle: Some(handle) }
    }

    fn connection(&self) -> GatewayRedisConnectionConfig {
        let mut connection = test_connection();
        connection.host = self.addr.ip().to_string();
        connection.port = self.addr.port();
        connection
    }
}

impl Drop for FakeRedis {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_fake_redis_client(stream: TcpStream, state: Arc<Mutex<HashMap<String, String>>>) {
    let mut writer = stream.try_clone().expect("clone stream");
    let mut reader = BufReader::new(stream);

    while let Ok(Some(parts)) = read_resp_command(&mut reader) {
        let response = fake_redis_response(&parts, &state);
        if writer.write_all(response.as_bytes()).is_err() {
            break;
        }
    }
}

fn read_resp_command(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<Vec<String>>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    if !line.starts_with('*') {
        return Ok(None);
    }
    let count = line[1..].trim().parse::<usize>().unwrap_or(0);
    let mut parts = Vec::with_capacity(count);
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line)?;
        if !line.starts_with('$') {
            return Ok(None);
        }
        let len = line[1..].trim().parse::<usize>().unwrap_or(0);
        let mut bytes = vec![0; len + 2];
        reader.read_exact(&mut bytes)?;
        parts.push(String::from_utf8_lossy(&bytes[..len]).to_string());
    }

    Ok(Some(parts))
}

fn fake_redis_response(parts: &[String], state: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let command_name = parts.first().map(|part| part.to_ascii_uppercase()).unwrap_or_default();
    match command_name.as_str() {
        "CLIENT" => simple("OK"),
        "PING" => parts.get(1).map_or_else(|| simple("PONG"), |message| bulk(message)),
        "INFO" => bulk(
            "# Server\r\nredis_version:7.2.4\r\nos:Linux test\r\nprocess_id:42\r\nused_memory_human:1M\r\nused_memory_peak_human:2M\r\nused_memory_lua_human:3K\r\nconnected_clients:5\r\ntotal_connections_received:9\r\ntotal_commands_processed:11\r\n# Keyspace\r\ndb1:keys=1,expires=0,avg_ttl=0\r\ndb0:keys=2,expires=1,avg_ttl=44\r\n",
        ),
        "SCAN" => {
            let cursor = parts.get(1).map(String::as_str).unwrap_or("0");
            if cursor == "7" {
                array(vec![bulk("0"), array(Vec::new())])
            } else {
                array(vec![bulk("7"), array(vec![bulk("z-key"), bulk("a-key")])])
            }
        }
        "TYPE" => {
            let key = parts.get(1).map(String::as_str).unwrap_or_default();
            let stored = state.lock().expect("state").get(key).cloned();
            let kind = stored.unwrap_or_else(|| match key {
                "missing" => "none".to_string(),
                "hash-key" => "hash".to_string(),
                "list-key" => "list".to_string(),
                "set-key" => "set".to_string(),
                "zset-key" => "zset".to_string(),
                "stream-key" => "stream".to_string(),
                "json-key" => "ReJSON-RL".to_string(),
                "unknown-key" => "module-type".to_string(),
                _ => "string".to_string(),
            });
            simple(&kind)
        }
        "TTL" => int(120),
        "MEMORY" => int(42),
        "GET" => {
            if parts.get(1).is_some_and(|key| key == "preview-error") {
                error("preview failed")
            } else {
                bulk("hello")
            }
        }
        "HGETALL" => array(vec![bulk("field"), bulk("value")]),
        "LRANGE" => array(vec![bulk("item-b"), bulk("item-a")]),
        "SMEMBERS" => array(vec![bulk("member-b"), bulk("member-a")]),
        "ZRANGE" => array(vec![bulk("member"), bulk("1")]),
        "XRANGE" => array(vec![array(vec![
            bulk("1690000000000-0"),
            array(vec![bulk("field"), bulk("value")]),
        ])]),
        "JSON.GET" => bulk(r#"{"ok":true}"#),
        "EXISTS" => {
            if parts.get(1).is_some_and(|key| key == "existing-key") {
                int(1)
            } else {
                int(0)
            }
        }
        "SET" => {
            remember_key(parts, state, "string");
            simple("OK")
        }
        "HSET" => {
            remember_key(parts, state, "hash");
            int(1)
        }
        "RPUSH" => {
            remember_key(parts, state, "list");
            int(1)
        }
        "SADD" => {
            remember_key(parts, state, "set");
            int(1)
        }
        "ZADD" => {
            remember_key(parts, state, "zset");
            int(1)
        }
        "XADD" => {
            remember_key(parts, state, "stream");
            bulk("1690000000000-0")
        }
        "JSON.SET" => {
            remember_key(parts, state, "ReJSON-RL");
            simple("OK")
        }
        _ => simple("OK"),
    }
}

fn remember_key(parts: &[String], state: &Arc<Mutex<HashMap<String, String>>>, kind: &str) {
    if let Some(key) = parts.get(1) {
        state.lock().expect("state").insert(key.clone(), kind.to_string());
    }
}

fn simple(value: &str) -> String {
    format!("+{value}\r\n")
}

fn bulk(value: &str) -> String {
    format!("${}\r\n{value}\r\n", value.as_bytes().len())
}

fn int(value: i64) -> String {
    format!(":{value}\r\n")
}

fn array(values: Vec<String>) -> String {
    format!("*{}\r\n{}", values.len(), values.concat())
}

fn error(value: &str) -> String {
    format!("-ERR {value}\r\n")
}

#[test]
fn parse_info_entries_ignores_comments_and_blank_lines() {
    let entries = parse_info_entries("# Server\nredis_version:7.2\n\nconnected_clients:3\n");

    assert_eq!(info_value(&entries, "redis_version").as_deref(), Some("7.2"));
    assert_eq!(parse_info_u64(&entries, "connected_clients"), 3);
    assert_eq!(parse_info_u64(&entries, "missing"), 0);
}

#[test]
fn parse_info_entries_trims_pairs_and_skips_malformed_lines() {
    let entries = parse_info_entries(" no_separator \n name : value with spaces \n bad: \n");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].key, "name");
    assert_eq!(entries[0].value, "value with spaces");
    assert_eq!(entries[1].key, "bad");
    assert_eq!(entries[1].value, "");
}

#[test]
fn parse_keyspace_stats_extracts_db_counts() {
    let entries = parse_info_entries(
        "db10:keys=bad,expires=3,avg_ttl=20\ndb0:keys=2,expires=1,avg_ttl=10\nnot_db:skip\n",
    );
    let stats = parse_keyspace_stats(&entries);

    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].db, "db0");
    assert_eq!(stats[0].keys, 2);
    assert_eq!(stats[0].expires, 1);
    assert_eq!(stats[0].avg_ttl, 10);
    assert_eq!(stats[1].db, "db10");
    assert_eq!(stats[1].keys, 0);
    assert_eq!(stats[1].expires, 3);
}

#[test]
fn parse_keyspace_stats_ignores_malformed_segments_and_unknown_fields() {
    let entries = parse_info_entries(
        "db0:keys=4,unknown=9,expires=2,avg_ttl=7\ndb1:keys=1,broken_segment,expires=1\n",
    );
    let stats = parse_keyspace_stats(&entries);

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].db, "db0");
    assert_eq!(stats[0].keys, 4);
    assert_eq!(stats[0].expires, 2);
    assert_eq!(stats[0].avg_ttl, 7);
}

#[test]
fn load_runtime_overview_maps_info_output_from_direct_connection() {
    let server = FakeRedis::start();
    let overview =
        load_redis_runtime_overview(&server.connection()).expect("overview from fake redis");

    assert_eq!(overview.connection_id, "redis-local");
    assert_eq!(overview.connection_label, "Local Redis");
    assert_eq!(overview.server_version, "7.2.4");
    assert_eq!(overview.os, "Linux test");
    assert_eq!(overview.process_id, "42");
    assert_eq!(overview.used_memory_human, "1M");
    assert_eq!(overview.used_memory_peak_human, "2M");
    assert_eq!(overview.used_memory_lua_human, "3K");
    assert_eq!(overview.connected_clients, 5);
    assert_eq!(overview.total_connections_received, 9);
    assert_eq!(overview.total_commands_processed, 11);
    assert_eq!(overview.keyspace.len(), 2);
    assert_eq!(overview.keyspace[0].db, "db0");
    assert_eq!(overview.keyspace[0].keys, 2);
    assert_eq!(overview.keyspace[0].expires, 1);
    assert_eq!(overview.keyspace[0].avg_ttl, 44);
    assert_eq!(overview.keyspace[1].db, "db1");
}

#[test]
fn scan_redis_keys_trims_pattern_sorts_keys_and_reports_cursor_state() {
    let server = FakeRedis::start();
    let connection = server.connection();

    let first_page = scan_redis_keys(&connection, 0, 10, " app:* ").expect("scan page");
    assert_eq!(first_page.connection_id, "redis-local");
    assert_eq!(first_page.pattern, "app:*");
    assert_eq!(first_page.keys, vec!["a-key".to_string(), "z-key".to_string()]);
    assert_eq!(first_page.next_cursor, 7);
    assert!(first_page.has_more);

    let final_page = scan_redis_keys(&connection, 7, 10, "").expect("final scan page");
    assert_eq!(final_page.pattern, "*");
    assert!(final_page.keys.is_empty());
    assert_eq!(final_page.next_cursor, 0);
    assert!(!final_page.has_more);
}

#[test]
fn summarize_command_name_uppercases_or_falls_back() {
    assert_eq!(summarize_command_name(" get secret-key "), "GET");
    assert_eq!(summarize_command_name(""), "UNKNOWN");
    assert_eq!(summarize_command_name("GET 'unterminated"), "UNKNOWN");
}

#[test]
fn summarize_command_args_reports_count_without_values() {
    assert_eq!(summarize_command_args("SET sensitive value"), "argc=2");
    assert_eq!(summarize_command_args("PING"), "argc=0");
    assert_eq!(summarize_command_args("SET 'unterminated"), "argc=parse_error");
}

#[test]
fn classify_write_command_treats_unknown_as_write() {
    assert!(!classify_write_command("GET"));
    assert!(!classify_write_command("ZRANGE"));
    assert!(classify_write_command("SET"));
    assert!(classify_write_command("CUSTOM"));
}

#[test]
fn classify_runtime_key_kind_normalizes_known_types() {
    assert!(matches!(classify_runtime_key_kind(" string "), RuntimeRedisKeyKind::String));
    assert!(matches!(classify_runtime_key_kind("HASH"), RuntimeRedisKeyKind::Hash));
    assert!(matches!(classify_runtime_key_kind("ReJSON-RL"), RuntimeRedisKeyKind::ReJson));

    let unknown = classify_runtime_key_kind("module-type");
    assert_eq!(unknown.label(), "module-type");
}

#[test]
fn parse_create_key_kind_accepts_supported_aliases() {
    assert!(matches!(parse_create_key_kind("string"), Ok(RuntimeRedisKeyKind::String)));
    assert!(matches!(parse_create_key_kind("hash"), Ok(RuntimeRedisKeyKind::Hash)));
    assert!(matches!(parse_create_key_kind("list"), Ok(RuntimeRedisKeyKind::List)));
    assert!(matches!(parse_create_key_kind("set"), Ok(RuntimeRedisKeyKind::Set)));
    assert!(matches!(parse_create_key_kind("zset"), Ok(RuntimeRedisKeyKind::Zset)));
    assert!(matches!(parse_create_key_kind("stream"), Ok(RuntimeRedisKeyKind::Stream)));
    assert!(matches!(parse_create_key_kind("json"), Ok(RuntimeRedisKeyKind::ReJson)));
    assert!(parse_create_key_kind("bitmap").is_err());
}

#[test]
fn create_default_command_for_key_kind_returns_minimal_seed_commands() {
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::String, "k"),
        ("SET", vec!["k".to_string(), String::new()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::Hash, "k"),
        ("HSET", vec!["k".to_string(), "field".to_string(), String::new()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::List, "k"),
        ("RPUSH", vec!["k".to_string(), "item".to_string()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::Set, "k"),
        ("SADD", vec!["k".to_string(), "member".to_string()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::Zset, "k"),
        ("ZADD", vec!["k".to_string(), "0".to_string(), "member".to_string()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::Stream, "k"),
        ("XADD", vec!["k".to_string(), "*".to_string(), "field".to_string(), "value".to_string()])
    );
    assert_eq!(
        create_default_command_for_key_kind(RuntimeRedisKeyKind::ReJson, "k"),
        ("JSON.SET", vec!["k".to_string(), "$".to_string(), "{}".to_string()])
    );
    assert_eq!(
        create_default_command_for_key_kind(
            RuntimeRedisKeyKind::Unknown("custom".to_string()),
            "k"
        ),
        ("SET", vec!["k".to_string(), String::new()])
    );
}

#[test]
fn preview_command_for_key_kind_matches_runtime_type() {
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::String, "k"),
        (Some("GET"), vec!["k".to_string()])
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::Hash, "k"),
        (Some("HGETALL"), vec!["k".to_string()])
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::List, "k"),
        (Some("LRANGE"), vec!["k".to_string(), "0".to_string(), "99".to_string()])
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::Set, "k"),
        (Some("SMEMBERS"), vec!["k".to_string()])
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::Zset, "k"),
        (
            Some("ZRANGE"),
            vec!["k".to_string(), "0".to_string(), "99".to_string(), "WITHSCORES".to_string()]
        )
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::Stream, "k"),
        (
            Some("XRANGE"),
            vec![
                "k".to_string(),
                "-".to_string(),
                "+".to_string(),
                "COUNT".to_string(),
                "50".to_string()
            ]
        )
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::ReJson, "k"),
        (Some("JSON.GET"), vec!["k".to_string(), "$".to_string()])
    );
    assert_eq!(
        preview_command_for_key_kind(&RuntimeRedisKeyKind::Unknown("geo".to_string()), "k"),
        (None, Vec::new())
    );
}

#[test]
fn build_preview_command_quotes_arguments() {
    let command = build_preview_command(&Some("GET"), &["key with space".to_string()]);

    assert_eq!(command, "GET 'key with space'");
    assert_eq!(build_preview_command(&None, &[]), "当前类型暂不支持预览");
}

#[test]
fn redis_value_formatting_covers_common_response_shapes() {
    assert_eq!(format_redis_value(&::redis::Value::Nil, 0), "(nil)");
    assert_eq!(format_redis_value(&::redis::Value::Int(42), 0), "42");
    assert_eq!(format_redis_value(&::redis::Value::BulkString(b"value".to_vec()), 0), "value");
    assert_eq!(format_redis_value(&::redis::Value::SimpleString("PONG".to_string()), 0), "PONG");
    assert_eq!(format_redis_value(&::redis::Value::Okay, 0), "OK");
    assert_eq!(format_redis_value(&::redis::Value::Array(vec![]), 0), "[]");
    assert_eq!(
        format_redis_value(
            &::redis::Value::Array(vec![
                ::redis::Value::BulkString(b"field".to_vec()),
                ::redis::Value::Array(vec![::redis::Value::Int(7)])
            ]),
            0
        ),
        "1. field\n2.   1. 7"
    );
    assert_eq!(format_redis_value(&::redis::Value::Boolean(true), 0), "boolean(true)");
}

#[test]
fn redis_numeric_value_conversions_accept_redis_number_shapes() {
    assert_eq!(redis_value_to_i64(&::redis::Value::Int(-5)), Some(-5));
    assert_eq!(redis_value_to_i64(&::redis::Value::SimpleString("12".to_string())), Some(12));
    assert_eq!(redis_value_to_i64(&::redis::Value::BulkString(b"34".to_vec())), Some(34));
    assert_eq!(redis_value_to_i64(&::redis::Value::Nil), None);
    assert_eq!(redis_value_to_u64(&::redis::Value::Int(8)), Some(8));
    assert_eq!(redis_value_to_u64(&::redis::Value::Int(-1)), None);
}

#[test]
fn runtime_operations_reject_empty_key_before_network_access() {
    let connection = test_connection();

    assert_eq!(analyze_redis_key(&connection, "   "), Err("Key 名不能为空".to_string()));
    assert_eq!(
        create_redis_key_with_default(&connection, "   ", "string"),
        Err("Key 名不能为空".to_string())
    );
}

#[test]
fn analyze_redis_key_formats_supported_previews_from_direct_connection() {
    let server = FakeRedis::start();
    let connection = server.connection();
    let cases = [
        ("string-key", "String", "GET string-key", "hello"),
        ("hash-key", "Hash", "HGETALL hash-key", "1. field\n2. value"),
        ("list-key", "List", "LRANGE list-key 0 99", "item-b"),
        ("set-key", "Set", "SMEMBERS set-key", "member-b"),
        ("zset-key", "Zset", "ZRANGE zset-key 0 99 WITHSCORES", "member"),
        ("stream-key", "Stream", "XRANGE stream-key - + COUNT 50", "1690000000000-0"),
        ("json-key", "ReJSON", "JSON.GET json-key '$'", r#"{"ok":true}"#),
    ];

    for (key, key_type, preview_command, preview_fragment) in cases {
        let analysis = analyze_redis_key(&connection, key).expect("analysis");

        assert_eq!(analysis.connection_id, "redis-local");
        assert_eq!(analysis.key, key);
        assert_eq!(analysis.key_type, key_type);
        assert_eq!(analysis.ttl_secs, 120);
        assert_eq!(analysis.memory_usage_bytes, Some(42));
        assert_eq!(analysis.preview_command, preview_command);
        assert!(analysis.preview_output.contains(preview_fragment));
    }
}

#[test]
fn analyze_redis_key_handles_missing_unknown_and_preview_error_paths() {
    let server = FakeRedis::start();
    let connection = server.connection();

    assert_eq!(analyze_redis_key(&connection, "missing"), Err("Key 不存在或已被删除".to_string()));

    let unknown = analyze_redis_key(&connection, "unknown-key").expect("unknown analysis");
    assert_eq!(unknown.key_type, "module-type");
    assert_eq!(unknown.preview_command, "当前类型暂不支持预览");
    assert_eq!(unknown.preview_output, "当前类型暂不支持预览");

    let preview_error =
        analyze_redis_key(&connection, "preview-error").expect("preview error analysis");
    assert_eq!(preview_error.key_type, "String");
    assert!(preview_error.preview_output.contains("preview failed"));
}

#[test]
fn create_key_rejects_read_only_before_network_access() {
    let mut connection = test_connection();
    connection.read_only = true;

    assert_eq!(
        create_redis_key_with_default(&connection, "new-key", "string"),
        Err("当前连接为只读模式，不能新增 Key".to_string())
    );
}

#[test]
fn create_key_rejects_unsupported_type_and_existing_key_before_write() {
    let server = FakeRedis::start();
    let connection = server.connection();

    assert_eq!(
        create_redis_key_with_default(&connection, "new-key", "bitmap"),
        Err("暂不支持该 Key 类型，当前仅支持 String / Hash / List / Set / Zset / Stream / ReJSON"
            .to_string())
    );
    assert_eq!(
        create_redis_key_with_default(&connection, "existing-key", "string"),
        Err("Key 已存在，请更换名称".to_string())
    );
}

#[test]
fn create_key_writes_default_value_then_returns_analysis() {
    let server = FakeRedis::start();
    let connection = server.connection();
    let cases = [
        ("created-string", "string", "String"),
        ("created-hash", "hash", "Hash"),
        ("created-list", "list", "List"),
        ("created-set", "set", "Set"),
        ("created-zset", "zset", "Zset"),
        ("created-stream", "stream", "Stream"),
        ("created-json", "json", "ReJSON"),
    ];

    for (key, key_type, expected_label) in cases {
        let analysis =
            create_redis_key_with_default(&connection, key, key_type).expect("created key");

        assert_eq!(analysis.key, key);
        assert_eq!(analysis.key_type, expected_label);
    }
}

#[test]
fn execute_command_rejects_empty_or_invalid_command_line() {
    let connection = test_connection();

    assert_eq!(execute_redis_command(&connection, ""), Err("请输入 Redis 命令".to_string()));
    assert!(
        execute_redis_command(&connection, "GET 'unterminated")
            .expect_err("invalid shell words should fail")
            .starts_with("命令解析失败:")
    );
}

#[test]
fn execute_command_formats_successful_value_response() {
    let server = FakeRedis::start();
    let connection = server.connection();

    assert_eq!(execute_redis_command(&connection, "PING 'hi there'").unwrap(), "hi there");
}

#[test]
fn ping_direct_connection_returns_pong() {
    let server = FakeRedis::start();

    assert_eq!(ping_redis_connection(&server.connection()).unwrap(), "PONG");
}

#[test]
fn redis_runtime_rejects_ssh_tunnel_mode_before_network_access() {
    let mut connection = test_connection();
    connection.ssh_tunnel.enabled = true;

    assert_eq!(
        ping_redis_connection(&connection),
        Err("当前版本暂不支持通过 SSH 隧道测试 Redis 连接".to_string())
    );
    assert_eq!(
        scan_redis_keys(&connection, 0, 10, ""),
        Err("当前版本暂不支持通过 SSH 隧道访问 Redis 运行时数据".to_string())
    );
    assert_eq!(
        analyze_redis_key(&connection, "key"),
        Err("当前版本暂不支持通过 SSH 隧道访问 Redis 运行时数据".to_string())
    );
    assert_eq!(
        execute_redis_command(&connection, "PING"),
        Err("当前版本暂不支持通过 SSH 隧道访问 Redis 运行时数据".to_string())
    );
}

#[test]
fn redis_runtime_rejects_incomplete_sentinel_mode_before_network_access() {
    let mut connection = test_connection();
    connection.sentinel.enabled = true;

    assert_eq!(
        ping_redis_connection(&connection),
        Err("启用 Sentinel 时必须填写 Master 组名称".to_string())
    );
    assert_eq!(
        scan_redis_keys(&connection, 0, 10, ""),
        Err("启用 Sentinel 时必须填写 Master 组名称".to_string())
    );
}

#[test]
fn redis_runtime_rejects_cluster_nonzero_db_before_network_access() {
    let mut connection = test_connection();
    connection.use_cluster = true;
    connection.db = 1;

    assert_eq!(ping_redis_connection(&connection), Err("Cluster 模式仅支持 DB 0".to_string()));
    assert_eq!(scan_redis_keys(&connection, 0, 10, ""), Err("Cluster 模式仅支持 DB 0".to_string()));
}

#[test]
fn runtime_paths_surface_blank_host_and_tls_material_errors_before_network_access() {
    let mut connection = test_connection();
    connection.host = "  ".to_string();

    assert_eq!(load_redis_runtime_overview(&connection), Err("Redis 主机地址不能为空".to_string()));
    assert_eq!(ping_redis_connection(&connection), Err("Redis 主机地址不能为空".to_string()));

    connection.host = "127.0.0.1".to_string();
    connection.use_tls = true;
    connection.tls_cert.ca_cert_path = "/definitely/missing/ca.pem".to_string();

    assert!(
        scan_redis_keys(&connection, 0, 10, "keys:*")
            .expect_err("missing tls material")
            .contains("读取CA 证书失败")
    );
    assert!(
        ping_redis_connection(&connection)
            .expect_err("missing tls material")
            .contains("读取CA 证书失败")
    );
}

#[test]
fn cluster_and_sentinel_tls_paths_load_custom_material_before_network_access() {
    let mut cluster = test_connection();
    cluster.use_cluster = true;
    cluster.use_tls = true;
    cluster.read_only = true;
    cluster.username = "user".to_string();
    cluster.password = "secret".to_string();
    cluster.tls_cert.ca_cert_path = "/definitely/missing/cluster-ca.pem".to_string();

    assert!(
        scan_redis_keys(&cluster, 0, 10, "keys:*")
            .expect_err("missing cluster tls material")
            .contains("读取CA 证书失败")
    );
    assert!(
        ping_redis_connection(&cluster)
            .expect_err("missing cluster tls material")
            .contains("读取CA 证书失败")
    );

    let mut sentinel = test_connection();
    sentinel.sentinel.enabled = true;
    sentinel.sentinel.master_name = "mymaster".to_string();
    sentinel.sentinel.node_password = "node-secret".to_string();
    sentinel.use_tls = true;
    sentinel.read_only = true;
    sentinel.db = 2;
    sentinel.username = "user".to_string();
    sentinel.password = "sentinel-secret".to_string();
    sentinel.tls_cert.ca_cert_path = "/definitely/missing/sentinel-ca.pem".to_string();

    assert!(
        scan_redis_keys(&sentinel, 0, 10, "keys:*")
            .expect_err("missing sentinel tls material")
            .contains("读取CA 证书失败")
    );
    assert!(
        ping_redis_connection(&sentinel)
            .expect_err("missing sentinel tls material")
            .contains("读取CA 证书失败")
    );
}

#[test]
fn cluster_seed_uri_requires_host_and_encodes_auth() {
    let mut connection = test_connection();
    connection.host = " ".to_string();
    assert_eq!(build_cluster_seed_uri(&connection), Err("Redis 主机地址不能为空".to_string()));

    connection.host = "redis.example.test".to_string();
    connection.username = "user name".to_string();
    connection.password = "p@ss word".to_string();
    assert_eq!(
        build_cluster_seed_uri(&connection).expect("valid seed uri"),
        "redis://user%20name:p%40ss%20word@redis.example.test:6379/"
    );

    connection.use_tls = true;
    connection.username.clear();
    connection.password = "secret".to_string();
    assert_eq!(
        build_cluster_seed_uri(&connection).expect("valid tls seed uri"),
        "rediss://:secret@redis.example.test:6379/"
    );

    connection.use_tls = false;
    connection.username = "user only".to_string();
    connection.password.clear();
    assert_eq!(
        build_cluster_seed_uri(&connection).expect("username only uri"),
        "redis://user%20only@redis.example.test:6379/"
    );

    connection.username.clear();
    assert_eq!(
        build_cluster_seed_uri(&connection).expect("no auth uri"),
        "redis://redis.example.test:6379/"
    );
}

#[test]
fn sentinel_addr_uses_tls_when_enabled() {
    let mut connection = test_connection();

    assert_eq!(
        build_sentinel_addr(&connection),
        ::redis::ConnectionAddr::Tcp("127.0.0.1".to_string(), 6379)
    );

    connection.use_tls = true;
    assert_eq!(
        build_sentinel_addr(&connection),
        ::redis::ConnectionAddr::TcpTls {
            host: "127.0.0.1".to_string(),
            port: 6379,
            insecure: false,
            tls_params: None,
        }
    );
}
