//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::config::update_gateway_config_async;
use crate::app::message::settings::util::parse_comma_or_newline_list;
use crate::app::{App, Message};
use iced::Task;

use super::messages::{GatewayMessage, SettingsMessage};

fn normalize_host(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() { "127.0.0.1".to_string() } else { trimmed.to_string() }
}

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod gateway_tests;

fn clamp_token_limit(value: u32) -> u32 {
    value.clamp(1, 100_000)
}

fn persist_gateway_settings(app: &mut App) -> Task<Message> {
    let s = &app.gateway_settings;
    let port = s.port.clamp(1, u16::MAX);
    let require_pairing = s.require_pairing;
    let allow_public_bind = s.allow_public_bind;
    let pair_rate_limit_per_minute = s.pair_rate_limit_per_minute.clamp(1, 10_000);
    let webhook_rate_limit_per_minute = s.webhook_rate_limit_per_minute.clamp(1, 100_000);
    let trust_forwarded_headers = s.trust_forwarded_headers;
    let rate_limit_max_keys = clamp_token_limit(s.rate_limit_max_keys) as usize;
    let idempotency_ttl_secs = s.idempotency_ttl_secs.clamp(1, 86_400) as u64;
    let idempotency_max_keys = clamp_token_limit(s.idempotency_max_keys) as usize;
    let node_control_enabled = s.node_control_enabled;
    let host = normalize_host(&s.host_input);
    let paired_tokens = s
        .paired_tokens
        .iter()
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let node_control_auth_token = s.node_control_auth_token_input.trim().to_string();
    let allowed_node_ids = parse_comma_or_newline_list(&s.node_control_allowed_node_ids_input);

    update_gateway_config_async(move |gateway| {
        gateway.port = port;
        gateway.host = host;
        gateway.require_pairing = require_pairing;
        gateway.allow_public_bind = allow_public_bind;
        gateway.paired_tokens = paired_tokens;
        gateway.pair_rate_limit_per_minute = pair_rate_limit_per_minute;
        gateway.webhook_rate_limit_per_minute = webhook_rate_limit_per_minute;
        gateway.trust_forwarded_headers = trust_forwarded_headers;
        gateway.rate_limit_max_keys = rate_limit_max_keys;
        gateway.idempotency_ttl_secs = idempotency_ttl_secs;
        gateway.idempotency_max_keys = idempotency_max_keys;
        gateway.node_control.enabled = node_control_enabled;
        gateway.node_control.auth_token =
            if node_control_auth_token.is_empty() { None } else { Some(node_control_auth_token) };
        gateway.node_control.allowed_node_ids = allowed_node_ids;
    })
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::Gateway(message) = message else {
        return Task::none();
    };

    if matches!(message, GatewayMessage::Refresh) {
        app.gateway_settings.save_error = None;
        return Task::none();
    }

    match message {
        GatewayMessage::PortChanged(value) => app.gateway_settings.port = value.clamp(1, u16::MAX),
        GatewayMessage::HostChanged(value) => app.gateway_settings.host_input = value,
        GatewayMessage::RequirePairingToggled(value) => {
            app.gateway_settings.require_pairing = value
        }
        GatewayMessage::AllowPublicBindToggled(value) => {
            app.gateway_settings.allow_public_bind = value
        }
        GatewayMessage::TrustForwardedHeadersToggled(value) => {
            app.gateway_settings.trust_forwarded_headers = value
        }
        GatewayMessage::NewPairedTokenChanged(value) => {
            app.gateway_settings.new_paired_token_input = value
        }
        GatewayMessage::AddPairedToken => {
            let token = app.gateway_settings.new_paired_token_input.trim().to_string();
            if token.is_empty() {
                app.gateway_settings.save_error = Some("配对令牌不能为空".to_string());
                return Task::none();
            }
            if !app.gateway_settings.paired_tokens.iter().any(|existing| existing == &token) {
                app.gateway_settings.paired_tokens.push(token);
            } else {
                app.gateway_settings.save_error = Some("配对令牌已存在".to_string());
                return Task::none();
            }
            app.gateway_settings.new_paired_token_input.clear();
        }
        GatewayMessage::RemovePairedToken(index) => {
            if index < app.gateway_settings.paired_tokens.len() {
                app.gateway_settings.paired_tokens.remove(index);
            }
        }
        GatewayMessage::PairRateLimitPerMinuteChanged(value) => {
            app.gateway_settings.pair_rate_limit_per_minute = value.clamp(1, 10_000)
        }
        GatewayMessage::WebhookRateLimitPerMinuteChanged(value) => {
            app.gateway_settings.webhook_rate_limit_per_minute = value.clamp(1, 100_000)
        }
        GatewayMessage::RateLimitMaxKeysChanged(value) => {
            app.gateway_settings.rate_limit_max_keys = clamp_token_limit(value)
        }
        GatewayMessage::IdempotencyTtlSecsChanged(value) => {
            app.gateway_settings.idempotency_ttl_secs = value.clamp(1, 86_400)
        }
        GatewayMessage::IdempotencyMaxKeysChanged(value) => {
            app.gateway_settings.idempotency_max_keys = clamp_token_limit(value)
        }
        GatewayMessage::NodeControlEnabledToggled(value) => {
            app.gateway_settings.node_control_enabled = value
        }
        GatewayMessage::NodeControlAuthTokenChanged(value) => {
            app.gateway_settings.node_control_auth_token_input = value
        }
        GatewayMessage::NodeControlAllowedNodeIdsChanged(value) => {
            app.gateway_settings.node_control_allowed_node_ids_input = value
        }
        GatewayMessage::HelpOpen => {
            app.gateway_settings.show_help_modal = true;
            return Task::none();
        }
        GatewayMessage::HelpClose => {
            app.gateway_settings.show_help_modal = false;
            return Task::none();
        }
        GatewayMessage::Refresh => unreachable!(),
    }

    app.gateway_settings.host_input = normalize_host(&app.gateway_settings.host_input);
    app.gateway_settings.save_error = None;
    persist_gateway_settings(app)
}
