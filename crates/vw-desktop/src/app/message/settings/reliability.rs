//! 可靠性设置消息处理模块
//!
//! 本模块负责处理应用程序中与可靠性相关的所有配置更新消息，
//! 包括 Provider 重试策略、退避时间、Channel 退避配置以及调度器参数等。
//!
//! # 主要功能
//!
//! - **Provider 可靠性配置**：重试次数、退避时间（毫秒级）
//! - **Channel 可靠性配置**：初始退避时间、最大退避时间（秒级）
//! - **调度器配置**：轮询间隔、重试次数
//! - **帮助模态框管理**：打开/关闭帮助信息的显示状态
//!
//! # 配置约束
//!
//! 所有配置项均有合理的值域限制，确保系统稳定性：
//! - Provider 重试次数：0-20 次
//! - Provider 退避时间：0-60000 毫秒（1 分钟）
//! - Channel 退避时间：1-3600 秒（1 小时），且最大值不小于初始值
//! - 调度器轮询间隔：1-3600 秒
//! - 调度器重试次数：0-20 次

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;

/// 持久化可靠性设置到配置文件
///
/// 将当前应用程序中的可靠性设置写入持久化存储。
/// 所有值在写入前都会进行边界约束，防止配置越界。
///
/// # 参数
///
/// - `app`：可变引用的应用程序实例，从中读取可靠性设置
///
/// # 配置项说明
///
/// | 配置项 | 约束范围 | 说明 |
/// |--------|----------|------|
/// | `provider_retries` | 0-20 | Provider 调用失败时的重试次数 |
/// | `provider_backoff_ms` | 0-60000 | Provider 重试的退避时间（毫秒） |
/// | `channel_initial_backoff_secs` | 1-3600 | Channel 初始退避时间（秒） |
/// | `channel_max_backoff_secs` | 初始值-3600 | Channel 最大退避时间（秒），不能小于初始值 |
/// | `scheduler_poll_secs` | 1-3600 | 调度器轮询间隔（秒） |
/// | `scheduler_retries` | 0-20 | 调度器任务重试次数 |
fn persist_reliability_settings(app: &mut App) -> Task<Message> {
    let s = &app.reliability_settings;
    let provider_retries = s.provider_retries.clamp(0, 20);
    let provider_backoff_ms = s.provider_backoff_ms.clamp(0, 60_000);
    let channel_initial_backoff_secs = s.channel_initial_backoff_secs.clamp(1, 3600);
    let channel_max_backoff_secs =
        s.channel_max_backoff_secs.clamp(s.channel_initial_backoff_secs.max(1), 3600);
    let scheduler_poll_secs = s.scheduler_poll_secs.clamp(1, 3600);
    let scheduler_retries = s.scheduler_retries.clamp(0, 20);
    // 调用全局配置更新函数，将可靠性设置持久化到配置文件
    // 使用 clamp 确保所有值都在安全范围内
    crate::app::update_reliability_config_async(move |reliability| {
        reliability.provider_retries = provider_retries;
        // Provider 退避时间：限制在 0-60000 毫秒（1分钟）之间
        reliability.provider_backoff_ms = provider_backoff_ms;
        // Channel 初始退避时间：限制在 1-3600 秒（1小时）之间
        reliability.channel_initial_backoff_secs = channel_initial_backoff_secs;
        // Channel 最大退避时间：必须 >= 初始退避时间，且 <= 3600 秒
        // max(1) 确保最小值为 1，避免与初始值比较时出现 0 的情况
        reliability.channel_max_backoff_secs = channel_max_backoff_secs;
        // 调度器轮询间隔：限制在 1-3600 秒之间
        reliability.scheduler_poll_secs = scheduler_poll_secs;
        // 调度器重试次数：限制在 0-20 次之间
        reliability.scheduler_retries = scheduler_retries;
    })
}

