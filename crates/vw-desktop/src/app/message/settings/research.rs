//! 研究模式设置消息处理模块
//!
//! 本模块负责处理研究模式（Research Mode）相关的设置消息，包括：
//! - 启用/禁用研究模式
//! - 配置触发器类型
//! - 管理关键词列表
//! - 设置最小消息长度和最大迭代次数
//! - 配置系统提示前缀
//! - 控制进度显示
//!
//! 所有设置变更会立即持久化到配置文件中。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;

/// 持久化研究模式设置到配置文件
///
/// 该函数将应用中的研究模式设置同步到持久化配置中，包括：
/// - 处理和清理关键词列表（去除空白、按逗号和换行分割）
/// - 去除系统提示前缀的首尾空白
/// - 限制最小消息长度和最大迭代次数的合理范围
///
/// # 参数
///
/// * `app` - 可变引用的应用实例，从中读取研究设置
fn persist_research_settings(app: &mut App) -> Task<Message> {
    let s = &app.research_settings;
    let enabled = s.enabled;
    let trigger = s.trigger;
    let min_message_length = s.min_message_length.clamp(1, 10_000) as usize;
    let max_iterations = s.max_iterations.clamp(1, 100) as usize;
    let show_progress = s.show_progress;

    // 解析关键词输入：按逗号和换行符分割，去除空白，过滤空值
    let keywords = s
        .keywords_input
        .split([',', '\n'])
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    // 去除系统提示前缀的首尾空白字符
    let system_prompt_prefix = s.system_prompt_prefix.trim().to_string();

    // 更新持久化配置
    crate::app::update_research_config_async(move |research| {
        research.enabled = enabled;
        research.trigger = trigger;
        research.keywords = keywords;
        // 限制最小消息长度在 1-10000 之间
        research.min_message_length = min_message_length;
        // 限制最大迭代次数在 1-100 之间
        research.max_iterations = max_iterations;
        research.show_progress = show_progress;
        research.system_prompt_prefix = system_prompt_prefix;
    })
}

/// 处理研究模式设置相关的消息
///
/// 该函数是研究模式设置的消息处理器，负责响应各类设置变更消息，
/// 更新应用状态并将设置持久化到配置文件。
///
/// # 参数
///
/// * `app` - 可变引用的应用实例
/// * `message` - 设置消息枚举，标识具体的设置操作
///
/// # 返回值
///
/// 返回 `Task<Message>`，可能包含需要执行的异步任务（通常为 `Task::none()`）
///
/// # 处理的消息类型
///
/// - `ResearchEnabledToggled` - 切换研究模式启用状态
/// - `ResearchTriggerChanged` - 更改触发器类型
/// - `ResearchKeywordsChanged` - 更新关键词列表
/// - `ResearchMinMessageLengthChanged` - 设置最小消息长度
/// - `ResearchMaxIterationsChanged` - 设置最大迭代次数
/// - `ResearchShowProgressToggled` - 切换进度显示
/// - `ResearchSystemPromptPrefixChanged` - 更新系统提示前缀
/// - `ResearchSave` - 手动保存设置
/// - `ResearchHelpOpen` - 打开帮助模态框
/// - `ResearchHelpClose` - 关闭帮助模态框
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        // 切换研究模式启用状态
        SettingsMessage::ResearchEnabledToggled(v) => {
            app.research_settings.enabled = v;
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 更改触发器类型
        SettingsMessage::ResearchTriggerChanged(v) => {
            app.research_settings.trigger = v;
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 更新关键词输入
        SettingsMessage::ResearchKeywordsChanged(v) => {
            app.research_settings.keywords_input = v;
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 设置最小消息长度（限制在 1-10000 之间）
        SettingsMessage::ResearchMinMessageLengthChanged(v) => {
            app.research_settings.min_message_length = v.clamp(1, 10_000);
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 设置最大迭代次数（限制在 1-100 之间）
        SettingsMessage::ResearchMaxIterationsChanged(v) => {
            app.research_settings.max_iterations = v.clamp(1, 100);
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 切换进度显示状态
        SettingsMessage::ResearchShowProgressToggled(v) => {
            app.research_settings.show_progress = v;
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 更新系统提示前缀
        SettingsMessage::ResearchSystemPromptPrefixChanged(v) => {
            app.research_settings.system_prompt_prefix = v;
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 手动保存设置
        SettingsMessage::ResearchSave => {
            app.research_settings.save_error = None;
            persist_research_settings(app)
        }

        // 打开帮助模态框
        SettingsMessage::ResearchHelpOpen => {
            app.research_settings.show_help_modal = true;
            Task::none()
        }

        // 关闭帮助模态框
        SettingsMessage::ResearchHelpClose => {
            app.research_settings.show_help_modal = false;
            Task::none()
        }

        // 其他非研究模式相关的消息，不做处理
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "research_tests.rs"]
mod research_tests;
