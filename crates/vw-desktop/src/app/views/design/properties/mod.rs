//! # 属性面板模块
//!
//! 本模块实现了设计视图中元素属性编辑面板的渲染和交互逻辑。
//!
//! ## 主要功能
//!
//! - 渲染属性面板 UI，包括标题栏、折叠按钮、内容区域
//! - 提供元素属性的编辑界面，支持位置、布局、外观、填充、排版等多种属性
//! - 支持 Tailwind CSS 类的可视化管理
//! - 提供帮助信息的弹窗展示
//!
//! ## 子模块
//!
//! - `appearance`: 外观属性编辑（阴影、效果等）
//! - `color_picker`: 颜色选择器组件
//! - `content`: 内容编辑相关（标题、上下文、文本内容）
//! - `export`: 导出功能
//! - `fill`: 填充属性编辑
//! - `icon`: 图标属性编辑
//! - `layout`: 布局属性编辑
//! - `number_input`: 数字输入组件
//! - `position`: 位置属性编辑
//! - `tailwind`: Tailwind CSS 相关功能
//! - `typography`: 排版属性编辑
//! - `utils`: 通用工具函数

use std::collections::HashMap;

use iced::widget::{
    Space, button, column, container, mouse_area, row, scrollable,
    scrollable::{Direction, Scrollbar},
    stack, svg, text, text_editor, text_input,
};
use iced::{Color, Element, Length, Point, Theme};
use serde_json::Value;

use super::canvas::find_element_by_id;
use super::models::{DesignElement, VariableDef};
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::{App, Message};

pub mod appearance;
pub mod color_picker;
mod content;
mod export;
pub mod fill;
pub mod icon;
mod layout;
pub mod number_input;
mod position;
mod tailwind;
pub mod typography;
mod utils;

pub(crate) use number_input::NumberInput;

use utils::{horizontal_rule, prop_section, prop_text_editor_style};

/// 活动的 Tailwind 类选择器状态
///
/// 用于跟踪当前打开的 Tailwind 类选择器弹窗的状态，
/// 包括目标元素 ID 和弹窗显示位置。
#[derive(Debug, Clone)]
pub struct ActiveTailwindClassPicker {
    /// 目标元素的唯一标识符
    pub element_id: String,
    /// 弹窗显示的屏幕位置坐标
    pub position: Point,
}

