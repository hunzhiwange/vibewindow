//! 输入面板渲染模块
//!
//! 本模块提供 TUI（终端用户界面）输入面板的渲染功能。
//! 负责显示用户输入区域、模型信息、执行状态指示器和操作提示。
//!
//! # 主要组件
//!
//! - `render_input_panel`: 渲染完整的输入面板，包括输入框、状态栏和提示信息
//! - `render_input_box`: 渲染输入框主体区域
//!
//! # 布局结构
//!
//! 输入面板分为两个主要区域：
//! 1. 上部：输入框区域（带装饰边框）
//! 2. 下部：状态栏（两行）
//!    - 第一行：执行指示器、模型名称、取消提示
//!    - 第二行：操作提示信息

use crate::app::agent::agent::loop_::cli::render_execution_indicator;
use crate::app::agent::agent::loop_::cli::theme::{
    SURFACE_ELEVATED, TEXT_MUTED, TEXT_PRIMARY, TEXT_SUBTLE, WARNING,
};
use crate::app::agent::agent::loop_::cli::tui_utils::cursor_position;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

/// 渲染完整的输入面板
///
/// 该函数渲染 TUI 界面中的输入面板，包括输入框、状态信息和光标定位。
/// 面板采用垂直布局，分为输入区域和底部状态栏两部分。
///
/// # 参数
///
/// * `f` - ratatui Frame 引用，用于渲染控件
/// * `area` - 面板占用的矩形区域
/// * `input` - 当前输入的文本内容
/// * `cursor_idx` - 光标在输入文本中的字符索引位置
/// * `busy` - 是否正在执行任务（显示不同状态）
/// * `model_name` - 当前使用的模型名称
/// * `spinner_idx` - 加载动画的当前帧索引
///
/// # 布局说明
///
/// ```text
/// ┌─────────────────────────┐
/// │                         │
/// │      输入框区域          │  area.height - 2 行
/// │                         │
/// ├─────────────────────────┤
/// │ [状态指示] [模型名] [ESC]│  第 1 行
/// │ [提示]                  │  第 2 行
/// └─────────────────────────┘
/// ```
///
/// # 示例
///
/// ```ignore
/// render_input_panel(
///     &mut f,
///     input_area,
///     "你好，请帮我分析这段代码",
///     12,
///     true,
///     "gpt-4",
///     3,
/// );
/// ```
pub(crate) fn render_input_panel(
    f: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    input: &str,
    cursor_idx: usize,
    busy: bool,
    model_name: &str,
    spinner_idx: usize,
) {
    let panel_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(area.height.saturating_sub(2)),
            Constraint::Length(1),
        ])
        .split(area);

    let status_row = panel_rows[0];
    let input_inner_area = panel_rows[1];
    let tip_row = panel_rows[2];

    // 渲染输入框内容
    let input_text_area = render_input_box(f, input_inner_area, input);

    let build_info = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(1), Constraint::Length(10)])
        .split(status_row);

    render_execution_indicator(f, build_info[0], busy, spinner_idx);

    let esc_line = if busy {
        Line::from(vec![
            Span::styled("esc", Style::default().fg(WARNING).add_modifier(Modifier::BOLD)),
            Span::styled(" 取消", Style::default().fg(TEXT_MUTED)),
        ])
    } else {
        Line::from(Span::raw(""))
    };
    f.render_widget(
        Paragraph::new(esc_line).alignment(ratatui::layout::Alignment::Right),
        build_info[2],
    );

    // 第二行：操作提示信息
    // 渲染模型信息和操作提示文字
    let tip_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(44)])
        .split(tip_row);
    let model_line =
        Line::from(vec![Span::styled(model_name.to_string(), Style::default().fg(TEXT_MUTED))]);
    f.render_widget(Paragraph::new(model_line), tip_chunks[0]);
    let tip_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(WARNING)),
        Span::styled(
            "提示：Shift+Enter 或 Ctrl+J 换行，Enter 发送，Ctrl+Z 挂起",
            Style::default().fg(TEXT_MUTED),
        ),
    ]);
    f.render_widget(
        Paragraph::new(tip_line).alignment(ratatui::layout::Alignment::Right),
        tip_chunks[1],
    );

    // 计算并设置光标位置
    // 确保光标索引不超过输入文本长度
    let safe_cursor = cursor_idx.min(input.chars().count());

    // 根据文本内容计算光标的行列位置
    let (row, col) = cursor_position(input, safe_cursor);

    // 确保光标位置在可视区域内
    let max_row = input_text_area.height.saturating_sub(1);
    let max_col = input_text_area.width.saturating_sub(1);
    let safe_row: u16 = row.min(max_row);
    let safe_col: u16 = col.min(max_col);

    // 计算光标的绝对坐标（相对于终端窗口）
    let cursor_x = input_text_area.x.saturating_add(safe_col);
    let cursor_y = input_text_area.y.saturating_add(safe_row);

    // 设置光标位置
    f.set_cursor_position((cursor_x, cursor_y));
}

/// 渲染输入框主体区域
///
/// 该函数渲染输入框的内容区域，包括左侧装饰条和输入文本（或占位符）。
/// 输入框采用水平布局，左侧为装饰性竖线，右侧为文本输入区域。
///
/// # 参数
///
/// * `f` - ratatui Frame 引用，用于渲染控件
/// * `area` - 输入框占用的矩形区域
/// * `input` - 当前输入的文本内容
///
/// # 布局说明
///
/// ```text
/// ┌──┬──────────────────┐
/// │┃ │                  │
/// │┃ │   输入文本区域    │
/// │┃ │                  │
/// └──┴──────────────────┘
///  ↑
///  装饰条 (2 字符宽)
/// ```
///
/// # 视觉效果
///
/// - 输入为空时：显示灰色占位符文字"随便问点什么..."
/// - 输入非空时：显示白色输入文本
/// - 左侧装饰条：亮蓝色竖线（┃）
///
/// # 示例
///
/// ```ignore
/// render_input_box(&mut f, input_area, "这是一段输入文本");
/// ```
pub(crate) fn render_input_box(
    f: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    input: &str,
) -> ratatui::layout::Rect {
    // 根据输入内容决定显示文本
    let input_text = if input.is_empty() {
        // 输入为空时显示占位符（深灰色）
        Text::from(Line::from(Span::styled(
            "随便问点什么...",
            Style::default().fg(TEXT_SUBTLE),
        )))
    } else {
        // 输入非空时显示实际文本
        Text::from(input.to_string())
    };

    let text_area = ratatui::layout::Rect {
        x: area.x.saturating_add(1),
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };
    let bg_widget = Paragraph::new(Text::from(""))
        .style(Style::default().fg(TEXT_PRIMARY).bg(SURFACE_ELEVATED));
    f.render_widget(bg_widget, area);
    let input_widget = Paragraph::new(input_text)
        .style(Style::default().fg(TEXT_PRIMARY).bg(SURFACE_ELEVATED));
    f.render_widget(input_widget, text_area);
    text_area
}
#[cfg(test)]
#[path = "input_panel_tests.rs"]
mod input_panel_tests;
