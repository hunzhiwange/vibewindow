//! MQTT 到 SOP 事件扇入监听器
//!
//! 本模块实现了一个 MQTT 消息监听器，用于将 MQTT 消息路由到 SOP（标准作业流程）引擎。
//! 这不是一个 `Channel` trait 的实现者——它通过 `dispatch_sop_event` 将 MQTT 消息
//! 分发到 SOP 引擎，而不是发送到聊天循环。
//!
//! # 主要功能
//!
//! - 订阅配置的 MQTT 主题
//! - 接收 MQTT 发布消息并转换为 SOP 事件
//! - 将事件分发到 SOP 引擎执行
//! - 支持 TLS 加密连接（mqtts://）
//! - 自动重连和健康状态报告
//!
//! # 架构说明
//!
//! ```
//! MQTT Broker → run_mqtt_sop_listener → dispatch_sop_event → SopEngine
//! ```

use std::sync::{Arc, Mutex};

use anyhow::Result;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, Transport};
use tracing::{info, warn};

use crate::app::agent::config::MqttConfig;
use crate::app::agent::sop::audit::SopAuditLogger;
use crate::app::agent::sop::dispatch::{dispatch_sop_event, process_headless_results};
use crate::app::agent::sop::engine::{now_iso8601, SopEngine};
use crate::app::agent::sop::types::{SopEvent, SopTriggerSource};

/// 运行 MQTT SOP 监听器循环
///
/// 此函数是 MQTT 监听器的主入口点，它会：
/// 1. 验证配置的有效性
/// 2. 建立 MQTT 连接（支持 TLS）
/// 3. 订阅配置的主题列表
/// 4. 持续监听并处理传入的 MQTT 消息
///
/// # 参数
///
/// * `config` - MQTT 配置，包含连接参数、认证信息和主题列表
/// * `engine` - SOP 引擎实例（线程安全），用于处理事件
/// * `audit` - SOP 审计日志记录器，用于记录操作审计信息
///
/// # 返回值
///
/// 返回 `Result<()>`：
/// - `Ok(())` - 监听器正常结束（通常不会发生，除非被取消）
/// - `Err(e)` - 初始化失败或致命错误
///
/// # 错误处理
///
/// - 配置验证失败会立即返回错误
/// - 连接错误会被记录但不会终止循环（rumqttc 自动重连）
/// - 健康状态会持续更新到系统健康检查系统
///
/// # 示例
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use vibe_agent::config::MqttConfig;
/// use vibe_agent::sop::engine::SopEngine;
/// use vibe_agent::sop::audit::SopAuditLogger;
/// use vibe_agent::channels::mqtt::run_mqtt_sop_listener;
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = MqttConfig {
///     broker_url: "mqtt://localhost:1883".to_string(),
///     client_id: "my-client".to_string(),
///     topics: vec!["sensors/#".to_string()],
///     username: None,
///     password: None,
///     qos: 1,
///     keep_alive_secs: 60,
///     use_tls: false,
/// };
///
/// let engine = Arc::new(Mutex::new(SopEngine::new()));
/// let audit = Arc::new(SopAuditLogger::new());
///
/// run_mqtt_sop_listener(&config, engine, audit).await?;
/// # Ok(())
/// # }
/// ```
///
/// # 阻塞行为
///
/// 此函数会阻塞直到连接断开或被显式取消。通常在独立的异步任务中运行。
pub async fn run_mqtt_sop_listener(
    config: &MqttConfig,
    engine: Arc<Mutex<SopEngine>>,
    audit: Arc<SopAuditLogger>,
) -> Result<()> {
    // 验证配置的有效性（broker_url、client_id、topics 等）
    config.validate()?;

    // 构建 MQTT 连接选项
    let mut mqtt_options = MqttOptions::new(
        &config.client_id,
        broker_host(&config.broker_url),
        broker_port(&config.broker_url),
    );

    // 设置心跳保持时间（秒）
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(config.keep_alive_secs));

    // 如果提供了用户名和密码，设置认证凭据
    if let (Some(ref user), Some(ref pass)) = (&config.username, &config.password) {
        mqtt_options.set_credentials(user, pass);
    }

    // 当使用 mqtts:// 协议时，配置 TLS 加密传输
    if config.use_tls {
        mqtt_options.set_transport(Transport::tls_with_default_config());
        info!("MQTT SOP listener: TLS transport enabled");
    }

    // 创建异步客户端和事件循环（64 为内部队列大小）
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 64);

    // 根据 QoS 配置值映射到相应的 QoS 枚举
    let qos = match config.qos {
        0 => QoS::AtMostOnce,      // 最多一次（可能丢失）
        1 => QoS::AtLeastOnce,     // 至少一次（可能重复）
        _ => QoS::ExactlyOnce,     // 恰好一次（最可靠）
    };

    // 订阅所有配置的主题
    for topic in &config.topics {
        client.subscribe(topic, qos).await?;
        info!("MQTT SOP listener: subscribed to '{topic}'");
    }

    // 标记 MQTT 组件健康状态为正常
    crate::app::agent::health::mark_component_ok("mqtt");

    // 事件循环：持续监听 MQTT 事件
    loop {
        match eventloop.poll().await {
            // 处理接收到的发布消息
            Ok(Event::Incoming(Packet::Publish(msg))) => {
                let topic = msg.topic.clone();
                // 将字节载荷转换为 UTF-8 字符串（使用 lossy 转换处理无效 UTF-8）
                let payload = String::from_utf8_lossy(&msg.payload).to_string();

                // 构建 SOP 事件对象
                let event = SopEvent {
                    source: SopTriggerSource::Mqtt,  // 标记来源为 MQTT
                    topic: Some(topic),
                    payload: Some(payload),
                    timestamp: now_iso8601(),  // 当前时间 ISO 8601 格式
                };

                // 将事件分发到 SOP 引擎执行
                let results = dispatch_sop_event(&engine, &audit, event).await;
                // 处理无头模式的结果（如自动化任务执行结果）
                process_headless_results(&results).await;
            }

            // 处理连接确认
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                // 连接成功，更新健康状态
                crate::app::agent::health::mark_component_ok("mqtt");
                info!("MQTT SOP listener: connected to broker");
            }

            // 其他事件（PingResp、SubAck 等）——忽略
            Ok(_) => {
                // 其他 MQTT 协议事件不需要特殊处理
            }

            // 处理连接错误
            Err(e) => {
                // 记录错误并更新健康状态
                crate::app::agent::health::mark_component_error("mqtt", e.to_string());
                warn!("MQTT SOP listener: connection error: {e}");
                // rumqttc 会自动处理重连，循环继续
            }
        }
    }
}

