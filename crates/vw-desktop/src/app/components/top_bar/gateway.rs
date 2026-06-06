//! 顶栏网关服务弹窗。

use super::widgets::{color_with_alpha, icon_svg, menu_item_btn};
use crate::app::assets::Icon;
use crate::app::components::overlays::BelowOverlay;
use crate::app::components::system_settings::SystemTab;
use crate::app::message::settings::{GatewayClientMessage, SettingsMessage};
use crate::app::message::view::MenuType;
use crate::app::state::TopBarGatewayTab;
use crate::app::{App, Message, message};
use iced::widget::svg;
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{Space, button, column, container, row, text};
use iced::{Color, Element, Length, Theme};

fn status_dot(active: bool) -> Element<'static, Message> {
    container(Space::new().width(Length::Fixed(8.0)).height(Length::Fixed(8.0)))
        .style(move |theme: &Theme| {
            let color = if active {
                Color::from_rgb8(18, 190, 35)
            } else {
                theme.palette().text.scale_alpha(0.28)
            };
            iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border { width: 0.0, color: Color::TRANSPARENT, radius: 4.0.into() },
                ..Default::default()
            }
        })
        .into()
}

fn gateway_server_healthy(app: &App, server: &crate::app::state::GatewayClientServerDraft) -> bool {
    crate::app::message::gateway_health::server_health_key(server)
        .and_then(|key| app.gateway_client_settings.health.get(&key).copied())
        .unwrap_or(false)
}

fn active_gateway_healthy(app: &App) -> bool {
    app.gateway_client_settings
        .servers
        .iter()
        .find(|server| server.id == app.gateway_client_settings.selected_server_id)
        .is_some_and(|server| gateway_server_healthy(app, server))
}

fn gateway_services_button(active: bool, usable: bool) -> Element<'static, Message> {
    let gateway_icon = container(icon_svg(Icon::HddNetwork).style(|theme: &Theme, _status| {
        svg::Style { color: Some(theme.palette().text.scale_alpha(0.88)) }
    }))
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0))
    .align_x(iced::Alignment::Center)
    .align_y(iced::Alignment::Center);

    let btn = button(
        container(
            row![
                gateway_icon,
                container(status_dot(usable))
                    .width(Length::Fixed(10.0))
                    .height(Length::Fixed(16.0))
                    .align_y(iced::Alignment::Start),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
        )
        .height(Length::Fixed(24.0))
        .align_y(iced::Alignment::Center),
    )
    .height(Length::Fixed(24.0))
    .padding([0, 6])
    .on_press(Message::View(message::ViewMessage::ToggleMenu(Some(MenuType::GatewayServices))))
    .style(move |theme: &Theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => {
                Some(color_with_alpha(theme.palette().text, 0.14).into())
            }
            iced::widget::button::Status::Pressed => {
                Some(color_with_alpha(theme.palette().text, 0.22).into())
            }
            _ if active => Some(color_with_alpha(theme.palette().text, 0.10).into()),
            _ => None,
        };
        iced::widget::button::Style {
            background: bg,
            border: iced::Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    });

    let tip_content = container(text("网关服务")).padding([6, 10]).style(|theme: &Theme| {
        iced::widget::container::Style {
            text_color: Some(theme.palette().text),
            background: Some(iced::Background::Color(theme.palette().background)),
            border: iced::Border {
                width: 1.0,
                color: theme.extended_palette().background.strong.color,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }
    });

    Tooltip::new(btn, tip_content, TooltipPosition::Bottom).gap(10).into()
}

fn gateway_menu_container<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .padding(6)
        .width(Length::Fixed(460.0))
        .style(|theme: &Theme| iced::widget::container::Style {
            text_color: Some(theme.palette().text),
            background: Some(iced::Background::Color(color_with_alpha(
                theme.palette().background,
                0.96,
            ))),
            border: iced::Border {
                width: 1.0,
                color: theme.extended_palette().background.strong.color.scale_alpha(0.70),
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.18),
                offset: iced::Vector::new(0.0, 10.0),
                blur_radius: 26.0,
            },
            snap: false,
        })
        .into()
}

