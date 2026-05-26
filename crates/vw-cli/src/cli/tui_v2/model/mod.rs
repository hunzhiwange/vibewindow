//! TUI v2 内部模型层。
//!
//! 本模块只定义 CLI 新 TUI 的内部 view model，不承担 reducer、renderer 或 gateway
//! 调度职责。当前阶段先收口三类核心模型：
//! - `ui_message`: 用户/助手/tool/thinking/step/system/error 等消息结构
//! - `prompt`: 输入框、历史、排队命令与一次提交的生命周期
//! - `overlay`: confirm/search/question/todo/task/palette/error 的弹层状态
//!
//! 这些结构的目标是为后续 S2-2 的 reducer/store 与 S4-1 的事件标准化提供稳定承接面。

pub(crate) mod overlay;
#[cfg(test)]
#[path = "overlay_tests.rs"]
mod overlay_tests;
pub(crate) mod prompt;
#[cfg(test)]
#[path = "prompt_tests.rs"]
mod prompt_tests;
pub(crate) mod ui_message;
#[cfg(test)]
#[path = "ui_message_tests.rs"]
mod ui_message_tests;

pub(crate) use overlay::{
    McpServerTransport, OverlayFocus, OverlayState, UiConfirmOverlay, UiErrorOverlay,
    UiMcpOverlay, UiMcpServerInfo, UiMemoryEntry, UiMemoryOverlay, UiOverlay, UiOverlayKind,
    UiQuestionOverlay, UiQuestionSurfaceKind, UiSearchMatch, UiSearchOverlay, UiTaskOverlay,
    UiTaskStepItem, UiTodoOverlay,
};
#[cfg(test)]
pub(crate) use overlay::{
    UiQuestionOption, UiQuestionPrompt, UiQuestionToolContext, UiTodoItem,
};
pub(crate) use prompt::{
    PromptMode, PromptMotion, PromptState, PromptSubmission,
    PromptSubmissionStatus, QueuedPromptCommand, QueuedPromptCommandKind,
};
pub(crate) use ui_message::{
    UiAssistantMessage, UiErrorMessage, UiMessage, UiMessageBase, UiMessageId, UiStep,
    UiStepState, UiSystemMessage, UiSystemMessageLevel, UiThinkingBlock, UiThinkingTiming,
    UiTokenUsage, UiToolCall, UiToolCallState, UiToolResult, UiTurnTerminal, UiUserMessage,
};
#[cfg(test)]
pub(crate) use ui_message::UiMessageKind;

#[cfg(test)]
mod tests;
