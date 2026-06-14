//! Markdown 工具视图模块
//!
//! 本模块提供 Markdown 编辑器的完整视图界面，包括：
//! - 编辑器工具栏（粗体、斜体、标题、代码块等格式化按钮）
//! - 视图模式切换（编辑/预览/分屏）
//! - 远程图片加载和本地图片插入
//! - HTML 转 Markdown 功能
//! - 流式渲染开关
//!
//! # 主要组件
//!
//! - [`view`] - 主视图函数，渲染完整的 Markdown 工具界面
//! - `markdown::Viewer` 实现 - 处理 Markdown 中的链接点击和图片渲染

use crate::app::components::editor_toolbar;
use crate::app::components::markdown_editor::{MarkdownViewMode, mode_switch};
use crate::app::components::system_settings_common::{
    primary_action_btn_style, round_icon_btn_style, rounded_action_btn_style,
    settings_muted_text_style, settings_panel, settings_panel_style, settings_text_editor_style,
    settings_text_input_style,
};
use crate::app::components::text_editor_context_menu::{
    TextEditorContextMenuMessages, TextEditorContextMenuState, wrap_with_context_menu,
};
use crate::app::components::text_editor_scroll_panel::{
    TextEditorScrollPanelMetrics, text_editor_scroll_panel,
};
use crate::app::message::{MarkdownToolMessage, ViewMessage};
use crate::app::{App, Message};
use iced::widget::image::Handle as ImageHandle;
use iced::widget::scrollable::Direction;
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{
    Image, Space, button, column, container, markdown, mouse_area, responsive, row, scrollable,
    svg, text, text_editor, toggler,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

/// 将编辑器动作转换为消息
///
/// 这是一个辅助函数，用于将底层的编辑器动作封装为应用层消息。
///
/// # 参数
///
/// * `a` - 文本编辑器产生的动作
///
/// # 返回值
///
/// 返回封装后的 `Message::MarkdownTool` 消息
fn on_editor_action(a: text_editor::Action) -> Message {
    Message::MarkdownTool(MarkdownToolMessage::EditorAction(a))
}

/// 将视图模式切换转换为消息
///
/// 这是一个辅助函数，用于将视图模式变更封装为应用层消息。
///
/// # 参数
///
/// * `mode` - 目标视图模式（编辑/预览/分屏）
///
/// # 返回值
///
/// 返回封装后的 `Message::MarkdownTool` 消息
fn on_mode_change(mode: MarkdownViewMode) -> Message {
    Message::MarkdownTool(MarkdownToolMessage::SetViewMode(mode))
}

#[derive(Debug, Clone, Copy)]
enum MarkdownBadgeTone {
    Loading,
    Success,
    Idle,
}

#[derive(Debug, Clone, Copy)]
enum MarkdownActionTone {
    Default,
    Primary,
    Success,
    Danger,
}

fn is_dark_theme(theme: &Theme) -> bool {
    theme.palette().background.r + theme.palette().background.g + theme.palette().background.b < 1.5
}

fn danger_color(theme: &Theme) -> Color {
    if is_dark_theme(theme) {
        Color::from_rgba8(0xF5, 0xA3, 0xA3, 0.96)
    } else {
        Color::from_rgba8(0xB4, 0x23, 0x18, 0.94)
    }
}

fn chrome_chip_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let is_dark = is_dark_theme(theme);

    iced::widget::container::Style {
        background: Some(Background::Color(if is_dark {
            palette.background.weak.color.scale_alpha(0.24)
        } else {
            Color::WHITE.scale_alpha(0.82)
        })),
        border: Border {
            width: 1.0,
            color: if is_dark {
                palette.background.strong.color.scale_alpha(0.82)
            } else {
                Color::from_rgba8(15, 23, 42, 0.08)
            },
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

fn editor_surface_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();

    iced::widget::container::Style {
        background: Some(palette.background.base.color.into()),
        border: Border { width: 1.0, color: palette.background.strong.color, radius: 10.0.into() },
        ..Default::default()
    }
}

fn tooltip_card_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let is_dark = is_dark_theme(theme);

    iced::widget::container::Style {
        background: Some(Background::Color(if is_dark {
            palette.background.base.color.scale_alpha(0.98)
        } else {
            Color::from_rgba8(255, 255, 255, 0.96)
        })),
        border: Border {
            width: 1.0,
            color: if is_dark {
                palette.background.strong.color.scale_alpha(0.88)
            } else {
                Color::from_rgba8(15, 23, 42, 0.08)
            },
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

fn build_metric_badge<'a>(label: impl Into<String>) -> Element<'a, Message> {
    let label = label.into();

    container(text(label).size(12).style(settings_muted_text_style))
        .padding([6, 10])
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = is_dark_theme(theme);

            iced::widget::container::Style {
                background: Some(Background::Color(if is_dark {
                    palette.background.weak.color.scale_alpha(0.34)
                } else {
                    Color::from_rgba8(248, 250, 252, 0.98)
                })),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        palette.background.strong.color.scale_alpha(0.80)
                    } else {
                        Color::from_rgba8(148, 163, 184, 0.18)
                    },
                    radius: 999.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_status_badge<'a>(
    label: impl Into<String>,
    tone: MarkdownBadgeTone,
) -> Element<'a, Message> {
    let label = label.into();

    container(text(label).size(12).style(move |theme: &Theme| {
        let is_dark = is_dark_theme(theme);

        iced::widget::text::Style {
            color: Some(match tone {
                MarkdownBadgeTone::Loading | MarkdownBadgeTone::Success => Color::WHITE,
                MarkdownBadgeTone::Idle if is_dark => theme.palette().text.scale_alpha(0.92),
                MarkdownBadgeTone::Idle => Color::from_rgba8(71, 85, 105, 1.0),
            }),
        }
    }))
    .padding([8, 12])
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = is_dark_theme(theme);

        iced::widget::container::Style {
            background: Some(Background::Color(match tone {
                MarkdownBadgeTone::Loading => Color::from_rgba8(37, 99, 235, 0.92),
                MarkdownBadgeTone::Success => Color::from_rgba8(22, 163, 74, 0.92),
                MarkdownBadgeTone::Idle if is_dark => {
                    palette.background.strong.color.scale_alpha(0.82)
                }
                MarkdownBadgeTone::Idle => Color::from_rgba8(241, 245, 249, 0.96),
            })),
            border: Border {
                width: 1.0,
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.88)
                } else {
                    Color::from_rgba8(148, 163, 184, 0.22)
                },
                radius: 999.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn toolbar_icon_color(theme: &Theme, tone: MarkdownActionTone) -> Color {
    match tone {
        MarkdownActionTone::Default => theme.palette().text.scale_alpha(0.88),
        MarkdownActionTone::Primary => theme.palette().primary.scale_alpha(0.96),
        MarkdownActionTone::Success => theme.extended_palette().success.base.color,
        MarkdownActionTone::Danger => danger_color(theme),
    }
}

fn toolbar_button_style(
    theme: &Theme,
    status: button::Status,
    tone: MarkdownActionTone,
) -> button::Style {
    let mut style = round_icon_btn_style(theme, status);

    if matches!(tone, MarkdownActionTone::Default) {
        return style;
    }

    let is_dark = is_dark_theme(theme);
    let accent = match tone {
        MarkdownActionTone::Primary => theme.palette().primary,
        MarkdownActionTone::Success => theme.extended_palette().success.base.color,
        MarkdownActionTone::Danger => danger_color(theme),
        MarkdownActionTone::Default => theme.palette().text,
    };

    let alpha = match status {
        button::Status::Hovered => {
            if is_dark {
                0.18
            } else {
                0.10
            }
        }
        button::Status::Pressed => {
            if is_dark {
                0.24
            } else {
                0.14
            }
        }
        _ => {
            if is_dark {
                0.12
            } else {
                0.06
            }
        }
    };

    style.background = Some(Background::Color(accent.scale_alpha(alpha)));
    style.border.color = accent.scale_alpha(if is_dark { 0.38 } else { 0.22 });
    style
}

fn build_toolbar_button(
    icon: crate::app::assets::Icon,
    tip: &'static str,
    msg: MarkdownToolMessage,
    tone: MarkdownActionTone,
) -> Element<'static, Message> {
    let button = button(editor_toolbar::icon_svg(icon).style(move |theme: &Theme, _status| {
        svg::Style { color: Some(toolbar_icon_color(theme, tone)) }
    }))
    .padding(8)
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .style(move |theme: &Theme, status| toolbar_button_style(theme, status, tone))
    .on_press(Message::MarkdownTool(msg));

    let tip_content = container(text(tip).size(12).style(settings_muted_text_style))
        .padding([6, 10])
        .style(tooltip_card_style);

    Tooltip::new(button, tip_content, TooltipPosition::Bottom).gap(8).into()
}

fn build_panel_card<'a>(
    title: &'a str,
    badge: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    settings_panel(
        column![
            row![text(title).size(14), Space::new().width(Length::Fill), badge]
                .align_y(Alignment::Center),
            content,
        ]
        .spacing(12)
        .height(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn build_markdown_image<'a>(app: &App, src: String) -> Element<'a, Message> {
    // 处理远程图片 (HTTP/HTTPS)
    if src.starts_with("http://") || src.starts_with("https://") {
        // 检查图片是否已缓存（已加载过）
        if let Some(handle) = app.markdown_tool_remote_images.get(&src).cloned() {
            return container(Image::new(handle).width(Length::Fill))
                .padding(10)
                .width(Length::Fill)
                .style(editor_surface_style)
                .into();
        }

        // 检查图片是否正在加载中
        if app.markdown_tool_remote_images_loading.contains(&src) {
            return container(text("远程图片加载中...").size(12).style(settings_muted_text_style))
                .padding([8, 10])
                .style(editor_surface_style)
                .into();
        }

        // 显示加载按钮，让用户主动触发图片加载
        return container(
            button(text("加载远程图片"))
                .style(rounded_action_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::FetchRemoteImage(src))),
        )
        .padding([8, 10])
        .style(editor_surface_style)
        .into();
    }

    // 处理本地文件图片
    // 移除 file:/// 或 file:// 前缀，获取实际文件路径
    let path_str = src
        .strip_prefix("file:///")
        .or_else(|| src.strip_prefix("file://"))
        .unwrap_or(src.as_str());

    let path = std::path::Path::new(path_str);

    // 如果文件存在，直接加载显示
    if path.exists() {
        return container(Image::new(ImageHandle::from_path(path)).width(Length::Fill))
            .padding(10)
            .width(Length::Fill)
            .style(editor_surface_style)
            .into();
    }

    // 文件不存在，显示原始 Markdown 语法
    container(text(format!("![image]({src})")).size(12).style(settings_muted_text_style))
        .padding([8, 10])
        .style(editor_surface_style)
        .into()
}

/// 为 App 实现 Markdown 查看器 trait
///
/// 该实现定义了 Markdown 渲染时的自定义行为：
/// - 链接点击：在外部浏览器中打开
/// - 图片渲染：支持远程图片（需手动加载）和本地文件图片
impl<'a> markdown::Viewer<'a, Message> for App {
    /// 处理链接点击事件
    ///
    /// 当用户点击 Markdown 中的链接时，在外部浏览器中打开该 URL。
    ///
    /// # 参数
    ///
    /// * `url` - 被点击的链接 URI
    ///
    /// # 返回值
    ///
    /// 返回打开外部 URL 的消息
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::View(ViewMessage::OpenUrlExternal(url.to_string()))
    }

    /// 渲染 Markdown 中的图片
    ///
    /// 根据图片 URL 的类型采用不同的渲染策略：
    /// - **远程图片 (http/https)**：显示"加载远程图片"按钮，点击后异步加载
    /// - **本地文件 (file://)**：直接从文件系统读取并显示
    /// - **无效路径**：显示原始 Markdown 语法
    ///
    /// # 参数
    ///
    /// * `_settings` - Markdown 渲染设置（当前未使用）
    /// * `url` - 图片的 URI
    /// * `_title` - 图片标题（当前未使用）
    /// * `_alt` - 图片替代文本（当前未使用）
    ///
    /// # 返回值
    ///
    /// 返回图片元素的 UI 组件
    fn image(
        &self,
        _settings: markdown::Settings,
        url: &'a markdown::Uri,
        _title: &'a str,
        _alt: &markdown::Text,
    ) -> Element<'a, Message> {
        let src = url.to_string();
        build_markdown_image(self, src)
    }
}

/// 渲染 Markdown 工具的主视图
///
/// 该函数构建完整的 Markdown 编辑器界面，包含：
/// - 标题栏和模式切换
/// - 格式化工具栏（粗体、斜体、标题、代码块等）
/// - 编辑器/预览区域
/// - 模态对话框（HTML转换、图片插入）
///
/// # 参数
///
/// * `app` - 应用状态引用，包含编辑器内容和配置
///
/// # 返回值
///
/// 返回完整的 UI 元素树
///
/// # 界面结构
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │ 标题        [模式切换] [流式开关] [通知] │
/// ├─────────────────────────────────────────┤
/// │ [HTML][粗体][斜体][删除线][标题]...     │
/// ├─────────────────────────────────────────┤
/// │                                         │
/// │         编辑器 / 预览区域               │
/// │                                         │
/// └─────────────────────────────────────────┘
/// ```
pub fn view(app: &App) -> Element<'_, Message> {
    let theme_now = app.theme();
    let copy_feedback_visible = app.markdown_tool_notification.as_deref() == Some("已复制");

    let view_mode_label = match app.markdown_tool_view_mode {
        MarkdownViewMode::Edit => "编辑模式",
        MarkdownViewMode::Preview => "预览模式",
        MarkdownViewMode::Split => "分屏模式",
    };

    let status_badge = if !app.markdown_tool_remote_images_loading.is_empty() {
        build_status_badge(
            format!("图片加载中 {}", app.markdown_tool_remote_images_loading.len()),
            MarkdownBadgeTone::Loading,
        )
    } else if let Some(msg) = &app.markdown_tool_notification {
        build_status_badge(msg.as_str().to_owned(), MarkdownBadgeTone::Success)
    } else {
        build_status_badge("已就绪", MarkdownBadgeTone::Idle)
    };

    // 构建格式化工具栏
    // 包含所有 Markdown 格式化功能的快捷按钮
    let toolbar = settings_panel(
        row![
            build_toolbar_button(
                crate::app::assets::Icon::Html,
                "HTML转Markdown",
                MarkdownToolMessage::OpenHtml2Md,
                MarkdownActionTone::Primary,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::TypeBold,
                "粗体",
                MarkdownToolMessage::InsertBold,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::TypeItalic,
                "斜体",
                MarkdownToolMessage::InsertItalic,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::TypeStrikethrough,
                "删除线",
                MarkdownToolMessage::InsertStrike,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::Markdown,
                "标题",
                MarkdownToolMessage::InsertHeading,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::ChatTextFill,
                "引用",
                MarkdownToolMessage::InsertQuote,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::Code,
                "代码块",
                MarkdownToolMessage::InsertCodeBlock,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::HandIndex,
                "链接",
                MarkdownToolMessage::InsertLink,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::Image,
                "图片",
                MarkdownToolMessage::InsertImage,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                crate::app::assets::Icon::Grid1x2,
                "表格",
                MarkdownToolMessage::InsertTable,
                MarkdownActionTone::Default,
            ),
            build_toolbar_button(
                if copy_feedback_visible {
                    crate::app::assets::Icon::Check
                } else {
                    crate::app::assets::Icon::Clipboard
                },
                if copy_feedback_visible { "已复制" } else { "复制" },
                MarkdownToolMessage::Copy,
                if copy_feedback_visible {
                    MarkdownActionTone::Success
                } else {
                    MarkdownActionTone::Default
                },
            ),
            build_toolbar_button(
                crate::app::assets::Icon::Trash,
                "清空",
                MarkdownToolMessage::Clear,
                MarkdownActionTone::Danger,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .wrap(),
    );

    // 流式渲染开关
    // 启用后 Markdown 内容将实时渲染预览
    let stream_toggle = row![
        toggler(app.markdown_tool_stream_enabled)
            .on_toggle(|b| Message::MarkdownTool(MarkdownToolMessage::ToggleStream(b))),
        text("流式预览").size(12).style(settings_muted_text_style)
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // 构建标题栏
    // 包含标题、模式切换、流式开关和通知消息
    let header = container(
        row![
            row![text("Markdown编辑器").size(20), build_metric_badge(view_mode_label)]
                .spacing(10)
                .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            container(mode_switch(app.markdown_tool_view_mode, on_mode_change))
                .padding([8, 10])
                .style(chrome_chip_style),
            container(stream_toggle).padding([8, 10]).style(chrome_chip_style),
            status_badge,
        ]
        .width(Length::Fill)
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([18, 20])
    .width(Length::Fill)
    .style(settings_panel_style);

    let body = build_body(app, &theme_now, on_editor_action);
    let body: Element<'_, Message> = body;

    // 构建基础容器
    // 包含标题栏、工具栏和编辑器主体
    let base = container(column![header, toolbar, body].spacing(16).padding([18, 24]))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(theme.extended_palette().background.base.color.into()),
            ..Default::default()
        });

    // 处理 HTML 转 Markdown 模态框
    if app.markdown_tool_show_html2md {
        // 半透明遮罩层，点击关闭模态框
        let overlay =
            mouse_area(container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                |_| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                },
            ))
            .on_press(Message::MarkdownTool(MarkdownToolMessage::CloseHtml2Md));

        // HTML 输入编辑器
        let html_editor = text_editor(&app.markdown_tool_html_editor)
            .placeholder("请输入 HTML 片段")
            .on_action(|a| Message::MarkdownTool(MarkdownToolMessage::HtmlEditorAction(a)))
            .height(Length::Fixed(260.0))
            .padding(12)
            .font(iced::Font::with_name("Noto Sans CJK SC"))
            .style(settings_text_editor_style);

        // 模态框标题栏
        let modal_header = row![
            row![text("HTML 转 Markdown").size(16), build_metric_badge("转换")]
                .spacing(8)
                .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            button(editor_toolbar::icon_svg(crate::app::assets::Icon::X))
                .padding(6)
                .style(round_icon_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::CloseHtml2Md))
        ]
        .align_y(Alignment::Center);

        // 模态框底部操作区
        let modal_footer = row![
            text("支持粘贴完整 HTML 片段").size(12).style(settings_muted_text_style),
            Space::new().width(Length::Fill),
            button(text("转换成markdown"))
                .style(primary_action_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::ConvertHtmlToMarkdown))
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // 模态框容器
        let modal = container(
            column![
                modal_header,
                text("将 HTML 片段快速转换为可继续编辑的 Markdown。")
                    .size(12)
                    .style(settings_muted_text_style),
                html_editor,
                modal_footer,
            ]
            .spacing(14),
        )
        .padding([18, 20])
        .width(Length::Fixed(680.0))
        .style(settings_panel_style);

        // 使用层叠布局显示模态框
        return iced::widget::stack![
            base,
            overlay,
            container(modal)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        ]
        .into();
    }

    // 处理图片插入模态框
    if app.markdown_tool_show_image {
        // 半透明遮罩层，点击关闭模态框
        let overlay =
            mouse_area(container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                |_| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                },
            ))
            .on_press(Message::MarkdownTool(MarkdownToolMessage::CloseImage));

        // 模态框标题栏
        let modal_header = row![
            row![text("插入图片").size(16), build_metric_badge("本地 / 远程")]
                .spacing(8)
                .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            button(editor_toolbar::icon_svg(crate::app::assets::Icon::X))
                .padding(6)
                .style(round_icon_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::CloseImage))
        ]
        .align_y(Alignment::Center);

        // 图片 URL 输入框
        let url_input =
            iced::widget::text_input("输入图片URL...", &app.markdown_tool_image_url_input)
                .on_input(|v| Message::MarkdownTool(MarkdownToolMessage::ImageUrlChanged(v)))
                .padding([10, 12])
                .style(settings_text_input_style)
                .width(Length::Fill);

        // 图片插入操作按钮
        let url_actions = row![
            button(text("插入网址图片"))
                .style(primary_action_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::InsertImageFromUrl)),
            button(text("选择本地图片"))
                .style(rounded_action_btn_style)
                .on_press(Message::MarkdownTool(MarkdownToolMessage::PickImageFile)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // 模态框容器
        let modal = container(
            column![
                modal_header,
                text("支持远程 URL 和本地文件，插入后会保持 Markdown 语法。")
                    .size(12)
                    .style(settings_muted_text_style),
                url_input,
                url_actions,
            ]
            .spacing(14),
        )
        .padding([18, 20])
        .width(Length::Fixed(640.0))
        .style(settings_panel_style);

        // 使用层叠布局显示模态框
        return iced::widget::stack![
            base,
            overlay,
            container(modal)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        ]
        .into();
    }

    // 返回基础视图（无模态框）
    base.into()
}