/// 从 broker URL 中提取主机名
///
/// 支持的 URL 格式：
/// - `mqtt://hostname:port`
/// - `mqtts://hostname:port`
/// - `hostname:port`（无协议前缀）
///
/// # 参数
///
/// * `url` - broker URL 字符串
///
/// # 返回值
///
/// 返回主机名字符串。如果无法解析，默认返回 `"localhost"`
///
/// # 示例
///
/// ```
/// assert_eq!(broker_host("mqtt://example.com:1883"), "example.com");
/// assert_eq!(broker_host("mqtts://secure.example.com:8883"), "secure.example.com");
/// assert_eq!(broker_host("invalid"), "localhost");
/// ```
fn broker_host(url: &str) -> String {
    // 移除协议前缀（mqtt:// 或 mqtts://）
    let without_scheme = url
        .strip_prefix("mqtt://")
        .or_else(|| url.strip_prefix("mqtts://"))
        .unwrap_or(url);

    // 分割并提取主机部分（冒号之前）
    without_scheme
        .split(':')
        .next()
        .unwrap_or("localhost")
        .to_string()
}

/// 从 broker URL 中提取端口号
///
/// 支持的 URL 格式：
/// - `mqtt://hostname:port` - 默认端口 1883
/// - `mqtts://hostname:port` - 默认端口 8883
/// - `hostname:port` - 根据 use_tls 配置决定默认端口
///
/// # 参数
///
/// * `url` - broker URL 字符串
///
/// # 返回值
///
/// 返回端口号：
/// - 如果 URL 中包含端口号，返回该端口号
/// - `mqtt://` 协议默认返回 `1883`
/// - `mqtts://` 协议默认返回 `8883`
/// - 无法解析时返回相应协议的默认端口
///
/// # 示例
///
/// ```
/// assert_eq!(broker_port("mqtt://example.com:1883"), 1883);
/// assert_eq!(broker_port("mqtts://secure.example.com:8883"), 8883);
/// assert_eq!(broker_port("mqtt://example.com"), 1883);  // 默认端口
/// assert_eq!(broker_port("mqtts://example.com"), 8883); // TLS 默认端口
/// ```
fn broker_port(url: &str) -> u16 {
    // 检测是否为 TLS 连接
    let is_tls = url.starts_with("mqtts://");

    // 移除协议前缀
    let without_scheme = url
        .strip_prefix("mqtt://")
        .or_else(|| url.strip_prefix("mqtts://"))
        .unwrap_or(url);

    // 根据协议类型确定默认端口
    let default_port: u16 = if is_tls { 8883 } else { 1883 };

    // 从右向左分割（处理 IPv6 地址情况）并提取端口号
    without_scheme
        .rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
