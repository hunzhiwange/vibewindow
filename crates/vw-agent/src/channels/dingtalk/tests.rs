//! 钉钉通道单元测试模块
//!
//! 本模块包含 DingTalkChannel 及相关配置的单元测试用例，覆盖以下功能：
//! - 通道名称验证
//! - 用户访问权限控制（白名单机制）
//! - 配置序列化/反序列化
//! - 流数据解析
//! - 聊天ID解析
//!
//! 所有测试遵循隔离原则，不依赖外部钉钉服务。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试通道名称返回值
    ///
    /// 验证 DingTalkChannel 实例的 `name()` 方法正确返回 "dingtalk" 标识符。
    #[test]
    fn test_name() {
        let ch = DingTalkChannel::new("id".into(), "secret".into(), vec![]);
        assert_eq!(ch.name(), "dingtalk");
    }

    /// 测试通配符用户访问权限
    ///
    /// 当允许用户列表包含 "*" 通配符时，任意用户都应被允许访问。
    /// 这是白名单的"全开放"模式。
    #[test]
    fn test_user_allowed_wildcard() {
        let ch = DingTalkChannel::new("id".into(), "secret".into(), vec!["*".into()]);
        assert!(ch.is_user_allowed("anyone"));
    }

    /// 测试特定用户访问权限
    ///
    /// 当允许用户列表包含特定用户ID时：
    /// - 列表中的用户应被允许访问
    /// - 不在列表中的用户应被拒绝
    #[test]
    fn test_user_allowed_specific() {
        let ch = DingTalkChannel::new("id".into(), "secret".into(), vec!["user123".into()]);
        assert!(ch.is_user_allowed("user123"));
        assert!(!ch.is_user_allowed("other"));
    }

    /// 测试空允许列表的访问权限
    ///
    /// 当允许用户列表为空时，所有用户都应被拒绝访问。
    /// 这是白名单的"全关闭"模式，用于默认安全策略。
    #[test]
    fn test_user_denied_empty() {
        let ch = DingTalkChannel::new("id".into(), "secret".into(), vec![]);
        assert!(!ch.is_user_allowed("anyone"));
    }

    /// 测试配置 TOML 反序列化
    ///
    /// 验证 DingTalkConfig 能正确从 TOML 字符串解析，包括：
    /// - client_id: 钉钉应用ID
    /// - client_secret: 钉钉应用密钥
    /// - allowed_users: 允许的用户列表
    #[test]
    fn test_config_serde() {
        let toml_str = r#"
    client_id = "app_id_123"
    client_secret = "secret_456"
    allowed_users = ["user1", "*"]
    "#;
        let config: crate::app::agent::config::schema::DingTalkConfig =
            toml::from_str(toml_str).unwrap();
        assert_eq!(config.client_id, "app_id_123");
        assert_eq!(config.client_secret, "secret_456");
        assert_eq!(config.allowed_users, vec!["user1", "*"]);
    }

    /// 测试配置 TOML 反序列化默认值
    ///
    /// 当 TOML 中未指定 allowed_users 字段时，
    /// 应使用默认值（空列表）。
    #[test]
    fn test_config_serde_defaults() {
        let toml_str = r#"
    client_id = "id"
    client_secret = "secret"
    "#;
        let config: crate::app::agent::config::schema::DingTalkConfig =
            toml::from_str(toml_str).unwrap();
        assert!(config.allowed_users.is_empty());
    }

    /// 测试流数据解析支持字符串载荷
    ///
    /// 钉钉回调可能将载荷作为转义的 JSON 字符串传递。
    /// 此测试验证 `parse_stream_data` 能正确解析字符串形式的载荷，
    /// 提取出嵌套的 text.content 字段。
    #[test]
    fn parse_stream_data_supports_string_payload() {
        let frame = serde_json::json!({
            "data": "{\"text\":{\"content\":\"hello\"}}"
        });
        let parsed = DingTalkChannel::parse_stream_data(&frame).unwrap();
        assert_eq!(
            parsed.get("text").and_then(|v| v.get("content")),
            Some(&serde_json::json!("hello"))
        );
    }

    /// 测试流数据解析支持对象载荷
    ///
    /// 钉钉回调也可能直接传递 JSON 对象作为载荷。
    /// 此测试验证 `parse_stream_data` 能正确处理对象形式的载荷，
    /// 提取出嵌套的 text.content 字段。
    #[test]
    fn parse_stream_data_supports_object_payload() {
        let frame = serde_json::json!({
            "data": {"text": {"content": "hello"}}
        });
        let parsed = DingTalkChannel::parse_stream_data(&frame).unwrap();
        assert_eq!(
            parsed.get("text").and_then(|v| v.get("content")),
            Some(&serde_json::json!("hello"))
        );
    }

    /// 测试聊天ID解析处理群聊类型
    ///
    /// 当 conversationType 为 2（群聊）时，应使用 conversationId 作为聊天ID，
    /// 而非发送者的 staffId。此测试验证群聊场景下的ID解析逻辑。
    #[test]
    fn resolve_chat_id_handles_numeric_group_conversation_type() {
        let data = serde_json::json!({
            "conversationType": 2,
            "conversationId": "cid-group",
        });
        let chat_id = DingTalkChannel::resolve_chat_id(&data, "staff-1");
        assert_eq!(chat_id, "cid-group");
    }
}