fn build_body<'a>(
    app: &'a App,
    theme: &Theme,
    on_editor_action: fn(text_editor::Action) -> Message,
) -> Element<'a, Message> {
    let preview = markdown::view_with(app.markdown_tool_content.items(), theme, app);
    let preview = scrollable(preview)
        .direction(Direction::Vertical(scrollable::Scrollbar::new().width(4).scroller_width(4)))
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill);

    let preview_panel: Element<'a, Message> = container(preview)
        .padding([12, 14])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(editor_surface_style)
        .into();

    let preview_card = build_panel_card(
        "预览区",
        build_metric_badge(if app.markdown_tool_stream_enabled {
            "实时预览"
        } else {
            "预览面板"
        }),
        preview_panel,
    );

    let editor_panel = responsive(move |size| build_editor_panel(app, size, on_editor_action));
    let editor_panel: Element<'a, Message> = editor_panel.into();
    let editor_card = build_panel_card(
        "编辑区",
        build_metric_badge(format!("{} 行", app.markdown_tool_editor.line_count().max(1))),
        editor_panel,
    );

    match app.markdown_tool_view_mode {
        MarkdownViewMode::Edit => editor_card,
        MarkdownViewMode::Preview => preview_card,
        MarkdownViewMode::Split => {
            row![editor_card, preview_card].spacing(16).height(Length::Fill).into()
        }
    }
}

