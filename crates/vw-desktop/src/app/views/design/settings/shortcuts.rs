//! 设计器设置视图模块，负责设置面板、快捷键说明与缩放控制的界面组合。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, scrollable, svg, text};
use iced::{Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::Message;

#[derive(Clone, Copy)]
struct ShortcutItem {
    keys: &'static [&'static str],
    description: &'static str,
}

#[derive(Clone, Copy)]
struct ShortcutSection {
    title: &'static str,
    items: &'static [ShortcutItem],
}

const AI_CHAT_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "K"], description: "切换 AI 聊天" },
    ShortcutItem { keys: &["⌘", "T"], description: "新建聊天标签" },
    ShortcutItem { keys: &["⌘", "W"], description: "关闭当前聊天标签" },
    ShortcutItem { keys: &["Ctrl", "⇥"], description: "切换到下一个聊天标签" },
    ShortcutItem { keys: &["Ctrl", "⇧", "⇥"], description: "切换到上一个聊天标签" },
    ShortcutItem { keys: &["↑", "↓"], description: "提示词历史" },
];

const TOOL_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["V"], description: "移动工具" },
    ShortcutItem { keys: &["H"], description: "抓手工具" },
    ShortcutItem { keys: &["R"], description: "矩形工具" },
    ShortcutItem { keys: &["O"], description: "椭圆工具" },
    ShortcutItem { keys: &["A / F"], description: "画板工具" },
    ShortcutItem { keys: &["T"], description: "文本工具" },
    ShortcutItem { keys: &["N"], description: "便签工具" },
];

const EDITING_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "C"], description: "复制所选内容到剪贴板" },
    ShortcutItem { keys: &["⌘", "V"], description: "粘贴剪贴板内容" },
    ShortcutItem { keys: &["⌘", "D"], description: "复制所选内容" },
    ShortcutItem { keys: &["⌘", "X"], description: "剪切所选内容" },
    ShortcutItem { keys: &["⌫"], description: "删除所选内容" },
    ShortcutItem {
        keys: &["Arrow keys"], description: "按 1px 移动所选内容（⇧ 为 10px）"
    },
    ShortcutItem { keys: &["Arrow keys"], description: "在弹性布局中调整元素顺序" },
    ShortcutItem { keys: &["⌘", "G"], description: "为所选元素编组" },
    ShortcutItem { keys: &["⌘", "⇧", "G"], description: "取消编组" },
    ShortcutItem { keys: &["⌘", "⌥", "G"], description: "为所选内容应用弹性布局" },
    ShortcutItem { keys: &["⌘", "["], description: "下移一层" },
    ShortcutItem { keys: &["⌘", "]"], description: "上移一层" },
    ShortcutItem { keys: &["["], description: "置于底层" },
    ShortcutItem { keys: &["]"], description: "置于顶层" },
    ShortcutItem { keys: &["⌘", "Z"], description: "撤销" },
    ShortcutItem { keys: &["⌘", "⇧", "Z"], description: "重做" },
];

const SELECTION_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "A"], description: "全选" },
    ShortcutItem { keys: &["⌘", "Click"], description: "直接选中" },
    ShortcutItem { keys: &["⇧", "Click"], description: "添加到选区" },
    ShortcutItem { keys: &["⇧", "↵"], description: "选中父级" },
    ShortcutItem { keys: &["↵"], description: "选中子级" },
    ShortcutItem { keys: &["Esc"], description: "清除选区" },
];

const COMPONENT_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "⌥", "K"], description: "将元素转换为组件" },
    ShortcutItem { keys: &["⌘", "⌥", "X"], description: "分离组件实例" },
];

const CANVAS_NAVIGATION_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "Scroll"], description: "缩放" },
    ShortcutItem { keys: &["Space", "Drag"], description: "平移画布" },
    ShortcutItem { keys: &["⇧", "Scroll"], description: "水平平移" },
    ShortcutItem { keys: &["="], description: "放大" },
    ShortcutItem { keys: &["-"], description: "缩小" },
    ShortcutItem { keys: &["0"], description: "缩放到 100%" },
    ShortcutItem { keys: &["1"], description: "缩放以适应画布" },
    ShortcutItem { keys: &["2"], description: "缩放到所选内容" },
];

const CANVAS_SETTINGS_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "'"], description: "切换像素网格" },
    ShortcutItem { keys: &["⌘", "⇧", "'"], description: "切换吸附到像素网格" },
];

const FILE_OPERATION_SHORTCUTS: &[ShortcutItem] = &[
    ShortcutItem { keys: &["⌘", "S"], description: "保存文件" },
    ShortcutItem { keys: &["⌘", "N"], description: "新建文件" },
    ShortcutItem { keys: &["⌘", "O"], description: "打开文件" },
];

const SHORTCUT_COLUMN_LEFT: &[ShortcutSection] = &[
    ShortcutSection { title: "AI 聊天", items: AI_CHAT_SHORTCUTS },
    ShortcutSection { title: "工具", items: TOOL_SHORTCUTS },
    ShortcutSection { title: "编辑", items: EDITING_SHORTCUTS },
];

const SHORTCUT_COLUMN_RIGHT: &[ShortcutSection] = &[
    ShortcutSection { title: "选择", items: SELECTION_SHORTCUTS },
    ShortcutSection { title: "组件", items: COMPONENT_SHORTCUTS },
    ShortcutSection { title: "画布导航", items: CANVAS_NAVIGATION_SHORTCUTS },
    ShortcutSection { title: "画布设置", items: CANVAS_SETTINGS_SHORTCUTS },
    ShortcutSection { title: "文件操作", items: FILE_OPERATION_SHORTCUTS },
];

