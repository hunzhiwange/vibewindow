//! Redis 连接配置的归一化与 URI/证书辅助逻辑。
//!
//! 本模块只处理配置形态转换、输入校验和连接字符串构造，不直接访问 Redis。
//! 这样可以让 HTTP handler、导入流程和运行时连接逻辑复用同一套边界检查，
//! 避免不同入口接受不一致的 Redis 配置。

use std::collections::HashSet;

use uuid::Uuid;
use vw_api_types::tool::{
    GatewayRedisConnectionConfig, GatewayRedisConnectionUpsertBody,
    GatewayRedisSentinelConfig, GatewayRedisSshTunnelConfig, GatewayRedisTlsCertConfig,
};

use crate::app::agent::gateway::ApiError;

/// 根据前端 upsert 请求创建新的 Redis 连接配置。
///
/// # 参数
///
/// * `body` - 前端提交的连接配置草稿。
/// * `now_ms` - 当前毫秒时间戳，用于生成 id 和更新时间。
///
/// # 返回值
///
/// 返回已归一化并带有稳定 id 的连接配置。
///
/// # 错误处理
///
/// 名称、主机、Sentinel/Cluster 互斥关系、SSH 必填项或 TLS 开关不合法时返回
/// [`ApiError`]。
pub(super) fn new_connection_from_upsert(
    body: &GatewayRedisConnectionUpsertBody,
    now_ms: u64,
) -> Result<GatewayRedisConnectionConfig, ApiError> {
    let normalized = normalize_upsert_body(body)?;
    Ok(GatewayRedisConnectionConfig {
        id: format!("redis-{now_ms}-{}", Uuid::new_v4()),
        name: normalized.name,
        host: normalized.host,
        port: normalized.port,
        db: normalized.db,
        username: normalized.username,
        password: normalized.password,
        use_tls: normalized.use_tls,
        tls_cert: normalized.tls_cert,
        ssh_tunnel: normalized.ssh_tunnel,
        sentinel: normalized.sentinel,
        use_cluster: normalized.use_cluster,
        read_only: normalized.read_only,
        key_pattern: normalized.key_pattern,
        last_used_ms: Some(now_ms),
        updated_at_ms: now_ms,
    })
}

/// 根据 upsert 请求更新现有 Redis 连接配置。
///
/// # 参数
///
/// * `existing` - 原连接配置，保留 id 与最后使用时间。
/// * `body` - 前端提交的新配置草稿。
/// * `now_ms` - 当前毫秒时间戳，用于更新时间。
///
/// # 返回值
///
/// 返回已归一化的新配置。
///
/// # 错误处理
///
/// 与 [`new_connection_from_upsert`] 相同，所有配置非法状态都会转为 [`ApiError`]。
pub(super) fn updated_connection_from_upsert(
    existing: &GatewayRedisConnectionConfig,
    body: &GatewayRedisConnectionUpsertBody,
    now_ms: u64,
) -> Result<GatewayRedisConnectionConfig, ApiError> {
    let normalized = normalize_upsert_body(body)?;
    Ok(GatewayRedisConnectionConfig {
        id: existing.id.clone(),
        name: normalized.name,
        host: normalized.host,
        port: normalized.port,
        db: normalized.db,
        username: normalized.username,
        password: normalized.password,
        use_tls: normalized.use_tls,
        tls_cert: normalized.tls_cert,
        ssh_tunnel: normalized.ssh_tunnel,
        sentinel: normalized.sentinel,
        use_cluster: normalized.use_cluster,
        read_only: normalized.read_only,
        key_pattern: normalized.key_pattern,
        last_used_ms: existing.last_used_ms,
        updated_at_ms: now_ms,
    })
}

