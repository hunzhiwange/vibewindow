//! 处理视图面板的外部打开、弹出层与使用量展示交互。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::ViewMessage;
use crate::app::{App, Message};
use iced::Task;

/// 执行 update 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub fn update(app: &mut App, message: ViewMessage) -> Task<Message> {
    match message {
        ViewMessage::UsageModelInfoLoaded(info) => {
            app.usage_model_info = info;
            Task::none()
        }
        ViewMessage::UsageSessionFilePathLoaded(session_id, result) => {
            if app.active_session_id.as_deref() != Some(session_id.as_str()) {
                return Task::none();
            }

            app.usage_session_file_path =
                result.ok().flatten().and_then(|path| path.to_str().map(str::to_owned));
            Task::none()
        }
        ViewMessage::UsageStepToggled(step_index) => {
            if app.usage_step_expanded.contains(&step_index) {
                app.usage_step_expanded.remove(&step_index);
            } else {
                app.usage_step_expanded.insert(step_index);
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

#[cfg(test)]
#[path = "usage_tests.rs"]
mod usage_tests;
