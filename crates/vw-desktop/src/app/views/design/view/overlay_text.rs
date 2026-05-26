//! 设计器视图浮层模块，负责画布和上下文选择器等叠加界面的渲染。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::font::{Font as IcedFont, Weight as IcedWeight};
use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Element, Length};

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::layout::parse_padding;
use crate::app::views::design::canvas::parse::{parse_color, parse_fill};
use crate::app::views::design::canvas::{find_element_by_id, get_element_screen_bounds, parse_font_size};
use crate::app::views::design::state::DesignState;
use crate::app::views::design::utils::transparent_editor_style;
use crate::app::{App, Message};

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn inline_text_editor_overlay<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let mut overlay: Element<'a, Message> =
        container(Space::new()).width(Length::Fill).height(Length::Fill).into();

    if let Some(edit_id) = &state.editing_id
        && let Some(rect) = get_element_screen_bounds(&state.doc, edit_id, state.pan, state.zoom)
        && let Some(el) = find_element_by_id(&state.doc.children, edit_id)
    {
        let theme_mode = state.doc.theme.as_ref().map(|t| t.mode.as_str());
        let font_size =
            parse_font_size(&el.font_size, &state.doc.variables, theme_mode) * state.zoom;
        let text_color = el
            .color
            .as_ref()
            .map(|s| parse_color(s, &state.doc.variables, theme_mode))
            .unwrap_or_else(|| parse_fill(&el.fill, &state.doc.variables, theme_mode));
        let padding = parse_padding(&el.padding, &state.doc.variables, theme_mode);

        let content_x = rect.x + padding.left * state.zoom;
        let content_y = rect.y + padding.top * state.zoom;
        let content_w = (rect.width - (padding.left + padding.right) * state.zoom).max(10.0);
        let content_h =
            (rect.height - (padding.top + padding.bottom) * state.zoom).max(font_size + 10.0);

        let weight = match el
            .font_weight
            .as_ref()
            .and_then(|v: &serde_json::Value| v.as_str())
        {
            Some("300") => IcedWeight::Light,
            Some("400") | None => IcedWeight::Normal,
            Some("500") => IcedWeight::Medium,
            Some("600") => IcedWeight::Semibold,
            Some("700") => IcedWeight::Bold,
            Some("800") => IcedWeight::ExtraBold,
            Some(_) => IcedWeight::Normal,
        };
        let font = IcedFont { weight, ..Default::default() };

        let editor = text_editor(&state.editing_editor)
            .on_action(|a| Message::Design(DesignMessage::EditEditorAction(a)))
            .size(font_size)
            .width(content_w)
            .height(content_h)
            .font(font)
            .style(move |_theme, _status| transparent_editor_style(text_color));

        overlay = container(editor)
            .padding(0)
            .padding(iced::Padding { top: content_y, right: 0.0, bottom: 0.0, left: content_x })
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    overlay
}

/// 执行本模块的界面辅助逻辑。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回按当前状态生成的列表，供调用方继续渲染或选择。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn html_preview_layers<'a>(app: &'a App) -> Vec<Element<'a, Message>> {
    if !app.show_element_html_preview {
        return vec![];
    }

    let content = container(
        column(vec![
            row(vec![
                text("HTML 预览")
                    .size(16)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .into(),
                Space::new().width(Length::Fill).into(),
                button("关闭")
                    .on_press(Message::Design(DesignMessage::CloseHtmlPreview))
                    .style(button::secondary)
                    .padding([4, 10])
                    .into(),
            ])
            .align_y(iced::Alignment::Center)
            .spacing(10)
            .into(),
            container(
                text_editor(&app.element_html_preview_editor)
                    .on_action(|a| Message::Design(DesignMessage::HtmlPreviewAction(a)))
                    .font(iced::Font::with_name("JetBrains Mono")),
            )
            .style(container::rounded_box)
            .padding(5)
            .height(Length::Fill)
            .into(),
        ])
        .spacing(10),
    )
    .padding(20)
    .width(Length::Fixed(800.0))
    .height(Length::Fixed(600.0))
    .style(container::rounded_box);

    vec![
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(iced::Color { a: 0.5, ..palette.background }.into()),
                    ..Default::default()
                }
            })
            .into(),
    ]
}
#[cfg(test)]
#[path = "overlay_text_tests.rs"]
mod overlay_text_tests;