/// 归一化导入包中的 Redis 连接配置。
///
/// # 参数
///
/// * `connection` - 导入文件中的连接配置。
/// * `now_ms` - 当前毫秒时间戳，用于补齐缺失时间或重建 id。
/// * `seen_ids` - 当前导入批次已使用的 id 集合。
///
/// # 返回值
///
/// 返回可直接保存的连接配置；空 id 或重复 id 会被替换为新 id。
///
/// # 错误处理
///
/// 导入配置未通过同一套 upsert 校验时返回 [`ApiError`]。
pub(super) fn normalize_import_connection(
    connection: GatewayRedisConnectionConfig,
    now_ms: u64,
    seen_ids: &mut HashSet<String>,
) -> Result<GatewayRedisConnectionConfig, ApiError> {
    let GatewayRedisConnectionConfig {
        id: original_id,
        name,
        host,
        port,
        db,
        username,
        password,
        use_tls,
        tls_cert,
        ssh_tunnel,
        sentinel,
        use_cluster,
        read_only,
        key_pattern,
        last_used_ms,
        updated_at_ms,
    } = connection;
    let draft = GatewayRedisConnectionUpsertBody {
        name,
        host,
        port,
        db,
        username,
        password,
        use_tls,
        tls_cert,
        ssh_tunnel,
        sentinel,
        use_cluster,
        read_only,
        key_pattern,
    };
    let normalized = normalize_upsert_body(&draft)?;
    // 导入配置可能来自外部文件；重复 id 会覆盖用户现有选择语义，因此在批次内强制去重。
    let id = if original_id.trim().is_empty() || seen_ids.contains(original_id.trim()) {
        format!("redis-{now_ms}-{}", Uuid::new_v4())
    } else {
        original_id.trim().to_string()
    };
    seen_ids.insert(id.clone());

    Ok(GatewayRedisConnectionConfig {
        id,
        name: normalized.name,
        host: normalized.host,
        port: normalized.port,
        db: normalized.db,
        username: normalized.username,
        password: normalized.password,
        use_tls: normalized.use_tls,
        tls_cert: normalized.tls_cert,
        ssh_tunnel: normalized.ssh_tunnel,
        sentinel: normalized.sentinel,
        use_cluster: normalized.use_cluster,
        read_only: normalized.read_only,
        key_pattern: normalized.key_pattern,
        last_used_ms: last_used_ms.or(Some(now_ms)),
        updated_at_ms: if updated_at_ms == 0 { now_ms } else { updated_at_ms },
    })
}

fn normalize_upsert_body(
    body: &GatewayRedisConnectionUpsertBody,
) -> Result<GatewayRedisConnectionUpsertBody, ApiError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(ApiError::bad_request("请输入连接名称"));
    }

    let host = body.host.trim();
    if host.is_empty() {
        return Err(ApiError::bad_request("请输入 Redis 主机地址"));
    }

    let key_pattern = if body.key_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        body.key_pattern.trim().to_string()
    };

    if body.sentinel.enabled && body.use_cluster {
        // Sentinel 和 Cluster 的连接发现机制不同，同时启用会让后续连接路径含义不明确。
        return Err(ApiError::bad_request("Sentinel 与 Cluster 不能同时启用"));
    }

    if body.use_cluster && body.db != 0 {
        // Redis Cluster 不支持按连接选择非 0 DB，提前拒绝可避免运行时才暴露协议错误。
        return Err(ApiError::bad_request("Cluster 模式仅支持 DB 0"));
    }

    if body.ssh_tunnel.enabled {
        if body.ssh_tunnel.host.trim().is_empty() {
            return Err(ApiError::bad_request("启用 SSH 时必须填写 SSH 地址"));
        }
        if body.ssh_tunnel.username.trim().is_empty() {
            return Err(ApiError::bad_request("启用 SSH 时必须填写 SSH 用户名"));
        }
    }

    if body.sentinel.enabled && body.sentinel.master_name.trim().is_empty() {
        return Err(ApiError::bad_request("启用 Sentinel 时必须填写 Master 组名称"));
    }

    if has_custom_tls_material(&body.tls_cert) && !body.use_tls {
        // 证书路径只有在 TLS 握手中才会被使用；未启用 TLS 时接受路径容易造成安全误解。
        return Err(ApiError::bad_request("使用证书文件前必须启用 SSL/TLS"));
    }

    Ok(GatewayRedisConnectionUpsertBody {
        name: name.to_string(),
        host: host.to_string(),
        port: body.port,
        db: body.db,
        username: body.username.trim().to_string(),
        password: body.password.trim().to_string(),
        use_tls: body.use_tls,
        tls_cert: GatewayRedisTlsCertConfig {
            private_key_path: body.tls_cert.private_key_path.trim().to_string(),
            public_cert_path: body.tls_cert.public_cert_path.trim().to_string(),
            ca_cert_path: body.tls_cert.ca_cert_path.trim().to_string(),
        },
        ssh_tunnel: GatewayRedisSshTunnelConfig {
            enabled: body.ssh_tunnel.enabled,
            host: body.ssh_tunnel.host.trim().to_string(),
            port: body.ssh_tunnel.port.max(1),
            username: body.ssh_tunnel.username.trim().to_string(),
            password: body.ssh_tunnel.password.trim().to_string(),
            private_key_path: body.ssh_tunnel.private_key_path.trim().to_string(),
            passphrase: body.ssh_tunnel.passphrase.trim().to_string(),
            timeout_secs: body.ssh_tunnel.timeout_secs.max(1),
        },
        sentinel: GatewayRedisSentinelConfig {
            enabled: body.sentinel.enabled,
            master_name: body.sentinel.master_name.trim().to_string(),
            node_password: body.sentinel.node_password.trim().to_string(),
        },
        use_cluster: body.use_cluster,
        read_only: body.read_only,
        key_pattern,
    })
}

