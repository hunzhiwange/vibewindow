//! Matrix 通道的出站消息发送实现。
//!
//! 本模块负责在发送前解析目标房间、同步一次缺失房间状态，并确保只向已加入
//! 且受支持的房间发送消息。

use super::MatrixChannel;
use crate::app::agent::channels::traits::SendMessage;
use matrix_sdk::{
    RoomState, config::SyncSettings, ruma::OwnedRoomId, ruma::events::room::message::RoomMessageEventContent,
};
use std::sync::atomic::Ordering;

impl MatrixChannel {
    /// 向配置的 Matrix 目标房间发送消息。
    ///
    /// 参数：`message` 是运行时标准消息，当前实现发送其 Markdown 文本正文。
    ///
    /// 返回值：发送成功返回 `Ok(())`。
    ///
    /// 错误处理：检测到 OTK 冲突、房间 ID 无法解析、目标房间不存在、未加入房间
    /// 或 Matrix SDK 发送失败时返回错误。
    pub(super) async fn send_impl(&self, message: &SendMessage) -> anyhow::Result<()> {
        if self.otk_conflict_detected.load(Ordering::Relaxed) {
            anyhow::bail!("{}", self.otk_conflict_recovery_message());
        }

        let client = self.matrix_client().await?;
        let target_room_id = self.target_room_id().await?;
        let target_room: OwnedRoomId = target_room_id.parse()?;

        let mut room = client.get_room(&target_room);
        if room.is_none() {
            // 本地 store 可能尚未有房间快照，发送前同步一次可以避免误报房间不存在。
            let _ = client.sync_once(SyncSettings::new()).await;
            room = client.get_room(&target_room);
        }

        let Some(room) = room else {
            anyhow::bail!("Matrix room '{}' not found in joined rooms", target_room_id);
        };

        if room.state() != RoomState::Joined {
            anyhow::bail!("Matrix room '{}' is not in joined state", target_room_id);
        }

        // 使用 Markdown 内容保留代理回复中的列表、代码块等格式。
        room.send(RoomMessageEventContent::text_markdown(&message.content)).await?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "outbound_tests.rs"]
mod outbound_tests;
