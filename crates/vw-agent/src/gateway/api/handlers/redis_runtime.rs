//! Redis 运行时访问与结果格式化。
//!
//! 本模块封装实际 Redis 网络连接、命令执行、key 分析和 PING 检测。
//! handler 层会把这些同步调用放入阻塞线程中运行，因此这里保持直白的同步控制流，
//! 并在不支持的连接模式上显式返回错误。

use vw_api_types::tool::{
    GatewayRedisConnectionConfig, GatewayRedisInfoEntry, GatewayRedisKeyAnalysis,
    GatewayRedisKeyPage, GatewayRedisKeyspaceStat, GatewayRedisRuntimeOverview,
};

use super::config_support::{
    build_connection_uri, has_custom_tls_material, load_tls_certificates_from_paths,
};

/// 加载 Redis 运行时概览。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
///
/// # 返回值
///
/// 返回 INFO 输出解析后的运行时概览。
///
/// # 错误处理
///
/// 连接失败、INFO 命令失败或当前连接模式不支持时返回错误字符串。
pub(super) fn load_redis_runtime_overview(
    connection: &GatewayRedisConnectionConfig,
) -> Result<GatewayRedisRuntimeOverview, String> {
    let info_output = query_redis_string(connection, "INFO", &[])?;
    let info_entries = parse_info_entries(&info_output);

    Ok(GatewayRedisRuntimeOverview {
        connection_id: connection.id.clone(),
        connection_label: connection.name.clone(),
        server_version: info_value(&info_entries, "redis_version").unwrap_or_default(),
        os: info_value(&info_entries, "os").unwrap_or_default(),
        process_id: info_value(&info_entries, "process_id").unwrap_or_default(),
        used_memory_human: info_value(&info_entries, "used_memory_human").unwrap_or_default(),
        used_memory_peak_human: info_value(&info_entries, "used_memory_peak_human")
            .unwrap_or_default(),
        used_memory_lua_human: info_value(&info_entries, "used_memory_lua_human")
            .unwrap_or_default(),
        connected_clients: parse_info_u64(&info_entries, "connected_clients"),
        total_connections_received: parse_info_u64(&info_entries, "total_connections_received"),
        total_commands_processed: parse_info_u64(&info_entries, "total_commands_processed"),
        keyspace: parse_keyspace_stats(&info_entries),
        info_entries,
    })
}

/// 使用 SCAN 分页查询 Redis key。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
/// * `cursor` - Redis SCAN 游标。
/// * `count` - 本次扫描建议数量。
/// * `pattern` - 匹配模式，空值会退回 `*`。
///
/// # 返回值
///
/// 返回排序后的 key 页和下一游标。
pub(super) fn scan_redis_keys(
    connection: &GatewayRedisConnectionConfig,
    cursor: u64,
    count: u32,
    pattern: &str,
) -> Result<GatewayRedisKeyPage, String> {
    let pattern =
        if pattern.trim().is_empty() { "*".to_string() } else { pattern.trim().to_string() };

    let (next_cursor, mut keys) = with_redis_connection(connection, |redis_connection| {
        let mut command = ::redis::cmd("SCAN");
        command.arg(cursor);
        command.arg("MATCH").arg(&pattern);
        command.arg("COUNT").arg(count);
        command.query::<(u64, Vec<String>)>(redis_connection).map_err(|error| error.to_string())
    })?;
    keys.sort();

    Ok(GatewayRedisKeyPage {
        connection_id: connection.id.clone(),
        pattern,
        keys,
        next_cursor,
        has_more: next_cursor != 0,
    })
}

