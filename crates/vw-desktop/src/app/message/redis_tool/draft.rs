//! 负责把 Redis 连接草稿转换为连接 URI 与网关保存请求，并集中校验草稿字段。

use super::{
    GatewayRedisConnectionUpsertBody, GatewayRedisSentinelConfig, GatewayRedisSshTunnelConfig,
    GatewayRedisTlsCertConfig, RedisConnectionDraft,
};

/// 处理 `build_draft_uri` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回 `Err` 时保留原始错误文本，交由上层设置页展示。
pub(super) fn build_draft_uri(draft: &RedisConnectionDraft) -> Result<String, String> {
    if let Some(reason) = unsupported_draft_mode_reason(draft) {
        return Err(reason);
    }

    let host = draft.host.trim();
    if host.is_empty() {
        return Err("请输入 Redis 主机地址".to_string());
    }

    let port =
        draft.port.trim().parse::<u16>().map_err(|_| "端口必须是 1-65535 的整数".to_string())?;
    let db = draft.db.trim().parse::<i64>().map_err(|_| "数据库编号必须是整数".to_string())?;
    let scheme = if draft.use_tls { "rediss" } else { "redis" };
    let username = urlencoding::encode(draft.username.trim());
    let password = urlencoding::encode(draft.password.trim());

    let auth = if draft.password.trim().is_empty() && draft.username.trim().is_empty() {
        String::new()
    } else if draft.username.trim().is_empty() {
        format!(":{password}@")
    } else if draft.password.trim().is_empty() {
        format!("{username}@")
    } else {
        format!("{username}:{password}@")
    };

    Ok(format!("{scheme}://{auth}{host}:{port}/{db}"))
}

#[cfg(test)]
#[path = "draft_tests.rs"]
mod draft_tests;

/// 处理 `draft_to_upsert_body` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回 `Err` 时保留原始错误文本，交由上层设置页展示。
pub(super) fn draft_to_upsert_body(
    draft: &RedisConnectionDraft,
) -> Result<GatewayRedisConnectionUpsertBody, String> {
    let name = draft.name.trim();
    if name.is_empty() {
        return Err("请输入连接名称".to_string());
    }

    let host = draft.host.trim();
    if host.is_empty() {
        return Err("请输入 Redis 主机地址".to_string());
    }

    let port =
        draft.port.trim().parse::<u16>().map_err(|_| "端口必须是 1-65535 的整数".to_string())?;
    let db = draft.db.trim().parse::<i64>().map_err(|_| "数据库编号必须是整数".to_string())?;
    let key_pattern = if draft.key_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        draft.key_pattern.trim().to_string()
    };

    if draft.sentinel.enabled && draft.use_cluster {
        return Err("Sentinel 与 Cluster 不能同时启用".to_string());
    }

    if draft.use_cluster && db != 0 {
        return Err("Cluster 模式仅支持 DB 0".to_string());
    }

    if draft.tls_cert.has_custom_paths() && !draft.use_tls {
        return Err("使用证书文件前必须启用 SSL/TLS".to_string());
    }

    if draft.ssh_tunnel.enabled {
        if draft.ssh_tunnel.host.trim().is_empty() {
            return Err("启用 SSH 时必须填写 SSH 地址".to_string());
        }
        if draft.ssh_tunnel.username.trim().is_empty() {
            return Err("启用 SSH 时必须填写 SSH 用户名".to_string());
        }
    }

    if draft.sentinel.enabled && draft.sentinel.master_name.trim().is_empty() {
        return Err("启用 Sentinel 时必须填写 Master 组名称".to_string());
    }

    let ssh_port = parse_u16_or_default(&draft.ssh_tunnel.port, 22, "SSH 端口")?;
    let ssh_timeout_secs = parse_u32_or_default(&draft.ssh_tunnel.timeout_secs, 30, "SSH 超时")?;

    Ok(GatewayRedisConnectionUpsertBody {
        name: name.to_string(),
        host: host.to_string(),
        port,
        db,
        username: draft.username.trim().to_string(),
        password: draft.password.trim().to_string(),
        use_tls: draft.use_tls,
        tls_cert: GatewayRedisTlsCertConfig {
            private_key_path: draft.tls_cert.private_key_path.trim().to_string(),
            public_cert_path: draft.tls_cert.public_cert_path.trim().to_string(),
            ca_cert_path: draft.tls_cert.ca_cert_path.trim().to_string(),
        },
        ssh_tunnel: GatewayRedisSshTunnelConfig {
            enabled: draft.ssh_tunnel.enabled,
            host: draft.ssh_tunnel.host.trim().to_string(),
            port: ssh_port,
            username: draft.ssh_tunnel.username.trim().to_string(),
            password: draft.ssh_tunnel.password.trim().to_string(),
            private_key_path: draft.ssh_tunnel.private_key_path.trim().to_string(),
            passphrase: draft.ssh_tunnel.passphrase.trim().to_string(),
            timeout_secs: ssh_timeout_secs,
        },
        sentinel: GatewayRedisSentinelConfig {
            enabled: draft.sentinel.enabled,
            master_name: draft.sentinel.master_name.trim().to_string(),
            node_password: draft.sentinel.node_password.trim().to_string(),
        },
        use_cluster: draft.use_cluster,
        read_only: draft.read_only,
        key_pattern,
    })
}

fn parse_u16_or_default(value: &str, default: u16, label: &str) -> Result<u16, String> {
    if value.trim().is_empty() {
        return Ok(default);
    }

    value.trim().parse::<u16>().map_err(|_| format!("{label}必须是 1-65535 的整数"))
}

fn parse_u32_or_default(value: &str, default: u32, label: &str) -> Result<u32, String> {
    if value.trim().is_empty() {
        return Ok(default);
    }

    value
        .trim()
        .parse::<u32>()
        .map(|parsed| parsed.max(1))
        .map_err(|_| format!("{label}必须是正整数"))
}

fn unsupported_draft_mode_reason(draft: &RedisConnectionDraft) -> Option<String> {
    if draft.ssh_tunnel.enabled {
        return Some("当前版本暂不支持通过 SSH 隧道复制标准 Redis URI".to_string());
    }

    if draft.sentinel.enabled {
        return Some("当前版本暂不支持为 Sentinel 模式生成单条 Redis URI".to_string());
    }

    if draft.use_cluster {
        return Some("当前版本暂不支持为 Cluster 模式生成单条 Redis URI".to_string());
    }

    if draft.use_tls && draft.tls_cert.has_custom_paths() {
        return Some("当前版本暂不支持将自定义 SSL 证书路径编码为连接 URI".to_string());
    }

    None
}