fn build_editor_panel<'a>(
    app: &'a App,
    size: Size,
    on_editor_action: fn(text_editor::Action) -> Message,
) -> Element<'a, Message> {
    let is_dark = is_dark_theme(&app.theme());
    let highlight_theme = if is_dark {
        iced::highlighter::Theme::Base16Ocean
    } else {
        iced::highlighter::Theme::InspiredGitHub
    };

    let editor = text_editor(&app.markdown_tool_editor)
        .id(app.markdown_tool_editor_id.clone())
        .placeholder("输入 Markdown")
        .on_action(on_editor_action)
        .height(Length::Fill)
        .padding(10)
        .font(iced::Font::with_name("Noto Sans CJK SC"))
        .highlight("markdown", highlight_theme)
        .style(|theme: &Theme, _status| {
            let palette = theme.extended_palette();
            text_editor::Style {
                background: palette.background.base.color.into(),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
                value: theme.palette().text,
                selection: theme.palette().primary.scale_alpha(0.30),
                placeholder: theme.palette().text.scale_alpha(0.55),
            }
        });

    let editor = wrap_with_context_menu(
        editor,
        TextEditorContextMenuState {
            open: app.markdown_tool_context_menu_open,
            position: app.markdown_tool_context_menu_pos,
        },
        |point| {
            Message::MarkdownTool(MarkdownToolMessage::OpenContextMenu { x: point.x, y: point.y })
        },
        TextEditorContextMenuMessages {
            close: Message::MarkdownTool(MarkdownToolMessage::CloseContextMenu),
            copy: Message::MarkdownTool(MarkdownToolMessage::ContextMenuCopy),
            cut: Message::MarkdownTool(MarkdownToolMessage::ContextMenuCut),
            paste: Message::MarkdownTool(MarkdownToolMessage::ContextMenuPaste),
            delete: Message::MarkdownTool(MarkdownToolMessage::ContextMenuDelete),
        },
    );

    text_editor_scroll_panel(
        editor,
        size,
        TextEditorScrollPanelMetrics {
            viewport_padding: 24.0,
            line_height: app.current_line_height,
            line_count: app.markdown_tool_editor.line_count(),
            scroll_top_line: app.markdown_tool_scroll_top_line,
        },
        |delta, viewport_height| {
            Message::MarkdownTool(MarkdownToolMessage::EditorWheelScrolled {
                delta,
                viewport_height,
            })
        },
        |top_line, viewport_height| {
            Message::MarkdownTool(MarkdownToolMessage::ScrollbarChanged {
                top_line,
                viewport_height,
            })
        },
    )
}
#[cfg(test)]
#[path = "markdown_tool_tests.rs"]
mod markdown_tool_tests;
