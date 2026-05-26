//! 处理渠道设置子模块的状态变更、字段转换和持久化。

use crate::app::config::update_channels_config_async;
use crate::app::{state::ChannelsSettingsState, App, Message};
use iced::Task;

fn enabled_channels(settings: &ChannelsSettingsState) -> Vec<&'static str> {
    let mut channels = Vec::new();
    if settings.cli {
        channels.push("CLI");
    }
    if settings.telegram.is_some() {
        channels.push("Telegram");
    }
    if settings.discord.is_some() {
        channels.push("Discord");
    }
    if settings.slack.is_some() {
        channels.push("Slack");
    }
    if settings.mattermost.is_some() {
        channels.push("Mattermost");
    }
    if settings.webhook.is_some() {
        channels.push("Webhook");
    }
    if settings.imessage.is_some() {
        channels.push("iMessage");
    }
    if settings.matrix.is_some() {
        channels.push("Matrix");
    }
    if settings.signal.is_some() {
        channels.push("Signal");
    }
    if settings.whatsapp.is_some() {
        channels.push("WhatsApp");
    }
    if settings.linq.is_some() {
        channels.push("Linq");
    }
    if settings.wati.is_some() {
        channels.push("WATI");
    }
    if settings.nextcloud_talk.is_some() {
        channels.push("Nextcloud Talk");
    }
    #[cfg(not(target_arch = "wasm32"))]
    if settings.email.is_some() {
        channels.push("Email");
    }
    if settings.irc.is_some() {
        channels.push("IRC");
    }
    if settings.lark.is_some() {
        channels.push("Lark");
    }
    if settings.feishu.is_some() {
        channels.push("Feishu");
    }
    if settings.dingtalk.is_some() {
        channels.push("DingTalk");
    }
    if settings.qq.is_some() {
        channels.push("QQ");
    }
    if settings.nostr.is_some() {
        channels.push("Nostr");
    }
    if settings.clawdtalk.is_some() {
        channels.push("ClawdTalk");
    }
    channels
}

#[cfg(test)]
#[path = "persist_tests.rs"]
mod persist_tests;

/// 处理 `persist_channels_settings` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
/// 返回 `None` 表示输入为空或当前状态不需要生成后续值。
pub(super) fn persist_channels_settings(app: &mut App) -> Option<Task<Message>> {
    let settings = &app.channels_settings;
    let project_dir = settings.project_dir();
    let cli = settings.cli;
    let telegram = settings.telegram.clone();
    let discord = settings.discord.clone();
    let slack = settings.slack.clone();
    let mattermost = settings.mattermost.clone();
    let webhook = settings.webhook.clone();
    let imessage = settings.imessage.clone();
    let matrix = settings.matrix.clone();
    let signal = settings.signal.clone();
    let whatsapp = settings.whatsapp.clone();
    let linq = settings.linq.clone();
    let wati = settings.wati.clone();
    let nextcloud_talk = settings.nextcloud_talk.clone();
    #[cfg(not(target_arch = "wasm32"))]
    let email = settings.email.clone();
    let irc = settings.irc.clone();
    let lark = settings.lark.clone();
    let feishu = settings.feishu.clone();
    let dingtalk = settings.dingtalk.clone();
    let qq = settings.qq.clone();
    let nostr = settings.nostr.clone();
    let clawdtalk = settings.clawdtalk.clone();
    let message_timeout_secs = settings.message_timeout_secs.max(1) as u64;

    let enabled = enabled_channels(settings);
    if enabled.is_empty() {
        app.channels_settings.save_error =
            Some("当前未启用任何通道。至少保留一个可用入口可避免运行时无消息源。".to_string());
        return None;
    }

    app.channels_settings.save_error = None;
    Some(update_channels_config_async(move |channels| {
        channels.project_dir = project_dir;
        channels.cli = cli;
        channels.telegram = telegram;
        channels.discord = discord;
        channels.slack = slack;
        channels.mattermost = mattermost;
        channels.webhook = webhook;
        channels.imessage = imessage;
        channels.matrix = matrix;
        channels.signal = signal;
        channels.whatsapp = whatsapp;
        channels.linq = linq;
        channels.wati = wati;
        channels.nextcloud_talk = nextcloud_talk;
        #[cfg(not(target_arch = "wasm32"))]
        {
            channels.email = email;
        }
        channels.irc = irc;
        channels.lark = lark;
        channels.feishu = feishu;
        channels.dingtalk = dingtalk;
        channels.qq = qq;
        channels.nostr = nostr;
        channels.clawdtalk = clawdtalk;
        channels.message_timeout_secs = message_timeout_secs;
    }))
}
