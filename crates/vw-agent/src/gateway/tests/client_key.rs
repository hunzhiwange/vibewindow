//! 客户端密钥（client_key）提取与规范化函数的单元测试模块
//!
//! 本模块测试网关层中客户端标识提取的正确性，主要覆盖两个核心函数：
//! - `client_key_from_request`: 从 HTTP 请求中提取客户端唯一标识（基于 IP 地址）
//! - `normalize_max_keys`: 规范化最大密钥数量配置值
//!
//! ## 测试场景覆盖
//!
//! ### 1. 客户端密钥提取（`client_key_from_request`）
//! - 非受信代理模式：应使用直接对端地址，忽略转发头
//! - 受信代理模式：应解析 `X-Forwarded-For` 头的第一个有效 IP
//! - 无效转发头回退：当转发头内容无效时，回退到对端地址
//!
//! ### 2. 最大密钥数规范化（`normalize_max_keys`）
//! - 零值处理：零值应使用配置的回退值
//! - 非零值保留：有效非零值应原样保留
//!
//! ## 安全上下文
//!
//! 客户端密钥用于速率限制、配额管理和安全审计。正确的 IP 提取对于
//! 防止绕过限制和准确识别客户端至关重要。受信代理模式仅在网关
//! 部署于已知可信的反向代理之后时启用。

use super::*;
use axum::http::HeaderMap;

/// 测试非受信代理模式下客户端密钥的默认行为
///
/// ## 测试目的
/// 验证当 `trusted_proxy_mode` 为 `false` 时，即使请求中包含
/// `X-Forwarded-For` 头，系统也应忽略该头，直接使用 TCP 连接的
/// 对端地址作为客户端标识。
///
/// ## 场景描述
/// - 对端地址: `10.0.0.5:42617`（内网地址）
/// - 转发头内容: `198.51.100.10, 203.0.113.11`（两个公网 IP）
/// - 代理模式: 非受信（`false`）
///
/// ## 预期结果
/// 返回对端 IP `10.0.0.5`，完全忽略转发头中的地址。
///
/// ## 安全考虑
/// 这是默认安全的行为：防止客户端通过伪造转发头绕过基于 IP 的限制。
#[test]
fn client_key_defaults_to_peer_addr_when_untrusted_proxy_mode() {
    // 构造测试用的对端地址（模拟内网客户端）
    let peer = SocketAddr::from(([10, 0, 0, 5], 42617));

    // 构造包含多个转发地址的 HTTP 头
    let mut headers = HeaderMap::new();
    headers.insert("X-Forwarded-For", HeaderValue::from_static("198.51.100.10, 203.0.113.11"));

    // 在非受信代理模式下提取客户端密钥
    let key = client_key_from_request(Some(peer), &headers, false);

    // 断言：应使用对端地址，忽略转发头
    assert_eq!(key, "10.0.0.5");
}

/// 测试受信代理模式下客户端密钥的提取行为
///
/// ## 测试目的
/// 验证当 `trusted_proxy_mode` 为 `true` 时，系统应解析
/// `X-Forwarded-For` 头并使用第一个 IP 地址作为客户端标识。
///
/// ## 场景描述
/// - 对端地址: `10.0.0.5:42617`（代理服务器地址）
/// - 转发头内容: `198.51.100.10, 203.0.113.11`（原始客户端 + 中间代理）
/// - 代理模式: 受信（`true`）
///
/// ## 预期结果
/// 返回转发头中的第一个 IP `198.51.100.10`，即原始客户端地址。
///
/// ## 使用前提
/// 此模式仅在网关部署于可信的反向代理（如 Nginx、Cloudflare）之后时启用。
/// 代理负责将真实客户端 IP 注入 `X-Forwarded-For` 头的起始位置。
#[test]
fn client_key_uses_forwarded_ip_only_in_trusted_proxy_mode() {
    // 构造测试用的对端地址（模拟代理服务器）
    let peer = SocketAddr::from(([10, 0, 0, 5], 42617));

    // 构造包含多个转发地址的 HTTP 头
    // 格式: 客户端IP, 代理1, 代理2, ...
    let mut headers = HeaderMap::new();
    headers.insert("X-Forwarded-For", HeaderValue::from_static("198.51.100.10, 203.0.113.11"));

    // 在受信代理模式下提取客户端密钥
    let key = client_key_from_request(Some(peer), &headers, true);

    // 断言：应使用转发头中的第一个 IP（原始客户端）
    assert_eq!(key, "198.51.100.10");
}