fn tab_label(
    label: &'static str,
    tab: TopBarGatewayTab,
    active: bool,
) -> Element<'static, Message> {
    let label_text =
        move |theme: &Theme, status: iced::widget::button::Status| iced::widget::text::Style {
            color: Some(if active {
                theme.palette().text
            } else if status == iced::widget::button::Status::Hovered {
                theme.palette().text.scale_alpha(0.82)
            } else {
                theme.palette().text.scale_alpha(0.62)
            }),
        };
    let underline = container(Space::new().width(Length::Fill).height(Length::Fixed(2.0))).style(
        move |theme: &Theme| iced::widget::container::Style {
            background: active.then_some(iced::Background::Color(theme.palette().text)),
            ..Default::default()
        },
    );

    button(column![text(label).size(13), underline].spacing(6).align_x(iced::Alignment::Center))
        .on_press(Message::View(message::ViewMessage::GatewayServicesTabSelected(tab)))
        .padding([4, 8])
        .style(move |theme: &Theme, status| iced::widget::button::Style {
            background: None,
            text_color: label_text(theme, status).color.unwrap_or(theme.palette().text),
            border: iced::Border { width: 0.0, color: Color::TRANSPARENT, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}

fn gateway_row<'a>(
    icon: Element<'a, Message>,
    label: String,
    mark: Option<&'static str>,
    msg: Option<Message>,
) -> Element<'a, Message> {
    let interactive = msg.is_some();
    let mark_el: Element<'a, Message> =
        mark.map(|value| text(value).size(12).into()).unwrap_or_else(|| Space::new().into());
    let content = row![
        container(icon)
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(18.0))
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fixed(4.0)),
        text(label).size(13),
        Space::new().width(Length::Fill),
        mark_el,
    ]
    .width(Length::Fill)
    .align_y(iced::alignment::Vertical::Center);

    let base = button(content)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let bg = if interactive && status == iced::widget::button::Status::Hovered {
                Some(palette.primary.base.color)
            } else {
                None
            };
            iced::widget::button::Style {
                background: bg.map(iced::Background::Color),
                text_color: if interactive && status == iced::widget::button::Status::Hovered {
                    Color::WHITE
                } else {
                    theme.palette().text
                },
                border: iced::Border { width: 0.0, color: Color::TRANSPARENT, radius: 5.0.into() },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .padding([6, 12]);

    if let Some(msg) = msg {
        base.on_press(Message::View(message::ViewMessage::MenuAction(Box::new(msg)))).into()
    } else {
        base.into()
    }
}

fn empty_tab_content<'a>() -> Vec<Element<'a, Message>> {
    vec![container(Space::new().height(Length::Fixed(72.0))).width(Length::Fill).into()]
}

fn gateway_tab_content(app: &App) -> Vec<Element<'_, Message>> {
    let mut rows: Vec<Element<'_, Message>> = Vec::new();
    for server in &app.gateway_client_settings.servers {
        let selected = server.id == app.gateway_client_settings.selected_server_id;
        let healthy = gateway_server_healthy(app, server);
        rows.push(gateway_row(
            status_dot(healthy),
            format!("{}  {}:{}", server.name, server.host, server.port),
            selected.then_some("✓"),
            Some(Message::Settings(SettingsMessage::GatewayClient(
                GatewayClientMessage::SelectServer(server.id.clone()),
            ))),
        ));
    }

    rows.push(menu_item_btn(
        "管理客户端网关",
        None,
        Some(Message::View(message::ViewMessage::OpenSystemSettingsTab(SystemTab::GatewayClient))),
    ));
    rows.push(menu_item_btn(
        "新增网关服务",
        None,
        Some(Message::Batch(vec![
            Message::Settings(SettingsMessage::GatewayClient(GatewayClientMessage::AddServer)),
            Message::View(message::ViewMessage::OpenSystemSettingsTab(SystemTab::GatewayClient)),
        ])),
    ));
    rows
}

