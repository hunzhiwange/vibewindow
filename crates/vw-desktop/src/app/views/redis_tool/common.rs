//! Redis 工具视图模块，负责连接列表、弹窗、状态徽标和表单控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    SETTINGS_LABEL_WIDTH, icon_svg, primary_action_btn_style, round_icon_btn_style,
    rounded_action_btn_style, settings_muted_text_style, settings_panel_style,
    settings_text_input_style,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::{RedisConnectionConfig, RedisConnectionDraft, RedisHistoryRecord};
use crate::app::{App, Message};
use chrono::{Local, TimeZone};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_status_badge<'a>(app: &'a App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Loading,
        Error,
        Success,
        Idle,
    }

    let (label, tone): (String, StatusTone) =
        if let Some(message) = &app.redis_tool.gateway_loading_label {
            (message.as_str().to_owned(), StatusTone::Loading)
        } else if let Some(error) = &app.redis_tool.gateway_error {
            (error.as_str().to_owned(), StatusTone::Error)
        } else if let Some(message) = &app.redis_tool.notification {
            (message.as_str().to_owned(), StatusTone::Success)
        } else {
            ("已就绪".to_string(), StatusTone::Idle)
        };

    container(text(label).size(12).style(move |theme: &Theme| iced::widget::text::Style {
        color: Some(match tone {
            StatusTone::Loading | StatusTone::Error | StatusTone::Success => Color::WHITE,
            StatusTone::Idle => theme.palette().text.scale_alpha(0.82),
        }),
    }))
    .padding([8, 12])
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(match tone {
                StatusTone::Loading => Color::from_rgba8(37, 99, 235, 0.92),
                StatusTone::Error => Color::from_rgba8(220, 38, 38, 0.92),
                StatusTone::Success => Color::from_rgba8(22, 163, 74, 0.92),
                StatusTone::Idle => palette.background.weak.color.scale_alpha(0.72),
            })),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color.scale_alpha(0.72),
                radius: 999.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `message`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_error_banner<'a>(message: &'a str) -> Element<'a, Message> {
    container(
        row![
            text(message).size(12).width(Length::Fill),
            button(text("关闭").size(12))
                .on_press(Message::RedisTool(RedisToolMessage::ClearGatewayError))
                .padding([8, 12])
                .style(rounded_action_btn_style),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([12, 14])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgba8(185, 28, 28, 0.16))),
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(220, 38, 38, 0.48),
                radius: 16.0.into(),
            },
            text_color: Some(palette.danger.base.color),
            ..Default::default()
        }
    })
    .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `icon`: 当前视图构建所需的状态、配置或消息。
/// - `message`: 当前视图构建所需的状态、配置或消息。
/// - `enabled`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_round_icon_action<'a>(
    icon: Icon,
    message: Message,
    enabled: bool,
) -> Element<'a, Message> {
    button(icon_svg(icon, 14.0))
        .on_press_maybe(enabled.then_some(message))
        .padding(10)
        .style(round_icon_btn_style)
        .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `message`: 当前视图构建所需的状态、配置或消息。
/// - `primary`: 当前视图构建所需的状态、配置或消息。
/// - `enabled`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_detail_action_button<'a>(
    label: &'a str,
    message: Message,
    primary: bool,
    enabled: bool,
) -> Element<'a, Message> {
    let action = button(text(label).size(13)).padding([10, 14]);
    if primary {
        action.on_press_maybe(enabled.then_some(message)).style(primary_action_btn_style).into()
    } else {
        action.on_press_maybe(enabled.then_some(message)).style(rounded_action_btn_style).into()
    }
}

/// 构建对应界面片段。
///
/// # 参数
/// - `placeholder`: 当前视图构建所需的状态、配置或消息。
/// - `value`: 当前视图构建所需的状态、配置或消息。
/// - `constructor`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    constructor: fn(String) -> RedisToolMessage,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(move |next| Message::RedisTool(constructor(next)))
        .padding([10, 12])
        .size(13)
        .width(Length::Fill)
        .style(settings_text_input_style)
        .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `placeholder`: 当前视图构建所需的状态、配置或消息。