/// 测试转发头无效时的回退行为
///
/// ## 测试目的
/// 验证当 `X-Forwarded-For` 头包含无效内容时，即使在受信代理模式下，
/// 系统也应安全回退到使用对端地址作为客户端标识。
///
/// ## 场景描述
/// - 对端地址: `10.0.0.5:42617`
/// - 转发头内容: `garbage-value`（非有效 IP 格式）
/// - 代理模式: 受信（`true`）
///
/// ## 预期结果
/// 由于无法解析转发头，回退到对端 IP `10.0.0.5`。
///
/// ## 健壮性保障
/// 此测试确保系统对畸形输入具有防御性，不会因解析失败而崩溃或
/// 返回空值。回退策略确保速率限制仍然有效。
#[test]
fn client_key_falls_back_to_peer_when_forwarded_header_invalid() {
    // 构造测试用的对端地址
    let peer = SocketAddr::from(([10, 0, 0, 5], 42617));

    // 构造包含无效内容的转发头
    let mut headers = HeaderMap::new();
    headers.insert("X-Forwarded-For", HeaderValue::from_static("garbage-value"));

    // 在受信代理模式下尝试提取（但头内容无效）
    let key = client_key_from_request(Some(peer), &headers, true);

    // 断言：应回退到对端地址
    assert_eq!(key, "10.0.0.5");
}

/// 测试 `normalize_max_keys` 函数对零值的处理
///
/// ## 测试目的
/// 验证当输入值为零时，函数应使用提供的回退值作为结果。
/// 这允许配置中使用零作为"使用默认值"的标记。
///
/// ## 测试用例
/// 1. `normalize_max_keys(0, 10_000)` → 返回 `10_000`（使用回退值）
/// 2. `normalize_max_keys(0, 0)` → 返回 `1`（最小值保护）
///
/// ## 设计考量
/// - 零值通常表示"未配置"或"使用默认"
/// - 当回退值也为零时，返回 `1` 作为绝对最小值，防止无意义的零限制
#[test]
fn normalize_max_keys_uses_fallback_for_zero() {
    // 零值应使用回退值 10_000
    assert_eq!(normalize_max_keys(0, 10_000), 10_000);

    // 即使回退值也是零，也应返回最小值 1
    assert_eq!(normalize_max_keys(0, 0), 1);
}

/// 测试 `normalize_max_keys` 函数对非零值的保留行为
///
/// ## 测试目的
/// 验证当输入值为有效的非零值时，函数应原样返回该值，
/// 不进行任何修改或截断。
///
/// ## 测试用例
/// 1. `normalize_max_keys(2_048, 10_000)` → 返回 `2_048`（保留用户配置）
/// 2. `normalize_max_keys(1, 10_000)` → 返回 `1`（最小有效值保留）
///
/// ## 设计考量
/// 非零值代表用户明确的配置意图，应被尊重。
/// 即使配置值小于回退值，也应按用户配置执行。
#[test]
fn normalize_max_keys_preserves_nonzero_values() {
    // 非零值应原样保留，即使小于回退值
    assert_eq!(normalize_max_keys(2_048, 10_000), 2_048);

    // 最小有效值 1 也应保留
    assert_eq!(normalize_max_keys(1, 10_000), 1);
}
