//! 文件搜索结果组件
//!
//! 该模块提供文件内容搜索功能的 UI 组件，包括搜索结果标签页和结果列表的构建。
//! 用户可以在项目范围内搜索关键字，并查看所有匹配结果。
//!
//! 主要功能：
//! - 多标签页支持：允许同时打开多个搜索会话
//! - 高级搜索选项：区分大小写、全词匹配、正则表达式
//! - 搜索结果预览：高亮显示匹配内容
//! - 上下文菜单：右键点击结果项可打开文件菜单
//!
//! # 组件结构
//!
//! - [`build_find_results_tabs`]: 构建搜索标签页栏，显示所有打开的搜索会话
//! - [`build_find_results_list`]: 构建搜索结果列表，包含搜索控件和匹配项列表

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    Space, button, checkbox, column, container, row, scrollable, text, text_editor,
};
use iced::{Background, Color, Element, Length, Theme};

use crate::app::assets::Icon;
use crate::app::components::file_tree::icons::{file_icon_for, static_icon_svg, themed_icon_svg};
use crate::app::components::file_tree::menu::build_file_tree_menu;
use crate::app::components::file_tree::widgets::RightClickArea;
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::message::project::ProjectMessage;
use crate::app::{App, Message, message};

/// 搜索匹配高亮样式枚举
///
/// 定义搜索结果中匹配文本的高亮显示风格，参考 VSCode 的两种常见高亮配色。
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FindMatchHighlightStyle {
    /// VSCode 风格的黄色高亮
    /// - 深色主题：金黄色半透明
    /// - 浅色主题：淡黄色半透明
    VscodeYellow,
    /// VSCode 风格的蓝色高亮
    /// - 深色主题：天蓝色半透明
    /// - 浅色主题：淡蓝色半透明
    VscodeBlue,
}

/// 当前使用的高亮样式
const FIND_MATCH_HIGHLIGHT_STYLE: FindMatchHighlightStyle = FindMatchHighlightStyle::VscodeYellow;

/// 搜索结果数量上限
///
/// 为避免性能问题，当搜索结果达到此数量时将停止继续搜索。
const FIND_RESULTS_LIMIT: usize = 3000;

/// 将绝对路径转换为相对于项目根目录的相对路径
///
/// # 参数
///
/// - `project_root`: 项目根目录的绝对路径
/// - `path`: 需要转换的绝对路径
///
/// # 返回值
///
/// 返回相对路径字符串，路径分隔符统一为 `/`。如果无法剥离前缀，则返回原路径。
///
/// # 示例
///
/// ```ignore
/// let rel = to_relative_path("/home/user/project", "/home/user/project/src/main.rs");
/// // rel = "src/main.rs"
/// ```
pub(super) fn to_relative_path(project_root: &str, path: &str) -> String {
    let normalized_root = project_root.trim().replace('\\', "/").trim_end_matches('/').to_string();
    let normalized_path = path.trim().replace('\\', "/");
    if let Some(relative) = normalized_path
        .strip_prefix(&normalized_root)
        .map(|value| value.trim_start_matches('/'))
        .filter(|value| !value.is_empty())
    {
        return relative.to_string();
    }
    normalized_path
}

/// 根据主题计算搜索匹配高亮的背景颜色
///
/// 该函数根据当前主题的明暗模式，选择合适的高亮颜色，确保在深色和浅色主题下
/// 都有良好的可读性和对比度。
///
/// # 参数
///
/// - `theme`: 当前的 iced 主题引用
///
/// # 返回值
///
/// 返回用于高亮匹配文本的背景颜色
///
/// # 算法说明
///
/// 1. 使用亮度公式计算背景颜色的感知亮度：
///    `luma = 0.2126 * R + 0.7152 * G + 0.0722 * B`
/// 2. 根据 `luma` 值判断是深色还是浅色主题（阈值为 0.5）
/// 3. 根据配置的高亮样式（黄色/蓝色）返回对应的颜色
pub(super) fn find_match_highlight_bg(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    let base = p.background.base.color;
    // 使用 ITU-R BT.709 标准计算感知亮度
    let luma = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
    // 亮度低于 0.5 认为是深色主题
    let is_dark = luma < 0.5;

    match FIND_MATCH_HIGHLIGHT_STYLE {
        FindMatchHighlightStyle::VscodeYellow => {
            if is_dark {
                // 深色主题：金黄色，透明度 34%
                Color::from_rgba8(0xEA, 0xD5, 0x6B, 0.34)
            } else {
                // 浅色主题：淡黄色，透明度 70%
                Color::from_rgba8(0xFF, 0xE8, 0x9A, 0.70)
            }
        }
        FindMatchHighlightStyle::VscodeBlue => {
            if is_dark {
                // 深色主题：天蓝色，透明度 34%
                Color::from_rgba8(0x4F, 0x99, 0xFF, 0.34)
            } else {
                // 浅色主题：淡蓝色，透明度 78%
                Color::from_rgba8(0xB8, 0xD7, 0xFF, 0.78)
            }
        }
    }
}

