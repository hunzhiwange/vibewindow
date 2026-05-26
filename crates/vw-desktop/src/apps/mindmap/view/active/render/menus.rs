//! 菜单渲染模块
//!
//! 本模块负责渲染思维导图应用中的各类菜单和控件，包括：
//! - 文件操作菜单（新建、打开、保存、导出等）
//! - 缩放控制器（放大、缩小、缩放比例显示）
//! - 缩放预设菜单（提供常用缩放比例快捷选项）
//!
//! 所有菜单均采用 Iced 框架的声明式 UI 构建方式，
//! 并遵循统一的设计风格（圆角、阴影、主题色适配等）。

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::svg;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// 渲染操作菜单覆盖层
///
/// 构建一个包含文件操作选项的下拉菜单，提供以下功能：
/// - 新建：创建新的思维导图文件
/// - 打开：打开现有思维导图文件
/// - 保存：保存当前思维导图
/// - 另存为：将当前思维导图保存为新文件
/// - 导出 PNG/JPEG/SVG：将思维导图导出为图片格式
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用
/// * `action_menu_w` - 菜单的宽度（像素值）
///
/// # 返回值
///
/// 返回渲染完成的菜单元素，可嵌入到 Iced 应用界面中
///
/// # 示例
///
/// ```ignore
/// let menu = action_menu_overlay(&tab, 200.0);
/// // 将 menu 添加到界面布局中
/// ```
pub(super) fn action_menu_overlay(_tab: &MindMapTab, action_menu_w: f32) -> Element<'_, Message> {
    // 菜单按钮的基础样式闭包
    // 根据按钮状态（悬停、按下等）动态计算背景色和边框样式
    let menu_btn_style = |theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        // 根据交互状态确定背景颜色
        let bg = match status {
            // 悬停状态：使用半透明弱背景色
            iced::widget::button::Status::Hovered => {
                let c = palette.background.weak.color;
                Some(Color::from_rgba(c.r, c.g, c.b, 0.55))
            }
            // 按下状态：使用强背景色
            iced::widget::button::Status::Pressed => Some(palette.background.strong.color),
            // 其他状态（如默认状态）：无背景
            _ => None,
        };
        iced::widget::button::Style {
            background: bg.map(Background::Color),
            // 圆角边框（8px 圆角）
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    };

    // 菜单项按钮构建闭包
    // 创建一个带图标、标签和快捷键提示的菜单项按钮
    //
    // # 参数
    // * `icon` - 图标类型
    // * `label` - 菜单项显示文本
    // * `shortcut` - 可选的快捷键提示文本（如 "Cmd+N"）
    // * `msg` - 可选的点击消息，None 表示禁用状态
    let menu_item_btn = |icon: Icon,
                         label: &'static str,
                         shortcut: Option<&'static str>,
                         msg: Option<Message>|
     -> Element<'_, Message> {
        // 根据消息是否存在判断按钮是否启用
        let enabled = msg.is_some();

        // 构建图标 SVG，根据启用状态调整颜色透明度
        let icon = svg(assets::get_icon(icon))
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .content_fit(iced::ContentFit::Contain)
            .style(move |theme: &Theme, _| {
                let c = if enabled {
                    theme.palette().text
                } else {
                    // 禁用状态：降低透明度
                    theme.palette().text.scale_alpha(0.35)
                };
                iced::widget::svg::Style { color: Some(c) }
            });

        // 构建标签文本，根据启用状态调整颜色
        let label_el: Element<'_, Message> = container(text(label).size(13))
            .style(move |theme: &Theme| iced::widget::container::Style {
                text_color: Some(if enabled {
                    theme.palette().text
                } else {
                    theme.palette().text.scale_alpha(0.35)
                }),
                ..Default::default()
            })
            .into();

        // 构建快捷键提示元素（如果有）
        let shortcut_el: Element<'_, Message> = if let Some(s) = shortcut {
            container(text(s).size(12))
                .align_x(iced::alignment::Horizontal::Right)
                .style(move |theme: &Theme| iced::widget::container::Style {
                    text_color: Some(if enabled {
                        // 启用状态：快捷键颜色稍淡
                        theme.palette().text.scale_alpha(0.55)
                    } else {
                        theme.palette().text.scale_alpha(0.35)
                    }),
                    ..Default::default()
                })
                .into()
        } else {
            // 无快捷键时使用占位空间
            Space::new().into()
        };

        // 组装按钮基础结构
        // 布局：[图标] [标签] [弹性空间] [快捷键]
        let base = button(
            container(
                row![
                    // 图标容器：固定宽度居中
                    container(icon)
                        .width(Length::Fixed(22.0))
                        .align_x(iced::alignment::Horizontal::Center),
                    label_el,
                    // 弹性空间：将快捷键推到右侧
                    Space::new().width(Length::Fill),
                    shortcut_el,
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .padding([6, 10]),
        )
        .style(move |theme: &Theme, status: iced::widget::button::Status| {
            let mut style = menu_btn_style(theme, status);
            if !enabled {
                // 禁用状态：移除背景色并降低文本透明度
                style.background = None;
                style.text_color = theme.palette().text.scale_alpha(0.35);
            }
            style
        })
        .width(Length::Fill);

        // 根据是否有消息决定是否添加点击事件
        if let Some(msg) = msg { base.on_press(msg).into() } else { base.into() }
    };

    // 构建文件操作菜单项列表
    let file_actions = column![
        // 新建文件
        menu_item_btn(
            Icon::FileEarmarkPlus,
            "新建",
            Some("Cmd+N"),
            Some(Message::MindMapTool(MindMapMessage::New))
        ),
        // 打开文件
        menu_item_btn(
            Icon::FolderOpen,
            "打开",
            Some("Cmd+O"),
            Some(Message::MindMapTool(MindMapMessage::Open))
        ),
        // 保存文件
        menu_item_btn(
            Icon::Save,
            "保存",
            Some("Cmd+S"),
            Some(Message::MindMapTool(MindMapMessage::Save))
        ),
        // 另存为
        menu_item_btn(
            Icon::Save,
            "另存为",
            Some("Cmd+Shift+S"),
            Some(Message::MindMapTool(MindMapMessage::SaveAs))
        ),
        // 导出 PNG
        menu_item_btn(
            Icon::CloudDownload,
            "导出 PNG",
            None,
            Some(Message::MindMapTool(MindMapMessage::ExportPng))
        ),
        // 导出 JPEG
        menu_item_btn(
            Icon::CloudDownload,
            "导出 JPEG",
            None,
            Some(Message::MindMapTool(MindMapMessage::ExportJpeg))
        ),
        // 导出 SVG
        menu_item_btn(
            Icon::CloudDownload,
            "导出 SVG",
            None,
            Some(Message::MindMapTool(MindMapMessage::ExportSvg))
        ),
    ]
    .spacing(2);

    // 构建最终的菜单容器
    // 应用统一的样式：背景色、边框、阴影效果
    container(column![file_actions].spacing(8))
        .padding(12)
        .width(Length::Fixed(action_menu_w))
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                // 背景色：使用主题基础背景色
                background: Some(Background::Color(p.background.base.color)),
                // 边框：1px 宽度，使用弱背景色，12px 圆角
                border: Border { width: 1.0, color: p.background.weak.color, radius: 12.0.into() },
                // 阴影效果：增加层次感
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.18),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 22.0,
                },
                ..Default::default()
            }
        })
        .into()
}

