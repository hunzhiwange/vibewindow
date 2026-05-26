//! 主题面板渲染模块
//!
//! 本模块负责渲染思维导图的主题选择面板，提供可视化的主题配色选择界面。
//! 用户可以在此面板中选择预设主题组或自定义主题，管理主题配色方案。
//!
//! ## 主要功能
//!
//! - **主题组选择**：通过下拉列表选择不同的主题组（预设组 + 自定义组）
//! - **主题变体预览**：以卡片网格形式展示主题变体的预览效果
//! - **主题管理操作**：提供保存、删除、取消背景等操作按钮
//! - **实时预览**：每张卡片展示主题的配色方案（中央、主分支、主题、叶子节点）

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::apps::mindmap::canvas::theme::{
    CUSTOM_THEME_GROUP_ID, CUSTOM_THEME_GROUP_NAME, THEME_GROUPS, resolve_theme,
    theme_group_variant_count,
};
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::svg;
use iced::widget::{Space, button, column, container, pick_list, row, scrollable, text, tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::super::super::common::rgba_u32_to_color;

/// 创建主题选择面板
///
/// 构建一个完整的主题选择面板，包含主题组下拉列表、主题卡片网格和操作按钮。
/// 面板支持滚动浏览多个主题变体，并提供主题管理功能。
///
/// # 参数
///
/// * `tab` - 当前思维导图标签页的状态，包含当前选中的主题组和变体信息
/// * `panel_w` - 面板宽度（像素）
/// * `panel_h` - 面板高度（像素）
///
/// # 返回值
///
/// 返回一个 Iced 元素，包含完整的主题面板 UI
///
/// # 示例
///
/// ```ignore
/// let panel = theme_panel(&mindmap_tab, 360.0, 480.0);
/// // 将 panel 添加到父容器中显示
/// ```
pub(in super::super) fn theme_panel(
    tab: &MindMapTab,
    panel_w: f32,
    panel_h: f32,
) -> Element<'static, Message> {
    // 主题组选项结构体
    //
    // 用于下拉列表中展示主题组的 ID 和显示名称
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ThemeGroupOption {
        // 主题组的唯一标识符
        id: &'static str,
        // 主题组的显示名称
        name: &'static str,
    }

    // 实现显示 trait，使选项在下拉列表中正确显示名称
    impl std::fmt::Display for ThemeGroupOption {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    // 构建主题组选项列表：预设主题组 + 自定义主题组
    let options: Vec<ThemeGroupOption> = THEME_GROUPS
        .iter()
        .map(|g| ThemeGroupOption { id: g.id, name: g.name })
        .chain(std::iter::once(ThemeGroupOption {
            id: CUSTOM_THEME_GROUP_ID,
            name: CUSTOM_THEME_GROUP_NAME,
        }))
        .collect();

    // 查找当前选中的主题组，若未找到则默认选择第一个
    let selected_group = options
        .iter()
        .copied()
        .find(|o| o.id == tab.theme_group.as_str())
        .or_else(|| options.first().copied());

    // 确定当前活动的主题组 ID，并进行有效性验证
    let active_group_id = selected_group.map(|g| g.id).unwrap_or(THEME_GROUPS[0].id);
    // 验证主题组 ID 是否有效（自定义组或在预设组列表中）
    let active_group_id = if active_group_id == CUSTOM_THEME_GROUP_ID
        || THEME_GROUPS.iter().any(|g| g.id == active_group_id)
    {
        active_group_id
    } else {
        THEME_GROUPS[0].id
    };

    // 创建主题组下拉选择器
    let group_pick: Element<'static, Message> = pick_list(options, selected_group, |o| {
        Message::MindMapTool(MindMapMessage::SetThemeGroup(o.id.to_string()))
    })
    .width(Length::Fixed(170.0))
    .text_size(12)
    .into();

    // 计算当前主题组的变体数量
    // 自定义主题组使用自定义主题列表长度，预设组使用预设变体数量
    let variant_count = if active_group_id == CUSTOM_THEME_GROUP_ID {
        tab.custom_themes.len().max(1)
    } else {
        theme_group_variant_count(active_group_id).max(1)
    };

    // 计算卡片布局参数
    let card_gap = 10.0; // 卡片间距
    let card_w = ((panel_w - 20.0 - card_gap) / 2.0).max(120.0); // 卡片宽度（两列布局）
    let card_h = 66.0; // 卡片高度

    // 创建单个主题预览卡片
    //
    // 每张卡片展示主题的配色方案，包含中央节点和三个层级分支的预览
    //
    // # 参数
    //
    // * `variant` - 主题变体索引
    //
    // # 返回值
    //
    // 返回一个可点击的按钮元素，点击后切换到该主题变体
    let card = |variant: usize| -> Element<'static, Message> {
        // 解析当前变体的主题配置
        let theme = resolve_theme(active_group_id, variant, &tab.custom_themes);
        // 判断是否为当前激活的主题
        let active = tab.theme_group == active_group_id && tab.theme_variant == variant;
        // 转换背景颜色
        let bg = rgba_u32_to_color(theme.background_color);

        // 创建颜色徽章（用于展示节点样式）
        //
        // # 参数
        //
        // * `label` - 徽章标签文本
        // * `fill` - 填充颜色（u32 RGBA 格式）
        // * `text_rgba` - 文字颜色（u32 RGBA 格式）
        let badge = |label: &'static str, fill: u32, text_rgba: u32| {
            let fill = rgba_u32_to_color(fill);
            let text_c = rgba_u32_to_color(text_rgba);
            container(text(label).size(10)).padding([4, 6]).style(move |_| {
                iced::widget::container::Style {
                    background: Some(Background::Color(fill)),
                    text_color: Some(text_c),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
                    ..Default::default()
                }
            })
        };

        // 创建各层级的徽章预览
        let root = badge("Central", theme.root_fill, theme.root_text); // 中央节点
        let b1 = badge("Main", theme.palette(0), theme.branch_text); // 主分支
        let b2 = badge("Topic", theme.palette(1), theme.branch_text); // 主题节点
        let b3 = badge("Leaf", theme.leaf_fill, theme.leaf_text); // 叶子节点

        // 组装右侧分支预览列
        let right = column![b1, b2, b3].spacing(4).width(Length::Fill);
        // 组装卡片内容行（中央节点 + 间距 + 分支预览）
        let inner = row![root, Space::new().width(Length::Fixed(8.0)), right]
            .spacing(0)
            .align_y(Alignment::Center);

        // 创建可点击的主题选择按钮
        button(inner)
            .on_press(Message::MindMapTool(MindMapMessage::SetThemeVariant(
                active_group_id.to_string(),
                variant,
            )))
            .width(Length::Fixed(card_w))
            .height(Length::Fixed(card_h))
            .padding(8)
            .style(move |t: &Theme, status| {
                let p = t.extended_palette();
                let is_hovered = status == iced::widget::button::Status::Hovered;
                iced::widget::button::Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        // 激活状态的卡片使用更粗的边框
                        width: if active { 2.0 } else { 1.0 },
                        color: if active {
                            p.primary.base.color
                        } else if is_hovered {
                            p.background.strong.color
                        } else {
                            p.background.weak.color
                        },
                        radius: 10.0.into(),
                    },
                    text_color: t.palette().text,
                    ..Default::default()
                }
            })
            .into()
    };

    // 构建卡片网格（两列布局）
    let mut grid = column![].spacing(10);
    let mut i = 0;
    while i < variant_count {
        // 创建第一列卡片
        let mut r = row![card(i)].spacing(card_gap).align_y(Alignment::Center);
        // 如果还有更多变体，添加第二列卡片；否则添加占位空间
        if i + 1 < variant_count {
            r = r.push(card(i + 1));
        } else {
            r = r.push(Space::new().width(Length::Fixed(card_w)).height(Length::Fixed(card_h)));
        }
        grid = grid.push(r);
        i += 2;
    }

    // 创建提示框内容容器
    //
    // # 参数
    //
    // * `tip` - 提示文本内容
    //
    // # 返回值
    //
    // 返回带有深色背景和阴影的提示框容器
    let tip_content = |tip: String| {
        container(text(tip).size(12)).padding([6, 8]).style(|_theme: &Theme| {
            iced::widget::container::Style {
                background: Some(Color::from_rgba8(16, 16, 16, 0.96).into()),
                text_color: Some(Color::WHITE),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.40),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 18.0,
                },
                ..Default::default()
            }
        })
    };

    // 创建带提示的图标按钮
    //
    // # 参数
    //
    // * `icon` - 图标类型
    // * `tip` - 悬停提示文本
    // * `on_press` - 点击时发送的消息
    //
    // # 返回值
    //
    // 返回带有底部提示框的图标按钮元素
    let icon_btn = |icon: Icon, tip: String, on_press: Message| -> Element<'static, Message> {
        // 创建图标按钮主体
        let btn = button(
            container(
                svg(assets::get_icon(icon))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .content_fit(iced::ContentFit::Contain)
                    .style(move |theme: &Theme, _| {
                        let c = theme.palette().text.scale_alpha(0.70);
                        iced::widget::svg::Style { color: Some(c) }
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .on_press(on_press)
        .padding(0)
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .style(|theme: &Theme, status| {
            let p = theme.extended_palette();
            // 根据按钮状态设置背景色
            let bg = match status {
                iced::widget::button::Status::Hovered => Some(p.background.weak.color.into()),
                iced::widget::button::Status::Pressed => Some(p.background.strong.color.into()),
                _ => None,
            };
            iced::widget::button::Style {
                background: bg,
                border: Border { width: 1.0, color: p.background.strong.color, radius: 8.0.into() },
                text_color: theme.palette().text.scale_alpha(0.80),
                ..Default::default()
            }
        });

        // 将按钮包装在提示框中
        tooltip::Tooltip::new(btn, tip_content(tip), tooltip::Position::Bottom).gap(8).into()
    };

    // 创建"保存到自定义组合"按钮
    let save_btn = icon_btn(
        Icon::Save,
        "保存到自定义组合".to_string(),
        Message::MindMapTool(MindMapMessage::SaveThemeToCustom),
    );

    // 创建"取消套图背景"按钮（仅在跟随主题背景且无自定义背景时显示）
    let cancel_btn = (tab.background.is_none() && tab.follow_theme_background).then(|| {
        icon_btn(
            Icon::EyeSlash,
            "取消套图背景".to_string(),
            Message::MindMapTool(MindMapMessage::CancelThemeBackground),
        )
    });

    // 创建"删除当前自定义配色"按钮（仅在自定义主题组且有自定义主题时显示）
    let delete_btn = (active_group_id == CUSTOM_THEME_GROUP_ID && !tab.custom_themes.is_empty())
        .then(|| {
            icon_btn(
                Icon::Trash,
                "删除当前自定义配色".to_string(),
                Message::MindMapTool(MindMapMessage::DeleteCustomTheme(tab.theme_variant)),
            )
        });

    // 组装面板头部：下拉列表 + 操作按钮
    let mut header =
        row![group_pick, Space::new().width(Length::Fill)].spacing(8).align_y(Alignment::Center);
    header = header.push(save_btn);
    if let Some(cancel_btn) = cancel_btn {
        header = header.push(cancel_btn);
    }
    if let Some(delete_btn) = delete_btn {
        header = header.push(delete_btn);
    }

    // 创建可滚动的卡片网格区域
    let grid_scroll =
        scrollable(grid).direction(Direction::Vertical(Scrollbar::new())).height(Length::Fill);

    // 组装面板内容：头部 + 可滚动网格
    let content = column![header, grid_scroll].spacing(10);

    // 包装在容器中并应用样式
    container(content)
        .padding([10, 10])
        .width(Length::Fixed(panel_w))
        .height(Length::Fixed(panel_h))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color,
                    radius: 12.0.into(),
                },
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
