//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::message::RedisToolMessage;
use crate::app::state::RedisConnectionTab;
use crate::app::{App, Message};
use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_divider,
};
use iced::widget::{button, checkbox, column, container, row, text};
use iced::{Alignment, Element};

use super::super::common::{
    build_input, build_path_picker_input, form_row,
};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `is_busy`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_tab_bar<'a>(app: &'a App, is_busy: bool) -> Element<'a, Message> {
    let mut tabs = row![].spacing(8).align_y(Alignment::Center);
    for tab in [
        RedisConnectionTab::Basic,
        RedisConnectionTab::Ssh,
        RedisConnectionTab::Tls,
        RedisConnectionTab::Sentinel,
        RedisConnectionTab::Cluster,
    ] {
        let active = app.redis_tool.draft_tab == tab;
        let tab_button: Element<'a, Message> = if active {
            button(text(tab.title()).size(13))
                .on_press_maybe((!is_busy).then_some(Message::RedisTool(
                    RedisToolMessage::DraftTabChanged(tab),
                )))
                .padding([8, 12])
                .style(primary_action_btn_style)
                .into()
        } else {
            button(text(tab.title()).size(13))
                .on_press_maybe((!is_busy).then_some(Message::RedisTool(
                    RedisToolMessage::DraftTabChanged(tab),
                )))
                .padding([8, 12])
                .style(rounded_action_btn_style)
                .into()
        };
        tabs = tabs.push(tab_button);
    }

    container(tabs).padding([12, 0]).into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `compact`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_active_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    match app.redis_tool.draft_tab {
        RedisConnectionTab::Basic => build_basic_tab(app, compact),
        RedisConnectionTab::Ssh => build_ssh_tab(app, compact),
        RedisConnectionTab::Tls => build_tls_tab(app, compact),
        RedisConnectionTab::Sentinel => build_sentinel_tab(app, compact),
        RedisConnectionTab::Cluster => build_cluster_tab(app, compact),
    }
}