fn keycap_bg(is_dark: bool) -> Color {
    if is_dark { Color::from_rgba8(56, 59, 64, 1.0) } else { Color::from_rgba8(244, 244, 245, 1.0) }
}

fn keycap_border(is_dark: bool) -> Color {
    if is_dark { Color::from_rgba8(255, 255, 255, 0.10) } else { Color::from_rgba8(0, 0, 0, 0.08) }
}

fn shortcut_keycap<'a>(label: &'static str) -> iced::widget::Container<'a, Message, Theme> {
    container(
        text(label)
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
    )
    .padding([5, 9])
    .style(move |theme: &Theme| {
        let palette = theme.palette();
        let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
        container::Style {
            background: Some(keycap_bg(is_dark).into()),
            text_color: Some(if is_dark {
                Color::from_rgba8(244, 245, 247, 1.0)
            } else {
                Color::from_rgba8(58, 58, 61, 1.0)
            }),
            border: Border { width: 1.0, color: keycap_border(is_dark), radius: 6.0.into() },
            ..Default::default()
        }
    })
}

fn shortcut_row(item: &ShortcutItem) -> Element<'static, Message> {
    let mut keys_row = row![].spacing(6).align_y(iced::Alignment::Center);
    for key in item.keys {
        keys_row = keys_row.push(shortcut_keycap(key));
    }

    row![
        container(keys_row).width(Length::Fixed(144.0)),
        text(item.description).size(15).line_height(1.45).style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            iced::widget::text::Style {
                color: Some(if is_dark {
                    Color::from_rgba8(215, 218, 223, 1.0)
                } else {
                    Color::from_rgba8(111, 114, 120, 1.0)
                }),
            }
        })
    ]
    .spacing(18)
    .align_y(iced::Alignment::Start)
    .into()
}

fn shortcut_section(section: &ShortcutSection) -> Element<'static, Message> {
    let mut items = column![].spacing(12);
    for item in section.items {
        items = items.push(shortcut_row(item));
    }

    column![
        text(section.title)
            .size(16)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        items
    ]
    .spacing(14)
    .into()
}

fn shortcut_column(sections: &'static [ShortcutSection]) -> Element<'static, Message> {
    let mut col = column![].spacing(24).width(Length::Fill);
    for section in sections {
        col = col.push(shortcut_section(section));
    }
    col.into()
}

/// 渲染对应界面。
///
/// # 参数
/// - `show`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_shortcuts_panel(show: bool) -> Element<'static, Message> {
    if !show {
        return Space::new().into();
    }

    let header = row![
        text("快捷键")
            .size(18)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        Space::new().width(Length::Fill),
        button(svg(assets::get_icon(Icon::X)).width(14).height(14).style(
            |theme: &Theme, _status| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                svg::Style {
                    color: Some(if is_dark {
                        Color::from_rgba8(204, 207, 212, 0.88)
                    } else {
                        Color::from_rgba8(104, 107, 114, 0.90)
                    }),
                }
            }
        ))
        .on_press(Message::Design(DesignMessage::ToggleShortcuts))
        .style(move |_theme: &Theme, status| button::Style {
            background: match status {
                button::Status::Hovered => Some(Color::from_rgba8(127, 127, 132, 0.12).into()),
                button::Status::Pressed => Some(Color::from_rgba8(127, 127, 132, 0.18).into()),
                _ => None,
            },
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
            ..Default::default()
        })
        .padding(6)
    ]
    .align_y(iced::Alignment::Center);

    let body = scrollable(
        container(
            row![
                container(shortcut_column(SHORTCUT_COLUMN_LEFT)).width(Length::FillPortion(1)),
                container(shortcut_column(SHORTCUT_COLUMN_RIGHT)).width(Length::FillPortion(1)),
            ]
            .spacing(36)
            .width(Length::Fill),
        )
        .padding(iced::Padding { top: 0.0, right: 18.0, bottom: 0.0, left: 0.0 }),
    )
    .direction(Direction::Vertical(Scrollbar::new().width(6).scroller_width(6)))
    .height(Length::Fixed(760.0));

    let modal = container(column![header, body].spacing(20))
        .padding(iced::Padding { top: 20.0, right: 14.0, bottom: 20.0, left: 20.0 })
        .width(Length::Fixed(720.0))
        .height(Length::Fixed(860.0))
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark {
                        Color::from_rgba8(36, 38, 42, 0.98)
                    } else {
                        Color::from_rgba8(247, 247, 248, 0.995)
                    }
                    .into(),
                ),
                text_color: Some(if is_dark {
                    Color::from_rgba8(247, 247, 248, 1.0)
                } else {
                    Color::from_rgba8(28, 29, 31, 1.0)
                }),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.10)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.08)
                    },
                    radius: 14.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(if is_dark { 0.32 } else { 0.16 }),
                    offset: iced::Vector::new(0.0, 18.0),
                    blur_radius: 48.0,
                },
                ..Default::default()
            }
        });

    container(container(modal).center_x(Length::Fill).center_y(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            container::Style {
                background: Some(
                    if is_dark {
                        Color::from_rgba8(0, 0, 0, 0.36)
                    } else {
                        Color::from_rgba8(30, 30, 34, 0.18)
                    }
                    .into(),
                ),
                ..Default::default()
            }
        })
        .into()
}

#[cfg(test)]
#[path = "shortcuts_tests.rs"]
mod shortcuts_tests;