#[cfg(not(target_arch = "wasm32"))]
fn lsp_tab_content(app: &App) -> Vec<Element<'_, Message>> {
    let mut server_keys = app
        .preview_tabs
        .iter()
        .filter_map(|tab| tab.lsp_server_key.map(str::to_string))
        .chain(app.lsp_progress.keys().cloned())
        .collect::<Vec<_>>();
    server_keys.sort();
    server_keys.dedup();

    let mut rows = Vec::new();
    if app.lsp_disabled {
        rows.push(gateway_row(status_dot(false), "LSP 已禁用".to_string(), None, None));
        return rows;
    }

    for server_key in server_keys {
        let progress = app.lsp_progress.get(&server_key);
        let working = progress.is_some_and(|entries| !entries.is_empty());
        let detail = progress
            .and_then(|entries| entries.values().next())
            .map(|progress| {
                let percent =
                    progress.percentage.map(|value| format!(" {value}%")).unwrap_or_default();
                let message = progress
                    .message
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| format!("  {value}"))
                    .unwrap_or_default();
                format!("LSP  {}{}{}", progress.title, percent, message)
            })
            .unwrap_or_else(|| format!("LSP  {server_key} 就绪"));
        rows.push(gateway_row(status_dot(true), detail, working.then_some("…"), None));
    }

    if rows.is_empty() {
        let status = app.lsp_status.clone().unwrap_or_else(|| "LSP 尚未连接".to_string());
        rows.push(gateway_row(status_dot(false), status, None, None));
    } else if let Some(status) = &app.lsp_status {
        rows.push(gateway_row(status_dot(true), status.clone(), None, None));
    }
    rows
}

#[cfg(target_arch = "wasm32")]
fn lsp_tab_content(app: &App) -> Vec<Element<'_, Message>> {
    let connected = active_gateway_healthy(app);
    let status = if connected {
        "LSP 由网关提供".to_string()
    } else {
        "LSP 等待网关连接".to_string()
    };
    vec![gateway_row(status_dot(connected), status, None, None)]
}

pub(super) fn gateway_services_module(app: &App) -> Element<'_, Message> {
    let active = app.active_menu == Some(MenuType::GatewayServices);
    let button = gateway_services_button(active, active_gateway_healthy(app));

    let tabs = row![
        tab_label(
            "网关",
            TopBarGatewayTab::Gateway,
            app.top_bar_gateway_tab == TopBarGatewayTab::Gateway
        ),
        tab_label("MCP", TopBarGatewayTab::Mcp, app.top_bar_gateway_tab == TopBarGatewayTab::Mcp),
        tab_label("LSP", TopBarGatewayTab::Lsp, app.top_bar_gateway_tab == TopBarGatewayTab::Lsp),
        tab_label(
            "插件",
            TopBarGatewayTab::Plugins,
            app.top_bar_gateway_tab == TopBarGatewayTab::Plugins
        ),
    ]
    .spacing(18)
    .padding([2, 8]);

    let mut rows: Vec<Element<'_, Message>> = vec![tabs.into()];
    rows.extend(match app.top_bar_gateway_tab {
        TopBarGatewayTab::Gateway => gateway_tab_content(app),
        TopBarGatewayTab::Mcp | TopBarGatewayTab::Plugins => empty_tab_content(),
        TopBarGatewayTab::Lsp => lsp_tab_content(app),
    });

    let content = container(column(rows).spacing(2)).width(Length::Fill);
    let menu = gateway_menu_container(content.into());

    BelowOverlay::new(button, menu)
        .show(active)
        .on_close(Message::View(message::ViewMessage::ToggleMenu(None)))
        .into()
}