/// - `value`: 当前视图构建所需的状态、配置或消息。
/// - `constructor`: 当前视图构建所需的状态、配置或消息。
/// - `pick_message`: 当前视图构建所需的状态、配置或消息。
/// - `enabled`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_path_picker_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    constructor: fn(String) -> RedisToolMessage,
    pick_message: Message,
    enabled: bool,
) -> Element<'a, Message> {
    row![
        container(build_input(placeholder, value, constructor)).width(Length::Fill),
        button(text("选择").size(12))
            .on_press_maybe(enabled.then_some(pick_message))
            .padding([10, 12])
            .style(rounded_action_btn_style),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// 构建 Redis 工具界面。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn redis_scroll_direction() -> Direction {
    Direction::Both {
        vertical: Scrollbar::new().width(4).scroller_width(4),
        horizontal: Scrollbar::new().width(4).scroller_width(4),
    }
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `description`: 当前视图构建所需的状态、配置或消息。
/// - `control`: 当前视图构建所需的状态、配置或消息。
/// - `compact`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn form_row<'a>(
    label: &'a str,
    description: &'a str,
    control: Element<'a, Message>,
    compact: bool,
) -> Element<'a, Message> {
    let meta =
        column![text(label).size(13), text(description).size(12).style(settings_muted_text_style),]
            .spacing(4)
            .width(if compact { Length::Fill } else { Length::Fixed(SETTINGS_LABEL_WIDTH) });

    if compact {
        column![meta, control].spacing(10).padding([12, 0]).into()
    } else {
        row![meta, container(control).width(Length::Fill)]
            .spacing(18)
            .align_y(Alignment::Center)
            .padding([12, 0])
            .into()
    }
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `label`: 当前视图构建所需的状态、配置或消息。
/// - `value`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn overview_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    row![
        text(label).size(12).style(settings_muted_text_style),
        Space::new().width(Length::Fill),
        text(value.to_string()).size(12),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// 构建侧栏界面。
///
/// # 参数
/// - `title`: 当前视图构建所需的状态、配置或消息。
/// - `description`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn empty_sidebar_hint<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    container(
        column![text(title).size(14), text(description).size(12).style(settings_muted_text_style),]
            .spacing(6),
    )
    .padding([20, 16])
    .width(Length::Fill)
    .style(settings_panel_style)
    .into()
}

/// 生成主题相关样式。
///
/// # 参数
/// - `theme`: 当前视图构建所需的状态、配置或消息。
/// - `selected`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回值遵循函数签名约定，调用方据此继续组装界面或更新状态。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn connection_item_style(
    theme: &Theme,
    selected: bool,
) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    if selected {
        iced::widget::container::Style {
            background: Some(Background::Color(palette.primary.base.color.scale_alpha(0.14))),
            border: Border {
                width: 1.0,
                color: palette.primary.base.color.scale_alpha(0.34),
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    } else {
        let mut style = settings_panel_style(theme);
        style.border.radius = 16.0.into();
        style
    }
}

/// 构建弹窗界面。
///
/// # 参数
/// - `content`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn modal_shell<'a>(
    content: Element<'a, Message>,
) -> iced::widget::Container<'a, Message> {
    container(content).padding([22, 24]).style(|theme: &Theme| {
        let mut style = settings_panel_style(theme);
        style.border.radius = 24.0.into();
        style.shadow = iced::Shadow {
            color: Color::BLACK.scale_alpha(0.22),
            offset: iced::Vector::new(0.0, 18.0),
            blur_radius: 34.0,
        };
        style
    })
}

