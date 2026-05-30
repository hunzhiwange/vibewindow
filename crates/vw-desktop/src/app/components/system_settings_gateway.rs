//! 系统设置中 gateway 配置页面的界面拼装与交互消息转换。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use crate::app::components::system_settings_common::{
    SETTINGS_CONTROL_PADDING, SETTINGS_CONTROL_TEXT_SIZE, SETTINGS_LABEL_WIDTH,
    rounded_action_btn_style, settings_checkbox_style, settings_divider, settings_error_banner,
    settings_help_button, settings_muted_text_style, settings_page_intro, settings_panel,
    settings_section_card, settings_text_input_style, settings_value_badge,
};
use crate::app::message::settings::{GatewayMessage, SettingsMessage};
use crate::app::views::design::properties::number_input::NumberInput;
use crate::app::{App, Message};
use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

fn field_row<'a>(
    label: &'static str,
    description: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(label).size(13),
                text(description).size(11).style(settings_muted_text_style),
            ]
            .spacing(4)
            .width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
            container(control.into()).width(Length::Fill),
        ]
        .spacing(22)
        .align_y(Alignment::Center),
    )
    .padding([14, 0])
    .width(Length::Fill)
    .into()
}

fn text_row<'a>(
    label: &'static str,
    description: &'static str,
    placeholder: &'static str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    field_row(
        label,
        description,
        text_input(placeholder, value)
            .on_input(on_input)
            .padding(SETTINGS_CONTROL_PADDING)
            .size(SETTINGS_CONTROL_TEXT_SIZE)
            .style(settings_text_input_style)
            .width(Length::Fill),
    )
}

fn secure_text_row<'a>(
    label: &'static str,
    description: &'static str,
    placeholder: &'static str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    field_row(
        label,
        description,
        text_input(placeholder, value)
            .secure(true)
            .on_input(on_input)
            .padding(SETTINGS_CONTROL_PADDING)
            .size(SETTINGS_CONTROL_TEXT_SIZE)
            .style(settings_text_input_style)
            .width(Length::Fill),
    )
}

fn bool_row<'a>(
    label: &'static str,
    description: &'static str,
    checked: bool,
    checkbox_label: &'static str,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    field_row(
        label,
        description,
        checkbox(checked).label(checkbox_label).on_toggle(on_toggle).style(settings_checkbox_style),
    )
}

fn hint_row<'a>(message: &'a str) -> Element<'a, Message> {
    row![
        container(text("")).width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
        text(message).size(12).style(settings_muted_text_style),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

fn number_row<'a>(
    label: &'static str,
    description: &'static str,
    value: u32,
    min: u32,
    max: u32,
    suffix: &'static str,
    on_change: impl Fn(u32) -> Message + 'a,
) -> Element<'a, Message> {
    let display = if suffix.is_empty() { value.to_string() } else { format!("{value} {suffix}") };

    field_row(
        label,
        description,
        row![
            NumberInput::new(value as f32, min as f32, max as f32, 1.0, 0, 0.15, move |raw| {
                on_change(raw.round() as u32)
            })
            .settings_style(),
            settings_value_badge(display),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    )
}

