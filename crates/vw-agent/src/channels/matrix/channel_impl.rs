//! Matrix 通道的 `Channel` trait 适配层。
//!
//! 这里把 Matrix 专有的发送、监听和健康检查实现暴露成通道运行时统一接口，
//! 使上层调度无需了解 Matrix SDK 的房间状态和会话恢复细节。

use super::MatrixChannel;
use crate::app::agent::channels::traits::{Channel, ChannelMessage, SendMessage};
use async_trait::async_trait;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Channel for MatrixChannel {
    /// 返回通道注册名。
    fn name(&self) -> &str {
        "matrix"
    }

    /// 发送一条通道消息。
    ///
    /// 参数：`message` 是运行时标准发送消息，包含正文和回复目标。
    ///
    /// 返回值：发送成功返回 `Ok(())`。
    ///
    /// 错误处理：Matrix 客户端、房间解析或 SDK 发送失败会向上传递。
    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        self.send_impl(message).await
    }

    /// 开始监听 Matrix 入站消息并转发到运行时队列。
    ///
    /// 参数：`tx` 用于把标准化后的 `ChannelMessage` 发送给通道管理器。
    ///
    /// 返回值：监听循环正常结束时返回 `Ok(())`。
    ///
    /// 错误处理：底层同步或事件处理失败会以 `anyhow::Error` 返回。
    async fn listen(&self, tx: mpsc::Sender<ChannelMessage>) -> anyhow::Result<()> {
        self.listen_impl(tx).await
    }

    /// 检查 Matrix 通道当前是否可用。
    ///
    /// 返回值：房间可解析、可支持且客户端可创建时返回 `true`。
    ///
    /// 错误处理：健康检查吞掉具体错误并返回 `false`，调用方只需要可用性信号。
    async fn health_check(&self) -> bool {
        // OTK 冲突代表加密会话可能不可信，健康检查必须失败以触发显式恢复。
        if self.otk_conflict_detected.load(Ordering::Relaxed) {
            return false;
        }

        let Ok(room_id) = self.target_room_id().await else {
            return false;
        };

        if self.ensure_room_supported(&room_id).await.is_err() {
            return false;
        }

        self.matrix_client().await.is_ok()
    }
}

#[cfg(test)]
#[path = "channel_impl_tests.rs"]
mod channel_impl_tests;
