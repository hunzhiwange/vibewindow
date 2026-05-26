//! Tailscale 隧道模块的单元测试
//!
//! 本模块包含 `TailscaleTunnel` 结构体的测试用例，验证其构造函数、
//! 公共 URL 获取、健康检查和停止等功能的基本行为。
//!
//! # 测试覆盖范围
//!
//! - 构造函数参数存储验证
//! - 启动前的公共 URL 状态
//! - 启动前的健康检查状态
//! - 未启动时的停止操作安全性

use super::*;
use serde_json::json;

/// 测试构造函数是否正确存储 hostname 和 mode 参数
///
/// # 验证内容
///
/// - `funnel` 标志应被正确存储
/// - `hostname` 应被正确存储且可通过 `as_deref()` 访问
///
/// # 示例
///
/// ```
/// let tunnel = TailscaleTunnel::new(true, Some("myhost.tailnet.ts.net".into()));
/// assert!(tunnel.funnel);
/// assert_eq!(tunnel.hostname.as_deref(), Some("myhost.tailnet.ts.net"));
/// ```
#[test]
fn constructor_stores_hostname_and_mode() {
    // 创建一个启用 funnel 模式并指定 hostname 的隧道实例
    let tunnel = TailscaleTunnel::new(true, Some("myhost.tailnet.ts.net".into()));

    // 验证 funnel 标志已正确存储
    assert!(tunnel.funnel);

    // 验证 hostname 已正确存储，且可通过 as_deref() 获取原始字符串引用
    assert_eq!(tunnel.hostname.as_deref(), Some("myhost.tailnet.ts.net"));
}

/// 测试在调用 start() 之前，public_url() 应返回 None
///
/// # 验证内容
///
/// - 未启动的隧道实例，其公共 URL 应为 `None`
/// - 确保初始状态的一致性和可预测性
///
/// # 设计意图
///
/// 此测试确保在隧道未启动时，调用者无法获取有效的公共 URL，
/// 从而避免使用未初始化的连接信息。
#[test]
fn public_url_is_none_before_start() {
    // 创建一个未启用 funnel 且未指定 hostname 的隧道实例
    let tunnel = TailscaleTunnel::new(false, None);

    // 验证在启动前，公共 URL 应为 None
    assert!(tunnel.public_url().is_none());
}

/// 测试在调用 start() 之前，health_check() 应返回 false
///
/// # 验证内容
///
/// - 未启动的隧道实例，其健康检查应返回 `false`
/// - 确保健康检查能够正确反映隧道的实际运行状态
///
/// # 异步说明
///
/// 此测试使用 `#[tokio::test]` 标记，因为 `health_check()` 是异步方法。
///
/// # 设计意图
///
/// 健康检查应在隧道未启动时返回 false，以便调用者能够识别服务不可用状态，
/// 而不是抛出错误或返回不确定的结果。
#[tokio::test]
async fn health_check_is_false_before_start() {
    // 创建一个未启用 funnel 且未指定 hostname 的隧道实例
    let tunnel = TailscaleTunnel::new(false, None);

    // 验证在启动前，健康检查应返回 false
    assert!(!tunnel.health_check().await);
}

/// 测试在未启动进程的情况下调用 stop() 应安全返回 Ok(())
///
/// # 验证内容
///
/// - 对未启动的隧道调用 `stop()` 不应导致错误
/// - 返回值应为 `Ok(())`，表示操作成功完成
///
/// # 异步说明
///
/// 此测试使用 `#[tokio::test]` 标记，因为 `stop()` 是异步方法。
///
/// # 设计意图
///
/// 此测试确保 `stop()` 方法是幂等的且安全的：
/// - 即使在没有运行中的进程时调用，也不应引发 panic 或错误
/// - 允许调用者在不检查启动状态的情况下安全地停止隧道
/// - 符合"优雅降级"的设计原则
#[tokio::test]
async fn stop_without_started_process_is_ok() {
    // 创建一个未启用 funnel 且未指定 hostname 的隧道实例
    let tunnel = TailscaleTunnel::new(false, None);

    // 尝试停止未启动的隧道，应安全返回 Ok
    let result = tunnel.stop().await;

    // 验证停止操作成功完成，即使没有运行中的进程
    assert!(result.is_ok());
}

#[test]
fn public_urls_from_status_prefers_funnel_enabled_entry() {
    let status = json!({
        "Web": {
            "xiangminmac-mini.tail47e3db.ts.net:443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:42617"
                    }
                }
            },
            "xiangminmac-mini.tail47e3db.ts.net:8443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:42617"
                    }
                }
            },
            "other-node.tail47e3db.ts.net:8443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:42617"
                    }
                }
            }
        },
        "AllowFunnel": {
            "xiangminmac-mini.tail47e3db.ts.net:443": true
        }
    });

    let urls = public_urls_from_status(
        &status,
        Some("xiangminmac-mini.tail47e3db.ts.net"),
        42617,
        true,
    );

    assert_eq!(urls, vec!["https://xiangminmac-mini.tail47e3db.ts.net".to_string()]);
}

#[test]
fn public_urls_from_status_prefers_tailnet_endpoint_for_serve() {
    let status = json!({
        "Web": {
            "xiangminmac-mini.tail47e3db.ts.net:443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:42617"
                    }
                }
            },
            "xiangminmac-mini.tail47e3db.ts.net:8443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:42617"
                    }
                }
            }
        },
        "AllowFunnel": {
            "xiangminmac-mini.tail47e3db.ts.net:443": true
        }
    });

    let urls = public_urls_from_status(
        &status,
        Some("xiangminmac-mini.tail47e3db.ts.net"),
        42617,
        false,
    );

    assert_eq!(
        urls,
        vec!["https://xiangminmac-mini.tail47e3db.ts.net:8443".to_string()]
    );
}

#[test]
fn public_urls_from_status_filters_out_other_backend_ports() {
    let status = json!({
        "Web": {
            "xiangminmac-mini.tail47e3db.ts.net:8443": {
                "Handlers": {
                    "/": {
                        "Proxy": "http://127.0.0.1:3000"
                    }
                }
            }
        }
    });

    let urls = public_urls_from_status(
        &status,
        Some("xiangminmac-mini.tail47e3db.ts.net"),
        42617,
        false,
    );

    assert!(urls.is_empty());
}

#[test]
fn fallback_public_url_uses_expected_frontend_port() {
    assert_eq!(
        fallback_public_url("xiangminmac-mini.tail47e3db.ts.net.", false),
        "https://xiangminmac-mini.tail47e3db.ts.net:8443"
    );
    assert_eq!(
        fallback_public_url("xiangminmac-mini.tail47e3db.ts.net.", true),
        "https://xiangminmac-mini.tail47e3db.ts.net"
    );
}
