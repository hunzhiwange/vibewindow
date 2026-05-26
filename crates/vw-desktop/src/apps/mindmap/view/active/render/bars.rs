//! 工具栏渲染模块
//!
//! 本模块负责渲染思维导图应用中的各类工具栏组件，包括：
//! - 操作栏（Action Bar）：提供撤销/重做、剪切/复制/粘贴等全局操作
//! - 节点工具栏覆盖层（Node Toolbar Overlay）：提供节点样式编辑、优先级设置等节点级操作
//!
//! 所有工具栏均采用 Iced 框架构建，支持主题切换和交互状态反馈。

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapColorTarget, MindMapTab};
use iced::widget::svg;
use iced::widget::{Space, button, container, row, stack, text, tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::super::common::{priority_color, rgba_u32_to_color};

/// 构建操作栏（Action Bar）UI 组件
///
/// 操作栏是思维导图顶部的主工具栏，提供以下功能按钮：
/// - 菜单切换按钮：显示/隐藏侧边操作菜单
/// - 主题面板切换按钮：打开主题设置面板
/// - 图表类型选择器：切换不同的图表布局类型
/// - Markdown 导入按钮：从 Markdown 文件导入大纲
/// - 撤销/重做按钮：支持历史操作回退和前进
/// - 剪切/复制/粘贴/删除按钮：节点编辑操作
///
/// # 参数
///
/// * `tab` - 当前标签页的状态引用，包含各面板的显示状态
/// * `action_btn_size` - 操作按钮的尺寸（宽高相同）
/// * `action_bar_padding` - 操作栏的内边距
/// * `action_bar_spacing` - 按钮之间的间距
/// * `can_undo` - 是否可以执行撤销操作
/// * `can_redo` - 是否可以执行重做操作
/// * `can_cut` - 是否可以执行剪切操作
/// * `can_copy` - 是否可以执行复制操作
/// * `can_paste` - 是否可以执行粘贴操作
/// * `can_delete` - 是否可以执行删除操作
///
/// # 返回值
///
/// 返回构建好的操作栏 `Element`，可直接嵌入 Iced 布局中
///
/// # 示例
///
/// ```ignore
/// let bar = action_bar(
///     &tab,
///     36.0,  // 按钮尺寸
///     8.0,   // 内边距
///     4.0,   // 间距
///     true,  // 可撤销
///     false, // 不可重做
///     true,  // 可剪切
///     true,  // 可复制
///     false, // 不可粘贴
///     true,  // 可删除
/// );
/// ```
pub(super) fn action_bar(
    tab: &MindMapTab,
    action_btn_size: f32,
    action_bar_padding: f32,
    action_bar_spacing: f32,
    can_undo: bool,
    can_redo: bool,
    can_cut: bool,
    can_copy: bool,
    can_paste: bool,
    can_delete: bool,
) -> Element<'_, Message> {
    // 创建圆角矩形样式的图标按钮
    //
    // 内部闭包，用于生成具有统一样式的工具栏按钮。
    // 按钮支持启用/禁用状态和激活高亮状态。
    //
    // # 参数
    // * `icon` - 按钮显示的图标
    // * `on` - 点击时发送的消息，`None` 表示按钮禁用
    // * `active` - 是否处于激活状态（显示高亮背景）
    //
    // # 样式说明
    // - 禁用状态：灰色图标，无交互
    // - 激活状态：主色调图标，带半透明主色背景
    // - 悬停/按下：显示背景色变化
    // - 圆角半径：8px
    let icon_btn = |icon: Icon, on: Option<Message>, active: bool| -> Element<'_, Message> {
        let enabled = on.is_some();

        // 构建 SVG 图标，根据状态设置颜色
        let icon: Element<'static, Message> = svg(assets::get_icon(icon))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .content_fit(iced::ContentFit::Contain)
            .style(move |theme: &Theme, _| {
                // 根据按钮状态决定图标颜色
                let c = if !enabled {
                    Color::from_rgba8(160, 160, 160, 1.0) // 禁用状态：灰色
                } else if active {
                    theme.palette().primary // 激活状态：主色调
                } else {
                    theme.palette().text // 正常状态：文本色
                };
                iced::widget::svg::Style { color: Some(c) }
            })
            .into();

        // 将图标居中放置在容器中
        let content: Element<'static, Message> = container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();

        // 构建按钮，设置尺寸和样式
        let base = button(content)
            .padding(0)
            .width(Length::Fixed(action_btn_size))
            .height(Length::Fixed(action_btn_size))
            .style(move |theme: &Theme, status| {
                let palette = theme.extended_palette();

                // 根据状态设置背景
                let bg = if !enabled {
                    None // 禁用状态：无背景
                } else if active {
                    // 激活状态：主色调半透明背景
                    Some(Background::Color(theme.palette().primary.scale_alpha(0.14)))
                } else {
                    // 根据交互状态设置背景
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(Background::Color(palette.background.weak.color))
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(palette.background.strong.color))
                        }
                        _ => None,
                    }
                };

                iced::widget::button::Style {
                    background: bg,
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
                    text_color: if enabled {
                        theme.palette().text
                    } else {
                        Color::from_rgba8(160, 160, 160, 1.0)
                    },
                    ..Default::default()
                }
            });

        // 如果有消息，绑定点击事件
        if let Some(msg) = on { base.on_press(msg).into() } else { base.into() }
    };

    // 创建圆形样式的图标按钮
    //
    // 与 `icon_btn` 功能相同，但使用完全圆角（999px）的边框样式，
    // 适用于需要突出显示的特殊按钮。
    let icon_btn_circle = |icon: Icon, on: Option<Message>, active: bool| -> Element<'_, Message> {
        let enabled = on.is_some();

        // 构建 SVG 图标
        let icon: Element<'static, Message> = svg(assets::get_icon(icon))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .content_fit(iced::ContentFit::Contain)
            .style(move |theme: &Theme, _| {
                let c = if !enabled {
                    Color::from_rgba8(160, 160, 160, 1.0)
                } else if active {
                    theme.palette().primary
                } else {
                    theme.palette().text
                };
                iced::widget::svg::Style { color: Some(c) }
            })
            .into();

        // 将图标居中放置
        let content: Element<'static, Message> = container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();

        // 构建按钮，使用圆形边框（radius: 999.0）
        let base = button(content)
            .padding(0)
            .width(Length::Fixed(action_btn_size))
            .height(Length::Fixed(action_btn_size))
            .style(move |theme: &Theme, status| {
                let palette = theme.extended_palette();
                let bg = if !enabled {
                    None
                } else if active {
                    Some(Background::Color(theme.palette().primary.scale_alpha(0.14)))
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(Background::Color(palette.background.weak.color))
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(palette.background.strong.color))
                        }
                        _ => None,
                    }
                };

                iced::widget::button::Style {
                    background: bg,
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
                    text_color: if enabled {
                        theme.palette().text
                    } else {
                        Color::from_rgba8(160, 160, 160, 1.0)
                    },
                    ..Default::default()
                }
            });

        if let Some(msg) = on { base.on_press(msg).into() } else { base.into() }
    };

    // 构建各个功能按钮

    // 菜单切换按钮
    let menu_btn = icon_btn(
        Icon::LayoutSidebar,
        Some(Message::MindMapTool(MindMapMessage::ToggleActionMenu)),
        tab.show_action_menu,
    );

    // 主题面板切换按钮
    let theme_btn = icon_btn(
        Icon::Palette,
        Some(Message::MindMapTool(MindMapMessage::ToggleThemePanel)),
        tab.show_theme_panel,
    );

    // 图表类型选择器按钮
    let diagram_type_btn = icon_btn(
        Icon::Grid1x2,
        Some(Message::MindMapTool(MindMapMessage::ToggleDiagramTypePicker)),
        tab.show_diagram_type_picker,
    );

    // Markdown 导入按钮（使用圆形样式突出显示）
    let markdown_outline_btn = icon_btn_circle(
        Icon::Markdown,
        Some(Message::MindMapTool(MindMapMessage::ToggleMarkdownImport)),
        tab.show_markdown_import,
    );

    // 撤销按钮：仅在可撤销时启用
    let undo_btn = icon_btn(
        Icon::ArrowCounterClockwise,
        can_undo.then_some(Message::MindMapTool(MindMapMessage::Undo)),
        false,
    );

    // 重做按钮：仅在可重做时启用
    let redo_btn = icon_btn(
        Icon::ArrowClockwise,
        can_redo.then_some(Message::MindMapTool(MindMapMessage::Redo)),
        false,
    );

    // 剪切按钮
    let cut_btn = icon_btn(
        Icon::Scissors,
        can_cut.then_some(Message::MindMapTool(MindMapMessage::CutNode)),
        false,
    );

    // 复制按钮
    let copy_btn = icon_btn(
        Icon::Copy,
        can_copy.then_some(Message::MindMapTool(MindMapMessage::CopyNode)),
        false,
    );

    // 粘贴按钮
    let paste_btn = icon_btn(
        Icon::Clipboard,
        can_paste.then_some(Message::MindMapTool(MindMapMessage::PasteNode)),
        false,
    );

    // 删除按钮
    let delete_btn = icon_btn(
        Icon::Trash,
        can_delete.then_some(Message::MindMapTool(MindMapMessage::DeleteNode)),
        false,
    );

    // 组装操作栏容器
    container(
        row![
            menu_btn,
            theme_btn,
            diagram_type_btn,
            markdown_outline_btn,
            undo_btn,
            redo_btn,
            cut_btn,
            copy_btn,
            paste_btn,
            delete_btn
        ]
        .spacing(action_bar_spacing)
        .align_y(Alignment::Center),
    )
    .padding(action_bar_padding)
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(p.background.base.color)),
            border: Border { width: 1.0, color: p.background.weak.color, radius: 12.0.into() },
            // 添加阴影效果增强视觉层次
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.12),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 18.0,
            },
            ..Default::default()
        }
    })
    .into()
}

