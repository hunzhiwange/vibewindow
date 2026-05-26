//! 协调设置消息处理模块
//!
//! 本模块负责处理与代理协调系统相关的设置消息。协调系统允许多个代理之间
//! 进行消息传递、任务分配和状态同步。
//!
//! # 主要功能
//!
//! - 启用/禁用协调系统
//! - 配置主导代理（Lead Agent）
//! - 设置各种容量限制（收件箱、死信队列、上下文条目、已见消息ID）
//! - 持久化协调设置到配置文件
//! - 显示/隐藏帮助模态框
//!
//! # 消息类型
//!
//! 本模块处理 `SettingsMessage` 枚举中所有 `Coordination*` 相关的变体。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;

/// 持久化协调设置到配置文件
///
/// 该函数将当前应用状态中的协调设置写入持久化存储。在保存前，
/// 会对某些数值字段进行范围限制（clamp），确保值在合理范围内。
///
/// # 参数
///
/// - `app`: 可变引用的应用实例，从中读取协调设置
///
/// # 处理的字段
///
/// - `enabled`: 协调系统是否启用
/// - `lead_agent`: 主导代理标识符（去除首尾空白）
/// - `max_inbox_messages_per_agent`: 每个代理收件箱最大消息数（1-10,000）
/// - `max_dead_letters`: 死信队列最大容量（1-10,000）
/// - `max_context_entries`: 上下文最大条目数（1-20,000）
/// - `max_seen_message_ids`: 已见消息ID最大追踪数（1-100,000）
fn persist_coordination_settings(app: &mut App) -> Task<Message> {
    let s = &app.coordination_settings;
    let enabled = s.enabled;
    let max_inbox_messages_per_agent = s.max_inbox_messages_per_agent.clamp(1, 10_000) as usize;
    let max_dead_letters = s.max_dead_letters.clamp(1, 10_000) as usize;
    let max_context_entries = s.max_context_entries.clamp(1, 20_000) as usize;
    let max_seen_message_ids = s.max_seen_message_ids.clamp(1, 100_000) as usize;
    // 去除主导代理输入的首尾空白字符
    let lead_agent = s.lead_agent_input.trim().to_string();

    crate::app::update_coordination_config_async(move |coordination| {
        // 启用状态直接保存
        coordination.enabled = enabled;
        // 主导代理标识
        coordination.lead_agent = lead_agent;
        // 每个代理收件箱最大消息数，限制在 1-10,000 范围内
        coordination.max_inbox_messages_per_agent = max_inbox_messages_per_agent;
        // 死信队列最大容量，限制在 1-10,000 范围内
        coordination.max_dead_letters = max_dead_letters;
        // 上下文最大条目数，限制在 1-20,000 范围内
        coordination.max_context_entries = max_context_entries;
        // 已见消息ID最大追踪数，限制在 1-100,000 范围内
        coordination.max_seen_message_ids = max_seen_message_ids;
    })
}

#[cfg(test)]
#[path = "coordination_tests.rs"]
mod coordination_tests;

/// 处理协调设置相关的消息
///
/// 该函数是协调设置模块的主入口点，负责处理所有与协调系统配置相关的
/// 用户交互消息。每个消息处理后都会返回一个 Iced Task 用于后续 UI 更新。
///
/// # 参数
///
/// - `app`: 可变引用的应用实例，用于读取和更新协调设置状态
/// - `message`: 设置消息枚举，指定要执行的操作
///
/// # 返回值
///
/// 返回 `Task<Message>`，通常为 `Task::none()`，因为大多数设置变更
/// 不需要额外的异步操作。
///
/// # 处理的消息类型
///
/// - `CoordinationEnabledToggled`: 切换协调系统启用状态
/// - `CoordinationLeadAgentChanged`: 更改主导代理标识
/// - `CoordinationMaxInboxMessagesPerAgentChanged`: 更改每代理收件箱容量
/// - `CoordinationMaxDeadLettersChanged`: 更改死信队列容量
/// - `CoordinationMaxContextEntriesChanged`: 更改上下文条目容量
/// - `CoordinationMaxSeenMessageIdsChanged`: 更改已见消息ID追踪容量
/// - `CoordinationSave`: 手动触发设置保存
/// - `CoordinationHelpOpen`: 打开帮助模态框
/// - `CoordinationHelpClose`: 关闭帮助模态框
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        // 处理协调系统启用状态切换
        SettingsMessage::CoordinationEnabledToggled(v) => {
            app.coordination_settings.enabled = v;
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理主导代理标识变更
        SettingsMessage::CoordinationLeadAgentChanged(v) => {
            app.coordination_settings.lead_agent_input = v;
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理每代理收件箱最大消息数变更
        SettingsMessage::CoordinationMaxInboxMessagesPerAgentChanged(v) => {
            // 限制值在 1-10,000 范围内
            app.coordination_settings.max_inbox_messages_per_agent = v.clamp(1, 10_000);
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理死信队列最大容量变更
        SettingsMessage::CoordinationMaxDeadLettersChanged(v) => {
            // 限制值在 1-10,000 范围内
            app.coordination_settings.max_dead_letters = v.clamp(1, 10_000);
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理上下文最大条目数变更
        SettingsMessage::CoordinationMaxContextEntriesChanged(v) => {
            // 限制值在 1-20,000 范围内
            app.coordination_settings.max_context_entries = v.clamp(1, 20_000);
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理已见消息ID最大追踪数变更
        SettingsMessage::CoordinationMaxSeenMessageIdsChanged(v) => {
            // 限制值在 1-100,000 范围内
            app.coordination_settings.max_seen_message_ids = v.clamp(1, 100_000);
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 处理手动保存设置请求
        SettingsMessage::CoordinationSave => {
            app.coordination_settings.save_error = None;
            persist_coordination_settings(app)
        }
        // 打开帮助模态框
        SettingsMessage::CoordinationHelpOpen => {
            app.coordination_settings.show_help_modal = true;
            Task::none()
        }
        // 关闭帮助模态框
        SettingsMessage::CoordinationHelpClose => {
            app.coordination_settings.show_help_modal = false;
            Task::none()
        }
        // 非协调相关消息，不做处理
        _ => Task::none(),
    }
}
