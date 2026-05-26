//! 系统设置中 gateway client 配置页面的界面拼装与交互消息转换。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use crate::app::components::system_settings_common::{
    SETTINGS_CONTROL_PADDING, SETTINGS_CONTROL_TEXT_SIZE, SETTINGS_LABEL_WIDTH,
    settings_error_banner, settings_help_button, settings_muted_text_style,
    settings_page_intro, settings_panel, settings_section_card, settings_text_input_style,
    settings_value_badge, with_settings_help_modal,
};
use crate::app::message::settings::{GatewayClientMessage, SettingsMessage};
use crate::app::views::design::properties::number_input::NumberInput;
use crate::app::{App, Message};
use iced::widget::{column, container, row, text, text_input};
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
    let s = &app.gateway_client_settings;

    let help_btn = settings_help_button(Message::Settings(SettingsMessage::GatewayClient(
        GatewayClientMessage::HelpOpen,
    )));

    let host_row = text_row("目标主机", "桌面端请求所连接的 Gateway 主机。", "127.0.0.1", &s.host_input, |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::HostChanged(value)))
    });
    let port_row = number_row("目标端口", "桌面端请求所连接的 Gateway 端口。", s.port as u32, 1, u16::MAX as u32, "", |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::PortChanged(
            value as u16,
        )))
    });
    let bearer_row = secure_text_row("Bearer Token", "优先使用的配对令牌；填写后会作为 Authorization: Bearer 发送。", "已配对 Bearer Token", &s.bearer_token_input, |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::BearerTokenChanged(
            value,
        )))
    });
    let username_row = text_row("用户名", "Basic Auth 用户名。", "vibewindow", &s.username_input, |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::UsernameChanged(
            value,
        )))
    });
    let password_row = secure_text_row("密码", "Basic Auth 密码；留空时不发送 Authorization 头。", "Basic Auth 密码", &s.password_input, |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::PasswordChanged(
            value,
        )))
    });
    let skey_row = secure_text_row("SKey", "可选共享密钥，会作为 x-skey 头发送。", "可选共享密钥", &s.skey_input, |value| {
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::SkeyChanged(value)))
    });

    let mut content = column![
        row![
            container(settings_page_intro(
                "客户端网关",
                "配置 Desktop 作为客户端访问 Gateway 时使用的地址与认证信息。",
            ))
            .width(Length::Fill),
            help_btn
        ]
        .align_y(Alignment::Start),
        settings_section_card(
            "桌面端连接目标",
            "这里配置 VibeWindow Desktop 要连接哪个 Gateway。它只影响客户端请求，不会修改服务端网关监听地址。",
        ),
        settings_panel(column![host_row, port_row].spacing(0)),
        settings_section_card(
            "认证",
            "可选填写 Bearer Token、Basic Auth 用户名/密码，以及额外的 SKey。Bearer Token 存在时优先发送 Authorization: Bearer。",
        ),
        settings_panel(column![bearer_row, username_row, password_row, skey_row].spacing(0)),
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
    let s = &app.gateway_client_settings;
    if !s.show_help_modal {
        return dialog;
    }

    let help_text = r#"客户端网关配置说明

一、作用
- 这里配置桌面端发请求时要连接的 Gateway 地址与认证信息。
- 这些设置只影响 Desktop 作为客户端访问网关，不会修改服务端网关的监听配置。

二、字段含义
1) host
- 目标 Gateway 主机地址，默认 127.0.0.1。

2) port
- 目标 Gateway 端口，默认 42617。

3) username
- Basic Auth 用户名。仅在同时提供密码时才会随请求发送。

4) bearer_token
- 已配对 Bearer Token。填写后会优先作为 Authorization: Bearer 发送。

5) password
- Basic Auth 密码。仅在 Bearer Token 为空时用于发送 Authorization 头。

6) skey
- 可选共享密钥。填写后会作为 x-skey 请求头发送给网关。

三、示例
{
  "gateway_client": {
    "host": "127.0.0.1",
    "port": 42617,
        "bearer_token": "",
    "username": "vibewindow",
    "password": "",
    "skey": ""
  }
}
"#;

    with_settings_help_modal(
        app,
        dialog,
        "客户端网关配置帮助",
        help_text,
        Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::HelpClose)),
    )
}
