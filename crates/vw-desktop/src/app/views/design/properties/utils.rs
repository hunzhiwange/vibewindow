//! 属性面板工具模块
//!
//! 本模块提供属性面板的通用工具函数和样式定义，用于构建统一风格的属性编辑界面。
//! 包括输入控件样式、布局组件、帮助提示图标等 UI 工具函数。
//!
//! ## 主要功能
//!
//! - **样式定义**: 为文本输入框、文本编辑器等控件提供统一样式
//! - **布局组件**: 创建属性区块、分割线等布局元素
//! - **帮助提示**: 提供悬浮提示和模态框提示两种帮助形式
//! - **数据转换**: JSON 值与字符串之间的转换工具

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use iced::widget::{
    Space, button, column, container, row, svg, text, text_editor, text_input, tooltip,
};
use iced::{Background, Color, Element, Length, Theme};

/// 属性输入框的圆角半径
///
/// 用于文本输入框、文本编辑器等输入控件的边框圆角，
/// 确保整个属性面板的视觉一致性。
pub const PROP_INPUT_RADIUS: f32 = 8.0;

/// 生成属性面板文本输入框样式
///
/// 根据当前主题和输入框状态（聚焦/非聚焦）生成统一样式。
/// 聚焦状态下会高亮边框颜色，使用较弱背景色。
///
/// # 参数
///
/// * `theme` - 当前 Iced 主题
/// * `status` - 文本输入框的状态（聚焦、悬停、激活等）
///
/// # 返回值
///
/// 返回配置好的 `text_input::Style` 实例
///
/// # 示例
///
/// ```ignore
/// let input = text_input("placeholder", &value)
///     .style(prop_text_input_style);
/// ```
pub fn prop_text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    let extended = theme.extended_palette();
    let focused = matches!(status, text_input::Status::Focused { .. });
    let border_color = if focused { palette.primary } else { extended.background.strong.color };
    let bg = if focused { extended.background.weak.color } else { extended.background.base.color };
    text_input::Style {
        background: Background::Color(bg),
        border: iced::Border { width: 1.0, color: border_color, radius: PROP_INPUT_RADIUS.into() },
        icon: palette.text.scale_alpha(0.5),
        placeholder: palette.text.scale_alpha(0.55),
        value: palette.text,
        selection: palette.primary.scale_alpha(0.30),
    }
}

/// 生成属性面板文本编辑器样式
///
/// 为多行文本编辑器提供统一样式，与文本输入框保持视觉一致性。
///
/// # 参数
///
/// * `theme` - 当前 Iced 主题
/// * `_status` - 文本编辑器状态（当前未使用，保留以备扩展）
///
/// # 返回值
///
/// 返回配置好的 `text_editor::Style` 实例
///
/// # 示例
///
/// ```ignore
/// let editor = text_editor(&content)
///     .style(prop_text_editor_style);
/// ```
pub fn prop_text_editor_style(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let palette = theme.palette();
    let extended = theme.extended_palette();
    text_editor::Style {
        background: Background::Color(extended.background.base.color),
        border: iced::Border {
            width: 1.0,
            color: extended.background.strong.color,
            radius: PROP_INPUT_RADIUS.into(),
        },
        value: palette.text,
        selection: palette.primary.scale_alpha(0.30),
        placeholder: palette.text.scale_alpha(0.55),
    }
}

/// 创建属性区块
///
/// 构建一个包含标签和输入控件的标准属性区块布局。
/// 标签显示在上方，输入控件显示在下方，两者之间有 8px 间距。
///
/// # 参数
///
/// * `label` - 属性标签文本（静态字符串）
/// * `input` - 输入控件元素，任何可转换为 `Element<Message>` 的组件
///
/// # 返回值
///
/// 返回包含标签和输入的垂直布局元素
///
/// # 示例
///
/// ```ignore
/// let input = text_input("请输入...", &value);
/// let section = prop_section("名称", input);
/// ```
pub fn prop_section<'a>(
    label: &'static str,
    input: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(label)
            .size(11)
            .line_height(iced::widget::text::LineHeight::Relative(1.2))
            .style(text::secondary),
        input.into()
    ]
    .spacing(8)
    .into()
}