/// 渲染属性面板
///
/// 根据应用状态渲染完整的属性面板 UI，包括：
/// - 面板标题和折叠/展开按钮
/// - 当前选中元素的属性编辑区域
/// - 帮助信息弹窗（如果存在）
///
/// # 参数
///
/// - `app`: 应用状态的引用，包含设计文档、选中元素等信息
///
/// # 返回值
///
/// 返回渲染后的属性面板 UI 元素
///
/// # 示例
///
/// ```ignore
/// let panel = render_properties(&app);
/// // panel 可以直接用于 iced 的 UI 树中
/// ```
pub fn render_properties(app: &App) -> Element<'_, Message> {
    /// 判断当前主题是否为深色主题
    ///
    /// 通过计算背景色的 RGB 值总和来判断，
    /// 总和小于 1.5 视为深色主题
    fn is_dark_theme(theme: &Theme) -> bool {
        let palette = theme.palette();
        palette.background.r + palette.background.g + palette.background.b < 1.5
    }

    if !app.show_properties_panel {
        return Space::new().width(0).into();
    }

    // 构建面板标题栏
    let header = row![
        text("属性面板")
            .size(13)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        Space::new().width(Length::Fill)
    ]
    .align_y(iced::Alignment::Center);

    // 根据当前状态渲染内容区域
    // 返回 (Tailwind 原始 HTML 编辑器, 主要属性内容)
    let (tailwind_raw_html, content): (Option<Element<'_, Message>>, Element<'_, Message>) =
        if let Some(state) = app.active_design_state() {
            if let Some(id) = &state.selected_element_id {
                let doc = &state.doc;
                if let Some(element) = find_element_by_id(&doc.children, id) {
                    let theme_mode = doc.theme.as_ref().map(|t| t.mode.as_str());
                    let available_themes =
                        doc.themes.as_ref().map(|t| t.mode.as_slice()).unwrap_or(&[]);
                    // 对于 Tailwind 元素且未选中子节点时，显示原始 HTML 编辑器
                    let raw_html = if element.kind.eq_ignore_ascii_case("tailwind")
                        && doc
                            .tailwind_selection
                            .as_ref()
                            .is_none_or(|(sel_id, _)| sel_id != &element.id)
                    {
                        Some(prop_section(
                            "Raw HTML",
                            text_editor(&state.tailwind_html_editor)
                                .placeholder("粘贴或编辑 HTML ...")
                                .on_action(|a| {
                                    Message::Design(DesignMessage::TailwindHtmlEditorAction(a))
                                })
                                .size(12)
                                .height(Length::Fixed(220.0))
                                .style(prop_text_editor_style)
                                .padding(8),
                        ))
                    } else {
                        None
                    };
                    (
                        raw_html,
                        render_element_properties(
                            element,
                            &doc.variables,
                            available_themes,
                            theme_mode,
                            &state.context_editor,
                            state.context_expanded,
                            &state.content_editor,
                            &state.tailwind_node_class_editor,
                            &state.tailwind_node_text_editor,
                            &state.tailwind_html_editor,
                            state.selected_fill_index,
                            state.selected_effect_index,
                            doc.tailwind_selection.as_ref(),
                            state.tailwind_class_input.as_str(),
                            state.tailwind_node_class_input.as_str(),
                            state.tailwind_node_class_dropdown_open,
                        ),
                    )
                } else {
                    (
                        None,
                        text("选择的元素没有找到")
                            .size(12)
                            .style(iced::widget::text::secondary)
                            .into(),
                    )
                }
            } else {
                (
                    None,
                    text("你可以选择一个元素，然后可以编辑其属性。")
                        .size(12)
                        .style(iced::widget::text::secondary)
                        .into(),
                )
            }
        } else {
            (None, text("没有打开的设计文档").size(12).style(iced::widget::text::secondary).into())
        };

    let tailwind_raw_html = tailwind_raw_html.unwrap_or_else(|| Space::new().height(0).into());

    // 构建主要内容容器
    let main_content: Element<'_, Message> = container(
        column![header]
            .spacing(12)
            .padding(iced::Padding { top: 12.0, right: 12.0, bottom: 12.0, left: 12.0 })
            .push(tailwind_raw_html)
            .push(
                scrollable(container(content).padding(iced::Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: 0.0,
                }))
                .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4))),
            ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    // 渲染帮助信息弹窗（如果存在）
    let overlay: Element<'_, Message> = if let Some(help) = &app.design_help_text {
        // 半透明背景遮罩
        let backdrop =
            mouse_area(container(Space::new()).width(Length::Fill).height(Length::Fill).style(
                |_theme: &Theme| container::Style {
                    background: Some(Color::from_rgba8(0, 0, 0, 0.35).into()),
                    ..Default::default()
                },
            ))
            .on_press(Message::Design(DesignMessage::CloseHelpModal));

        // 帮助对话框
        let dialog = container(
            column![
                row![
                    text("帮助").size(12).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::new().width(Length::Fill),
                    button(text("关闭").size(12))
                        .on_press(Message::Design(DesignMessage::CloseHelpModal))
                        .padding(4)
                        .style(button::secondary)
                ]
                .align_y(iced::Alignment::Center),
                text(help.clone()).size(12).width(Length::Fill),
            ]
            .spacing(10),
        )
        .padding([12, 12])
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.base.color.into()),
                border: iced::Border {
                    color: p.background.strong.color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.20),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }
        });

        container(stack![
            backdrop,
            container(dialog)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .into()
    } else {
        Space::new().into()
    };

    // 组合所有 UI 层：左侧分隔线、主要内容、帮助弹窗
    container(row![
        container(Space::new()).width(Length::Fixed(1.0)).height(Length::Fill).style(
            |theme: &Theme| {
                let is_dark = is_dark_theme(theme);
                let divider_color = if is_dark {
                    Color::from_rgb8(60, 60, 60)
                } else {
                    Color::from_rgb8(224, 224, 224)
                };
                container::Style { background: Some(divider_color.into()), ..Default::default() }
            }
        ),
        stack![main_content, overlay].width(Length::Fill).height(Length::Fill)
    ])
    .width(Length::Fixed(app.properties_panel_width))
    .height(Length::Fill)
    .padding(0)
    .style(|theme: &Theme| {
        let is_dark = is_dark_theme(theme);
        let palette = theme.extended_palette();
        let background = if is_dark { palette.background.base.color } else { Color::WHITE };
        container::Style {
            background: Some(background.into()),
            border: iced::Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    })
    .into()
}

/// 分割类名字符串为独立的 token 列表
///
/// 将空格分隔的类名字符串拆分为独立的类名列表，
/// 自动去除空白字符并去重。
///
/// # 参数
///
/// - `s`: 包含一个或多个类名的字符串，类名之间用空格分隔
///
/// # 返回值
///
/// 返回去重后的类名向量
///
/// # 示例
///
/// ```ignore
/// let tokens = split_class_tokens("flex items-center  justify-center flex");
/// // 返回: ["flex", "items-center", "justify-center"]
/// ```
pub(crate) fn split_class_tokens(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in s.split_whitespace() {
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        // 去重：只添加尚未存在的类名
        if !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    }
    out
}

/// 将 Tailwind 类名分组到语义类别
///
/// 根据类名前缀和模式，将 Tailwind CSS 类名归类到不同的功能组，
/// 便于在 UI 中进行分组展示。
///
/// # 参数
///
/// - `class_name`: Tailwind CSS 类名
///
/// # 返回值
///
/// 返回类名所属的分组名称（中文字符串）
///
/// # 分组规则
///
/// - **布局**: flex, grid, items-*, justify-*, block, inline, hidden, relative, absolute, fixed
/// - **间距**: p-*, m-*, gap-*, px-*, py-*, mx-*, my-*, pt-*, pb-*, pl-*, pr-*, mt-*, mb-*, ml-*, mr-*
/// - **边框**: rounded*, border*, divide*
/// - **尺寸**: w-*, h-*, max-w-*, w-full, h-full, w-screen, h-screen, w-auto, h-auto
/// - **排版**: font-*, tracking-*, leading-*, italic, underline, text-left 等
/// - **颜色**: bg-*, text-*
/// - **其它**: 不匹配以上规则的类名
pub(crate) fn group_tailwind_class(class_name: &str) -> &'static str {
    // 处理响应式前缀（如 md:flex, lg:grid），取最后一部分
    let token = class_name.split(':').next_back().unwrap_or(class_name);
    // 处理分数值类名（如 w-1/2），取基础部分
    let base = token.split('/').next().unwrap_or(token);

    // 布局相关类名
    if base.starts_with("flex")
        || base == "grid"
        || base.starts_with("grid-cols-")
        || base.starts_with("items-")
        || base.starts_with("justify-")
        || matches!(
            base,
            "block" | "inline-block" | "inline" | "hidden" | "relative" | "absolute" | "fixed"
        )
    {
        return "布局";
    }

    // 间距相关类名
    if base.starts_with("p-")
        || base.starts_with("m-")
        || base.starts_with("gap-")
        || base.starts_with("px-")
        || base.starts_with("py-")
        || base.starts_with("mx-")
        || base.starts_with("my-")
        || base.starts_with("pt-")
        || base.starts_with("pb-")
        || base.starts_with("pl-")
        || base.starts_with("pr-")
        || base.starts_with("mt-")
        || base.starts_with("mb-")
        || base.starts_with("ml-")
        || base.starts_with("mr-")
        || matches!(base, "mx-auto" | "my-auto")
    {
        return "间距";
    }

    // 边框相关类名
    if base.starts_with("rounded") || base.starts_with("border") || base.starts_with("divide") {
        return "边框";
    }

    // 尺寸相关类名
    if base.starts_with("w-")
        || base.starts_with("h-")
        || base.starts_with("max-w-")
        || matches!(base, "w-full" | "h-full" | "w-screen" | "h-screen" | "w-auto" | "h-auto")
    {
        return "尺寸";
    }

    // 排版相关类名
    if base.starts_with("font-")
        || base.starts_with("tracking-")
        || base.starts_with("leading-")
        || matches!(
            base,
            "italic"
                | "not-italic"
                | "underline"
                | "line-through"
                | "no-underline"
                | "uppercase"
                | "lowercase"
                | "capitalize"
        )
        || matches!(base, "text-left" | "text-center" | "text-right" | "text-justify")
        || matches!(
            base,
            "text-xs"
                | "text-sm"
                | "text-base"
                | "text-lg"
                | "text-xl"
                | "text-2xl"
                | "text-3xl"
                | "text-4xl"
        )
    {
        return "排版";
    }

    // 颜色相关类名
    if base.starts_with("bg-") || base.starts_with("text-") {
        return "颜色";
    }

    "其它"
}