/// 构建搜索结果标签页栏
///
/// 创建一个水平滚动的标签页栏，显示所有打开的搜索会话。每个标签页包含：
/// - 搜索查询字符串和匹配数量
/// - 点击切换到该搜索会话
/// - 关闭按钮用于关闭该标签页
///
/// # 参数
///
/// - `app`: 应用状态引用，包含所有搜索标签页数据和当前活动标签页 ID
///
/// # 返回值
///
/// 返回包含所有标签页的可滚动 UI 元素
///
/// # UI 结构
///
/// ```text
/// [标签1 (10)] [标签2 (5)] [标签3 (8)] ...
/// ```
///
/// 每个标签包含：
/// - 标题按钮：显示查询文本和匹配数量，点击切换活动标签
/// - 关闭按钮：点击关闭该搜索会话
pub fn build_find_results_tabs(app: &App) -> Element<'_, Message> {
    let mut tabs = row![].spacing(4).width(Length::Fill);

    for tab in &app.find_results_tabs {
        // 判断当前标签页是否为活动状态
        let active = app.active_find_results_tab_id.as_deref() == Some(tab.id.as_str());
        let tab_id = tab.id.clone();
        let close_id = tab.id.clone();
        // 标题格式：查询文本 (匹配数量)
        let title = format!("{} ({})", tab.query, tab.matches.len());

        // 构建标题按钮
        let title_btn = button(text(title).size(12))
            .on_press(Message::Project(ProjectMessage::FileTreeFindTabSelected(tab_id)))
            .padding([4, 8])
            .style(move |theme: &Theme, status| {
                let p = theme.extended_palette();
                let base_bg = theme.palette().background;
                // 通过 RGB 值简单判断是否为深色主题
                let is_dark = base_bg.r + base_bg.g + base_bg.b < 1.5;
                // 活动标签使用主色调，非活动标签使用背景色调
                let active_bg = if is_dark { p.primary.weak.color } else { p.primary.base.color };
                let bg = if active {
                    Some(active_bg)
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => Some(p.background.weak.color),
                        iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                        _ => None,
                    }
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    text_color: if active {
                        // 活动标签的文字颜色：深色主题用默认文字色，浅色主题用主色文字
                        if is_dark { theme.palette().text } else { p.primary.base.text }
                    } else {
                        theme.palette().text
                    },
                    border: iced::Border { radius: 6.0.into(), ..Default::default() },
                    ..Default::default()
                }
            });

        // 构建关闭按钮
        let close_btn =
            button(themed_icon_svg(Icon::X).width(Length::Fixed(12.0)).height(Length::Fixed(12.0)))
                .on_press(Message::Project(ProjectMessage::FileTreeFindTabClosed(close_id)))
                .padding([3, 4])
                .style(|theme: &Theme, status| {
                    let p = theme.extended_palette();
                    let bg = match status {
                        iced::widget::button::Status::Hovered => Some(p.background.weak.color),
                        iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                        _ => None,
                    };
                    iced::widget::button::Style {
                        background: bg.map(Background::Color),
                        text_color: theme.palette().text,
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                });

        // 将标题按钮和关闭按钮组合成一个标签
        tabs = tabs.push(
            container(row![title_btn, close_btn].spacing(2).align_y(iced::Alignment::Center))
                .padding([2, 2]),
        );
    }

    // 将标签页栏放入水平滚动容器
    scrollable(container(tabs).width(Length::Fill).padding([0, 2]))
        .direction(Direction::Horizontal(Scrollbar::new().width(4).scroller_width(4)))
        .height(Length::Fixed(34.0))
        .into()
}