/// 渲染缩放控制器
///
/// 构建一个水平布局的缩放控件，包含三个部分：
/// - 减号按钮：缩小视图（每次缩放倍率除以 1.1）
/// - 缩放比例标签：显示当前缩放百分比，点击可展开预设菜单
/// - 加号按钮：放大视图（每次缩放倍率乘以 1.1）
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用，用于获取当前缩放值
/// * `zoom_control_w` - 控件的宽度（像素值）
/// * `zoom_control_h` - 控件的高度（像素值）
///
/// # 返回值
///
/// 返回渲染完成的缩放控制器元素
///
/// # 示例
///
/// ```ignore
/// let control = zoom_control(&tab, 120.0, 32.0);
/// // 将 control 添加到界面布局中
/// ```
pub(super) fn zoom_control(
    tab: &MindMapTab,
    zoom_control_w: f32,
    zoom_control_h: f32,
) -> Element<'_, Message> {
    // 缩放按钮样式生成器
    // 返回一个闭包，根据指定的圆角半径生成按钮样式
    //
    // # 参数
    // * `radius` - 按钮的圆角半径配置
    let zoom_btn_style = |radius: iced::border::Radius| {
        move |theme: &Theme, status: iced::widget::button::Status| {
            let p = theme.extended_palette();
            // 根据交互状态确定背景颜色
            let bg = match status {
                // 按下状态：强背景色
                iced::widget::button::Status::Pressed => {
                    Some(Background::Color(p.background.strong.color))
                }
                // 悬停状态：弱背景色
                iced::widget::button::Status::Hovered => {
                    Some(Background::Color(p.background.weak.color))
                }
                // 默认状态：无背景
                _ => None,
            };
            iced::widget::button::Style {
                background: bg,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius },
                text_color: theme.palette().text,
                ..Default::default()
            }
        }
    };

    // 格式化当前缩放比例为百分比字符串（如 "100%"）
    let zoom_label = format!("{:.0}%", (tab.zoom * 100.0).round());

    // 构建缩小按钮（左侧，左圆角）
    let zoom_minus: Element<'_, Message> = button(
        container(text("-").size(14).line_height(1.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::MindMapTool(MindMapMessage::Zoom(1.0 / 1.1, None)))
    .style(zoom_btn_style(iced::border::Radius {
        // 左侧圆角：12px
        top_left: 12.0,
        top_right: 0.0,
        bottom_left: 12.0,
        bottom_right: 0.0,
    }))
    .width(Length::Fixed(34.0))
    .height(Length::Fixed(zoom_control_h))
    .padding(0)
    .into();

    // 构建放大按钮（右侧，右圆角）
    let zoom_plus: Element<'_, Message> = button(
        container(text("+").size(14).line_height(1.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::MindMapTool(MindMapMessage::Zoom(1.1, None)))
    .style(zoom_btn_style(iced::border::Radius {
        // 右侧圆角：12px
        top_left: 0.0,
        top_right: 12.0,
        bottom_left: 0.0,
        bottom_right: 12.0,
    }))
    .width(Length::Fixed(34.0))
    .height(Length::Fixed(zoom_control_h))
    .padding(0)
    .into();

    // 构建缩放比例标签按钮（中间，无圆角）
    // 点击时切换缩放预设菜单的显示状态
    let zoom_label_btn: Element<'_, Message> = button(
        container(text(zoom_label).size(12).line_height(1.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::MindMapTool(MindMapMessage::ToggleZoomMenu))
    .style(zoom_btn_style(0.0.into()))
    .width(Length::Fill)
    .height(Length::Fixed(zoom_control_h))
    .padding(0)
    .into();

    // 分隔线构建闭包
    // 在按钮之间创建垂直分隔线
    let zoom_divider = || -> Element<'_, Message> {
        container(Space::new().width(Length::Fixed(1.0)))
            .height(Length::Fixed(zoom_control_h))
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                iced::widget::container::Style {
                    background: Some(Background::Color(p.background.strong.color)),
                    ..Default::default()
                }
            })
            .into()
    };

    // 组装完整的缩放控制器
    // 布局：[缩小按钮] [分隔线] [比例标签] [分隔线] [放大按钮]
    container(
        row![zoom_minus, zoom_divider(), zoom_label_btn, zoom_divider(), zoom_plus]
            .spacing(0)
            .align_y(Alignment::Center),
    )
    .width(Length::Fixed(zoom_control_w))
    .height(Length::Fixed(zoom_control_h))
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(p.background.base.color)),
            border: Border { width: 1.0, color: p.background.weak.color, radius: 12.0.into() },
            // 阴影效果：比菜单稍轻
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.10),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 18.0,
            },
            ..Default::default()
        }
    })
    .into()
}