/// 处理可靠性设置相关的消息更新
///
/// 根据不同的设置消息类型，更新应用程序的可靠性配置，
/// 并自动持久化到配置文件。
///
/// # 参数
///
/// - `app`：可变引用的应用程序实例
/// - `message`：设置消息枚举，指定要更新的配置项
///
/// # 返回值
///
/// 返回 `Task<Message>`，通常为 `Task::none()`，因为这些操作不需要额外的异步任务
///
/// # 消息处理逻辑
///
/// 每个配置变更消息都会：
/// 1. 更新应用程序内存中的设置值（并进行边界约束）
/// 2. 调用 `persist_reliability_settings` 持久化到文件
/// 3. 清除之前可能存在的保存错误信息
///
/// # 特殊处理
///
/// - **Channel 初始退避时间变更**：如果新的初始值大于当前最大值，
///   会自动将最大值调整为初始值，保持 max >= initial 的约束
/// - **Channel 最大退避时间变更**：确保最大值不小于当前初始值
/// - **未匹配的消息**：返回 `Task::none()`，不做任何操作
///
/// # 示例
///
/// ```ignore
/// // 处理 Provider 重试次数变更
/// let task = update(&mut app, SettingsMessage::ReliabilityProviderRetriesChanged(5));
/// // 此时 app.reliability_settings.provider_retries 已更新为 5
/// // 配置也已持久化到文件
/// ```
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        // Provider 重试次数变更
        SettingsMessage::ReliabilityProviderRetriesChanged(v) => {
            // 约束值在 0-20 范围内
            app.reliability_settings.provider_retries = v.clamp(0, 20);
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // Provider 退避时间变更（毫秒）
        SettingsMessage::ReliabilityProviderBackoffMsChanged(v) => {
            // 约束值在 0-60000 毫秒范围内
            app.reliability_settings.provider_backoff_ms = v.clamp(0, 60_000);
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // Channel 初始退避时间变更（秒）
        SettingsMessage::ReliabilityChannelInitialBackoffSecsChanged(v) => {
            // 约束值在 1-3600 秒范围内
            app.reliability_settings.channel_initial_backoff_secs = v.clamp(1, 3600);
            // 如果最大退避时间小于新的初始退避时间，则自动调整最大值
            // 这确保了 max_backoff >= initial_backoff 的约束
            if app.reliability_settings.channel_max_backoff_secs
                < app.reliability_settings.channel_initial_backoff_secs
            {
                app.reliability_settings.channel_max_backoff_secs =
                    app.reliability_settings.channel_initial_backoff_secs;
            }
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // Channel 最大退避时间变更（秒）
        SettingsMessage::ReliabilityChannelMaxBackoffSecsChanged(v) => {
            // 最小值为当前初始退避时间（至少为 1）
            let min_v = app.reliability_settings.channel_initial_backoff_secs.max(1);
            // 约束值在 [初始值, 3600] 范围内
            app.reliability_settings.channel_max_backoff_secs = v.clamp(min_v, 3600);
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // 调度器轮询间隔变更（秒）
        SettingsMessage::ReliabilitySchedulerPollSecsChanged(v) => {
            // 约束值在 1-3600 秒范围内
            app.reliability_settings.scheduler_poll_secs = v.clamp(1, 3600);
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // 调度器重试次数变更
        SettingsMessage::ReliabilitySchedulerRetriesChanged(v) => {
            // 约束值在 0-20 次范围内
            app.reliability_settings.scheduler_retries = v.clamp(0, 20);
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // 手动保存可靠性设置
        SettingsMessage::ReliabilitySave => {
            app.reliability_settings.save_error = None;
            persist_reliability_settings(app)
        }
        // 打开帮助模态框
        SettingsMessage::ReliabilityHelpOpen => {
            app.reliability_settings.show_help_modal = true;
            Task::none()
        }
        // 关闭帮助模态框
        SettingsMessage::ReliabilityHelpClose => {
            app.reliability_settings.show_help_modal = false;
            Task::none()
        }
        // 未匹配的消息，不做任何处理
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "reliability_tests.rs"]
mod reliability_tests;