/// 构建搜索结果列表面板
///
/// 创建完整的搜索界面，包含搜索控件和结果列表。该函数是搜索功能的主 UI 入口。
///
/// # 参数
///
/// - `app`: 应用状态引用，包含搜索配置、结果数据和 UI 状态
///
/// # 返回值
///
/// 返回包含搜索控件和结果列表的 UI 元素。如果没有活动搜索标签页，返回空状态提示。
///
/// # UI 结构
///
/// ```text
/// ┌────────────────────────────────────────┐
/// │ [搜索输入框]                            │
/// │ [替换输入框（可选）]                     │
/// │ [区分大小写] [全词匹配] [使用正则] [搜索] │
/// │ [错误信息（如有）]                       │
/// │ 搜索选项摘要                            │
/// ├────────────────────────────────────────┤
/// │ [文件图标] 文件路径:相对路径    行号:列号 │
/// │            匹配上下文预览（高亮显示）      │
/// │ [文件图标] 文件路径:相对路径    行号:列号 │
/// │            匹配上下文预览（高亮显示）      │
/// │ ...                                    │
/// └────────────────────────────────────────┘
/// ```
///
/// # 功能特性
///
/// - 多行搜索输入支持
/// - Enter 键触发搜索（无修饰键时）
/// - 可选的文本替换功能
/// - 搜索选项：区分大小写、全词匹配、正则表达式
/// - 结果数量限制保护
/// - 右键菜单和拖拽支持
pub fn build_find_results_list(app: &App) -> Element<'_, Message> {
    // 检查是否有活动的搜索标签页
    let Some(tab_id) = app.active_find_results_tab_id.as_deref() else {
        return column![text("无查找结果").size(13)].padding(10).into();
    };
    let Some(tab) = app.find_results_tabs.iter().find(|t| t.id == tab_id) else {
        return column![text("无查找结果").size(13)].padding(10).into();
    };

    let root = app.project_path.as_deref().unwrap_or("");
    let active_id = tab.id.clone();

    // 构建搜索输入框
    let query_input = text_editor(&tab.query_editor)
        .placeholder("搜索关键字")
        .on_action({
            let active_id = active_id.clone();
            move |a| {
                Message::Project(ProjectMessage::FileTreeFindQueryEditorAction(
                    active_id.clone(),
                    a,
                ))
            }
        })
        // 自定义键盘绑定：Enter 键触发搜索
        .key_binding({
            let active_id = active_id.clone();
            move |kp| {
                // 检测 Enter 键（包括命名键和字符形式）
                if matches!(
                    kp.key,
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                ) || matches!(kp.key, iced::keyboard::Key::Character(ref c) if c == "\n" || c == "\r")
                {
                    // 仅在无修饰键时触发搜索
                    if !kp.modifiers.shift()
                        && !kp.modifiers.control()
                        && !kp.modifiers.alt()
                        && !kp.modifiers.command()
                    {
                        Some(iced::widget::text_editor::Binding::Custom(Message::Project(
                            ProjectMessage::FileTreeFindRun(active_id.clone()),
                        )))
                    } else {
                        iced::widget::text_editor::Binding::from_key_press(kp)
                    }
                } else {
                    iced::widget::text_editor::Binding::from_key_press(kp)
                }
            }
        })
        .size(13)
        .padding([6, 8])
        .height(Length::Fixed(64.0))
        .style(|theme: &Theme, _status: text_editor::Status| {
            let p = theme.extended_palette();
            iced::widget::text_editor::Style {
                background: Background::Color(p.background.base.color),
                border: iced::Border {
                    width: 1.0,
                    color: p.background.strong.color,
                    radius: 6.0.into(),
                },
                value: theme.palette().text,
                selection: theme.palette().primary.scale_alpha(0.30),
                placeholder: p.background.strong.text.scale_alpha(0.8),
            }
        });

    // 构建替换输入框（可选功能）
    let replace_input = text_editor(&tab.replace_editor)
        .placeholder("替换词语（可选）")
        .on_action({
            let active_id = active_id.clone();
            move |a| {
                Message::Project(ProjectMessage::FileTreeFindReplaceEditorAction(
                    active_id.clone(),
                    a,
                ))
            }
        })
        .size(13)
        .padding([6, 8])
        .height(Length::Fixed(64.0))
        .style(|theme: &Theme, _status: text_editor::Status| {
            let p = theme.extended_palette();
            iced::widget::text_editor::Style {
                background: Background::Color(p.background.base.color),
                border: iced::Border {
                    width: 1.0,
                    color: p.background.strong.color,
                    radius: 6.0.into(),
                },
                value: theme.palette().text,
                selection: theme.palette().primary.scale_alpha(0.30),
                placeholder: p.background.strong.text.scale_alpha(0.8),
            }
        });

    // 构建搜索选项复选框组
    let options = row![
        checkbox(tab.case_sensitive)
            .label("区分大小写")
            .on_toggle({
                let active_id = active_id.clone();
                move |v| {
                    Message::Project(ProjectMessage::FileTreeFindCaseSensitiveToggled(
                        active_id.clone(),
                        v,
                    ))
                }
            })
            .size(14)
            .text_size(13),
        checkbox(tab.whole_word)
            .label("全词匹配")
            .on_toggle({
                let active_id = active_id.clone();
                move |v| {
                    Message::Project(ProjectMessage::FileTreeFindWholeWordToggled(
                        active_id.clone(),
                        v,
                    ))
                }
            })
            .size(14)
            .text_size(13),
        checkbox(tab.use_regex)
            .label("使用正则")
            .on_toggle({
                let active_id = active_id.clone();
                move |v| {
                    Message::Project(ProjectMessage::FileTreeFindRegexToggled(active_id.clone(), v))
                }
            })
            .size(14)
            .text_size(13),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    // 构建搜索按钮（搜索进行中时禁用）
    let run_btn =
        button(if tab.running { text("搜索中...").size(12) } else { text("搜索").size(12) })
            .on_press_maybe(
                (!tab.running).then_some(Message::Project(ProjectMessage::FileTreeFindRun(
                    active_id.clone(),
                ))),
            )
            .padding([5, 12])
            .style(|theme: &Theme, status| {
                let p = theme.extended_palette();
                let bg = match status {
                    iced::widget::button::Status::Hovered => Some(p.primary.strong.color),
                    iced::widget::button::Status::Pressed => Some(p.primary.base.color),
                    _ => Some(p.primary.base.color),
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    text_color: p.primary.base.text,
                    border: iced::Border { radius: 999.0.into(), ..Default::default() },
                    ..Default::default()
                }
            });

    // 格式化搜索选项摘要文本
    let opts_text = format!(
        "作用域: {}  |  大小写: {}  全词: {}  正则: {}  替换: {}",
        to_relative_path(root, &tab.scope_path),
        if tab.case_sensitive { "开" } else { "关" },
        if tab.whole_word { "开" } else { "关" },
        if tab.use_regex { "开" } else { "关" },
        if tab.replace_text.trim().is_empty() {
            "(空)".to_string()
        } else {
            tab.replace_text.replace('\n', " ")
        }
    );

    // 显示错误信息（如果存在）
    let error_line: Element<'_, Message> = if let Some(err) = &tab.error {
        text(err.clone()).size(12).color(Color::from_rgb(0.82, 0.2, 0.2)).into()
    } else {
        Space::new().height(Length::Fixed(2.0)).into()
    };

    // 组合所有控件到控制面板
    let controls = column![
        query_input,
        replace_input,
        row![options, Space::new().width(Length::Fill), run_btn]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        error_line,
        text(opts_text).size(11).color(Color::from_rgb(0.5, 0.5, 0.5))
    ]
    .spacing(4)
    .width(Length::Fill);

    // 构建搜索结果列表
    let mut results = column![].spacing(4).width(Length::Fill);

    // 显示结果数量限制提示
    if tab.limit_reached {
        results = results.push(
            text(format!("结果已达上限 {} 条，已停止搜索。", FIND_RESULTS_LIMIT))
                .size(12)
                .color(Color::from_rgb(0.78, 0.48, 0.10)),
        );
    }

    if tab.matches.is_empty() {
        // 无匹配结果
        results = results.push(text("没有匹配项").size(13));
    } else {
        // 遍历所有匹配项并构建 UI
        for item in &tab.matches {
            // 构建标题行：文件图标 + 相对路径 + 行号:列号
            let rel = to_relative_path(root, &item.path);
            let line_col = format!("{}:{}", item.line, item.column);
            let title = row![
                container(static_icon_svg(file_icon_for(&rel))).width(Length::Fixed(16.0)),
                text(rel).size(12),
                Space::new().width(Length::Fill),
                text(line_col).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
                Space::new().width(Length::Fixed(10.0))
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center);

            // 构建预览行：高亮显示匹配的文本片段
            let start = item.column.saturating_sub(1).min(item.preview.len());
            let end = start.saturating_add(item.match_len).min(item.preview.len());
            let preview: Element<'_, Message> = if start < end
                && item.preview.is_char_boundary(start)
                && item.preview.is_char_boundary(end)
            {
                // 安全地分割字符串并高亮匹配部分
                let before = &item.preview[..start];
                let matched = &item.preview[start..end];
                let after = &item.preview[end..];
                row![
                    text(before).size(12),
                    container(text(matched).size(12)).padding([0, 3]).style(|theme: &Theme| {
                        container::Style {
                            background: Some(Background::Color(find_match_highlight_bg(theme))),
                            border: iced::Border { radius: 3.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
                    text(after).size(12)
                ]
                .spacing(0)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                // 边界检查失败时显示完整预览
                text(item.preview.clone()).size(12).into()
            };

            // 保存点击所需的数据
            let path = item.path.clone();
            let line = item.line;
            let column = item.column;
            // 生成菜单源标识符，用于区分不同位置的右键菜单
            let menu_source = format!("find:{}:{}:{}", item.path, line, column);

            // 构建结果项按钮
            let btn = button(column![title, preview].spacing(2).width(Length::Fill))
                .width(Length::Fill)
                .padding([6, 10])
                .style(|theme: &Theme, status| {
                    let p = theme.palette().primary;
                    // 悬停和按下时使用半透明主色作为背景
                    let hover_bg = Color::from_rgba(p.r, p.g, p.b, 0.06);
                    let pressed_bg = Color::from_rgba(p.r, p.g, p.b, 0.10);
                    let bg = match status {
                        iced::widget::button::Status::Hovered => Some(Background::Color(hover_bg)),
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(pressed_bg))
                        }
                        _ => None,
                    };
                    iced::widget::button::Style {
                        background: bg,
                        text_color: theme.palette().text,
                        border: iced::Border { radius: 6.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                })
                .on_press(Message::Preview(message::PreviewMessage::Open(path)));

            // 为右键菜单和拖拽功能准备数据
            let abs_for_right_click = item.path.clone();
            let menu_source_for_right_click = menu_source.clone();
            let drag_path = item.path.clone();

            // 包装按钮以支持右键点击和拖拽
            let right_click = Element::new(RightClickArea::new(
                btn.into(),
                Box::new(move |pos| {
                    Message::Project(ProjectMessage::FileTreeRightClicked(
                        abs_for_right_click.clone(),
                        menu_source_for_right_click.clone(),
                        pos.x,
                        pos.y,
                    ))
                }),
                Some(Message::Project(ProjectMessage::FileTreeDragStart(
                    drag_path,
                    Some((line, column)),
                ))),
                Some(Message::Project(ProjectMessage::FileTreeDragEnd)),
            ));

            // 检查是否需要显示右键菜单
            let item = if app.file_tree_menu_path.as_deref() == Some(item.path.as_str())
                && app.file_tree_menu_source.as_deref() == Some(menu_source.as_str())
            {
                // 显示带有菜单覆盖层的结果项
                PointBelowOverlay::new(right_click, build_file_tree_menu(app, false))
                    .show(true)
                    .anchor(app.file_tree_menu_anchor.unwrap_or(iced::Point::ORIGIN))
                    .on_close(Message::Project(ProjectMessage::FileTreeMenuClose))
                    .into()
            } else {
                right_click
            };
            results = results.push(item);
        }
    }

    // 组合控制面板和结果列表到最终布局
    column![
        controls,
        scrollable(container(results).width(Length::Fill).padding([4, 0]))
            .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
            .height(Length::Fill)
    ]
    .spacing(8)
    .padding([8, 12])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