/// 构建或处理 `view` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub fn view(app: &App) -> Element<'_, Message> {
    let s = &app.gateway_settings;

    let help_btn =
        settings_help_button(Message::Settings(SettingsMessage::Gateway(GatewayMessage::HelpOpen)));

    let port_row = number_row(
        "端口",
        "控制服务端 Gateway 的监听端口。",
        s.port as u32,
        1,
        u16::MAX as u32,
        "",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::PortChanged(value as u16)))
        },
    );
    let host_row = text_row(
        "主机地址",
        "控制服务端 Gateway 的监听地址。",
        "127.0.0.1",
        &s.host_input,
        |value| Message::Settings(SettingsMessage::Gateway(GatewayMessage::HostChanged(value))),
    );

    let require_pairing_row = bool_row(
        "需要配对",
        "未配对请求必须先通过 /v1/pair。",
        s.require_pairing,
        "未配对请求必须先通过 /v1/pair",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::RequirePairingToggled(
                value,
            )))
        },
    );
    let allow_public_bind_row = bool_row(
        "允许公网绑定",
        "允许绑定到非本地地址。",
        s.allow_public_bind,
        "允许绑定到非本地地址",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::AllowPublicBindToggled(
                value,
            )))
        },
    );
    let trust_forwarded_headers_row = bool_row(
        "信任转发头",
        "仅在可信反向代理后开启。",
        s.trust_forwarded_headers,
        "信任 X-Forwarded-For / X-Real-IP",
        |value| {
            Message::Settings(SettingsMessage::Gateway(
                GatewayMessage::TrustForwardedHeadersToggled(value),
            ))
        },
    );

    let paired_tokens_section = settings_panel(
        column![field_row(
            "新增令牌",
            "手动添加已配对 bearer token。",
            row![
                text_input("输入 bearer token", &s.new_paired_token_input)
                    .secure(true)
                    .on_input(|value| Message::Settings(SettingsMessage::Gateway(
                        GatewayMessage::NewPairedTokenChanged(value)
                    )))
                    .padding(SETTINGS_CONTROL_PADDING)
                    .size(SETTINGS_CONTROL_TEXT_SIZE)
                    .style(settings_text_input_style)
                    .width(Length::Fill),
                button(text("添加"))
                    .padding([6, 12])
                    .on_press(Message::Settings(SettingsMessage::Gateway(
                        GatewayMessage::AddPairedToken
                    )))
                    .style(rounded_action_btn_style),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        )]
        .spacing(0),
    );

    let paired_tokens_list: Element<'_, Message> = if s.paired_tokens.is_empty() {
        settings_panel(
            column![
                text("当前没有已保存的配对令牌。开启 require_pairing 后，未配对客户端将被拒绝。",)
                    .size(12)
                    .style(settings_muted_text_style)
            ]
            .spacing(0),
        )
        .into()
    } else {
        let list = s.paired_tokens.iter().enumerate().fold(
            column![].spacing(10),
            |column, (index, token)| {
                let masked = if token.len() <= 8 {
                    "*".repeat(token.len().max(1))
                } else {
                    format!("{}***{}", &token[..4], &token[token.len() - 4..])
                };

                column.push(
                    row![
                        text(format!("令牌 #{}", index + 1))
                            .size(13)
                            .width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
                        text(masked).width(Length::Fill),
                        button(text("删除"))
                            .padding([6, 12])
                            .on_press(Message::Settings(SettingsMessage::Gateway(
                                GatewayMessage::RemovePairedToken(index),
                            )))
                            .style(rounded_action_btn_style),
                    ]
                    .spacing(20)
                    .align_y(Alignment::Center),
                )
            },
        );

        settings_panel(list).into()
    };

    let pair_rate_row = number_row(
        "配对速率限制/分钟",
        "限制 /pair 每分钟可接受的请求数。",
        s.pair_rate_limit_per_minute,
        1,
        10_000,
        "次/分钟",
        |value| {
            Message::Settings(SettingsMessage::Gateway(
                GatewayMessage::PairRateLimitPerMinuteChanged(value),
            ))
        },
    );
    let webhook_rate_row = number_row(
        "Webhook速率限制/分钟",
        "限制 /webhook 每分钟可接受的请求数。",
        s.webhook_rate_limit_per_minute,
        1,
        100_000,
        "次/分钟",
        |value| {
            Message::Settings(SettingsMessage::Gateway(
                GatewayMessage::WebhookRateLimitPerMinuteChanged(value),
            ))
        },
    );
    let rate_limit_max_keys_row = number_row(
        "速率限制最大键数",
        "约束网关内部 key map 规模。",
        s.rate_limit_max_keys,
        1,
        100_000,
        "个键",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::RateLimitMaxKeysChanged(
                value,
            )))
        },
    );

    let idempotency_ttl_row = number_row(
        "幂等性TTL(秒)",
        "控制 webhook 幂等 key 的存活时间。",
        s.idempotency_ttl_secs,
        1,
        86_400,
        "秒",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::IdempotencyTtlSecsChanged(
                value,
            )))
        },
    );
    let idempotency_max_keys_row = number_row(
        "幂等性最大键数",
        "控制幂等缓存允许保留的 key 数量。",
        s.idempotency_max_keys,
        1,
        100_000,
        "个键",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::IdempotencyMaxKeysChanged(
                value,
            )))
        },
    );

    let node_control_enabled_row = bool_row(
        "节点控制.已启用",
        "启用实验性的 node-control API。",
        s.node_control_enabled,
        "启用实验性的 node-control API",
        |value| {
            Message::Settings(SettingsMessage::Gateway(GatewayMessage::NodeControlEnabledToggled(
                value,
            )))
        },
    );
    let node_control_auth_token_row = secure_text_row(
        "节点控制.认证令牌",
        "额外共享令牌，客户端需通过 X-Node-Control-Token 传递。",
        "可选共享令牌",
        &s.node_control_auth_token_input,
        |value| {
            Message::Settings(SettingsMessage::Gateway(
                GatewayMessage::NodeControlAuthTokenChanged(value),
            ))
        },
    );
    let node_control_allowed_ids_row = text_row(
        "节点控制.允许的节点ID",
        "留空表示不设置显式 allowlist；可使用 * 允许所有节点。",
        "逗号或换行分隔，支持 *",
        &s.node_control_allowed_node_ids_input,
        |value| {
            Message::Settings(SettingsMessage::Gateway(
                GatewayMessage::NodeControlAllowedNodeIdsChanged(value),
            ))
        },
    );

    let mut content = column![
        row![
            container(settings_page_intro(
                "服务端网关配置",
                "配置 Gateway 的监听地址、配对安全、限流和实验性 node-control 行为。",
            ))
            .width(Length::Fill),
            help_btn
        ]
        .align_y(Alignment::Start),
        settings_section_card(
            "监听地址",
            "控制网关监听 host / port。默认仅绑定回环地址，避免意外暴露到公网。"
        ),
        settings_panel(column![port_row, settings_divider(), host_row].spacing(0)),
        hint_row("若 host 非本地地址，建议仅在可信反向代理或隧道环境下使用。"),
        settings_section_card("安全开关", "控制配对要求、公网绑定与反向代理头信任策略。"),
        settings_panel(
            column![
                require_pairing_row,
                settings_divider(),
                allow_public_bind_row,
                settings_divider(),
                trust_forwarded_headers_row,
            ]
            .spacing(0),
        ),
        hint_row("仅在你完全信任前置代理时开启 forwarded headers。"),
        settings_section_card(
            "配对令牌",
            "维护 gateway.paired_tokens 列表。此列表会被写入加密配置存储，用于已配对客户端访问。",
        ),
        paired_tokens_section,
        paired_tokens_list,
        settings_section_card(
            "速率限制",
            "限制 /pair 和 /webhook 请求频率，并约束网关内部 key map 规模。"
        ),
        settings_panel(
            column![
                pair_rate_row,
                settings_divider(),
                webhook_rate_row,
                settings_divider(),
                rate_limit_max_keys_row
            ]
            .spacing(0),
        ),
        settings_section_card("幂等性", "控制 webhook 幂等 key 的存活时间与内存上限。"),
        settings_panel(
            column![idempotency_ttl_row, settings_divider(), idempotency_max_keys_row].spacing(0)
        ),
        settings_section_card(
            "Node Control",
            "配置实验性的 node-control 协议，包括额外鉴权和允许的远端节点 ID。"
        ),
        settings_panel(
            column![
                node_control_enabled_row,
                settings_divider(),
                node_control_auth_token_row,
                settings_divider(),
                node_control_allowed_ids_row,
            ]
            .spacing(0),
        ),
        hint_row("allowed_node_ids 留空表示不设置显式 allowlist；可使用 * 允许所有节点。"),
    ]
    .spacing(16)
    .width(Length::Fill);

    if let Some(err) = &s.save_error {
        content = content.push(settings_error_banner(err));
    }

    content.into()
}