fn build_basic_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    column![
        form_row(
            "连接名称",
            "建议使用环境或业务名，例如 production-cache。",
            build_input(
                "例如：本地开发",
                &app.redis_tool.draft.name,
                RedisToolMessage::DraftNameChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "主机地址",
            "支持域名或 IP，例如 127.0.0.1。",
            build_input(
                "127.0.0.1",
                &app.redis_tool.draft.host,
                RedisToolMessage::DraftHostChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "端口",
            "默认 Redis 端口为 6379。",
            build_input("6379", &app.redis_tool.draft.port, RedisToolMessage::DraftPortChanged),
            compact,
        ),
        settings_divider(),
        form_row(
            "数据库",
            "默认使用 DB 0，可按实例需要切换。",
            build_input("0", &app.redis_tool.draft.db, RedisToolMessage::DraftDbChanged),
            compact,
        ),
        settings_divider(),
        form_row(
            "用户名",
            "Redis 6 ACL 可填写用户名；传统密码模式可留空。",
            build_input(
                "可留空",
                &app.redis_tool.draft.username,
                RedisToolMessage::DraftUsernameChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "密码",
            "密码会保存在本地连接配置中，但不会写入历史日志。",
            build_input(
                "可留空",
                &app.redis_tool.draft.password,
                RedisToolMessage::DraftPasswordChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "键匹配模式",
            "后续键浏览与加载会优先使用该模式，默认 *。",
            build_input(
                "*",
                &app.redis_tool.draft.key_pattern,
                RedisToolMessage::DraftKeyPatternChanged,
            ),
            compact,
        ),
    ]
    .spacing(0)
    .into()
}

fn build_ssh_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let enabled = !app.redis_tool.is_gateway_loading();
    column![
        form_row(
            "SSH 隧道",
            "用于先打通堡垒机 / 跳板机，再访问 Redis。当前版本仅保存配置。",
            checkbox(app.redis_tool.draft.ssh_tunnel.enabled)
                .label("启用 SSH")
                .on_toggle(|value| Message::RedisTool(RedisToolMessage::DraftSshEnabledToggled(value)))
                .into(),
            compact,
        ),
        settings_divider(),
        form_row(
            "SSH 地址",
            "通常为堡垒机或跳板机地址。",
            build_input(
                "bastion.example.com",
                &app.redis_tool.draft.ssh_tunnel.host,
                RedisToolMessage::DraftSshHostChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "SSH 端口",
            "默认 22。",
            build_input(
                "22",
                &app.redis_tool.draft.ssh_tunnel.port,
                RedisToolMessage::DraftSshPortChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "SSH 用户名",
            "启用 SSH 时必须填写用户名。",
            build_input(
                "deploy",
                &app.redis_tool.draft.ssh_tunnel.username,
                RedisToolMessage::DraftSshUsernameChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "SSH 密码",
            "如使用私钥，可留空。",
            build_input(
                "可留空",
                &app.redis_tool.draft.ssh_tunnel.password,
                RedisToolMessage::DraftSshPasswordChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "私钥路径",
            "填写本地私钥路径，例如 ~/.ssh/id_rsa。",
            build_path_picker_input(
                "~/.ssh/id_rsa",
                &app.redis_tool.draft.ssh_tunnel.private_key_path,
                RedisToolMessage::DraftSshPrivateKeyPathChanged,
                Message::RedisTool(RedisToolMessage::PickSshPrivateKeyFile),
                enabled,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "Passphrase",
            "私钥带口令时填写。",
            build_input(
                "可留空",
                &app.redis_tool.draft.ssh_tunnel.passphrase,
                RedisToolMessage::DraftSshPassphraseChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "超时",
            "SSH 建连超时，单位秒。",
            build_input(
                "30",
                &app.redis_tool.draft.ssh_tunnel.timeout_secs,
                RedisToolMessage::DraftSshTimeoutSecsChanged,
            ),
            compact,
        ),
    ]
    .spacing(0)
    .into()
}

fn build_tls_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let enabled = !app.redis_tool.is_gateway_loading();
    column![
        form_row(
            "SSL/TLS",
            "启用 rediss 连接；如需 mTLS 或自定义 CA，可直接选择本地证书文件。",
            checkbox(app.redis_tool.draft.use_tls)
                .label("启用 SSL/TLS")
                .on_toggle(|value| Message::RedisTool(RedisToolMessage::DraftTlsToggled(value)))
                .into(),
            compact,
        ),
        settings_divider(),
        form_row(
            "客户端私钥",
            "填写 PEM 私钥文件路径。",
            build_path_picker_input(
                "client.key",
                &app.redis_tool.draft.tls_cert.private_key_path,
                RedisToolMessage::DraftTlsPrivateKeyPathChanged,
                Message::RedisTool(RedisToolMessage::PickTlsPrivateKeyFile),
                enabled,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "客户端证书",
            "填写 PEM 证书文件路径。",
            build_path_picker_input(
                "client.crt",
                &app.redis_tool.draft.tls_cert.public_cert_path,
                RedisToolMessage::DraftTlsPublicCertPathChanged,
                Message::RedisTool(RedisToolMessage::PickTlsPublicCertFile),
                enabled,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "CA 证书",
            "填写签发方 CA 证书路径。",
            build_path_picker_input(
                "ca.crt",
                &app.redis_tool.draft.tls_cert.ca_cert_path,
                RedisToolMessage::DraftTlsCaCertPathChanged,
                Message::RedisTool(RedisToolMessage::PickTlsCaCertFile),
                enabled,
            ),
            compact,
        ),
    ]
    .spacing(0)
    .into()
}

fn build_sentinel_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    column![
        form_row(
            "Sentinel",
            "基础页中的主机和端口应填写 Sentinel 节点入口。",
            checkbox(app.redis_tool.draft.sentinel.enabled)
                .label("启用 Sentinel")
                .on_toggle(|value| {
                    Message::RedisTool(RedisToolMessage::DraftSentinelEnabledToggled(value))
                })
                .into(),
            compact,
        ),
        settings_divider(),
        form_row(
            "Master 组名称",
            "例如 mymaster。启用 Sentinel 时必须填写。",
            build_input(
                "mymaster",
                &app.redis_tool.draft.sentinel.master_name,
                RedisToolMessage::DraftSentinelMasterNameChanged,
            ),
            compact,
        ),
        settings_divider(),
        form_row(
            "节点密码",
            "用于 Redis 节点鉴权，可与基础密码不同。",
            build_input(
                "可留空",
                &app.redis_tool.draft.sentinel.node_password,
                RedisToolMessage::DraftSentinelNodePasswordChanged,
            ),
            compact,
        ),
    ]
    .spacing(0)
    .into()
}

fn build_cluster_tab<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    column![
        form_row(
            "Cluster 模式",
            "用于保存 Redis Cluster 连接意图，当前已支持测试、运行态读取与命令执行。",
            checkbox(app.redis_tool.draft.use_cluster)
                .label("启用 Cluster")
                .on_toggle(|value| Message::RedisTool(RedisToolMessage::DraftClusterToggled(value)))
                .into(),
            compact,
        ),
        settings_divider(),
        form_row(
            "只读访问",
            "记录后续键浏览 / 命令执行是否按只读节点策略接入。",
            checkbox(app.redis_tool.draft.read_only)
                .label("Readonly")
                .on_toggle(|value| Message::RedisTool(RedisToolMessage::DraftReadOnlyToggled(value)))
                .into(),
            compact,
        ),
    ]
    .spacing(0)
    .into()
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod connection_tests;
