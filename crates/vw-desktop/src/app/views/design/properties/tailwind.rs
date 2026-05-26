//! Tailwind CSS 元素属性面板渲染模块
//!
//! 本模块提供 Tailwind 类型设计元素的属性编辑界面，包括：
//! - HTML 转换为图层的功能按钮
//! - DOM 节点选择器和属性编辑器
//! - Tailwind CSS 类名搜索和快速选择
//! - 节点文本内容编辑
//!
//! 主要用于设计视图中选中 Tailwind 元素时的属性面板渲染。

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::BelowOverlay;
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::tailwind::dom::{self, TailwindNode};
use crate::app::views::design::canvas::tailwind::get_tailwind_classes;
use crate::app::views::design::models::DesignElement;
use crate::app::views::design::properties::group_tailwind_class;
use crate::app::views::design::properties::utils::{prop_section, prop_text_editor_style};
use iced::widget::{
    Space, button, column, container, row, scrollable,
    scrollable::{Direction, Scrollbar},
    svg, text, text_editor, text_input,
};
use iced::{Background, Border, Color, Element, Length, Theme};

/// 渲染 Tailwind 元素的属性面板
///
/// 该函数为选中的 Tailwind 类型设计元素生成属性编辑界面，包括：
/// - 转换按钮：将 HTML 结构转换为独立图层
/// - 节点编辑器：编辑选中 DOM 节点的属性和内容
/// - 类名管理：添加、删除和搜索 Tailwind CSS 类名
/// - 文本编辑：修改节点的文本内容
///
/// # 参数
///
/// * `element` - 当前设计元素的引用，包含元素类型和内容
/// * `selection` - 可选的选中信息，包含元素 ID 和节点路径
/// * `tailwind_node_class_editor` - 类名编辑器的文本内容
/// * `tailwind_node_text_editor` - 文本内容编辑器的文本内容
/// * `_tailwind_html_editor` - HTML 编辑器内容（当前未使用）
/// * `tailwind_node_class_input` - 类名输入框的当前值
/// * `tailwind_node_class_dropdown_open` - 类名下拉列表是否展开
///
/// # 返回值
///
/// 返回一个 iced Element，包含属性面板的所有 UI 组件
pub fn render<'a>(
    element: &'a DesignElement,
    selection: Option<&(String, Vec<usize>)>,
    tailwind_node_class_editor: &'a iced::widget::text_editor::Content,
    tailwind_node_text_editor: &'a iced::widget::text_editor::Content,
    _tailwind_html_editor: &'a iced::widget::text_editor::Content,
    tailwind_node_class_input: &'a str,
    tailwind_node_class_dropdown_open: bool,
) -> Element<'a, Message> {
    // 如果元素类型不是 Tailwind，返回空界面
    if !element.kind.eq_ignore_ascii_case("tailwind") {
        return column![].into();
    }

    // 解析 HTML 内容为 DOM 节点树
    let content = element.content.as_deref().unwrap_or("");
    let nodes = dom::parse_html(content);

    // 创建"转换为图层"按钮（仅对 Tailwind 元素显示）
    let convert_button: Element<'a, Message> = if element.kind.eq_ignore_ascii_case("tailwind") {
        button(text("转换为图层").size(12))
            .on_press(Message::Design(DesignMessage::ConvertHtmlToLayers(element.id.clone())))
            .style(|theme: &Theme, status| {
                // 根据按钮状态设置不同的背景颜色
                let p = theme.extended_palette();
                let bg = match status {
                    iced::widget::button::Status::Hovered => Some(p.background.strong.color),
                    iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                    _ => Some(p.background.weak.color),
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .padding([8, 10])
            .into()
    } else {
        column![].into()
    };

    // 选中节点的属性编辑器
    let editor = if let Some((sel_id, sel_path)) = selection {
        // 检查选中的元素是否为当前元素
        if sel_id == &element.id {
            // 根据路径获取选中的 DOM 节点
            if let Some(node) = dom::get_node_by_path(&nodes, sel_path) {
                // 生成节点类型标签（如"段落 (P)"、"标题 1 (H1)"等）
                let type_label = node_type_label(node);
                // 生成节点内容预览文本
                let preview = node_preview_text(node);

                // 确定类名的目标路径
                // 如果是文本节点，类名应设置在父元素上
                let class_target_path = if node.text.is_some() && sel_path.len() > 1 {
                    sel_path[..sel_path.len() - 1].to_vec()
                } else {
                    sel_path.clone()
                };

                // 创建删除节点按钮的图标
                let delete_icon = svg(assets::get_icon(Icon::Trash))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .style(|theme: &Theme, _| iced::widget::svg::Style {
                        color: Some(theme.palette().text.scale_alpha(0.75)),
                    });

                // 创建删除节点按钮
                let delete_btn = button(delete_icon)
                    .on_press(Message::Design(DesignMessage::DeleteTailwindNode(
                        element.id.clone(),
                        sel_path.clone(),
                    )))
                    .style(|theme: &Theme, status| {
                        let palette = theme.extended_palette();
                        let bg = match status {
                            iced::widget::button::Status::Hovered => {
                                Some(palette.background.weak.color)
                            }
                            iced::widget::button::Status::Pressed => {
                                Some(palette.background.strong.color)
                            }
                            _ => None,
                        };
                        iced::widget::button::Style {
                            background: bg.map(Background::Color),
                            border: Border {
                                width: 0.0,
                                color: Color::TRANSPARENT,
                                radius: 8.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .padding(6);

                // 创建编辑器头部：显示节点类型和删除按钮
                let header = row![
                    text(type_label).size(13).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::new().width(Length::Fill),
                    delete_btn
                ]
                .align_y(iced::Alignment::Center);

                // 创建内容预览容器（显示节点内容的前 120 个字符）
                let content_preview = container(text(preview).size(12))
                    .style(iced::widget::container::bordered_box)
                    .padding(8)
                    .width(Length::Fill);

                // 文本编辑区域（仅对文本节点显示）
                let text_section: Element<'a, Message> = if node.text.is_some() {
                    // 创建提交消息，用于 Ctrl+Enter 提交文本更改
                    let commit_msg = Message::Design(DesignMessage::TailwindNodeTextCommit(
                        element.id.clone(),
                        sel_path.clone(),
                    ));
                    prop_section(
                        "文本",
                        text_editor(tailwind_node_text_editor)
                            .placeholder("输入文本...")
                            .on_action(|a| {
                                Message::Design(DesignMessage::TailwindNodeTextEditorAction(a))
                            })
                            // 设置键盘快捷键：Ctrl+Enter 提交
                            .key_binding(move |kp| {
                                if matches!(
                                    kp.key.clone(),
                                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                                ) && kp.modifiers.command()
                                {
                                    Some(iced::widget::text_editor::Binding::Custom(
                                        commit_msg.clone(),
                                    ))
                                } else {
                                    iced::widget::text_editor::Binding::from_key_press(kp)
                                }
                            })
                            .style(prop_text_editor_style)
                            .padding([6, 8])
                            .size(12)
                            .height(Length::Fixed(96.0)),
                    )
                } else {
                    Space::new().height(Length::Fixed(0.0)).into()
                };

                // 从编辑器中提取当前的类名列表（按空格分割）
                let class_tokens = tailwind_node_class_editor
                    .text()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();

                // 定义类名输入框的样式
                let input_style = |theme: &Theme, status: iced::widget::text_input::Status| {
                    let palette = theme.palette();
                    let extended = theme.extended_palette();
                    let focused =
                        matches!(status, iced::widget::text_input::Status::Focused { .. });
                    // 根据焦点状态设置边框颜色
                    let border_color =
                        if focused { palette.primary } else { extended.background.strong.color };
                    // 根据焦点状态设置背景颜色
                    let bg = if focused {
                        extended.background.weak.color
                    } else {
                        extended.background.base.color
                    };
                    iced::widget::text_input::Style {
                        background: iced::Background::Color(bg),
                        border: iced::Border {
                            width: 1.0,
                            color: border_color,
                            radius: 8.0.into(),
                        },
                        icon: palette.text.scale_alpha(0.5),
                        placeholder: palette.text.scale_alpha(0.55),
                        value: palette.text,
                        selection: palette.primary.scale_alpha(0.30),
                    }
                };

                // 创建类名输入框
                let class_token_input =
                    text_input("输入 class，回车创建（空格分割）", tailwind_node_class_input)
                        .on_input({
                            let element_id = element.id.clone();
                            let path = class_target_path.clone();
                            move |s| {
                                Message::Design(DesignMessage::TailwindNodeClassInputChanged(
                                    element_id.clone(),
                                    path.clone(),
                                    s,
                                ))
                            }
                        })
                        .on_submit(Message::Design(DesignMessage::TailwindNodeClassInputSubmit(
                            element.id.clone(),
                            class_target_path.clone(),
                        )))
                        .size(12)
                        .padding([6, 8])
                        .style(input_style);

                // 创建已选类名的标签显示区域
                let mut chips_row = row![].spacing(6);
                for token in &class_tokens {
                    // 计算移除当前标签后的新类名字符串
                    let new_value = class_tokens
                        .iter()
                        .filter(|t| *t != token)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ");

                    // 创建移除标签的消息
                    let remove_msg = Message::Design(DesignMessage::UpdateTailwindNodeClass(
                        element.id.clone(),
                        class_target_path.clone(),
                        new_value,
                    ));

                    // 创建移除按钮（X 图标）
                    let remove_btn =
                        button(svg(assets::get_icon(Icon::X)).width(8).height(8).style(
                            |theme: &Theme, _| iced::widget::svg::Style {
                                color: Some(theme.palette().text.scale_alpha(0.85)),
                            },
                        ))
                        .on_press(remove_msg)
                        .style(|theme: &Theme, status| {
                            let p = theme.extended_palette();
                            iced::widget::button::Style {
                                background: if status == iced::widget::button::Status::Hovered {
                                    Some(p.background.weak.color.into())
                                } else {
                                    None
                                },
                                border: Border {
                                    color: Color::TRANSPARENT,
                                    width: 0.0,
                                    radius: 7.0.into(),
                                },
                                text_color: theme.palette().text,
                                ..iced::widget::button::Style::default()
                            }
                        })
                        .padding(0)
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0));

                    // 创建标签容器（包含类名和移除按钮）
                    let chip = container(
                        row![text(token.clone()).size(12), remove_btn]
                            .spacing(6)
                            .align_y(iced::Alignment::Center),
                    )
                    .padding([4, 8])
                    .width(Length::Shrink)
                    .style(|theme: &Theme| {
                        let p = theme.extended_palette();
                        iced::widget::container::Style {
                            background: Some(p.background.strong.color.into()),
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 999.0.into(),
                            },
                            ..Default::default()
                        }
                    });

                    chips_row = chips_row.push(chip);
                }

                // 创建标签显示区域（如果没有类名则不显示）
                let chips_area: Element<'a, Message> = if class_tokens.is_empty() {
                    Space::new().height(Length::Fixed(0.0)).into()
                } else {
                    container(
                        scrollable(container(chips_row.wrap()).width(Length::Fill))
                            .height(Length::Fixed(120.0)),
                    )
                    .style(iced::widget::container::bordered_box)
                    .padding(6)
                    .into()
                };

                // 获取搜索关键字并过滤可用的 Tailwind 类名
                let needle = tailwind_node_class_input.trim().to_ascii_lowercase();
                let classes = get_tailwind_classes();
                // 根据搜索关键字过滤类名列表（空关键字则显示全部）
                let filtered: Vec<String> = if needle.is_empty() {
                    classes
                } else {
                    classes
                        .into_iter()
                        .filter(|c| c.to_ascii_lowercase().contains(&needle))
                        .collect()
                };

                // 构建类名选择列表（按分组显示）
                let mut list = column![].spacing(0);
                let mut current_group: Option<&'static str> = None;
                for class_name in filtered {
                    // 获取当前类名的分组
                    let group = group_tailwind_class(&class_name);
                    // 如果是新分组，添加分组标题
                    if current_group != Some(group) {
                        current_group = Some(group);
                        list = list.push(
                            container(text(group).size(11).style(iced::widget::text::secondary))
                                .width(Length::Fill)
                                .padding(iced::Padding {
                                    top: 10.0,
                                    right: 10.0,
                                    bottom: 6.0,
                                    left: 10.0,
                                }),
                        );
                    }

                    // 检查当前类名是否已被选中
                    let is_selected = class_tokens.iter().any(|t| t == &class_name);
                    // 根据选中状态计算点击后的新类名字符串
                    let new_value = if is_selected {
                        // 已选中则移除
                        class_tokens
                            .iter()
                            .filter(|t| *t != &class_name)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else {
                        // 未选中则添加
                        let mut next = class_tokens.clone();
                        next.push(class_name.clone());
                        next.join(" ")
                    };
                    let msg = Message::Design(DesignMessage::UpdateTailwindNodeClass(
                        element.id.clone(),
                        class_target_path.clone(),
                        new_value,
                    ));

                    // 创建类名选择按钮
                    let label = class_name.clone();
                    list = list.push(
                        button(
                            container(text(label).size(14)).width(Length::Fill).padding([8, 10]),
                        )
                        .on_press(msg)
                        .style(move |theme: &Theme, status| {
                            let ext = theme.extended_palette();
                            // 根据选中状态和按钮状态设置背景颜色
                            let bg = if is_selected {
                                theme.palette().primary
                            } else if status == button::Status::Hovered {
                                ext.background.weak.color
                            } else {
                                ext.background.base.color
                            };
                            button::Style {
                                background: Some(bg.into()),
                                // 已选中的按钮使用白色文字
                                text_color: if is_selected {
                                    Color::WHITE
                                } else {
                                    theme.palette().text
                                },
                                border: iced::Border {
                                    color: ext.background.strong.color,
                                    width: 0.0,
                                    radius: 0.0.into(),
                                },
                                ..button::Style::default()
                            }
                        })
                        .padding(0),
                    );
                }

                // 创建类名搜索下拉列表（仅在展开时显示）
                let class_search_overlay: Element<'a, Message> =
                    if tailwind_node_class_dropdown_open {
                        container(
                            scrollable(container(list).padding(4))
                                .direction(Direction::Vertical(
                                    Scrollbar::new().width(4).scroller_width(4),
                                ))
                                .height(Length::Fixed(480.0)),
                        )
                        .style(|theme: &Theme| {
                            let ext = theme.extended_palette();
                            iced::widget::container::Style {
                                background: Some(ext.background.base.color.into()),
                                border: iced::Border {
                                    color: ext.background.strong.color,
                                    width: 1.0,
                                    radius: 10.0.into(),
                                },
                                // 添加阴影效果
                                shadow: iced::Shadow {
                                    color: Color::BLACK.scale_alpha(0.20),
                                    offset: iced::Vector::new(0.0, 8.0),
                                    blur_radius: 20.0,
                                },
                                ..Default::default()
                            }
                        })
                        .width(Length::Fill)
                        .into()
                    } else {
                        column![].into()
                    };

                // 将输入框和下拉列表组合为叠加层组件
                let class_token_input_with_dropdown: Element<'a, Message> =
                    BelowOverlay::new(class_token_input, class_search_overlay)
                        .show(tailwind_node_class_dropdown_open)
                        .gap(4.0)
                        .on_close(Message::Design(DesignMessage::TailwindNodeClassDropdownClose(
                            element.id.clone(),
                            class_target_path.clone(),
                        )))
                        .into();

                // 组合所有编辑器部分
                column![
                    header,
                    // 如果是文本节点则不显示内容预览
                    if node.text.is_some() {
                        Space::new().height(Length::Fixed(0.0)).into()
                    } else {
                        prop_section("内容", content_preview)
                    },
                    text_section,
                    prop_section(
                        "Class",
                        column![class_token_input_with_dropdown, chips_area].spacing(6),
                    )
                ]
                .spacing(10)
            } else {
                // 节点不存在时显示错误信息
                column![text("选中的节点不存在").size(12)].spacing(5)
            }
        } else {
            // 选中的元素不是当前元素时返回空界面
            column![].spacing(5)
        }
    } else {
        // 没有选中任何元素时返回空界面
        column![].spacing(5)
    };

    // 最终布局：编辑器 + 转换按钮
    column![editor, convert_button].spacing(10).into()
}

/// 生成 DOM 节点的类型标签
///
/// 该函数根据节点的类型生成一个可读的中文标签，用于在属性面板中显示。
/// 对于文本节点，返回"文本"；对于元素节点，返回"标签名 (英文)"格式。
///
/// # 参数
///
/// * `node` - TailwindNode 节点的引用
///
/// # 返回值
///
/// 返回节点类型的可读标签字符串
///
/// # 示例
///
/// ```ignore
/// // 对于 <h1> 元素，返回 "标题 1 (H1)"
/// // 对于 <div> 元素，返回 "容器 (Div)"
/// // 对于文本节点，返回 "文本"
/// ```
fn node_type_label(node: &TailwindNode) -> String {
    // 如果是文本节点，直接返回"文本"
    if node.text.is_some() {
        return "文本".to_string();
    }

    // 根据标签名生成对应的中文标签
    let tag = node.tag.to_ascii_lowercase();
    match tag.as_str() {
        "p" => "段落 (P)".to_string(),
        "h1" => "标题 1 (H1)".to_string(),
        "h2" => "标题 2 (H2)".to_string(),
        "h3" => "标题 3 (H3)".to_string(),
        "button" => "按钮 (Button)".to_string(),
        "img" => "图片 (Img)".to_string(),
        "svg" => "图标 (Svg)".to_string(),
        "a" => "链接 (A)".to_string(),
        "div" => "容器 (Div)".to_string(),
        "span" => "行内 (Span)".to_string(),
        _ => format!("<{}>", node.tag), // 未识别的标签直接显示
    }
}

/// 生成 DOM 节点的内容预览文本
///
/// 该函数提取节点的文本内容用于预览显示。
/// 对于文本节点，直接返回其文本内容；对于元素节点，递归查找第一个文本内容。
/// 返回的文本最多包含 120 个字符，如果节点没有文本内容则返回"（无文本内容）"。
///
/// # 参数
///
/// * `node` - TailwindNode 节点的引用
///
/// # 返回值
///
/// 返回节点文本内容的预览字符串（最多 120 个字符）
///
/// # 示例
///
/// ```ignore
/// // 对于文本节点，返回其文本内容
/// // 对于 <div>Hello World</div>，返回 "Hello World"
/// // 对于空节点，返回 "（无文本内容）"
/// ```
fn node_preview_text(node: &TailwindNode) -> String {
    // 如果是文本节点，直接返回其文本内容
    if let Some(t) = node.text.as_deref() {
        return t.to_string();
    }

    // 递归查找第一个文本内容的辅助函数
    fn first_text(n: &TailwindNode) -> Option<&str> {
        // 如果当前节点有文本内容，返回它
        if let Some(t) = n.text.as_deref() {
            return Some(t);
        }
        // 否则递归查找子节点
        for c in &n.children {
            if let Some(t) = first_text(c) {
                return Some(t);
            }
        }
        None
    }

    // 提取文本并截取前 120 个字符
    let t = first_text(node).unwrap_or("");
    let out = t.chars().take(120).collect::<String>();
    if out.is_empty() { "（无文本内容）".to_string() } else { out }
}

#[cfg(test)]
#[path = "tailwind_tests.rs"]
mod tailwind_tests;