/// 渲染缩放预设菜单覆盖层
///
/// 构建一个包含缩放预设选项的下拉菜单，提供以下功能：
/// - "适合窗口"：自动调整缩放以适应窗口大小
/// - 预设缩放比例列表：如 50%、75%、100%、125%、200% 等
///
/// 当前选中的缩放比例会高亮显示。
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态引用，用于获取当前缩放值
/// * `zoom_control_w` - 菜单的宽度（通常与缩放控制器宽度一致）
/// * `zoom_menu_item_h` - 每个菜单项的高度
/// * `zoom_menu_spacing` - 菜单项之间的间距
/// * `zoom_menu_padding` - 菜单内容的内边距
/// * `zoom_preset_percents` - 预设缩放百分比数组（如 &[50, 75, 100, 125, 200]）
///
/// # 返回值
///
/// 返回渲染完成的缩放预设菜单元素
///
/// # 示例
///
/// ```ignore
/// let presets = &[50, 75, 100, 125, 150, 200];
/// let menu = zoom_menu_overlay(&tab, 120.0, 28.0, 2.0, 8.0, presets);
/// // 将 menu 添加到界面布局中
/// ```
pub(super) fn zoom_menu_overlay(
    tab: &MindMapTab,
    zoom_control_w: f32,
    zoom_menu_item_h: f32,
    zoom_menu_spacing: f32,
    zoom_menu_padding: f32,
    zoom_preset_percents: &[u32],
) -> Element<'static, Message> {
    // 计算当前缩放百分比，并限制在合理范围内（0-10000%）
    let current_percent = (tab.zoom * 100.0).round().clamp(0.0, 10_000.0) as u32;

    // 缩放菜单按钮样式闭包
    // 与操作菜单按钮类似，但增加了激活状态的高亮处理
    //
    // # 参数
    // * `theme` - 当前主题
    // * `status` - 按钮交互状态
    // * `active` - 是否为当前选中的缩放比例
    let menu_btn_style = |theme: &Theme, status: iced::widget::button::Status, active: bool| {
        let p = theme.extended_palette();
        // 根据激活状态和交互状态确定背景颜色
        let bg = if active {
            // 激活状态：使用主题色半透明高亮
            Some(Background::Color(theme.palette().primary.scale_alpha(0.14)))
        } else {
            match status {
                // 悬停状态
                iced::widget::button::Status::Hovered => {
                    let c = p.background.weak.color;
                    Some(Background::Color(Color::from_rgba(c.r, c.g, c.b, 0.55)))
                }
                // 按下状态
                iced::widget::button::Status::Pressed => {
                    Some(Background::Color(p.background.strong.color))
                }
                // 默认状态
                _ => None,
            }
        };
        iced::widget::button::Style {
            background: bg,
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    };

    // 缩放菜单项按钮构建闭包
    //
    // # 参数
    // * `label` - 菜单项显示文本
    // * `msg` - 点击时发送的消息
    // * `active` - 是否为当前选中的缩放比例
    let menu_item_btn = |label: String,
                         msg: MindMapMessage,
                         active: bool|
     -> Element<'static, Message> {
        button(
            container(
                row![text(label).size(13), Space::new().width(Length::Fill)]
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .padding(iced::Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 10.0 }),
        )
        .style(move |theme: &Theme, status: iced::widget::button::Status| {
            menu_btn_style(theme, status, active)
        })
        .width(Length::Fill)
        .height(Length::Fixed(zoom_menu_item_h))
        .on_press(Message::MindMapTool(msg))
        .into()
    };

    // 构建菜单项列表
    // 预分配容量：1 个"适合窗口"项 + 预设百分比数量
    let mut items: Vec<Element<'static, Message>> =
        Vec::with_capacity(1 + zoom_preset_percents.len());

    // 添加"适合窗口"选项（始终不激活高亮）
    items.push(menu_item_btn("适合窗口".to_string(), MindMapMessage::ZoomFit, false));

    // 添加预设缩放比例选项
    // 与当前缩放百分比匹配的项会高亮显示
    for percent in zoom_preset_percents {
        let active = current_percent == *percent;
        items.push(menu_item_btn(
            format!("{percent}%"),
            MindMapMessage::ZoomSet(*percent as f32 / 100.0),
            active,
        ));
    }

    // 构建最终的菜单容器
    container(column(items).spacing(zoom_menu_spacing).padding(zoom_menu_padding))
        .width(Length::Fixed(zoom_control_w))
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border { width: 1.0, color: p.background.weak.color, radius: 12.0.into() },
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
