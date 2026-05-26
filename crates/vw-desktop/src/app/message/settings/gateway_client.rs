//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message, update_gateway_client_config};
use iced::Task;

use super::messages::{GatewayClientMessage, SettingsMessage};

fn normalize_host(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() { "127.0.0.1".to_string() } else { trimmed.to_string() }
}

#[cfg(test)]
#[path = "gateway_client_tests.rs"]
mod gateway_client_tests;

fn persist_gateway_client_settings(app: &mut App) {
    let settings = &app.gateway_client_settings;
    let host = normalize_host(&settings.host_input);
    let bearer_token = settings.bearer_token_input.trim().to_string();
    let username = settings.username_input.trim().to_string();
    let password = settings.password_input.trim().to_string();
    let skey = settings.skey_input.trim().to_string();

    update_gateway_client_config(|cfg| {
        cfg.host = host;
        cfg.port = settings.port.clamp(1, u16::MAX);
        cfg.bearer_token = bearer_token;
        cfg.username = username;
        cfg.password = password;
        cfg.skey = skey;
    });
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::GatewayClient(message) = message else {
        return Task::none();
    };

    match message {
        GatewayClientMessage::HostChanged(value) => app.gateway_client_settings.host_input = value,
        GatewayClientMessage::PortChanged(value) => {
            app.gateway_client_settings.port = value.clamp(1, u16::MAX)
        }
        GatewayClientMessage::BearerTokenChanged(value) => {
            app.gateway_client_settings.bearer_token_input = value
        }
        GatewayClientMessage::UsernameChanged(value) => {
            app.gateway_client_settings.username_input = value
        }
        GatewayClientMessage::PasswordChanged(value) => {
            app.gateway_client_settings.password_input = value
        }
        GatewayClientMessage::SkeyChanged(value) => app.gateway_client_settings.skey_input = value,
        GatewayClientMessage::HelpOpen => {
            app.gateway_client_settings.show_help_modal = true;
            return Task::none();
        }
        GatewayClientMessage::HelpClose => {
            app.gateway_client_settings.show_help_modal = false;
            return Task::none();
        }
    }

    persist_gateway_client_settings(app);
    app.gateway_client_settings.host_input =
        normalize_host(&app.gateway_client_settings.host_input);
    app.gateway_client_settings.save_error = None;
    Task::none()
}
