//! 验证 Cloudflare 隧道实现的基础生命周期行为。
//! 这些测试聚焦未启动状态和构造器语义，避免在单元测试中启动外部进程。

use super::*;

#[test]
fn constructor_stores_token() {
    let tunnel = CloudflareTunnel::new("cf-token".into());
    assert_eq!(tunnel.token, "cf-token");
}

#[test]
fn public_url_is_none_before_start() {
    let tunnel = CloudflareTunnel::new("cf-token".into());
    assert!(tunnel.public_url().is_none());
}

#[tokio::test]
async fn stop_without_started_process_is_ok() {
    let tunnel = CloudflareTunnel::new("cf-token".into());
    let result = tunnel.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn health_check_is_false_before_start() {
    let tunnel = CloudflareTunnel::new("cf-token".into());
    assert!(!tunnel.health_check().await);
}
