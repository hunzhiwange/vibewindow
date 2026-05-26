//! 通道 trait 默认行为测试。
//!
//! 本模块使用最小 `DummyChannel` 实现验证通道契约的基础语义，包括消息克隆、
//! 默认辅助方法、草稿与 reaction 的空实现，以及监听接口向队列发送消息的能力。
//! 这些测试用于保护 trait 默认方法在新增通道能力时不被意外改成破坏性行为。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    /// 最小通道实现，用于隔离测试 `Channel` trait 的默认方法。
    ///
    /// 该替身只实现必需的名称、发送和监听逻辑；其他能力全部走 trait
    /// 默认实现，从而能直接观察默认契约是否仍保持向后兼容。
    struct DummyChannel;

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Channel for DummyChannel {
        fn name(&self) -> &str {
            "dummy"
        }

        async fn send(&self, _message: &SendMessage) -> anyhow::Result<()> {
            Ok(())
        }

        async fn listen(
            &self,
            tx: tokio::sync::mpsc::Sender<ChannelMessage>,
        ) -> anyhow::Result<()> {
            // 监听测试只需要证明消息能进入调用方提供的队列，因此使用固定消息
            // 避免引入外部平台连接或时间相关的不确定性。
            tx.send(ChannelMessage {
                id: "1".into(),
                sender: "tester".into(),
                reply_target: "tester".into(),
                content: "hello".into(),
                channel: "dummy".into(),
                timestamp: 123,
                thread_ts: None,
            })
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))
        }
    }

    #[test]
    fn channel_message_clone_preserves_fields() {
        // `ChannelMessage` 会跨任务和队列传递，克隆必须保持路由与回复字段完整。
        let message = ChannelMessage {
            id: "42".into(),
            sender: "alice".into(),
            reply_target: "alice".into(),
            content: "ping".into(),
            channel: "dummy".into(),
            timestamp: 999,
            thread_ts: None,
        };

        let cloned = message.clone();
        assert_eq!(cloned.id, "42");
        assert_eq!(cloned.sender, "alice");
        assert_eq!(cloned.reply_target, "alice");
        assert_eq!(cloned.content, "ping");
        assert_eq!(cloned.channel, "dummy");
        assert_eq!(cloned.timestamp, 999);
    }

    #[tokio::test]
    async fn default_trait_methods_return_success() {
        let channel = DummyChannel;

        // 默认方法应是安全的 no-op，便于不支持 typing 等能力的平台渐进接入。
        assert!(channel.health_check().await);
        assert!(channel.start_typing("bob").await.is_ok());
        assert!(channel.stop_typing("bob").await.is_ok());
        assert!(channel.send(&SendMessage::new("hello", "bob")).await.is_ok());
    }

    #[tokio::test]
    async fn default_reaction_methods_return_success() {
        let channel = DummyChannel;

        // reaction 能力不是所有平台都支持；默认成功能让调用方无需为缺省通道
        // 添加额外分支，同时具体通道仍可覆盖为真实实现。
        assert!(channel.add_reaction("chan_1", "msg_1", "\u{1F440}").await.is_ok());
        assert!(channel.remove_reaction("chan_1", "msg_1", "\u{1F440}").await.is_ok());
    }

    #[tokio::test]
    async fn default_draft_methods_return_success() {
        let channel = DummyChannel;

        // 草稿能力默认声明为不支持，但相关操作保持 no-op 成功，
        // 这样上层可统一调用清理流程而不破坏旧通道。
        assert!(!channel.supports_draft_updates());
        assert!(channel.send_draft(&SendMessage::new("draft", "bob")).await.unwrap().is_none());
        assert!(channel.update_draft("bob", "msg_1", "text").await.is_ok());
        assert!(channel.finalize_draft("bob", "msg_1", "final text").await.is_ok());
        assert!(channel.cancel_draft("bob", "msg_1").await.is_ok());
    }

    #[tokio::test]
    async fn listen_sends_message_to_channel() {
        let channel = DummyChannel;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // 使用容量为 1 的队列足以覆盖单条消息传递，并避免测试依赖后台循环。
        channel.listen(tx).await.unwrap();

        let received = rx.recv().await.expect("message should be sent");
        assert_eq!(received.sender, "tester");
        assert_eq!(received.content, "hello");
        assert_eq!(received.channel, "dummy");
    }
}
