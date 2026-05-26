//! 背景面板渲染模块
//!
//! 该模块负责渲染思维导图的背景颜色选择面板。
//! 提供两种背景模式：
//! - **跟随主题**：背景颜色随当前画布主题自动变化
//! - **固定颜色**：从预设色板中选择固定的背景颜色
//!
//! ## 主要组件
//!
//! - [`background_panel`]：主面板渲染函数，返回完整的背景选择器UI元素
//!
//! ## 交互
//!
//! 用户点击色块或跟随主题按钮时，会发送 [`MindMapMessage::SetBackground`] 消息
//! 更新思维导图的背景设置。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::super::super::common::rgba_u32_to_color;
use crate::apps::mindmap::canvas::theme::resolve_theme;

/// 渲染背景颜色选择面板
///
/// 创建一个包含跟随主题按钮和预设色板的背景选择器面板。
/// 面板采用圆角容器设计，带有阴影效果以提升视觉层次感。
///
/// # 参数
///
/// - `tab`: 思维导图标签页状态引用，包含当前背景设置、主题配置等信息
/// - `panel_w`: 面板宽度（像素）
/// - `panel_h`: 面板高度（像素）
///
/// # 返回值
///
/// 返回一个 [`Element`]，可嵌入到 iced UI 树中渲染
///
/// # 布局结构
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │ [⊘] [■][■][■][■][■][■][■][■][■][■][■] │
/// │  ↑    ↑                                │
/// │  │    └── 预设颜色色块（11种）          │
/// │  └── 跟随主题按钮                       │
/// └─────────────────────────────────────────┘
/// ```
///
/// # 交互行为
///
/// - 点击跟随主题按钮：发送 `SetBackground(None)`，启用跟随主题模式
/// - 点击色块：发送 `SetBackground(Some(color))`，设置固定背景色
pub(in super::super) fn background_panel(
    tab: &MindMapTab,
    panel_w: f32,
    panel_h: f32,
) -> Element<'static, Message> {
    // 跟随主题按钮的尺寸（正方形）
    let follow_size = 24.0;
    // 颜色色块的尺寸（正方形）
    let swatch_size = 16.0;
    // 色块之间的间距
    let swatch_gap = 5.0;

    // 预设背景色调色板（RGBA 格式，u32 存储）
    //
    // 颜色从左到右依次为：
    // - 0xFFFFFFFF: 纯白色
    // - 0xF3F4F6FF: 浅灰色（Gray 100）
    // - 0xFAFAF9FF: 暖白色（Stone 50）
    // - 0xFFF1F2FF: 浅粉红（Rose 50）
    // - 0xFEF3C7FF: 浅琥珀色（Amber 50）
    // - 0xECFDF5FF: 浅翠绿色（Emerald 50）
    // - 0xEFF6FFFF: 浅蓝色（Blue 50）
    // - 0x374151FF: 深灰色（Gray 700）
    // - 0x1F2937FF: 更深灰色（Gray 800）
    // - 0x111827FF: 接近黑色（Gray 900）
    // - 0x0B1220FF: 深蓝黑色（自定义深色）
    let palette: [u32; 11] = [
        0xFFFFFFFF, 0xF3F4F6FF, 0xFAFAF9FF, 0xFFF1F2FF, 0xFEF3C7FF, 0xECFDF5FF, 0xEFF6FFFF,
        0x374151FF, 0x1F2937FF, 0x111827FF, 0x0B1220FF,
    ];

    // 获取当前激活的背景颜色（None 表示跟随主题）
    let active_bg = tab.background;

    // 按钮样式生成闭包
    //
    // 根据按钮状态生成对应的样式，包括：
    // - 背景颜色：优先使用传入颜色，否则使用主题色
    // - 边框：激活状态使用主色调边框，否则使用背景色边框
    // - 文字颜色：使用主题文字颜色
    //
    // 参数：
    // - active: 按钮是否处于激活状态（被选中）
    // - is_swatch: 是否为色块按钮（影响边框颜色计算）
    // - bg: 可选的背景颜色
    let btn_style = |active: bool, is_swatch: bool, bg: Option<Color>| {
        move |t: &Theme, status| {
            let p = t.extended_palette();
            // 检测鼠标是否悬停在按钮上
            let hovered = status == iced::widget::button::Status::Hovered;
            // 根据激活状态选择边框颜色
            let border_color =
                if active { p.primary.base.color } else { p.background.strong.color };
            iced::widget::button::Style {
                // 背景颜色优先级：传入颜色 > 悬停/激活时背景色 > 非色块时弱背景色
                background: bg
                    .map(Background::Color)
                    .or_else(|| (hovered || active).then_some(p.background.strong.color.into()))
                    .or_else(|| (!is_swatch).then_some(p.background.weak.color.into())),
                border: Border {
                    // 激活状态使用更粗的边框
                    width: if active { 2.0 } else { 1.0 },
                    // 色块按钮的边框需要考虑背景明暗度
                    color: if is_swatch && !active {
                        // 根据颜色亮度选择边框颜色：深色背景用浅边框，浅色背景用深边框
                        if bg.is_some_and(|c| c.r + c.g + c.b < 1.0) {
                            Color::from_rgba8(255, 255, 255, 0.18)
                        } else {
                            Color::from_rgba8(0, 0, 0, 0.10)
                        }
                    } else {
                        border_color
                    },
                    // 圆角边框，形成圆形/胶囊形外观
                    radius: 999.0.into(),
                },
                text_color: t.palette().text,
                ..Default::default()
            }
        }
    };

    // 构建跟随主题按钮
    let follow_btn = {
        // 从当前主题解析背景颜色
        let theme_bg = rgba_u32_to_color(
            resolve_theme(&tab.theme_group, tab.theme_variant, &tab.custom_themes).background_color,
        );
        // 激活状态：当前没有设置固定背景色（即跟随主题）
        let active = active_bg.is_none();
        // 是否跟随主题背景的标志
        let follow_theme_bg = tab.follow_theme_background;
        button(
            container(text("⊘").size(18).line_height(1.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        // 点击时发送设置背景为 None 的消息，启用跟随主题模式
        .on_press(Message::MindMapTool(MindMapMessage::SetBackground(None)))
        .width(Length::Fixed(follow_size))
        .height(Length::Fixed(follow_size))
        .padding(0)
        .style(move |t: &Theme, status| {
            let p = t.extended_palette();
            // 确定跟随主题按钮的背景色
            let follow_bg = if follow_theme_bg { theme_bg } else { p.background.base.color };
            // 判断背景是否为深色（RGB 通道之和小于 1.0）
            let is_dark = follow_bg.r + follow_bg.g + follow_bg.b < 1.0;
            let hovered = status == iced::widget::button::Status::Hovered;
            // 根据状态和背景明暗度计算边框颜色
            let border_color = if active {
                p.primary.base.color
            } else if hovered {
                p.background.strong.color
            } else if is_dark {
                Color::from_rgba8(255, 255, 255, 0.18)
            } else {
                Color::from_rgba8(0, 0, 0, 0.10)
            };
            // 根据背景明暗度选择合适的文字颜色
            let text_c = if is_dark {
                Color::from_rgba8(255, 255, 255, 0.72)
            } else {
                Color::from_rgba8(107, 114, 128, 0.85)
            };
            iced::widget::button::Style {
                background: Some(Background::Color(follow_bg)),
                border: Border {
                    width: if active { 2.0 } else { 1.0 },
                    color: border_color,
                    radius: 999.0.into(),
                },
                text_color: text_c,
                ..Default::default()
            }
        })
    };

    // 创建颜色色块按钮
    //
    // 参数：
    // - rgba: 颜色值（RGBA 格式的 u32）
    //
    // 返回值：
    // 返回配置好的按钮，点击时发送设置该颜色的消息
    let swatch_btn = |rgba: u32| {
        // 检查该色块是否处于激活状态
        let active = active_bg == Some(rgba);
        let c = rgba_u32_to_color(rgba);
        button(container(Space::new()).width(Length::Fill).height(Length::Fill))
            // 点击时发送设置该背景色的消息
            .on_press(Message::MindMapTool(MindMapMessage::SetBackground(Some(rgba))))
            .width(Length::Fixed(swatch_size))
            .height(Length::Fixed(swatch_size))
            .padding(0)
            .style(btn_style(active, true, Some(c)))
    };

    // 构建色板行容器，包含所有预设颜色
    let mut swatches = row![].spacing(swatch_gap).align_y(Alignment::Center);
    for rgba in palette {
        swatches = swatches.push(swatch_btn(rgba));
    }

    // 组装内部布局：跟随主题按钮 + 色板
    let inner =
        row![follow_btn, swatches].spacing(8).height(Length::Fill).align_y(Alignment::Center);

    // 包装在外层容器中，添加样式
    container(inner)
        .padding([4, 8])
        .width(Length::Fixed(panel_w))
        .height(Length::Fixed(panel_h))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                // 容器背景使用主题基础背景色
                background: Some(palette.background.base.color.into()),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color,
                    // 圆角容器
                    radius: 12.0.into(),
                },
                // 添加阴影以增强视觉层次
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
