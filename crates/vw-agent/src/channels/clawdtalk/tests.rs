//! # ClawdTalk 通道测试模块
//!
//! 本模块包含 ClawdTalk 通道实现的单元测试，用于验证通道的创建、
//! 目的地访问控制以及 Webhook 事件反序列化等功能。
//!
//! ## 测试范围
//!
//! - 通道实例化与基本属性验证
//! - 目的地允许列表的精确匹配与通配符匹配逻辑
//! - Telnyx Webhook 事件的 JSON 反序列化

use super::*;

/// ClawdTalk 通道测试用例集合
///
/// 该模块包含所有与 ClawdTalk 通道相关的测试函数，
/// 涵盖配置验证、权限检查和事件解析等场景。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 创建用于测试的 ClawdTalk 配置实例
    ///
    /// 返回一个预配置的 `ClawdTalkConfig` 实例，包含测试所需的
    /// API 密钥、连接 ID、发送号码以及允许的目的地前缀。
    ///
    /// # 返回值
    ///
    /// 返回一个 `ClawdTalkConfig` 实例，其中：
    /// - `api_key`: 测试用 API 密钥 "test-key"
    /// - `connection_id`: 测试用连接标识 "test-connection"
    /// - `from_number`: 发送方号码 "+15551234567"
    /// - `allowed_destinations`: 允许的目的地前缀列表 ["+1555"]
    /// - `webhook_secret`: 未设置（None）
    fn test_config() -> ClawdTalkConfig {
        ClawdTalkConfig {
            api_key: "test-key".to_string(),
            connection_id: "test-connection".to_string(),
            from_number: "+15551234567".to_string(),
            allowed_destinations: vec!["+1555".to_string()],
            webhook_secret: None,
        }
    }

    /// 测试 ClawdTalk 通道的创建
    ///
    /// 验证使用测试配置创建的通道实例能够正确返回其名称标识。
    /// 通道名称应为 "ClawdTalk"。
    #[test]
    fn creates_channel() {
        let channel = ClawdTalkChannel::new(test_config());
        assert_eq!(channel.name(), "ClawdTalk");
    }

    /// 测试目的地允许列表的精确前缀匹配
    ///
    /// 验证 `is_destination_allowed` 方法能够正确判断目的地号码
    /// 是否在允许列表中。使用前缀匹配逻辑，即目的地号码以
    /// 允许列表中的某个前缀开头则视为允许。
    ///
    /// # 测试用例
    ///
    /// - "+15559876543" 应被允许（以 "+1555" 开头）
    /// - "+14449876543" 应被拒绝（不以任何允许前缀开头）
    #[test]
    fn destination_allowed_exact_match() {
        let channel = ClawdTalkChannel::new(test_config());
        assert!(channel.is_destination_allowed("+15559876543"));
        assert!(!channel.is_destination_allowed("+14449876543"));
    }

    /// 测试通配符 "*" 允许所有目的地
    ///
    /// 当 `allowed_destinations` 包含通配符 "*" 时，
    /// 所有目的地号码都应被允许访问。
    ///
    /// # 测试用例
    ///
    /// - "+15559876543" 应被允许
    /// - "+14449876543" 应被允许
    #[test]
    fn destination_allowed_wildcard() {
        let mut config = test_config();
        config.allowed_destinations = vec!["*".to_string()];
        let channel = ClawdTalkChannel::new(config);
        assert!(channel.is_destination_allowed("+15559876543"));
        assert!(channel.is_destination_allowed("+14449876543"));
    }

    /// 测试空允许列表视为允许所有目的地
    ///
    /// 当 `allowed_destinations` 为空列表时，
    /// 应视为不限制访问，所有目的地号码都被允许。
    ///
    /// # 测试用例
    ///
    /// - "+15559876543" 应被允许
    /// - "+14449876543" 应被允许
    #[test]
    fn destination_allowed_empty_means_all() {
        let mut config = test_config();
        config.allowed_destinations = vec![];
        let channel = ClawdTalkChannel::new(config);
        assert!(channel.is_destination_allowed("+15559876543"));
        assert!(channel.is_destination_allowed("+14449876543"));
    }

    /// 测试 Telnyx Webhook 事件的 JSON 反序列化
    ///
    /// 验证 `TelnyxWebhookEvent` 结构体能够正确解析来自
    /// Telnyx 平台的 Webhook 事件 JSON 数据。
    ///
    /// # 测试数据
    ///
    /// 使用包含呼叫发起事件（call.initiated）的 JSON 数据，
    /// 包含完整的呼叫控制 ID、呼叫腿 ID、会话 ID、方向、
    /// 发送方和接收方号码以及呼叫状态等字段。
    #[test]
    fn webhook_event_deserializes() {
        // Telnyx Webhook 事件的 JSON 示例数据
        let json = r#"{
                "data": {
                    "event_type": "call.initiated",
                    "payload": {
                        "call_control_id": "call-123",
                        "call_leg_id": "leg-123",
                        "call_session_id": "session-123",
                        "direction": "incoming",
                        "from": "+15551112222",
                        "to": "+15553334444",
                        "state": "ringing"
                    }
                }
            }"#;

        // 反序列化 JSON 并验证关键字段
        let event: TelnyxWebhookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.data.event_type, "call.initiated");
        assert_eq!(event.data.payload.call_control_id, Some("call-123".to_string()));
        assert_eq!(event.data.payload.from, Some("+15551112222".to_string()));
    }
}
