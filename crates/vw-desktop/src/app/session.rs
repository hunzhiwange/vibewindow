//! 聚合会话子模块并导出会话 UI 与加载相关能力。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::App;
use super::models::{
    ChatMessage, ChatRenderCacheEntry, ChatRole, ChatSession, ChatSessionMeta, ChatSessionStep,
    TokenUsage,
};
use super::state::StepUiMeta;
use std::collections::HashMap;
use std::sync::Arc;

mod active;
mod chat_ui;
mod loading;
mod runtime;

/// 对外暴露当前模块需要复用的能力。
pub use self::chat_ui::{
    CHAT_UI_CHUNK_SIZE, PreparedChatUiChunk, PreparedChatUiPhase, chat_ui_chunk_bounds,
    chat_ui_chunk_start_idx, prepare_chat_ui_chunk_phase,
};
/// 对外暴露当前模块需要复用的能力。
pub use self::loading::{
    SharedChatMessages, build_session_previews, session_usage, shared_chat_messages,
};

#[cfg(test)]
use self::chat_ui::{explore_summary_animation_key, prioritize_chat_ui_chunk_starts};

const EMPTY_SESSION_ID: &str = "__empty__";

#[cfg(test)]
mod tests;