/// 渲染 Tailwind 类编辑器
///
/// 为元素渲染 Tailwind CSS 类的可视化编辑界面，包括：
/// - 类名输入框
/// - 已有类名的标签展示（可删除）
/// - 添加新类的按钮
///
/// # 参数
///
/// - `element`: 当前选中的设计元素
/// - `class_input`: 当前输入框中的文本内容
///
/// # 返回值
///
/// 返回渲染后的 Tailwind 类编辑器 UI 元素
fn render_tailwind_class<'a>(
    element: &'a DesignElement,
    class_input: &'a str,
) -> Element<'a, Message> {
    let id = element.id.clone();
    let class_value = element.class.as_deref().unwrap_or("");
    let tokens = split_class_tokens(class_value);

    // 类名输入框
    let token_input = text_input("输入 class，回车创建（空格分割）", class_input)
        .on_input({
            let id = id.clone();
            move |s| Message::Design(DesignMessage::TailwindClassInputChanged(id.clone(), s))
        })
        .on_submit(Message::Design(DesignMessage::TailwindClassInputSubmit(id.clone())))
        .size(12)
        .padding([6, 8])
        .style(utils::prop_text_input_style);

    // 构建类名标签行
    let mut chips_row = row![].spacing(6);
    for token in &tokens {
        // 计算移除当前类名后的新值
        let new_value =
            tokens.iter().filter(|t| *t != token).cloned().collect::<Vec<_>>().join(" ");

        let remove_msg = Message::Design(DesignMessage::PropertyUpdate(
            id.clone(),
            "class".to_string(),
            if new_value.trim().is_empty() { Value::Null } else { Value::String(new_value) },
        ));

        // 类名移除按钮
        let remove_btn =
            button(svg(assets::get_icon(Icon::X)).width(8).height(8).style(|theme: &Theme, _| {
                iced::widget::svg::Style { color: Some(theme.palette().text.scale_alpha(0.85)) }
            }))
            .on_press(remove_msg)
            .style(|theme: &Theme, status| {
                let p = theme.extended_palette();
                button::Style {
                    background: if status == button::Status::Hovered {
                        Some(p.background.weak.color.into())
                    } else {
                        None
                    },
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 7.0.into(),
                    },
                    text_color: theme.palette().text,
                    ..button::Style::default()
                }
            })
            .padding(0)
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0));

        // 单个类名标签（chip）
        let chip = container(
            row![text(token.clone()).size(12), remove_btn]
                .spacing(6)
                .align_y(iced::Alignment::Center),
        )
        .padding([4, 8])
        .width(Length::Shrink)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            }
        });

        chips_row = chips_row.push(chip);
    }
    let chips = chips_row.wrap();

    // 类名标签展示区域
    let chips_area: Element<'a, Message> = if tokens.is_empty() {
        Space::new().height(Length::Fixed(0.0)).into()
    } else {
        container(scrollable(container(chips).width(Length::Fill)).height(Length::Fixed(120.0)))
            .style(iced::widget::container::bordered_box)
            .padding(6)
            .into()
    };

    // 添加新类按钮
    let open_picker_btn = button(text("添加新类").size(12))
        .on_press(Message::Design(DesignMessage::OpenTailwindClassPicker(id.clone(), None)))
        .style(|theme: &Theme, status: button::Status| {
            let p = theme.extended_palette();
            let bg = match status {
                button::Status::Hovered => p.background.weak.color,
                _ => p.background.base.color,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    color: p.background.strong.color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: iced::Shadow::default(),
                ..button::Style::default()
            }
        })
        .padding([8, 10])
        .width(Length::Fill);

    column![
        text("Tailwind 类").size(11).style(iced::widget::text::secondary),
        open_picker_btn,
        token_input,
        chips_area,
    ]
    .spacing(8)
    .into()
}