/// 分析单个 Redis key 的类型、TTL、内存和预览内容。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
/// * `key` - 待分析 key。
///
/// # 返回值
///
/// 返回面向桌面 UI 的 key 分析 DTO。
///
/// # 错误处理
///
/// key 为空、key 不存在、连接失败或命令失败时返回错误字符串。
pub(super) fn analyze_redis_key(
    connection: &GatewayRedisConnectionConfig,
    key: &str,
) -> Result<GatewayRedisKeyAnalysis, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Key 名不能为空".to_string());
    }

    let raw_type = query_redis_string(connection, "TYPE", &[key])?;
    let key_kind = classify_runtime_key_kind(&raw_type);
    if matches!(key_kind, RuntimeRedisKeyKind::Unknown(_)) && raw_type.eq_ignore_ascii_case("none")
    {
        return Err("Key 不存在或已被删除".to_string());
    }

    let ttl_secs = query_redis_value(connection, "TTL", &[key.to_string()])
        .ok()
        .and_then(|value| redis_value_to_i64(&value))
        .unwrap_or(-2);
    let memory_usage_bytes =
        query_redis_value(connection, "MEMORY", &["USAGE".to_string(), key.to_string()])
            .ok()
            .and_then(|value| redis_value_to_u64(&value));

    let (preview_command_name, preview_args) = preview_command_for_key_kind(&key_kind, key);
    let preview_command = build_preview_command(&preview_command_name, &preview_args);
    let preview_output = if let Some(command_name) = preview_command_name {
        query_redis_value(connection, command_name, &preview_args)
            .map(|value| format_redis_value(&value, 0))
            .unwrap_or_else(|error| error)
    } else {
        "当前类型暂不支持预览".to_string()
    };

    Ok(GatewayRedisKeyAnalysis {
        connection_id: connection.id.clone(),
        key: key.to_string(),
        key_type: key_kind.label().to_string(),
        ttl_secs,
        memory_usage_bytes,
        preview_command,
        preview_output,
    })
}

/// 创建指定类型的 Redis key，并写入最小默认内容。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
/// * `key` - 新 key 名。
/// * `key_type` - 目标类型。
///
/// # 返回值
///
/// 返回创建后 key 的分析结果。
///
/// # 错误处理
///
/// 只读连接、key 已存在、类型不支持或写入失败时返回错误字符串。
pub(super) fn create_redis_key_with_default(
    connection: &GatewayRedisConnectionConfig,
    key: &str,
    key_type: &str,
) -> Result<GatewayRedisKeyAnalysis, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Key 名不能为空".to_string());
    }
    if connection.read_only {
        return Err("当前连接为只读模式，不能新增 Key".to_string());
    }

    let key_kind = parse_create_key_kind(key_type)?;
    let exists = query_redis_value(connection, "EXISTS", &[key.to_string()])?;
    if redis_value_to_i64(&exists).unwrap_or_default() > 0 {
        return Err("Key 已存在，请更换名称".to_string());
    }

    let (command_name, command_args) = create_default_command_for_key_kind(key_kind, key);
    query_redis_value(connection, command_name, &command_args)?;
    analyze_redis_key(connection, key)
}

/// 执行用户输入的 Redis 命令行。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
/// * `command_line` - 使用 shell-like 引号规则解析的命令行。
///
/// # 返回值
///
/// 返回格式化后的 Redis 响应文本。
///
/// # 错误处理
///
/// 命令解析、连接或 Redis 执行失败时返回错误字符串。
pub(super) fn execute_redis_command(
    connection: &GatewayRedisConnectionConfig,
    command_line: &str,
) -> Result<String, String> {
    let parts =
        shell_words::split(command_line).map_err(|error| format!("命令解析失败: {error}"))?;
    let Some(command_name) = parts.first() else {
        return Err("请输入 Redis 命令".to_string());
    };

    let arguments = parts[1..].to_vec();
    let value = query_redis_value(connection, command_name, &arguments)?;
    Ok(format_redis_value(&value, 0))
}

fn parse_info_entries(info_output: &str) -> Vec<GatewayRedisInfoEntry> {
    info_output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let (key, value) = trimmed.split_once(':')?;
            Some(GatewayRedisInfoEntry {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
            })
        })
        .collect()
}

fn info_value(entries: &[GatewayRedisInfoEntry], key: &str) -> Option<String> {
    entries.iter().find(|entry| entry.key == key).map(|entry| entry.value.clone())
}

fn parse_info_u64(entries: &[GatewayRedisInfoEntry], key: &str) -> u64 {
    info_value(entries, key).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0)
}

fn parse_keyspace_stats(entries: &[GatewayRedisInfoEntry]) -> Vec<GatewayRedisKeyspaceStat> {
    let mut stats = entries
        .iter()
        .filter_map(|entry| {
            if !entry.key.starts_with("db") {
                return None;
            }

            let mut keys = 0;
            let mut expires = 0;
            let mut avg_ttl = 0;
            for segment in entry.value.split(',') {
                let (name, value) = segment.split_once('=')?;
                match name.trim() {
                    "keys" => keys = value.trim().parse::<u64>().ok().unwrap_or(0),
                    "expires" => expires = value.trim().parse::<u64>().ok().unwrap_or(0),
                    "avg_ttl" => avg_ttl = value.trim().parse::<i64>().ok().unwrap_or(0),
                    _ => {}
                }
            }

            Some(GatewayRedisKeyspaceStat { db: entry.key.clone(), keys, expires, avg_ttl })
        })
        .collect::<Vec<_>>();

    stats.sort_by(|left, right| left.db.cmp(&right.db));
    stats
}

