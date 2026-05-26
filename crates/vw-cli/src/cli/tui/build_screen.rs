//! TUI 构建屏幕渲染模块
//!
//! 本模块提供 VibeWindow 代理运行时 TUI 界面的构建屏幕渲染功能。
//! 主要包含两个核心渲染函数：
//!
//! - [`draw_home_screen`][]: 绘制主屏幕界面，包含 Logo、输入框和快捷键提示
//! - [`draw_main_screen`][]: 绘制会话面板和修改文件列表
//!
//! # 布局结构
//!
//! 主屏幕采用垂直分层布局：
//! ```text
//! ┌─────────────────────────────────────┐
//! │           Logo 区域                  │
//! ├─────────────────────────────────────┤
//! │ [执行指示器] [输入框] [取消提示]      │
//! ├─────────────────────────────────────┤
//! │ [模型名称]     [快捷键提示]           │
//! ├─────────────────────────────────────┤
//! │           提示信息区域                │
//! ├─────────────────────────────────────┤
//! │           工作区路径                  │
//! └─────────────────────────────────────┘
//! ```

use super::input_panel::render_input_box;
use super::layout::{centered_overlay_rect, home_layout, main_layout};
use crate::app::agent::agent::loop_::cli::logo_text_lines;
use crate::app::agent::agent::loop_::cli::render_execution_indicator;
use crate::app::agent::agent::loop_::cli::theme::{TEXT_MUTED, TEXT_SUBTLE, WARNING};
use crate::app::agent::agent::loop_::cli::transcript::wrap_trim_disabled;
use crate::app::agent::agent::loop_::cli::tui_utils::cursor_position;
use ratatui::layout::{Alignment, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

fn adaptive_logo_scale(area: ratatui::layout::Rect, input_height: u16) -> usize {
    let max_by_height = area.height.saturating_sub(input_height.saturating_add(6));
    let max_by_width = area.width.saturating_sub(3) / 16;
    usize::from(max_by_height.min(max_by_width).clamp(1, 8))
}

/// 绘制 TUI 主屏幕界面
///
/// 此函数负责渲染 VibeWindow 代理的主交互界面，包括：
/// - 居中的 ASCII Logo
/// - 带边框的用户输入框
/// - 执行状态指示器（左侧旋转动画）
/// - 模型名称和快捷键提示（底部元信息行）
/// - 操作提示信息
/// - 工作区路径页脚
/// - 可选的命令菜单覆盖层
///
/// # 参数
///
/// * `f` - ratatui 帧引用，用于渲染 widget
/// * `area` - 可用的渲染区域矩形
/// * `input` - 当前输入框中的文本内容
/// * `cursor_idx` - 光标在输入文本中的字符索引位置
/// * `busy` - 代理是否正在执行任务（显示旋转动画）
/// * `model_name` - 当前使用的模型名称
/// * `workspace` - 当前工作区路径字符串
/// * `input_height` - 输入框区域的高度（行数）
/// * `neon` - 主题高亮颜色（用于快捷键提示等）
/// * `spinner_idx` - 旋转动画的当前帧索引（0-3 循环）
/// * `show_menu` - 是否显示命令菜单覆盖层
///
/// # 布局说明
///
/// 屏幕被划分为多个水平分区：
/// 1. Logo 区域：居中显示项目标识
/// 2. 输入行：三列布局（指示器 | 输入框 | 取消提示）
/// 3. 元信息行：显示模型名称和快捷键
/// 4. 提示行：显示 Shift+Enter 等操作提示
/// 5. 页脚：显示当前工作区路径
///
/// # 光标处理
///
/// 函数会计算光标在多行输入中的实际位置，
/// 并通过 `set_cursor_position` 设置终端光标位置。
/// 光标位置会被限制在输入框区域内，防止越界。
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_home_screen(
    f: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    input: &str,
    cursor_idx: usize,
    busy: bool,
    model_name: &str,
    _workspace: &str,
    input_height: u16,
    neon: Color,
    spinner_idx: usize,
    show_menu: bool,
) {
    // 获取 Logo 文本行并计算所需高度
    let logo_lines = logo_text_lines(adaptive_logo_scale(area, input_height));
    let logo_height = u16::try_from(logo_lines.len()).unwrap_or(u16::MAX);

    // 计算主屏幕布局分区
    let (chunks, center_chunks) = home_layout(area, logo_height, input_height);

    // 渲染 Logo 区域：居中对齐显示
    f.render_widget(
        Paragraph::new(Text::from(logo_lines)).alignment(Alignment::Center),
        center_chunks[0],
    );

    // 输入行布局：三列均分（指示器 20% | 输入框 60% | 提示 20%）
    let input_row = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(12),
            ratatui::layout::Constraint::Percentage(76),
            ratatui::layout::Constraint::Percentage(12),
        ])
        .split(center_chunks[2]);

    let input_inner_area = input_row[1];

    // 渲染输入框内容和执行指示器
    let input_text_area = render_input_box(f, input_inner_area, input);
    render_execution_indicator(f, input_row[0], busy, spinner_idx);

    // 根据执行状态显示取消提示
    // 忙碌时显示 "esc 取消"，否则显示空行
    let esc_line = if busy {
        Line::from(vec![
            Span::styled("esc", Style::default().fg(WARNING).add_modifier(Modifier::BOLD)),
            Span::styled(" 取消", Style::default().fg(TEXT_MUTED)),
        ])
    } else {
        Line::from(Span::raw(""))
    };
    f.render_widget(Paragraph::new(esc_line).alignment(Alignment::Right), input_row[2]);

    // 元信息行布局：显示模型名称和快捷键提示
    let meta_row = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(12),
            ratatui::layout::Constraint::Percentage(76),
            ratatui::layout::Constraint::Percentage(12),
        ])
        .split(center_chunks[3]);
    // 元信息内部分区：模型名称 45% | 快捷键 55%
    let meta_inner = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(45),
            ratatui::layout::Constraint::Percentage(55),
        ])
        .split(meta_row[1]);

    // 渲染模型名称（左侧灰色显示）
    let active_mode_style = Style::default().fg(neon).add_modifier(Modifier::BOLD);
    let build_line = Line::from(vec![Span::styled(model_name, Style::default().fg(TEXT_MUTED))]);
    f.render_widget(Paragraph::new(build_line), meta_inner[0]);

    // 渲染快捷键提示行（右侧显示）
    // 包含：ctrl+p 命令、ctrl+t 工具详情、ctrl+y 思考详情
    let hint_line = Line::from(vec![
        Span::styled("ctrl+p", active_mode_style),
        Span::styled(" 命令  ", Style::default().fg(TEXT_SUBTLE)),
        Span::styled("ctrl+t", active_mode_style),
        Span::styled(" 工具详情  ", Style::default().fg(TEXT_SUBTLE)),
        Span::styled("ctrl+y", active_mode_style),
        Span::styled(" 思考详情", Style::default().fg(TEXT_SUBTLE)),
    ]);
    f.render_widget(Paragraph::new(hint_line).alignment(Alignment::Right), meta_inner[1]);

    // 提示行布局：显示操作提示信息
    let tip_row = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(12),
            ratatui::layout::Constraint::Percentage(76),
            ratatui::layout::Constraint::Percentage(12),
        ])
        .split(center_chunks[4]);

    // 渲染黄色圆点 + 操作提示文本
    let tip_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(WARNING)),
        Span::styled(
            "提示：Shift+Enter 或 Ctrl+J 换行，Enter 发送，Ctrl+Z 挂起",
            Style::default().fg(TEXT_MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(tip_line).alignment(Alignment::Center), tip_row[1]);

    f.render_widget(Paragraph::new(Line::from(Span::raw(""))), chunks[3]);

    // 计算并设置光标位置
    // 1. 确保光标索引不超过输入文本长度
    // 2. 将字符索引转换为行/列坐标
    // 3. 限制光标在输入框区域内
    // 4. 计算光标的绝对屏幕坐标
    let safe_cursor = cursor_idx.min(input.chars().count());
    let (row, col) = cursor_position(input, safe_cursor);
    let max_row = input_text_area.height.saturating_sub(1);
    let max_col = input_text_area.width.saturating_sub(1);
    let safe_row: u16 = row.min(max_row);
    let safe_col: u16 = col.min(max_col);
    let cursor_x = input_text_area.x.saturating_add(safe_col);
    let cursor_y = input_text_area.y.saturating_add(safe_row);
    f.set_cursor_position((cursor_x, cursor_y));

    // 条件渲染：命令菜单覆盖层
    // 当 show_menu 为 true 时，在屏幕中央显示命令菜单弹窗
    if show_menu {
        let overlay = centered_overlay_rect(area, 2, 5, 10);
        let lines = vec![Line::from(Span::styled("暂无命令", Style::default().fg(TEXT_SUBTLE)))];
        let popup = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Commands"));
        f.render_widget(Clear, overlay);
        f.render_widget(popup, overlay);
    }
}

