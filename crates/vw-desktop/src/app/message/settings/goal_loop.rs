//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::config::update_goal_loop_config_async;
use crate::app::{App, Message};
use iced::Task;

use super::messages::{GoalLoopMessage, SettingsMessage};

fn parse_positive_u32(input: &str, field: &str) -> Result<u32, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} 不能为空"));
    }

    let value = trimmed.parse::<u32>().map_err(|_| format!("{field} 必须是正整数"))?;
    if value == 0 {
        return Err(format!("{field} 必须大于 0"));
    }

    Ok(value)
}

#[cfg(test)]
#[path = "goal_loop_tests.rs"]
mod goal_loop_tests;

fn persist_goal_loop_settings(app: &mut App) -> Result<Task<Message>, String> {
    let s = &app.goal_loop_settings;
    let enabled = s.enabled;
    let interval_minutes =
        parse_positive_u32(&s.interval_minutes_input, "goal_loop.interval_minutes")?;
    let step_timeout_secs =
        parse_positive_u32(&s.step_timeout_secs_input, "goal_loop.step_timeout_secs")?;
    let max_steps_per_cycle =
        parse_positive_u32(&s.max_steps_per_cycle_input, "goal_loop.max_steps_per_cycle")?;
    let channel = s.channel_input.trim().to_string();
    let target = s.target_input.trim().to_string();

    Ok(update_goal_loop_config_async(move |goal_loop| {
        goal_loop.enabled = enabled;
        goal_loop.interval_minutes = interval_minutes;
        goal_loop.step_timeout_secs = step_timeout_secs as u64;
        goal_loop.max_steps_per_cycle = max_steps_per_cycle;
        goal_loop.channel = if channel.is_empty() { None } else { Some(channel) };
        goal_loop.target = if target.is_empty() { None } else { Some(target) };
    }))
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::GoalLoop(message) = message else {
        return Task::none();
    };

    match message {
        GoalLoopMessage::EnabledToggled(value) => app.goal_loop_settings.enabled = value,
        GoalLoopMessage::IntervalMinutesChanged(value) => {
            app.goal_loop_settings.interval_minutes_input = value
        }
        GoalLoopMessage::StepTimeoutSecsChanged(value) => {
            app.goal_loop_settings.step_timeout_secs_input = value
        }
        GoalLoopMessage::MaxStepsPerCycleChanged(value) => {
            app.goal_loop_settings.max_steps_per_cycle_input = value
        }
        GoalLoopMessage::ChannelChanged(value) => app.goal_loop_settings.channel_input = value,
        GoalLoopMessage::TargetChanged(value) => app.goal_loop_settings.target_input = value,
    }

    match persist_goal_loop_settings(app) {
        Ok(task) => {
            app.goal_loop_settings.save_error = None;
            task
        }
        Err(err) => {
            app.goal_loop_settings.save_error = Some(err);
            Task::none()
        }
    }
}