/// 创建带悬浮提示的帮助图标
///
/// 创建一个问号圆形图标，鼠标悬停时显示提示信息。
/// 图标使用 SVG 矢量图形，支持主题颜色适配。
///
/// # 参数
///
/// * `tip` - 提示文本内容
///
/// # 返回值
///
/// 返回一个包含按钮和悬浮提示的元素
///
/// # 示例
///
/// ```ignore
/// let help = help_icon("这是帮助提示内容");
/// ```
pub fn help_icon<'a>(tip: &'a str) -> Element<'a, Message> {
    // 创建问号圆形 SVG 图标，尺寸为 14x14 像素
    let icon = svg(assets::get_icon(Icon::QuestionCircle))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|theme: &Theme, _| iced::widget::svg::Style {
            color: Some(theme.palette().text.scale_alpha(0.55)),
        });

    // 创建按钮容器，尺寸为 16x16 像素，居中显示图标
    let btn = button(
        container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .style(button::text)
    .padding(0)
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0));

    // 创建提示框内容，深色背景，带阴影效果
    let tip_content =
        container(text(tip).size(12)).padding([6, 8]).style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::from_rgba8(24, 24, 24, 0.96).into()),
            text_color: Some(iced::Color::WHITE),
            border: iced::Border { color: Color::TRANSPARENT, width: 0.0, radius: 8.0.into() },
            shadow: iced::Shadow {
                color: iced::Color::BLACK.scale_alpha(0.30),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        });

    // 将按钮和提示框组合成 Tooltip 组件
    tooltip::Tooltip::new(btn, tip_content, tooltip::Position::Top).gap(2.0).into()
}

/// 创建带模态框的帮助图标
///
/// 创建一个问号圆形图标，点击后弹出模态框显示帮助信息。
/// 与 `help_icon` 不同，此函数用于显示较长的帮助内容。
///
/// # 参数
///
/// * `tip` - 帮助文本内容，将在模态框中显示
///
/// # 返回值
///
/// 返回一个可点击的帮助按钮元素，点击时触发 `ShowHelpModal` 消息
///
/// # 示例
///
/// ```ignore
/// let help = help_icon_modal("详细的帮助说明...");
/// ```
#[allow(dead_code)]
pub fn help_icon_modal<'a>(tip: &'a str) -> Element<'a, Message> {
    // 创建问号文本容器，带背景色和圆角边框
    let content = container(text("?").size(11).style(text::secondary))
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.weak.color.into()),
                border: iced::Border {
                    color: p.background.strong.color,
                    width: 1.0,
                    radius: 9.0.into(),
                },
                ..Default::default()
            }
        });

    // 创建按钮，点击时显示帮助模态框
    button(content)
        .on_press(Message::Design(DesignMessage::ShowHelpModal(tip.to_string())))
        .style(button::text)
        .padding(0)
        .into()
}

/// 创建带悬浮提示帮助的属性区块
///
/// 构建一个包含标签、帮助图标（悬浮提示）和输入控件的属性区块。
/// 适用于需要简短说明的属性字段。
///
/// # 参数
///
/// * `label` - 属性标签文本
/// * `help` - 帮助提示文本，鼠标悬停在图标上时显示
/// * `input` - 输入控件元素
///
/// # 返回值
///
/// 返回包含标签、帮助图标和输入控件的垂直布局元素
///
/// # 示例
///
/// ```ignore
/// let input = text_input("请输入...", &value);
/// let section = prop_section_with_help("名称", "请输入组件名称", input);
/// ```
pub fn prop_section_with_help<'a>(
    label: &'static str,
    help: &'static str,
    input: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(11)
                .line_height(iced::widget::text::LineHeight::Relative(1.2))
                .style(text::secondary),
            Space::new().width(Length::Shrink),
            help_icon(help)
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        input.into()
    ]
    .spacing(8)
    .into()
}