/// 绘制 TUI 主界面（会话面板和文件列表）
///
/// 此函数负责渲染 VibeWindow 代理的辅助面板区域，包括：
/// - Session 标题面板：显示当前会话标题
/// - 修改文件面板：显示被修改的文件列表（支持折叠和溢出省略）
/// - 页脚：显示工作区路径和清除确认提示
/// - 可选的命令菜单覆盖层
///
/// # 参数
///
/// * `f` - ratatui 帧引用，用于渲染 widget
/// * `right_chunks` - 右侧面板区域的布局分区（Vec 包含多个 Rect）
/// * `session_title` - 当前会话的标题文本
/// * `modified_files` - 被修改的文件路径列表
/// * `files_collapsed` - 文件列表是否处于折叠状态
/// * `neon_border` - 边框的主题样式（霓虹色风格）
/// * `workspace` - 当前工作区路径字符串
/// * `awaiting_clear_confirm` - 是否正在等待清除确认（输入 y/yes）
/// * `show_menu` - 是否显示命令菜单覆盖层
///
/// # 文件列表处理
///
/// 文件列表有三种显示状态：
/// 1. 折叠状态：显示 "已折叠" 文本
/// 2. 空列表：显示 "暂无修改文件" 文本
/// 3. 正常显示：列出所有文件路径，超出高度时显示省略
///
/// 当文件数量超过可视区域高度时，会截断列表并显示
/// 省略提示（如 "… +5" 表示还有 5 个文件未显示）。
pub(crate) fn draw_main_screen(
    f: &mut ratatui::Frame<'_>,
    right_chunks: Vec<ratatui::layout::Rect>,
    session_title: &str,
    modified_files: &[String],
    files_collapsed: bool,
    neon_border: Style,
    _workspace: &str,
    awaiting_clear_confirm: bool,
    show_menu: bool,
) {
    // 渲染 Session 标题面板
    // 带有 "Session" 标题和霓虹色边框
    let session_widget = Paragraph::new(session_title)
        .block(Block::default().borders(Borders::ALL).title("Session").border_style(neon_border));
    f.render_widget(session_widget, right_chunks[0]);

    // 构建修改文件列表的显示内容
    // 根据折叠状态和文件数量决定显示内容
    let mut files_lines: Vec<Line> = if files_collapsed {
        // 折叠状态：显示提示文本
        vec![Line::from(Span::styled("已折叠", Style::default().fg(TEXT_SUBTLE)))]
    } else if modified_files.is_empty() {
        // 空列表：显示暂无文件提示
        vec![Line::from(Span::styled("暂无修改文件", Style::default().fg(TEXT_SUBTLE)))]
    } else {
        // 正常状态：格式化显示所有文件路径
        modified_files.iter().map(|path| Line::from(Span::raw(format!("• {}", path)))).collect()
    };

    // 计算文件列表可视区域高度（减去边框占用的 2 行）
    let files_visible_height = right_chunks[1].height.saturating_sub(2) as usize;

    // 处理文件列表溢出
    if files_visible_height == 0 {
        // 高度为 0 时清空列表（无显示空间）
        files_lines.clear();
    } else if !files_collapsed && files_lines.len() > files_visible_height {
        // 文件数超出可视区域：截断并添加省略提示
        let hidden = files_lines.len().saturating_sub(files_visible_height).saturating_add(1);
        files_lines.truncate(files_visible_height);
        // 移除最后一行，为省略提示腾出空间
        if !files_lines.is_empty() {
            files_lines.pop();
        }
        // 添加省略提示行，显示隐藏文件数量
        files_lines.push(Line::from(Span::styled(
            format!("… +{hidden}"),
            Style::default().fg(TEXT_SUBTLE),
        )));
    }

    // 渲染修改文件面板
    // 带有 "修改文件" 标题、霓虹色边框和自动换行
    let files_widget = Paragraph::new(Text::from(files_lines))
        .block(Block::default().borders(Borders::ALL).title("修改文件").border_style(neon_border))
        .wrap(wrap_trim_disabled());
    f.render_widget(files_widget, right_chunks[1]);

    // 构建页脚内容
    // 基础内容为工作区路径，等待清除确认时追加黄色警告文本
    let mut footer = Vec::new();
    if awaiting_clear_confirm {
        footer.push(Span::styled(
            "confirm clear: type y or yes",
            Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
        ));
    }

    // 渲染页脚到主布局的 footer 区域
    let layout = main_layout(f.area());
    let footer_area = layout.footer_area();
    f.render_widget(Paragraph::new(Line::from(footer)), footer_area);

    // 条件渲染：命令菜单覆盖层
    // 当 show_menu 为 true 时，在屏幕中央显示命令菜单弹窗
    if show_menu {
        let overlay = centered_overlay_rect(f.area(), 2, 5, 10);
        let lines = vec![Line::from(Span::styled("暂无命令", Style::default().fg(TEXT_SUBTLE)))];
        let popup = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Commands"));
        f.render_widget(Clear, overlay);
        f.render_widget(popup, overlay);
    }
}
#[cfg(test)]
#[path = "build_screen_tests.rs"]
mod build_screen_tests;
