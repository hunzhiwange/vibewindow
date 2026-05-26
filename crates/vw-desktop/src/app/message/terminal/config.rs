//! 管理终端相关消息、配置和标签页状态更新。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::TerminalMessage;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::terminal::build_palette;
use crate::app::{App, Message, Shell, TerminalTheme, update_system_settings_config};
use iced::Task;

/// 执行 update 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub fn update(app: &mut App, message: TerminalMessage) -> Task<Message> {
    match message {
        TerminalMessage::ShellSelected(sh) => {
            app.terminal.shell = sh;
            let shell_key = match sh {
                Shell::Bash => "bash".to_string(),
                Shell::Zsh => "zsh".to_string(),
            };
            update_system_settings_config(|system| {
                system.terminal_shell = shell_key;
            });
            Task::none()
        }
        TerminalMessage::ThemeSelected(tt) => {
            app.terminal.theme = tt;

            #[cfg(not(target_arch = "wasm32"))]
            {
                let palette = build_palette(&app.app_theme);
                for tab in &mut app.terminal.tabs {
                    let _ =
                        tab.term.handle(iced_term::Command::ChangeTheme(Box::new(palette.clone())));
                }
            }

            let theme_key = match tt {
                TerminalTheme::System => "system".to_string(),
            };
            update_system_settings_config(|system| {
                system.terminal_theme = theme_key;
            });
            Task::none()
        }
        TerminalMessage::FontSelected(f) => {
            app.terminal.set_font_family(f.clone());
            update_system_settings_config(|system| {
                system.terminal_font_family = f;
            });
            Task::none()
        }
        TerminalMessage::FontSizeChanged(size) => {
            app.terminal.set_font_size(size);
            update_system_settings_config(|system| {
                system.terminal_font_size = size;
            });
            Task::none()
        }
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