/// 构造普通 Redis 连接 URI。
///
/// # 参数
///
/// * `connection` - 已归一化的连接配置。
///
/// # 返回值
///
/// 返回 `redis://` 或 `rediss://` URI，用户名和密码会做 URL 编码。
///
/// # 错误处理
///
/// 主机为空时返回 [`ApiError`]。
pub(super) fn build_connection_uri(
    connection: &GatewayRedisConnectionConfig,
) -> Result<String, ApiError> {
    if connection.host.trim().is_empty() {
        return Err(ApiError::bad_request("Redis 主机地址不能为空"));
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

    Ok(format!(
        "{scheme}://{auth}{}:{}/{}",
        connection.host, connection.port, connection.db
    ))
}

/// 从配置路径加载 TLS 证书材料。
///
/// # 参数
///
/// * `config` - 客户端证书、私钥和 CA 证书路径配置。
///
/// # 返回值
///
/// 返回 redis crate 可使用的证书结构；未配置路径时对应字段为空。
///
/// # 错误处理
///
/// 文件读取失败，或客户端证书与私钥只提供其一时返回错误字符串。
pub(super) fn load_tls_certificates_from_paths(
    config: &GatewayRedisTlsCertConfig,
) -> Result<::redis::TlsCertificates, String> {
    let client_cert = read_optional_tls_file(&config.public_cert_path, "客户端证书")?;
    let client_key = read_optional_tls_file(&config.private_key_path, "客户端私钥")?;
    let root_cert = read_optional_tls_file(&config.ca_cert_path, "CA 证书")?;

    let client_tls = match (client_cert, client_key) {
        (Some(client_cert), Some(client_key)) => Some(::redis::ClientTlsConfig {
            client_cert,
            client_key,
        }),
        (None, None) => None,
        _ => {
            return Err("客户端证书和私钥必须同时提供，或同时留空".to_string());
        }
    };

    Ok(::redis::TlsCertificates {
        client_tls,
        root_cert,
    })
}

fn read_optional_tls_file(path: &str, label: &str) -> Result<Option<Vec<u8>>, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let expanded = shellexpand::full(trimmed)
        .map_err(|error| format!("{label}路径无效: {error}"))?;
    std::fs::read(expanded.as_ref())
        .map(Some)
        .map_err(|error| format!("读取{label}失败: {error}"))
}

/// 判断 TLS 配置中是否提供了任意自定义证书材料。
///
/// # 参数
///
/// * `config` - TLS 证书路径配置。
///
/// # 返回值
///
/// 任意证书路径非空时返回 `true`。
///
/// # 错误处理
///
/// 本函数不执行 IO，不产生错误。
pub(super) fn has_custom_tls_material(config: &GatewayRedisTlsCertConfig) -> bool {
    !config.private_key_path.trim().is_empty()
        || !config.public_cert_path.trim().is_empty()
        || !config.ca_cert_path.trim().is_empty()
}

#[cfg(test)]
#[path = "redis_config_support_tests.rs"]
mod redis_config_support_tests;