/// 提取并规范化 Redis 命令名。
///
/// # 参数
///
/// * `command_line` - 原始命令行。
///
/// # 返回值
///
/// 返回大写命令名；解析失败或命令为空时返回 `UNKNOWN`。
pub(super) fn summarize_command_name(command_line: &str) -> String {
    shell_words::split(command_line)
        .ok()
        .and_then(|parts| parts.into_iter().next())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_ascii_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

/// 生成命令参数数量摘要。
///
/// # 参数
///
/// * `command_line` - 原始命令行。
///
/// # 返回值
///
/// 返回参数数量或解析失败摘要；不会包含原始参数值，避免历史记录泄露敏感内容。
pub(super) fn summarize_command_args(command_line: &str) -> String {
    match shell_words::split(command_line) {
        Ok(parts) => format!("argc={}", parts.len().saturating_sub(1)),
        Err(_) => "argc=parse_error".to_string(),
    }
}

/// 判断 Redis 命令是否应按写操作记录。
///
/// # 参数
///
/// * `command_name` - 已大写化的 Redis 命令名。
///
/// # 返回值
///
/// 明确列出的只读命令返回 `false`，其他命令保守视为写操作。
pub(super) fn classify_write_command(command_name: &str) -> bool {
    !matches!(
        command_name,
        "APPEND"
            | "BITCOUNT"
            | "BITFIELD_RO"
            | "BITPOS"
            | "DBSIZE"
            | "ECHO"
            | "EVALSHA_RO"
            | "EVAL_RO"
            | "EXISTS"
            | "FCALL_RO"
            | "GEODIST"
            | "GEOHASH"
            | "GEOPOS"
            | "GET"
            | "GETBIT"
            | "GETDEL"
            | "GETEX"
            | "GETRANGE"
            | "HGET"
            | "HGETALL"
            | "HEXISTS"
            | "HKEYS"
            | "HLEN"
            | "HMGET"
            | "HRANDFIELD"
            | "HSCAN"
            | "HSTRLEN"
            | "HVALS"
            | "INFO"
            | "KEYS"
            | "LINDEX"
            | "LLEN"
            | "LOLWUT"
            | "LRANGE"
            | "MEMORY"
            | "MGET"
            | "MODULE"
            | "OBJECT"
            | "PING"
            | "PTTL"
            | "RANDOMKEY"
            | "ROLE"
            | "SCAN"
            | "SCARD"
            | "SDIFF"
            | "SINTER"
            | "SISMEMBER"
            | "SMEMBERS"
            | "SMISMEMBER"
            | "SORT_RO"
            | "SRANDMEMBER"
            | "SSCAN"
            | "STRLEN"
            | "SUNION"
            | "TIME"
            | "TTL"
            | "TYPE"
            | "XINFO"
            | "XLEN"
            | "XPENDING"
            | "XRANGE"
            | "XREAD"
            | "XREADGROUP"
            | "XREVRANGE"
            | "ZCARD"
            | "ZCOUNT"
            | "ZDIFF"
            | "ZINTER"
            | "ZLEXCOUNT"
            | "ZMSCORE"
            | "ZRANDMEMBER"
            | "ZRANGE"
            | "ZRANGEBYLEX"
            | "ZRANGEBYSCORE"
            | "ZRANK"
            | "ZREVRANGE"
            | "ZREVRANGEBYLEX"
            | "ZREVRANGEBYSCORE"
            | "ZREVRANK"
            | "ZSCAN"
            | "ZSCORE"
    )
}

fn query_redis_string(
    connection: &GatewayRedisConnectionConfig,
    command_name: &str,
    arguments: &[&str],
) -> Result<String, String> {
    with_redis_connection(connection, |redis_connection| {
        let mut command = ::redis::cmd(command_name);
        for argument in arguments {
            command.arg(argument);
        }
        command.query::<String>(redis_connection).map_err(|error| error.to_string())
    })
}

fn query_redis_value(
    connection: &GatewayRedisConnectionConfig,
    command_name: &str,
    arguments: &[String],
) -> Result<::redis::Value, String> {
    with_redis_connection(connection, |redis_connection| {
        let mut command = ::redis::cmd(command_name);
        for argument in arguments {
            command.arg(argument);
        }
        command.query::<::redis::Value>(redis_connection).map_err(|error| error.to_string())
    })
}

fn with_redis_connection<T>(
    connection: &GatewayRedisConnectionConfig,
    operation: impl FnOnce(&mut dyn ::redis::ConnectionLike) -> Result<T, String>,
) -> Result<T, String> {
    if connection.ssh_tunnel.enabled {
        // SSH 隧道需要额外生命周期管理；当前运行时路径未实现时直接拒绝，避免伪支持。
        return Err("当前版本暂不支持通过 SSH 隧道访问 Redis 运行时数据".to_string());
    }

    if connection.sentinel.enabled {
        let service_name = connection.sentinel.master_name.trim();
        if service_name.is_empty() {
            return Err("启用 Sentinel 时必须填写 Master 组名称".to_string());
        }

        let server_type = if connection.read_only {
            ::redis::sentinel::SentinelServerType::Replica
        } else {
            ::redis::sentinel::SentinelServerType::Master
        };
        let mut builder = ::redis::sentinel::SentinelClientBuilder::new(
            vec![build_sentinel_addr(connection)],
            service_name.to_string(),
            server_type,
        )
        .map_err(|error| error.to_string())?;

        if connection.use_tls {
            builder = builder.set_client_to_sentinel_tls_mode(::redis::TlsMode::Secure);
            builder = builder.set_client_to_redis_tls_mode(::redis::TlsMode::Secure);
        }
        if connection.db != 0 {
            builder = builder.set_client_to_redis_db(connection.db);
        }
        if !connection.username.trim().is_empty() {
            let username = connection.username.trim().to_string();
            builder = builder.set_client_to_sentinel_username(username.clone());
            builder = builder.set_client_to_redis_username(username);
        }
        if !connection.password.trim().is_empty() {
            builder =
                builder.set_client_to_sentinel_password(connection.password.trim().to_string());
        }

        let redis_password = if !connection.sentinel.node_password.trim().is_empty() {
            connection.sentinel.node_password.trim()
        } else {
            connection.password.trim()
        };
        if !redis_password.is_empty() {
            builder = builder.set_client_to_redis_password(redis_password.to_string());
        }

        if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
            let certificates = load_tls_certificates_from_paths(&connection.tls_cert)?;
            builder = builder.set_client_to_sentinel_certificates(certificates.clone());
            builder = builder.set_client_to_redis_certificates(certificates);
        }

        let mut client = builder.build().map_err(|error| error.to_string())?;
        let mut redis_connection = client.get_connection().map_err(|error| error.to_string())?;
        return operation(&mut redis_connection);
    }

    if connection.use_cluster {
        if connection.db != 0 {
            // Redis Cluster 固定使用 DB 0，提前报错比让底层客户端返回模糊错误更清晰。
            return Err("Cluster 模式仅支持 DB 0".to_string());
        }

        let seed_uri = build_cluster_seed_uri(connection)?;
        let mut builder = ::redis::cluster::ClusterClientBuilder::new(vec![seed_uri]);

        if !connection.username.trim().is_empty() {
            builder = builder.username(connection.username.trim().to_string());
        }
        if !connection.password.trim().is_empty() {
            builder = builder.password(connection.password.trim().to_string());
        }
        if connection.use_tls {
            builder = builder.tls(::redis::TlsMode::Secure);
        }
        if connection.read_only {
            builder = builder.read_from_replicas();
        }
        if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
            builder = builder.certs(load_tls_certificates_from_paths(&connection.tls_cert)?);
        }

        let client = builder.build().map_err(|error| error.to_string())?;
        let mut redis_connection = client.get_connection().map_err(|error| error.to_string())?;
        return operation(&mut redis_connection);
    }

    let uri = build_connection_uri(connection).map_err(|error| error.to_string())?;
    if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
        let certificates = load_tls_certificates_from_paths(&connection.tls_cert)?;
        let client = ::redis::Client::build_with_tls(uri, certificates)
            .map_err(|error| error.to_string())?;
        let mut redis_connection = client.get_connection().map_err(|error| error.to_string())?;
        return operation(&mut redis_connection);
    }

    let client = ::redis::Client::open(uri).map_err(|error| error.to_string())?;
    let mut redis_connection = client.get_connection().map_err(|error| error.to_string())?;
    operation(&mut redis_connection)
}

