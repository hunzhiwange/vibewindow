use serde::Deserialize;

/// Matrix 同步响应结构
///
/// 包含同步 API 返回的下一批次标记和房间信息。
#[derive(Debug, Deserialize)]
pub(super) struct SyncResponse {
    /// 下一批次标记，用于增量同步
    pub(super) next_batch: String,
    /// 房间信息
    #[serde(default)]
    pub(super) rooms: Rooms,
}

/// 房间信息集合
#[derive(Debug, Deserialize, Default)]
pub(super) struct Rooms {
    /// 已加入的房间
    #[serde(default)]
    pub(super) join: std::collections::HashMap<String, JoinedRoom>,
}

/// 已加入的房间信息
#[derive(Debug, Deserialize)]
pub(super) struct JoinedRoom {
    /// 房间时间线（消息事件列表）
    #[serde(default)]
    pub(super) timeline: Timeline,
}

/// 房间时间线
///
/// 包含房间内的事件（如消息）。
#[derive(Debug, Deserialize, Default)]
pub(super) struct Timeline {
    /// 时间线事件列表
    #[serde(default)]
    pub(super) events: Vec<TimelineEvent>,
}

/// 时间线事件
///
/// 表示房间时间线中的单个事件（如消息）。
#[derive(Debug, Deserialize)]
pub(super) struct TimelineEvent {
    /// 事件类型（如 `m.room.message`）
    #[serde(rename = "type")]
    pub(super) event_type: String,
    /// 事件发送者
    pub(super) sender: String,
    /// 事件 ID
    #[serde(default)]
    pub(super) event_id: Option<String>,
    /// 事件内容
    #[serde(default)]
    pub(super) content: EventContent,
}

/// 事件内容
///
/// 包含消息体和消息类型等信息。
#[derive(Debug, Deserialize, Default)]
pub(super) struct EventContent {
    /// 消息体（纯文本）
    #[serde(default)]
    pub(super) body: Option<String>,
    /// 消息类型（如 `m.text`、`m.audio`）
    #[serde(default)]
    pub(super) msgtype: Option<String>,
}

/// whoami API 响应
///
/// 包含当前认证用户的身份信息。
#[derive(Debug, Deserialize)]
pub(super) struct WhoAmIResponse {
    /// 用户 ID
    pub(super) user_id: String,
    /// 设备 ID
    #[serde(default)]
    pub(super) device_id: Option<String>,
}

/// 房间别名解析响应
///
/// 包含房间别名对应的实际房间 ID。
#[derive(Debug, Deserialize)]
pub(super) struct RoomAliasResponse {
    /// 解析后的房间 ID
    pub(super) room_id: String,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