/// 构建或处理 `view_overlays` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub fn view_overlays<'a>(app: &'a App, dialog: Element<'a, Message>) -> Element<'a, Message> {
    let s = &app.gateway_settings;
    if !s.show_help_modal {
        return dialog;
    }

    let help_text = r#"服务端网关配置说明

一、监听与暴露
- gateway.port / gateway.host 控制 HTTP gateway 的监听地址。
- allow_public_bind 默认关闭，防止在无隧道/代理保护的情况下直接暴露公网。

二、安全与配对
- require_pairing 开启后，客户端需先完成 /pair 才能访问受保护端点。
- paired_tokens 保存已经配对的 bearer token，通常由自动配对流程维护，也可在桌面端手动管理。
- trust_forwarded_headers 仅应在可信反向代理后开启。

三、限流与幂等
- pair_rate_limit_per_minute / webhook_rate_limit_per_minute 控制不同端点的每分钟请求上限。
- rate_limit_max_keys 控制内存中追踪的客户端 key 数量。
- idempotency_ttl_secs / idempotency_max_keys 控制 webhook 幂等缓存行为。

四、Node Control
- node_control.enabled 打开实验性 node-control API。
- node_control.auth_token 为额外共享令牌，客户端需通过 X-Node-Control-Token 传递。
- node_control.allowed_node_ids 为空表示不显式限制；可使用 * 允许所有节点。

五、推荐默认
- host 使用 127.0.0.1。
- require_pairing 保持开启。
- allow_public_bind 保持关闭。
- trust_forwarded_headers 仅在受控代理链路后启用。"#;

    crate::app::components::system_settings_common::with_settings_help_modal(
        app,
        dialog,
        "Gateway 配置帮助",
        help_text,
        Message::Settings(SettingsMessage::Gateway(GatewayMessage::HelpClose)),
    )
}