/// 构建弹窗界面。
///
/// # 参数
/// - `title`: 当前视图构建所需的状态、配置或消息。
/// - `close_message`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn modal_header<'a>(title: &'a str, close_message: Message) -> Element<'a, Message> {
    row![
        text(title).size(20),
        Space::new().width(Length::Fill),
        button(icon_svg(Icon::X, 14.0))
            .on_press(close_message)
            .padding(8)
            .style(round_icon_btn_style),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// 构建 Redis 工具界面。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn history_table_header<'a>() -> Element<'a, Message> {
    history_table_row_cells(["Time", "Connection", "CMD", "Args", "Cost(ms)"], true)
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `record`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn history_table_row<'a>(record: &'a RedisHistoryRecord) -> Element<'a, Message> {
    history_table_row_cells(
        [
            format_timestamp(record.time_ms),
            record.connection_label.clone(),
            record.command.clone(),
            record.args.clone(),
            record.cost_ms.to_string(),
        ],
        false,
    )
}

fn history_table_row_cells<'a, T>(cells: [T; 5], header: bool) -> Element<'a, Message>
where
    T: ToString,
{
    let mut row_widget = row![].spacing(10).align_y(Alignment::Center);
    let widths = [110.0, 150.0, 120.0, 1.0, 90.0];

    for (index, cell) in cells.into_iter().enumerate() {
        let cell_text = text(cell.to_string()).size(if header { 12 } else { 11 }).style(
            move |theme: &Theme| iced::widget::text::Style {
                color: Some(if header {
                    theme.palette().text.scale_alpha(0.76)
                } else {
                    theme.palette().text.scale_alpha(0.9)
                }),
            },
        );

        let width = if index == 3 { Length::Fill } else { Length::Fixed(widths[index]) };
        row_widget = row_widget.push(container(cell_text).width(width));
    }

    container(row_widget)
        .padding([8, 12])
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(if header {
                    palette.background.weak.color.scale_alpha(0.72)
                } else {
                    palette.background.base.color.scale_alpha(0.36)
                })),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color.scale_alpha(0.18),
                    radius: 14.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn history_page_label(app: &App) -> String {
    if app.redis_tool.history_total == 0 {
        return "暂无历史记录".to_string();
    }

    let start = app.redis_tool.history_page_offset.saturating_add(1);
    let end = app
        .redis_tool
        .history_page_offset
        .saturating_add(app.redis_tool.history.len())
        .min(app.redis_tool.history_total);
    format!("第 {start}-{end} 条 / 共 {} 条", app.redis_tool.history_total)
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回值遵循函数签名约定，调用方据此继续组装界面或更新状态。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn current_load_count(app: &App) -> u32 {
    app.redis_tool
        .default_load_count_input
        .trim()
        .parse::<u32>()
        .ok()
        .unwrap_or(500)
        .clamp(1, 10_000)
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn masked_connection_preview(app: &App) -> String {
    let draft = &app.redis_tool.draft;
    let direct_uri = masked_direct_uri(draft);

    if draft.ssh_tunnel.enabled {
        let host = fallback_value(&draft.ssh_tunnel.host, "<ssh-host>");
        let port = fallback_value(&draft.ssh_tunnel.port, "22");
        let username = fallback_value(&draft.ssh_tunnel.username, "<user>");
        return format!("ssh://{username}@{host}:{port} -> {direct_uri}");
    }

    if draft.sentinel.enabled {
        let host = fallback_value(&draft.host, "<host>");
        let port = fallback_value(&draft.port, "6379");
        let master = fallback_value(&draft.sentinel.master_name, "<master>");
        return format!("sentinel://{host}:{port}?master={master}");
    }

    if draft.use_cluster {
        let host = fallback_value(&draft.host, "<host>");
        let port = fallback_value(&draft.port, "6379");
        let mode = if draft.read_only { "readonly" } else { "readwrite" };
        return format!("redis-cluster://{host}:{port}?mode={mode}");
    }

    direct_uri
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `draft`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn connection_mode_label(draft: &RedisConnectionDraft) -> String {
    let mut modes = Vec::new();
    if draft.ssh_tunnel.enabled {
        modes.push("SSH 隧道");
    }
    if draft.use_tls {
        modes.push("SSL/TLS");
    }
    if draft.sentinel.enabled {
        modes.push("Sentinel");
    }
    if draft.use_cluster {
        modes.push("Cluster");
    }
    if draft.read_only {
        modes.push("Readonly");
    }

    if modes.is_empty() { "直连".to_string() } else { modes.join(" / ") }
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `draft`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn enabled_feature_summary(draft: &RedisConnectionDraft) -> String {
    let mut features = Vec::new();
    if draft.use_tls {
        features.push("TLS");
    }
    if draft.tls_cert.has_custom_paths() {
        features.push("证书路径");
    }
    if draft.ssh_tunnel.enabled {
        features.push("SSH");
    }
    if draft.sentinel.enabled {
        features.push("Sentinel");
    }
    if draft.use_cluster {
        features.push("Cluster");
    }
    if draft.read_only {
        features.push("Readonly");
    }

    if features.is_empty() { "基础直连".to_string() } else { features.join(" / ") }
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `draft`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回匹配到的值；无法安全转换或当前状态不适用时返回 `None`。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn advanced_execution_note(draft: &RedisConnectionDraft) -> Option<String> {
    if draft.ssh_tunnel.enabled {
        return Some(
            "当前版本会保存 SSH 隧道配置，但测试连接、运行态读取、命令执行和复制 URI 暂不支持 SSH。"
                .to_string(),
        );
    }

    if draft.sentinel.enabled || draft.use_cluster {
        return Some(
            "当前版本已支持测试连接、运行态读取与命令执行，但复制 URI 仍仅支持直连 URI。"
                .to_string(),
        );
    }

    if draft.use_tls && draft.tls_cert.has_custom_paths() {
        return Some(
            "当前版本已支持自定义 SSL 证书测试、运行态读取与命令执行，但复制 URI 不会包含证书路径。"
                .to_string(),
        );
    }

    None
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `connection`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn connection_badge_labels(connection: &RedisConnectionConfig) -> Vec<&'static str> {
    let mut labels = Vec::new();
    if connection.use_tls {
        labels.push("TLS");
    }
    if connection.ssh_tunnel.enabled {
        labels.push("SSH");
    }
    if connection.sentinel.enabled {
        labels.push("Sentinel");
    }
    if connection.use_cluster {
        labels.push("Cluster");
    }
    if connection.read_only {
        labels.push("RO");
    }
    labels
}

/// 构建 Redis 工具界面。
///
/// # 参数
/// - `connection`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn connection_mode_summary(connection: &RedisConnectionConfig) -> String {
    let mut modes = Vec::new();
    if connection.ssh_tunnel.enabled {
        modes.push("SSH");
    }
    if connection.use_tls {
        modes.push("TLS");
    }
    if connection.sentinel.enabled {
        modes.push("Sentinel");
    }
    if connection.use_cluster {
        modes.push("Cluster");
    }
    if connection.read_only {
        modes.push("Readonly");
    }

    if modes.is_empty() { "基础直连".to_string() } else { modes.join(" / ") }
}

/// 格式化展示值。
///
/// # 参数
/// - `timestamp_ms`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回面向界面展示或后续消息处理的字符串。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn format_timestamp(timestamp_ms: u64) -> String {
    let Some(dt) = Local.timestamp_millis_opt(timestamp_ms as i64).single() else {
        return "--".to_string();
    };
    dt.format("%m-%d %H:%M:%S").to_string()
}

fn masked_direct_uri(draft: &RedisConnectionDraft) -> String {
    let scheme = if draft.use_tls { "rediss" } else { "redis" };
    let host = fallback_value(&draft.host, "<host>");
    let port = fallback_value(&draft.port, "6379");
    let db = fallback_value(&draft.db, "0");
    let auth = if draft.password.trim().is_empty() && draft.username.trim().is_empty() {
        String::new()
    } else if draft.username.trim().is_empty() {
        ":******@".to_string()
    } else if draft.password.trim().is_empty() {
        format!("{}@", draft.username.trim())
    } else {
        format!("{}:******@", draft.username.trim())
    };

    format!("{scheme}://{auth}{host}:{port}/{db}")
}

fn fallback_value<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() { fallback } else { value.trim() }
}

#[cfg(test)]
#[path = "common_tests.rs"]
mod common_tests;
