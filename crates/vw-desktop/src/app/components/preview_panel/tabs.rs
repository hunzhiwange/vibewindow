use crate::app::{Message, message};
use iced::widget::{Space, button, column, container, text};
use iced::{Background, Color, Element, Length, Theme, Vector};
use iced_code_editor::i18n::Language;

/// 构建预览标签页右键菜单。
pub fn build_preview_tab_menu<'a>(path: &str) -> Element<'a, Message> {
    let path_clone = path.to_string();
    let path_left = path.to_string();
    let path_right = path.to_string();

    let menu_btn = |label: String, msg: Message| -> Element<'a, Message> {
        button(container(text(label).size(13)).width(Length::Fill).padding([2, 8]))
            .on_press(msg)
            .style(|theme: &Theme, status| {
                let p = theme.extended_palette();
                let bg = match status {
                    iced::widget::button::Status::Hovered => p.background.weak.color,
                    iced::widget::button::Status::Pressed => p.background.strong.color,
                    _ => Color::TRANSPARENT,
                };
                iced::widget::button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: theme.palette().text,
                    border: iced::Border { radius: 4.0.into(), ..iced::Border::default() },
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .into()
    };

    let separator = || -> Element<'a, Message> {
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                let bg = theme.palette().background;
                let is_dark = bg.r + bg.g + bg.b < 1.5;
                let separator_color = if is_dark {
                    let text = theme.palette().text;
                    Color::from_rgba(text.r, text.g, text.b, 0.15)
                } else {
                    p.background.strong.color
                };
                container::Style { background: Some(separator_color.into()), ..Default::default() }
            })
            .into()
    };

    let content = column![
        menu_btn("关闭".to_string(), Message::Preview(message::PreviewMessage::Close(path_clone))),
        separator(),
        menu_btn(
            "关闭左侧".to_string(),
            Message::Preview(message::PreviewMessage::TabMenuCloseLeft(path_left))
        ),
        menu_btn(
            "关闭右侧".to_string(),
            Message::Preview(message::PreviewMessage::TabMenuCloseRight(path_right))
        ),
        separator(),
        menu_btn(
            "关闭所有".to_string(),
            Message::Preview(message::PreviewMessage::TabMenuCloseAll)
        ),
    ]
    .width(Length::Fixed(140.0))
    .padding(2)
    .spacing(1);

    container(content)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            let bg = theme.palette().background;
            let is_dark = bg.r + bg.g + bg.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark { p.background.weak.color } else { p.background.base.color }.into(),
                ),
                border: iced::Border {
                    color: if is_dark {
                        p.background.weak.color.scale_alpha(0.65)
                    } else {
                        p.background.strong.color
                    },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: if is_dark {
                        Color::BLACK.scale_alpha(0.30)
                    } else {
                        Color::BLACK.scale_alpha(0.10)
                    },
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            }
        })
        .into()
}

/// 编辑器界面语言包装类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DisplayLanguage(pub(super) Language);

/// 返回可选界面语言列表。
pub(super) fn all_languages() -> &'static [DisplayLanguage] {
    const LANGUAGES: [DisplayLanguage; 8] = [
        DisplayLanguage(Language::English),
        DisplayLanguage(Language::ChineseSimplified),
        DisplayLanguage(Language::French),
        DisplayLanguage(Language::Spanish),
        DisplayLanguage(Language::German),
        DisplayLanguage(Language::Italian),
        DisplayLanguage(Language::PortugueseBR),
        DisplayLanguage(Language::PortuguesePT),
    ];
    &LANGUAGES
}

impl std::fmt::Display for DisplayLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Language::English => write!(f, "English"),
            Language::ChineseSimplified => write!(f, "简体中文"),
            Language::French => write!(f, "Français"),
            Language::Spanish => write!(f, "Español"),
            Language::German => write!(f, "Deutsch"),
            Language::Italian => write!(f, "Italiano"),
            Language::PortugueseBR => write!(f, "Português (BR)"),
            Language::PortuguesePT => write!(f, "Português (PT)"),
        }
    }
}