/// 创建带模态框帮助的属性区块
///
/// 构建一个包含标签、帮助图标（点击显示模态框）和输入控件的属性区块。
/// 适用于需要详细说明的属性字段。
///
/// # 参数
///
/// * `label` - 属性标签文本
/// * `help` - 帮助文本内容，点击图标后在模态框中显示
/// * `input` - 输入控件元素
///
/// # 返回值
///
/// 返回包含标签、帮助图标和输入控件的垂直布局元素
///
/// # 示例
///
/// ```ignore
/// let input = text_input("请输入...", &value);
/// let section = prop_section_with_help_modal("配置", "详细配置说明...", input);
/// ```
#[allow(dead_code)]
pub fn prop_section_with_help_modal<'a>(
    label: &'static str,
    help: &'static str,
    input: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        row![
            text(label)
                .size(11)
                .line_height(iced::widget::text::LineHeight::Relative(1.2))
                .style(text::secondary),
            Space::new().width(Length::Shrink),
            help_icon_modal(help)
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        input.into()
    ]
    .spacing(8)
    .into()
}

/// 创建标签输入行
///
/// 创建一个水平布局，左侧为标签，右侧为文本输入框。
/// 适用于简短的标签-输入组合，如键值对编辑。
///
/// # 参数
///
/// * `label` - 输入框标签（宽度固定为 15）
/// * `value` - 输入框当前值
/// * `on_change` - 输入变化时的回调函数，接收新字符串并返回消息
///
/// # 返回值
///
/// 返回包含标签和输入框的水平布局元素
///
/// # 示例
///
/// ```ignore
/// let input = prop_label_input("键", &key, |s| Message::UpdateKey(s));
/// ```
#[allow(dead_code)]
pub fn prop_label_input<'a>(
    label: &'static str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label)
            .size(11)
            .width(15)
            .line_height(iced::widget::text::LineHeight::Relative(1.2))
            .style(text::secondary)
            .align_y(iced::alignment::Vertical::Center),
        text_input("", value).on_input(on_change).style(prop_text_input_style).padding(6).size(12)
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

/// 创建水平分割线
///
/// 创建一条水平分割线，可以指定总高度。
/// 当高度大于 1 时，分割线会垂直居中显示。
///
/// # 参数
///
/// * `height` - 分割线区域的总高度（像素）
///
/// # 返回值
///
/// 返回包含分割线的元素：
/// - 当 height <= 1 时，返回单条分割线
/// - 当 height > 1 时，返回带有上下留白的居中分割线
///
/// # 示例
///
/// ```ignore
/// // 创建一条简单的分割线
/// let line = horizontal_rule(1);
///
/// // 创建一条高度为 20 的居中分割线
/// let line = horizontal_rule(20);
/// ```
pub fn horizontal_rule(height: u16) -> Element<'static, Message> {
    // 创建分割线本身（高度为 1）
    let line = container(Space::new()).width(Length::Fill).height(1.0).style(|theme: &Theme| {
        let palette = theme.palette();
        container::Style {
            background: Some(Color { a: 0.10, ..palette.text }.into()),
            ..Default::default()
        }
    });

    // 如果请求高度 <= 1，直接返回单条线
    if height <= 1 {
        return line.height(height as f32).into();
    }

    // 计算分割线上下的留白高度，使分割线垂直居中
    let top = (height - 1) / 2;
    let bottom = height - 1 - top;

    // 构建带有上下留白的分割线布局
    column![
        Space::new().height(Length::Fixed(top as f32)),
        line,
        Space::new().height(Length::Fixed(bottom as f32))
    ]
    .into()
}

/// 将 JSON 值转换为字符串
///
/// 处理可选的 JSON 值，将其转换为字符串格式。
/// 对于字符串类型的 JSON 值，直接返回其内容；
/// 对于其他类型，序列化为 JSON 字符串；
/// 对于 None 值，返回空字符串。
///
/// # 参数
///
/// * `v` - 可选的 JSON 值引用
///
/// # 返回值
///
/// 返回转换后的字符串：
/// - `Some(String)` -> 返回字符串内容
/// - `Some(其他类型)` -> 返回 JSON 序列化字符串
/// - `None` -> 返回空字符串
///
/// # 示例
///
/// ```ignore
/// let val = Some(serde_json::json!("hello"));
/// assert_eq!(json_value_to_string(&val), "hello");
///
/// let val = Some(serde_json::json!({"key": "value"}));
/// assert_eq!(json_value_to_string(&val), r#"{"key":"value"}"#);
///
/// let val: Option<serde_json::Value> = None;
/// assert_eq!(json_value_to_string(&val), "");
/// ```
#[allow(dead_code)]
pub fn json_value_to_string(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(val) => serde_json::to_string(val).unwrap_or_default(),
        None => String::new(),
    }
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod utils_tests;