/// 渲染元素属性编辑面板
///
/// 根据元素类型和状态，渲染完整的属性编辑界面，包括：
/// - 节点标题和上下文（文本元素）
/// - 位置属性
/// - 布局属性
/// - Tailwind 类编辑
/// - 外观属性
/// - 填充属性
/// - 排版属性
/// - 效果属性
/// - Tailwind 特殊编辑（对于 Tailwind 元素）
/// - 导出选项
///
/// # 参数
///
/// - `element`: 当前选中的设计元素
/// - `variables`: 文档中定义的变量映射
/// - `available_themes`: 可用的主题列表
/// - `theme_mode`: 当前主题模式
/// - `context_editor`: 上下文编辑器内容
/// - `context_expanded`: 上下文区域是否展开
/// - `content_editor`: 内容编辑器内容
/// - `tailwind_node_class_editor`: Tailwind 节点类名编辑器内容
/// - `tailwind_node_text_editor`: Tailwind 节点文本编辑器内容
/// - `tailwind_html_editor`: Tailwind HTML 编辑器内容
/// - `selected_fill_index`: 当前选中的填充索引
/// - `selected_effect_index`: 当前选中的效果索引
/// - `tailwind_selection`: Tailwind 子节点选择状态
/// - `tailwind_class_input`: Tailwind 类名输入框内容
/// - `tailwind_node_class_input`: Tailwind 节点类名输入框内容
/// - `tailwind_node_class_dropdown_open`: Tailwind 节点类名下拉菜单是否打开
///
/// # 返回值
///
/// 返回渲染后的属性编辑面板 UI 元素
fn render_element_properties<'a>(
    element: &'a DesignElement,
    variables: &'a HashMap<String, VariableDef>,
    _available_themes: &[String],
    theme_mode: Option<&'a str>,
    context_editor: &'a iced::widget::text_editor::Content,
    context_expanded: bool,
    content_editor: &'a iced::widget::text_editor::Content,
    tailwind_node_class_editor: &'a iced::widget::text_editor::Content,
    tailwind_node_text_editor: &'a iced::widget::text_editor::Content,
    tailwind_html_editor: &'a iced::widget::text_editor::Content,
    selected_fill_index: Option<usize>,
    selected_effect_index: Option<usize>,
    tailwind_selection: Option<&'a (String, Vec<usize>)>,
    tailwind_class_input: &'a str,
    tailwind_node_class_input: &'a str,
    tailwind_node_class_dropdown_open: bool,
) -> Element<'a, Message> {
    let mut col = column![].spacing(12);

    let is_tailwind = element.kind.eq_ignore_ascii_case("tailwind");
    let is_sticky_note = element.kind.eq_ignore_ascii_case("sticky_note");
    let has_tailwind_sub_selection =
        is_tailwind && tailwind_selection.is_some_and(|(id, _)| id == &element.id);
    let is_text = element.kind.eq_ignore_ascii_case("text")
        || element.content.is_some()
        || element.context.is_some();

    // 对于文本元素且未选中 Tailwind 子节点时，渲染标题和上下文
    if is_text && !has_tailwind_sub_selection {
        col = col.push(content::render_node_title(element));
        if is_sticky_note {
            col = col.push(content::render_text_content(element, content_editor));
        } else {
            col = col.push(content::render_context(element, context_editor, context_expanded));
        }
        if !is_tailwind && !is_sticky_note {
            col = col.push(content::render_text_content(element, content_editor));
        }
        col = col.push(horizontal_rule(8));
    }

    // 渲染位置属性（除非选中了 Tailwind 子节点）
    if !has_tailwind_sub_selection {
        col = col.push(position::render(element));
        col = col.push(horizontal_rule(8));
    }

    // 对于非 Tailwind 元素，渲染完整的属性面板
    if !is_tailwind {
        col = col.push(layout::render(element, variables, theme_mode));
        col = col.push(horizontal_rule(8));
        col = col.push(render_tailwind_class(element, tailwind_class_input));
        col = col.push(horizontal_rule(8));
        col = col.push(appearance::render_appearance(element));
        col = col.push(horizontal_rule(8));
        col = col.push(fill::render(element, selected_fill_index));
        col = col.push(horizontal_rule(8));
        col = col.push(typography::render(element, variables, theme_mode));
        col = col.push(horizontal_rule(8));
        col = col.push(appearance::render_effects(element, selected_effect_index));
        col = col.push(horizontal_rule(8));
        if element.kind.eq_ignore_ascii_case("icon_font") {
            col = col.push(icon::render(element));
            col = col.push(horizontal_rule(8));
        }
    }

    // 对于 Tailwind 元素，渲染 Tailwind 专用编辑器
    if is_tailwind {
        col = col.push(tailwind::render(
            element,
            tailwind_selection,
            tailwind_node_class_editor,
            tailwind_node_text_editor,
            tailwind_html_editor,
            tailwind_node_class_input,
            tailwind_node_class_dropdown_open,
        ));
        col = col.push(horizontal_rule(8));
    }

    // 渲染导出选项（除非选中了 Tailwind 子节点）
    if !has_tailwind_sub_selection {
        col = col.push(export::render(element));
    }

    col.into()
}

#[cfg(test)]
mod tests;
