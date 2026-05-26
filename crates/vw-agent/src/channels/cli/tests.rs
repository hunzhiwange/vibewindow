//! CLI 通道模块的单元测试
//!
//! 本模块包含对 [`CliChannel`] 及相关数据结构的测试用例，覆盖以下功能：
//!
//! - 通道名称验证
//! - 消息发送行为
//! - 健康检查功能
//! - 消息结构体的字段访问与克隆能力
//!
//! ## 测试范围
//!
//! | 测试项 | 覆盖内容 |
//! |--------|----------|
//! | `cli_channel_name` | 通道名称返回值 |
//! | `cli_channel_send_does_not_panic` | 正常消息发送 |
//! | `cli_channel_send_empty_message` | 空消息发送 |
//! | `cli_channel_health_check` | 健康检查 |
//! | `channel_message_struct` | 消息结构体字段 |
//! | `channel_message_clone` | 消息克隆 |

use super::*;

/// CLI 通道测试模块
///
/// 包含所有针对 CLI 通道实现的单元测试。
/// 使用 `#[allow(dead_code)]` 属性是因为该模块仅在测试配置下编译。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试 CLI 通道的名称返回值
    ///
    /// # 验证点
    ///
    /// - 新创建的 `CliChannel` 实例应返回 `"cli"` 作为其通道名称
    ///
    /// # 示例
    ///
    /// ```
    /// let channel = CliChannel::new();
    /// assert_eq!(channel.name(), "cli");
    /// ```
    #[test]
    fn cli_channel_name() {
        assert_eq!(CliChannel::new().name(), "cli");
    }

    /// 测试 CLI 通道发送消息不会触发 panic
    ///
    /// # 验证点
    ///
    /// - 发送包含正常内容的消息应成功完成
    /// - `send` 方法应返回 `Ok(())`
    ///
    /// # 输入数据
    ///
    /// - `content`: `"hello"`
    /// - `recipient`: `"user"`
    /// - `subject`: `None`
    /// - `thread_ts`: `None`
    #[tokio::test]
    async fn cli_channel_send_does_not_panic() {
        let ch = CliChannel::new();

        // 构造包含正常内容的发送消息请求
        let result = ch
            .send(&SendMessage {
                content: "hello".into(),
                recipient: "user".into(),
                subject: None,
                thread_ts: None,
            })
            .await;

        // 验证发送操作成功完成
        assert!(result.is_ok());
    }

    /// 测试 CLI 通道发送空消息的行为
    ///
    /// # 验证点
    ///
    /// - 发送空字符串内容应不会导致错误
    /// - CLI 通道应对边界情况（空内容、空接收者）保持健壮
    ///
    /// # 输入数据
    ///
    /// - `content`: 空字符串
    /// - `recipient`: 空字符串
    /// - `subject`: `None`
    /// - `thread_ts`: `None`
    #[tokio::test]
    async fn cli_channel_send_empty_message() {
        let ch = CliChannel::new();

        // 构造边界情况：所有字符串字段均为空
        let result = ch
            .send(&SendMessage {
                content: String::new(),
                recipient: String::new(),
                subject: None,
                thread_ts: None,
            })
            .await;

        // 验证即使是空消息也能成功发送
        assert!(result.is_ok());
    }

    /// 测试 CLI 通道的健康检查功能
    ///
    /// # 验证点
    ///
    /// - `health_check` 方法应始终返回 `true`
    /// - CLI 通道作为本地通道，不依赖外部服务，因此始终健康
    #[tokio::test]
    async fn cli_channel_health_check() {
        let ch = CliChannel::new();

        // CLI 通道是本地通道，健康检查应始终通过
        assert!(ch.health_check().await);
    }

    /// 测试 `ChannelMessage` 结构体的字段访问
    ///
    /// # 验证点
    ///
    /// - 所有字段在创建后可正确访问
    /// - 字段值与构造时传入的值一致
    ///
    /// # 测试数据
    ///
    /// | 字段 | 测试值 |
    /// |------|--------|
    /// | `id` | `"test-id"` |
    /// | `sender` | `"user"` |
    /// | `reply_target` | `"user"` |
    /// | `content` | `"hello"` |
    /// | `channel` | `"cli"` |
    /// | `timestamp` | `1234567890` |
    /// | `thread_ts` | `None` |
    #[test]
    fn channel_message_struct() {
        // 创建包含所有字段的消息实例
        let msg = ChannelMessage {
            id: "test-id".into(),
            sender: "user".into(),
            reply_target: "user".into(),
            content: "hello".into(),
            channel: "cli".into(),
            timestamp: 1_234_567_890,
            thread_ts: None,
        };

        // 逐一验证各字段值
        assert_eq!(msg.id, "test-id");
        assert_eq!(msg.sender, "user");
        assert_eq!(msg.reply_target, "user");
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.channel, "cli");
        assert_eq!(msg.timestamp, 1_234_567_890);
    }

    /// 测试 `ChannelMessage` 的克隆功能
    ///
    /// # 验证点
    ///
    /// - `ChannelMessage` 实现了 `Clone` trait
    /// - 克隆后的实例与原实例的字段值完全一致
    ///
    /// # 背景
    ///
    /// 消息克隆功能在需要将同一消息分发到多个处理器或进行消息副本操作时非常重要。
    #[test]
    fn channel_message_clone() {
        // 创建原始消息
        let msg = ChannelMessage {
            id: "id".into(),
            sender: "s".into(),
            reply_target: "s".into(),
            content: "c".into(),
            channel: "ch".into(),
            timestamp: 0,
            thread_ts: None,
        };

        // 执行克隆操作
        let cloned = msg.clone();

        // 验证克隆后的关键字段与原始消息一致
        assert_eq!(cloned.id, msg.id);
        assert_eq!(cloned.content, msg.content);
    }
}