fn format_redis_value(value: &::redis::Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    match value {
        ::redis::Value::Nil => "(nil)".to_string(),
        ::redis::Value::Int(number) => number.to_string(),
        ::redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
        ::redis::Value::Array(items) => {
            if items.is_empty() {
                return "[]".to_string();
            }

            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    format!(
                        "{}{}. {}",
                        indent,
                        index + 1,
                        format_redis_value(item, depth.saturating_add(1))
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        ::redis::Value::SimpleString(value) => value.clone(),
        ::redis::Value::Okay => "OK".to_string(),
        other => format!("{other:?}"),
    }
}

#[derive(Debug, Clone)]
enum RuntimeRedisKeyKind {
    String,
    Hash,
    List,
    Set,
    Zset,
    Stream,
    ReJson,
    Unknown(String),
}

impl RuntimeRedisKeyKind {
    fn label(&self) -> &str {
        match self {
            Self::String => "String",
            Self::Hash => "Hash",
            Self::List => "List",
            Self::Set => "Set",
            Self::Zset => "Zset",
            Self::Stream => "Stream",
            Self::ReJson => "ReJSON",
            Self::Unknown(raw) => raw.as_str(),
        }
    }
}

fn classify_runtime_key_kind(raw_type: &str) -> RuntimeRedisKeyKind {
    let normalized = raw_type.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "string" => RuntimeRedisKeyKind::String,
        "hash" => RuntimeRedisKeyKind::Hash,
        "list" => RuntimeRedisKeyKind::List,
        "set" => RuntimeRedisKeyKind::Set,
        "zset" => RuntimeRedisKeyKind::Zset,
        "stream" => RuntimeRedisKeyKind::Stream,
        _ if normalized.contains("json") => RuntimeRedisKeyKind::ReJson,
        _ => RuntimeRedisKeyKind::Unknown(raw_type.to_string()),
    }
}

fn parse_create_key_kind(value: &str) -> Result<RuntimeRedisKeyKind, String> {
    let normalized = value.trim().to_ascii_lowercase();
    let kind = match normalized.as_str() {
        "string" => RuntimeRedisKeyKind::String,
        "hash" => RuntimeRedisKeyKind::Hash,
        "list" => RuntimeRedisKeyKind::List,
        "set" => RuntimeRedisKeyKind::Set,
        "zset" => RuntimeRedisKeyKind::Zset,
        "stream" => RuntimeRedisKeyKind::Stream,
        "rejson" | "json" => RuntimeRedisKeyKind::ReJson,
        _ => return Err(
            "暂不支持该 Key 类型，当前仅支持 String / Hash / List / Set / Zset / Stream / ReJSON"
                .to_string(),
        ),
    };

    Ok(kind)
}

fn create_default_command_for_key_kind(
    key_kind: RuntimeRedisKeyKind,
    key: &str,
) -> (&'static str, Vec<String>) {
    match key_kind {
        RuntimeRedisKeyKind::String => ("SET", vec![key.to_string(), String::new()]),
        RuntimeRedisKeyKind::Hash => {
            ("HSET", vec![key.to_string(), "field".to_string(), String::new()])
        }
        RuntimeRedisKeyKind::List => ("RPUSH", vec![key.to_string(), "item".to_string()]),
        RuntimeRedisKeyKind::Set => ("SADD", vec![key.to_string(), "member".to_string()]),
        RuntimeRedisKeyKind::Zset => {
            ("ZADD", vec![key.to_string(), "0".to_string(), "member".to_string()])
        }
        RuntimeRedisKeyKind::Stream => (
            "XADD",
            vec![key.to_string(), "*".to_string(), "field".to_string(), "value".to_string()],
        ),
        RuntimeRedisKeyKind::ReJson => {
            ("JSON.SET", vec![key.to_string(), "$".to_string(), "{}".to_string()])
        }
        RuntimeRedisKeyKind::Unknown(_) => ("SET", vec![key.to_string(), String::new()]),
    }
}

fn preview_command_for_key_kind(
    key_kind: &RuntimeRedisKeyKind,
    key: &str,
) -> (Option<&'static str>, Vec<String>) {
    match key_kind {
        RuntimeRedisKeyKind::String => (Some("GET"), vec![key.to_string()]),
        RuntimeRedisKeyKind::Hash => (Some("HGETALL"), vec![key.to_string()]),
        RuntimeRedisKeyKind::List => {
            (Some("LRANGE"), vec![key.to_string(), "0".to_string(), "99".to_string()])
        }
        RuntimeRedisKeyKind::Set => (Some("SMEMBERS"), vec![key.to_string()]),
        RuntimeRedisKeyKind::Zset => (
            Some("ZRANGE"),
            vec![key.to_string(), "0".to_string(), "99".to_string(), "WITHSCORES".to_string()],
        ),
        RuntimeRedisKeyKind::Stream => (
            Some("XRANGE"),
            vec![
                key.to_string(),
                "-".to_string(),
                "+".to_string(),
                "COUNT".to_string(),
                "50".to_string(),
            ],
        ),
        RuntimeRedisKeyKind::ReJson => (Some("JSON.GET"), vec![key.to_string(), "$".to_string()]),
        RuntimeRedisKeyKind::Unknown(_) => (None, Vec::new()),
    }
}

fn build_preview_command(command_name: &Option<&'static str>, arguments: &[String]) -> String {
    let Some(command_name) = command_name else {
        return "当前类型暂不支持预览".to_string();
    };

    std::iter::once(command_name.to_string())
        .chain(arguments.iter().map(|argument| shell_words::quote(argument).into_owned()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn redis_value_to_i64(value: &::redis::Value) -> Option<i64> {
    match value {
        ::redis::Value::Int(number) => Some(*number),
        ::redis::Value::SimpleString(value) => value.parse::<i64>().ok(),
        ::redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).parse::<i64>().ok(),
        _ => None,
    }
}

fn redis_value_to_u64(value: &::redis::Value) -> Option<u64> {
    redis_value_to_i64(value).and_then(|value| u64::try_from(value).ok())
}

/// 测试 Redis 连接。
///
/// # 参数
///
/// * `connection` - 目标 Redis 连接配置。
///
/// # 返回值
///
/// 返回 PING 响应，并在 Sentinel/Cluster 模式下附加所选角色说明。
///
/// # 错误处理
///
/// 不支持 SSH 隧道、连接失败或 PING 失败时返回错误字符串。
pub(super) fn ping_redis_connection(
    connection: &GatewayRedisConnectionConfig,
) -> Result<String, String> {
    if connection.ssh_tunnel.enabled {
        // 与运行时查询保持一致：未实现隧道连接前，测试入口也显式拒绝。
        return Err("当前版本暂不支持通过 SSH 隧道测试 Redis 连接".to_string());
    }

    if connection.sentinel.enabled {
        return ping_sentinel_connection(connection);
    }

    if connection.use_cluster {
        return ping_cluster_connection(connection);
    }

    ping_direct_connection(connection)
}

fn ping_direct_connection(connection: &GatewayRedisConnectionConfig) -> Result<String, String> {
    if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
        let uri = build_connection_uri(connection).map_err(|error| error.to_string())?;
        let certificates = load_tls_certificates_from_paths(&connection.tls_cert)?;
        let client = ::redis::Client::build_with_tls(uri, certificates)
            .map_err(|error| error.to_string())?;
        let mut direct_connection = client.get_connection().map_err(|error| error.to_string())?;
        return ::redis::cmd("PING")
            .query::<String>(&mut direct_connection)
            .map_err(|error| error.to_string());
    }

    let uri = build_connection_uri(connection).map_err(|error| error.to_string())?;
    ping_redis(uri)
}

fn ping_cluster_connection(connection: &GatewayRedisConnectionConfig) -> Result<String, String> {
    if connection.db != 0 {
        return Err("Cluster 模式仅支持 DB 0".to_string());
    }

    let seed_uri = build_cluster_seed_uri(connection)?;
    let mut builder = ::redis::cluster::ClusterClientBuilder::new(vec![seed_uri]);

    if !connection.username.trim().is_empty() {
        builder = builder.username(connection.username.trim().to_string());
    }
    if !connection.password.trim().is_empty() {
        builder = builder.password(connection.password.trim().to_string());
    }
    if connection.use_tls {
        builder = builder.tls(::redis::TlsMode::Secure);
    }
    if connection.read_only {
        builder = builder.read_from_replicas();
    }
    if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
        builder = builder.certs(load_tls_certificates_from_paths(&connection.tls_cert)?);
    }

    let client = builder.build().map_err(|error| error.to_string())?;
    let mut cluster_connection = client.get_connection().map_err(|error| error.to_string())?;
    let pong = ::redis::cmd("PING")
        .query::<String>(&mut cluster_connection)
        .map_err(|error| error.to_string())?;
    let route = if connection.read_only { "replica" } else { "primary" };
    Ok(format!("Cluster {pong} ({route})"))
}

fn ping_sentinel_connection(connection: &GatewayRedisConnectionConfig) -> Result<String, String> {
    let service_name = connection.sentinel.master_name.trim();
    if service_name.is_empty() {
        return Err("启用 Sentinel 时必须填写 Master 组名称".to_string());
    }

    let server_type = if connection.read_only {
        ::redis::sentinel::SentinelServerType::Replica
    } else {
        ::redis::sentinel::SentinelServerType::Master
    };
    let mut builder = ::redis::sentinel::SentinelClientBuilder::new(
        vec![build_sentinel_addr(connection)],
        service_name.to_string(),
        server_type,
    )
    .map_err(|error| error.to_string())?;

    if connection.use_tls {
        builder = builder.set_client_to_sentinel_tls_mode(::redis::TlsMode::Secure);
        builder = builder.set_client_to_redis_tls_mode(::redis::TlsMode::Secure);
    }
    if connection.db != 0 {
        builder = builder.set_client_to_redis_db(connection.db);
    }
    if !connection.username.trim().is_empty() {
        let username = connection.username.trim().to_string();
        builder = builder.set_client_to_sentinel_username(username.clone());
        builder = builder.set_client_to_redis_username(username);
    }
    if !connection.password.trim().is_empty() {
        builder = builder.set_client_to_sentinel_password(connection.password.trim().to_string());
    }

    let redis_password = if !connection.sentinel.node_password.trim().is_empty() {
        connection.sentinel.node_password.trim()
    } else {
        connection.password.trim()
    };
    if !redis_password.is_empty() {
        builder = builder.set_client_to_redis_password(redis_password.to_string());
    }

    if connection.use_tls && has_custom_tls_material(&connection.tls_cert) {
        let certificates = load_tls_certificates_from_paths(&connection.tls_cert)?;
        builder = builder.set_client_to_sentinel_certificates(certificates.clone());
        builder = builder.set_client_to_redis_certificates(certificates);
    }

    let mut client = builder.build().map_err(|error| error.to_string())?;
    let mut sentinel_connection = client.get_connection().map_err(|error| error.to_string())?;
    let pong = ::redis::cmd("PING")
        .query::<String>(&mut sentinel_connection)
        .map_err(|error| error.to_string())?;
    let role = if connection.read_only { "replica" } else { "master" };
    Ok(format!("Sentinel {pong} ({role})"))
}

fn build_cluster_seed_uri(connection: &GatewayRedisConnectionConfig) -> Result<String, String> {
    if connection.host.trim().is_empty() {
        return Err("Redis 主机地址不能为空".to_string());
    }

    let scheme = if connection.use_tls { "rediss" } else { "redis" };
    let username = urlencoding::encode(connection.username.trim());
    let password = urlencoding::encode(connection.password.trim());
    let auth = if connection.password.trim().is_empty() && connection.username.trim().is_empty() {
        String::new()
    } else if connection.username.trim().is_empty() {
        format!(":{password}@")
    } else if connection.password.trim().is_empty() {
        format!("{username}@")
    } else {
        format!("{username}:{password}@")
    };

    Ok(format!("{scheme}://{auth}{}:{}/", connection.host, connection.port))
}

fn build_sentinel_addr(connection: &GatewayRedisConnectionConfig) -> ::redis::ConnectionAddr {
    if connection.use_tls {
        ::redis::ConnectionAddr::TcpTls {
            host: connection.host.clone(),
            port: connection.port,
            insecure: false,
            tls_params: None,
        }
    } else {
        ::redis::ConnectionAddr::Tcp(connection.host.clone(), connection.port)
    }
}

fn ping_redis(uri: String) -> Result<String, String> {
    let client = ::redis::Client::open(uri).map_err(|error| error.to_string())?;
    let mut connection = client.get_connection().map_err(|error| error.to_string())?;
    ::redis::cmd("PING").query::<String>(&mut connection).map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "redis_runtime_tests.rs"]
mod redis_runtime_tests;