/// 构建节点工具栏覆盖层（Node Toolbar Overlay）UI 组件
///
/// 节点工具栏是当选中节点时显示的浮动工具栏，提供以下功能：
/// - 编辑操作：剪切、复制、粘贴、删除
/// - 样式设置：优先级、文字颜色、节点填充、边框颜色、连线颜色
/// - 链接设置：为节点添加 URL 链接
/// - 文本编辑：打开节点文本编辑器
///
/// # 参数
///
/// * `tab` - 当前标签页的状态引用
/// * `node_toolbar_btn_w` - 工具栏按钮的宽度
/// * `node_toolbar_btn_h` - 工具栏按钮的高度
/// * `node_toolbar_padding` - 工具栏的内边距
/// * `node_toolbar_divider_h` - 分隔线的高度
/// * `selected_path_is_root` - 当前选中的是否为根节点
/// * `can_cut` - 是否可以执行剪切操作
/// * `can_copy` - 是否可以执行复制操作
/// * `can_paste` - 是否可以执行粘贴操作
/// * `can_delete` - 是否可以执行删除操作
/// * `can_style` - 是否可以修改节点样式
/// * `current_priority` - 当前节点的优先级（0-255）
/// * `current_url_present` - 当前节点是否已设置 URL 链接
///
/// # 返回值
///
/// 返回构建好的节点工具栏 `Element`，可直接嵌入 Iced 布局中
///
/// # 设计说明
///
/// 工具栏分为两个区域：
/// 1. 编辑操作区（左侧）：剪切、复制、粘贴、删除
/// 2. 样式操作区（右侧）：优先级、颜色、链接、编辑文本
///
/// 两个区域之间用竖向分隔线分开。
pub(super) fn node_toolbar_overlay(
    tab: &MindMapTab,
    node_toolbar_btn_w: f32,
    node_toolbar_btn_h: f32,
    node_toolbar_padding: f32,
    node_toolbar_divider_h: f32,
    selected_path_is_root: bool,
    can_cut: bool,
    can_copy: bool,
    can_paste: bool,
    can_delete: bool,
    can_style: bool,
    current_priority: Option<u8>,
    current_url_present: bool,
) -> Element<'_, Message> {
    // 创建带可选色块标记和工具提示的图标按钮
    //
    // 此闭包用于生成节点工具栏中的按钮，支持以下特性：
    // - 可选的右上角色块标记（用于显示当前颜色值）
    // - 鼠标悬停时显示工具提示
    // - 启用/禁用状态视觉反馈
    //
    // # 参数
    // * `icon` - 按钮显示的图标
    // * `swatch` - 可选的颜色色块，显示在按钮右上角
    // * `on` - 点击时发送的消息，`None` 表示按钮禁用
    // * `width` - 按钮宽度
    // * `tooltip_text` - 工具提示文本，`None` 表示不显示提示
    let icon_btn = |icon: Icon,
                    swatch: Option<Color>,
                    on: Option<Message>,
                    width: f32,
                    tooltip_text: Option<&'static str>|
     -> Element<'_, Message> {
        let enabled = on.is_some();
        let disabled_icon_color = Color::from_rgba8(160, 160, 160, 1.0);

        // 构建 SVG 图标
        let icon_el: Element<'static, Message> = svg(assets::get_icon(icon))
            .width(14)
            .height(14)
            .style(move |theme: &Theme, _| {
                let c = if enabled { theme.palette().text } else { disabled_icon_color };
                iced::widget::svg::Style { color: Some(c) }
            })
            .into();

        // 图标居中容器
        let icon_layer: Element<'static, Message> = container(icon_el)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();

        // 可选的色块标记层（显示在右上角）
        let badge_layer: Option<Element<'static, Message>> = swatch.map(|c| {
            let bg = if enabled { c } else { disabled_icon_color };

            // 创建圆形色块
            let dot: Element<'static, Message> = container(Space::new())
                .width(Length::Fixed(9.0))
                .height(Length::Fixed(9.0))
                .style(move |_| iced::widget::container::Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        width: 1.0,
                        color: Color::from_rgba8(0, 0, 0, if enabled { 0.14 } else { 0.08 }),
                        radius: 999.0.into(),
                    },
                    ..Default::default()
                })
                .into();

            // 将色块定位到右上角
            container(dot)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Top)
                .padding([2, 2])
                .into()
        });

        // 根据是否有色块标记，选择布局方式
        let content: Element<'static, Message> = if let Some(badge) = badge_layer {
            stack![icon_layer, badge].into() // 叠加布局
        } else {
            icon_layer
        };

        // 构建按钮
        let base = button(content)
            .padding([4, 8])
            .width(Length::Fixed(width))
            .height(Length::Fixed(node_toolbar_btn_h))
            .style(move |theme: &Theme, status| {
                let palette = theme.extended_palette();
                let bg = if enabled {
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(palette.background.weak.color)
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(palette.background.strong.color)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
                    text_color: if enabled { theme.palette().text } else { disabled_icon_color },
                    ..Default::default()
                }
            });

        let btn = if let Some(on) = on { base.on_press(on).into() } else { base.into() };

        // 可选：添加工具提示
        if let Some(txt) = tooltip_text {
            // 构建工具提示样式
            let tip_content =
                container(text(txt).size(12)).padding([6, 8]).style(|_theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(Color::from_rgba8(24, 24, 24, 0.96).into()),
                        text_color: Some(Color::WHITE),
                        border: Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 8.0.into(),
                        },
                        shadow: iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.30),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 16.0,
                        },
                        ..Default::default()
                    }
                });
            tooltip::Tooltip::new(btn, tip_content, tooltip::Position::Top).gap(4.0).into()
        } else {
            btn
        }
    };

    // === 构建编辑操作按钮 ===

    // 剪切按钮
    let cut_btn = icon_btn(
        Icon::Scissors,
        None,
        can_cut.then_some(Message::MindMapTool(MindMapMessage::CutNode)),
        node_toolbar_btn_w,
        Some("剪切 (Ctrl+X)"),
    );

    // 复制按钮
    let copy_btn = icon_btn(
        Icon::Copy,
        None,
        can_copy.then_some(Message::MindMapTool(MindMapMessage::CopyNode)),
        node_toolbar_btn_w,
        Some("拷贝 (Ctrl+C)"),
    );

    // 粘贴按钮
    let paste_btn = icon_btn(
        Icon::Clipboard,
        None,
        can_paste.then_some(Message::MindMapTool(MindMapMessage::PasteNode)),
        node_toolbar_btn_w,
        Some("粘贴 (Ctrl+V)"),
    );

    // 删除按钮
    let delete_btn = icon_btn(
        Icon::Trash,
        None,
        can_delete.then_some(Message::MindMapTool(MindMapMessage::DeleteNode)),
        node_toolbar_btn_w,
        Some("删除 (Delete)"),
    );

    // === 构建样式操作按钮 ===

    // 优先级按钮：显示当前优先级对应的颜色作为标记
    let priority_btn: Element<'_, Message> = icon_btn(
        Icon::Speedometer2,
        current_priority.map(priority_color),
        can_style.then_some(Message::MindMapTool(MindMapMessage::TogglePriorityPicker)),
        node_toolbar_btn_w,
        Some("优先级"),
    );

    // 连线颜色按钮：根节点不显示（根节点没有入边）
    let edge_color_btn: Element<'_, Message> = if selected_path_is_root {
        container(text("")).width(Length::Fixed(0.0)).into()
    } else {
        // 获取当前连线颜色，未设置则使用默认灰色
        let edge_initial = tab
            .selected_path
            .as_ref()
            .and_then(|p| tab.edge_colors.get(p))
            .copied()
            .map(rgba_u32_to_color)
            .unwrap_or(Color::from_rgba8(208, 215, 222, 1.0));

        icon_btn(
            Icon::Bezier,
            None,
            can_style.then_some(Message::MindMapTool(MindMapMessage::OpenColorPicker(
                MindMapColorTarget::EdgeStroke,
                edge_initial,
            ))),
            node_toolbar_btn_w,
            Some("连线颜色"),
        )
    };

    // 获取当前节点填充颜色，未设置则使用默认白色
    let node_fill_initial = tab
        .selected_path
        .as_ref()
        .and_then(|p| tab.node_fills.get(p))
        .copied()
        .map(rgba_u32_to_color)
        .unwrap_or(Color::from_rgba8(255, 255, 255, 1.0));

    // 节点填充颜色按钮
    let node_fill_btn: Element<'_, Message> = icon_btn(
        Icon::PaintBucket,
        None,
        can_style.then_some(Message::MindMapTool(MindMapMessage::OpenColorPicker(
            MindMapColorTarget::NodeFill,
            node_fill_initial,
        ))),
        node_toolbar_btn_w,
        Some("节点填充"),
    );

    // 获取当前节点文字颜色，未设置则使用默认深色
    let node_text_initial = tab
        .selected_path
        .as_ref()
        .and_then(|p| tab.node_text_colors.get(p))
        .copied()
        .map(rgba_u32_to_color)
        .unwrap_or(Color::from_rgba8(17, 24, 39, 1.0));

    // 文字颜色按钮
    let node_text_btn: Element<'_, Message> = icon_btn(
        Icon::Type,
        None,
        can_style.then_some(Message::MindMapTool(MindMapMessage::OpenColorPicker(
            MindMapColorTarget::NodeText,
            node_text_initial,
        ))),
        node_toolbar_btn_w,
        Some("文字颜色"),
    );

    // 获取当前节点边框颜色，未设置则使用默认灰色
    let node_border_initial = tab
        .selected_path
        .as_ref()
        .and_then(|p| tab.node_border_colors.get(p))
        .copied()
        .map(rgba_u32_to_color)
        .unwrap_or(Color::from_rgba8(208, 215, 222, 1.0));

    // 边框颜色按钮
    let node_border_btn: Element<'_, Message> = icon_btn(
        Icon::BorderStyle,
        None,
        can_style.then_some(Message::MindMapTool(MindMapMessage::OpenColorPicker(
            MindMapColorTarget::NodeBorder,
            node_border_initial,
        ))),
        node_toolbar_btn_w,
        Some("边框颜色"),
    );

    // 链接按钮：如果已设置链接，显示蓝色标记
    let url_btn = icon_btn(
        Icon::Link,
        current_url_present.then_some(Color::from_rgba8(59, 130, 246, 1.0)),
        can_style.then_some(Message::MindMapTool(MindMapMessage::ToggleNodeUrlEditor)),
        node_toolbar_btn_w,
        Some("链接"),
    );

    // 编辑文本按钮
    let edit_text_btn = icon_btn(
        Icon::Pencil,
        None,
        can_style.then_some(Message::MindMapTool(MindMapMessage::ToggleNodeTextEditor)),
        node_toolbar_btn_w,
        Some("编辑文本"),
    );

    // === 组装工具栏布局 ===

    // 编辑操作区：剪切、复制、粘贴、删除
    let edit_actions =
        row![cut_btn, copy_btn, paste_btn, delete_btn].spacing(6).align_y(Alignment::Center);

    // 样式操作区：优先级、颜色、链接、编辑
    let node_actions = row![
        priority_btn,
        node_text_btn,
        node_fill_btn,
        node_border_btn,
        edge_color_btn,
        url_btn,
        edit_text_btn
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    // 分隔线：视觉上区分两个功能区
    let divider: Element<'_, Message> = container(Space::new().width(Length::Fixed(1.0)))
        .height(Length::Fixed(node_toolbar_divider_h))
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(p.background.strong.color)),
                ..Default::default()
            }
        })
        .into();

    // 最终工具栏容器：将编辑区和样式区通过分隔线组合
    container(row![edit_actions, divider, node_actions].spacing(10).align_y(Alignment::Center))
        .padding(node_toolbar_padding)
        .height(Length::Fixed(node_toolbar_padding * 2.0 + node_toolbar_btn_h))
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border { width: 1.0, color: p.background.weak.color, radius: 12.0.into() },
                // 添加阴影效果，比操作栏更明显
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.12),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 22.0,
                },
                ..Default::default()
            }
        })
        .into()
}
