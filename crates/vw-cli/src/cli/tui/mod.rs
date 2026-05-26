//! 终端用户界面（TUI）模块
//!
//! 本模块提供 VibeWindow CLI 的交互式终端界面实现。使用 ratatui 库构建响应式界面，
//! 支持会话对话显示、用户输入面板、会话统计和文件变更列表等功能。
//!
//! # 架构概览
//!
//! - [`CliTui`][]: 核心 TUI 状态管理器，负责终端初始化、渲染循环和鼠标交互
//! - [`MouseAction`][]: 鼠标交互行为枚举，用于处理用户点击事件
//!
//! # 子模块
//!
//! - `build_screen`: 主屏幕和首页渲染逻辑
//! - `input_panel`: 用户输入面板渲染
//! - `layout`: 界面布局计算
//! - `scroll`: 滚动条渲染
//!
//! # 使用示例
//!
//! ```ignore
//! let mut tui = CliTui::new()?;
//! tui.draw(&transcript, &input, cursor_idx, busy, ...)?;
//! tui.tick(); // 推进动画帧
//! ```

mod build_screen;
mod input_panel;
mod layout;
mod scroll;

use super::render::render_scanline_background;
use super::stats::CliStats;
use super::theme::{ACCENT_CYAN, ACCENT_RED, SUCCESS, TEXT_MUTED, TEXT_SUBTLE, WARNING};
use super::transcript::{
    ThinkBlockMeta, TranscriptEntry, think_block_expanded, transcript_to_lines,
    wrap_trim_disabled,
};
use super::tui_utils::neon_breath_color;
use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Instant;

use build_screen::draw_home_screen;

#[cfg(unix)]
use std::fs::File;

/// 动画帧序列，用于显示"思考中"状态的旋转指示器
///
/// 包含 10 个 Unicode Braille 字符，按顺序循环播放形成动画效果。
/// 通过 [`CliTui::tick`] 方法推进帧索引。
const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const CLI_TUI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn compact_cwd_path(fallback: &str) -> String {
    let Ok(cwd) = std::env::current_dir() else {
        return fallback.to_string();
    };
    if let Some(home) = std::env::var_os("HOME") {
        let home_path = PathBuf::from(home);
        if let Ok(stripped) = cwd.strip_prefix(&home_path) {
            if stripped.as_os_str().is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", stripped.display());
        }
    }
    cwd.display().to_string()
}

fn u16_from_usize_saturating(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

/// Unix 平台下的终端后端写入器类型
///
/// 在 Unix 系统上直接打开 `/dev/tty` 以确保输出到控制终端，
/// 即使标准输出被重定向也能正常显示 TUI。
#[cfg(unix)]
type CliBackendWriter = File;

/// 非 Unix 平台下的终端后端写入器类型
///
/// 在 Windows 等非 Unix 系统上使用标准输出作为 TUI 后端。
#[cfg(not(unix))]
type CliBackendWriter = std::io::Stdout;

/// CLI 终端用户界面（TUI）状态管理器
///
/// 封装 ratatui 终端实例和相关状态，负责：
/// - 终端的初始化和清理（通过 `Drop` trait）
/// - 管理渲染缓存以避免不必要的重绘
/// - 追踪会话区域状态以支持鼠标交互
/// - 管理工具块和思考块的展开/折叠状态
///
/// # 生命周期
///
/// 1. 调用 [`CliTui::new`] 初始化终端（进入 raw 模式和备用屏幕）
/// 2. 调用 [`CliTui::draw`] 渲染界面
/// 3. 调用 [`CliTui::tick`] 推进动画帧
/// 4. 销毁时自动清理终端状态
///
/// # 线程安全
///
/// 此类型不实现 `Send` 或 `Sync`，应在单个线程中使用。
pub(crate) struct CliTui {
    /// ratatui 终端实例，管理底层渲染
    terminal: Terminal<CrosstermBackend<CliBackendWriter>>,
    /// 当前动画帧索引，用于显示旋转指示器
    pub(crate) spinner_idx: usize,
    /// 是否展开所有工具调用详情块
    pub(crate) expand_tool_blocks: bool,
    /// 是否展开所有思考过程块
    pub(crate) expand_think_all: bool,
    /// 被手动切换过默认展开状态的思考块 ID 集合
    pub(crate) think_detail_overrides: BTreeSet<u64>,
    /// 上次渲染内容的哈希值，用于跳过无变化的渲染
    pub(crate) last_render_hash: Option<u64>,
    /// 上次渲染的会话区域边界，用于鼠标点击检测
    pub(crate) last_conversation_area: Option<ratatui::layout::Rect>,
    /// 上次渲染的会话滚动位置
    pub(crate) last_conversation_scroll: u16,
    /// 上次渲染的会话行内容，用于鼠标点击时定位
    pub(crate) last_conversation_lines: Vec<String>,
    /// 会话行到思考块 ID 的映射，用于处理思考块点击
    pub(crate) last_conversation_think_map: Vec<Option<ThinkBlockMeta>>,
}

/// 鼠标交互行为类型
///
/// 表示用户在 TUI 中点击后应执行的操作，
/// 由 [`CliTui::resolve_mouse_action`] 返回。
#[derive(Clone, Copy)]
pub(crate) enum MouseAction {
    /// 无操作（点击位置不响应）
    None,
    /// 切换工具详情块的展开/折叠状态
    ToggleToolDetails,
    /// 切换思考过程块的展开/折叠状态
    ToggleThinkDetails,
    /// 设置滚动位置到指定值（点击滚动条时使用）
    SetScrollBack(u16),
}

impl CliTui {
    /// 创建并初始化 TUI 实例
    ///
    /// 执行以下初始化步骤：
    /// 1. 启用终端 raw 模式（禁用行缓冲和本地回显）
    /// 2. 进入备用屏幕缓冲区（退出时恢复原屏幕）
    /// 3. 启用鼠标事件捕获
    /// 4. 创建 ratatui 终端实例
    ///
    /// # 平台差异
    ///
    /// - Unix: 直接打开 `/dev/tty` 作为输出目标
    /// - 非 Unix: 使用标准输出
    ///
    /// # 错误
    ///
    /// 当终端不支持 raw 模式或无法打开 TTY 设备时返回错误。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let tui = CliTui::new()?;
    /// // 使用 tui 进行渲染...
    /// // 离开作用域时自动清理
    /// ```
    pub(crate) fn new() -> Result<Self> {
        enable_raw_mode()?;

        // Unix 平台直接打开 TTY 设备，确保即使 stdout 被重定向也能显示界面
        #[cfg(unix)]
        let mut screen: CliBackendWriter =
            OpenOptions::new().read(true).write(true).open("/dev/tty")?;

        // 非 Unix 平台使用标准输出
        #[cfg(not(unix))]
        let mut screen: CliBackendWriter = std::io::stdout();

        screen.execute(EnterAlternateScreen)?;
        screen.execute(EnableMouseCapture)?;
        let _ = screen.execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ));
        let backend = CrosstermBackend::new(screen);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            spinner_idx: 0,
            expand_tool_blocks: false,
            expand_think_all: false,
            think_detail_overrides: BTreeSet::new(),
            last_render_hash: None,
            last_conversation_area: None,
            last_conversation_scroll: 0,
            last_conversation_lines: Vec::new(),
            last_conversation_think_map: Vec::new(),
        })
    }

    /// 计算当前渲染状态的哈希值
    ///
    /// 将所有影响界面显示的参数哈希为一个 64 位值，
    /// 用于检测状态变化并跳过不必要的重绘。
    ///
    /// # 参数
    ///
    /// - `transcript`: 会话消息列表
    /// - `input`: 当前用户输入文本
    /// - `cursor_idx`: 光标在输入中的位置
    /// - `busy`: Agent 是否正在处理
    /// - `awaiting_clear_confirm`: 是否等待确认清屏
    /// - `provider_name`: 模型提供者名称
    /// - `model_name`: 模型名称
    /// - `stats`: 会话统计信息
    /// - `workspace`: 工作区路径
    /// - `draft`: 草稿内容（思考过程的流式输出）
    /// - `session_title`: 会话标题
    /// - `modified_files`: 已修改文件列表
    /// - `files_collapsed`: 文件列表是否折叠
    /// - `scroll_back`: 滚动回退值（从底部向上的偏移）
    /// - `show_menu`: 是否显示菜单
    ///
    /// # 返回值
    ///
    /// 表示当前状态的 64 位哈希值。
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub(crate) fn render_hash(
        &self,
        transcript: &[TranscriptEntry],
        input: &str,
        cursor_idx: usize,
        busy: bool,
        awaiting_clear_confirm: bool,
        provider_name: &str,
        model_name: &str,
        stats: &CliStats,
        workspace: &str,
        draft: &str,
        session_title: &str,
        modified_files: &[String],
        files_collapsed: bool,
        scroll_back: u16,
        show_menu: bool,
    ) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        // 哈希所有可能影响渲染的状态
        busy.hash(&mut h);
        awaiting_clear_confirm.hash(&mut h);
        provider_name.hash(&mut h);
        model_name.hash(&mut h);
        workspace.hash(&mut h);
        input.hash(&mut h);
        cursor_idx.hash(&mut h);
        draft.hash(&mut h);
        session_title.hash(&mut h);
        files_collapsed.hash(&mut h);
        scroll_back.hash(&mut h);
        show_menu.hash(&mut h);
        self.expand_tool_blocks.hash(&mut h);
        self.expand_think_all.hash(&mut h);
        // 哈希手动覆盖默认状态的思考块 ID 集合
        for think_id in &self.think_detail_overrides {
            think_id.hash(&mut h);
        }
        // 哈希统计信息
        stats.user_messages.hash(&mut h);
        stats.assistant_messages.hash(&mut h);
        stats.tool_events.hash(&mut h);
        stats.input_tokens.hash(&mut h);
        stats.output_tokens.hash(&mut h);
        // 哈希修改的文件列表
        for f in modified_files {
            f.hash(&mut h);
        }
        // 哈希会话内容（角色和文本）
        for e in transcript {
            (e.role as u8).hash(&mut h);
            e.text.hash(&mut h);
        }
        h.finish()
    }

    /// 处理鼠标点击事件（简化版）
    ///
    /// 仅返回是否触发了展开/折叠操作，
    /// 不处理滚动条拖动。
    ///
    /// # 参数
    ///
    /// - `x`: 鼠标点击的列坐标
    /// - `y`: 鼠标点击的行坐标
    ///
    /// # 返回值
    ///
    /// 如果点击触发了工具详情或思考块的状态切换则返回 `true`。
    pub(crate) fn handle_mouse_click(&mut self, x: u16, y: u16) -> bool {
        matches!(
            self.resolve_mouse_action(x, y, 0),
            MouseAction::ToggleToolDetails | MouseAction::ToggleThinkDetails
        )
    }

    /// 解析鼠标点击并返回应执行的操作
    ///
    /// 根据点击位置判断用户意图：
    /// 1. 点击滚动条 → 返回滚动位置
    /// 2. 点击思考块 → 切换该块的展开状态
    /// 3. 点击工具详情 → 切换工具块展开状态
    /// 4. 点击其他区域 → 保持当前滚动
    ///
    /// # 参数
    ///
    /// - `x`: 鼠标点击的列坐标
    /// - `y`: 鼠标点击的行坐标
    /// - `current_scroll_back`: 当前滚动回退值
    ///
    /// # 返回值
    ///
    /// 表示应执行操作的 [`MouseAction`] 枚举值。
    pub(crate) fn resolve_mouse_action(
        &mut self,
        x: u16,
        y: u16,
        current_scroll_back: u16,
    ) -> MouseAction {
        // 检查是否有有效的会话区域信息
        let Some(area) = self.last_conversation_area else {
            return MouseAction::None;
        };
        // 区域太小则忽略
        if area.width < 2 || area.height < 1 {
            return MouseAction::None;
        }

        let inner_x0 = area.x;
        let inner_x1 = area.x.saturating_add(area.width.saturating_sub(1));
        let inner_y0 = area.y;
        let inner_y1 = area.y.saturating_add(area.height.saturating_sub(1));

        // 点击在会话区域外部
        if x < inner_x0 || x > inner_x1 || y < inner_y0 || y > inner_y1 {
            return MouseAction::None;
        }

        let inner_h = usize::from(area.height);
        let total = self.last_conversation_lines.len();
        let base_scroll = u16_from_usize_saturating(total.saturating_sub(inner_h));
        let scrollbar_x = area.x.saturating_add(area.width.saturating_sub(1));
        let scrollbar_hit_x = scrollbar_x.saturating_sub(1);

        // 处理滚动条点击
        if x >= scrollbar_hit_x && x <= scrollbar_x && inner_h > 0 && base_scroll > 0 {
            let row = usize::from(y.saturating_sub(inner_y0));
            let max_row = inner_h.saturating_sub(1).max(1);
            // 将点击位置映射到滚动位置
            let row_u64 = u64::try_from(row).unwrap_or(u64::MAX);
            let max_row_u64 = u64::try_from(max_row).unwrap_or(u64::MAX);
            let target_effective = u16::try_from(
                row_u64 * u64::from(base_scroll) / max_row_u64,
            )
            .unwrap_or(base_scroll);
            let target_scroll_back = base_scroll.saturating_sub(target_effective);
            return MouseAction::SetScrollBack(target_scroll_back);
        }

        // 计算点击对应的行索引（考虑滚动偏移）
        let row = usize::from(y.saturating_sub(inner_y0));
        let line_idx = usize::from(self.last_conversation_scroll) + row;
        let Some(line) = self.last_conversation_lines.get(line_idx) else {
            return MouseAction::None;
        };

        // 处理思考块点击
        if let Some(think_meta) = self.last_conversation_think_map.get(line_idx).and_then(|v| *v)
        {
            let current_expanded =
                think_block_expanded(think_meta, self.expand_think_all, &self.think_detail_overrides);
            let next_expanded = !current_expanded;
            self.expand_think_all = false;

            if next_expanded == think_meta.open {
                self.think_detail_overrides.remove(&think_meta.id);
            } else {
                self.think_detail_overrides.insert(think_meta.id);
            }
            return MouseAction::ToggleThinkDetails;
        }

        // 处理工具详情点击（通过关键词检测）
        if line.contains("[折叠") || line.contains("展开详情") || line.contains("收起详情")
        {
            self.expand_tool_blocks = !self.expand_tool_blocks;
            return MouseAction::ToggleToolDetails;
        }

        // 点击其他区域，保持当前滚动
        MouseAction::SetScrollBack(current_scroll_back)
    }

    /// 使渲染缓存失效
    ///
    /// 强制下一次 [`draw`] 调用执行完整重绘，
    /// 即使状态哈希值未改变。通常在需要刷新显示时调用。
    pub(crate) fn invalidate_render_cache(&mut self) {
        self.last_render_hash = None;
    }

    /// 推进动画帧
    ///
    /// 将动画帧索引前进到下一帧，循环使用 [`SPINNER_FRAMES`] 中的字符。
    /// 应在主循环中定期调用以产生动画效果。
    pub(crate) fn tick(&mut self) {
        self.spinner_idx = (self.spinner_idx + 1) % SPINNER_FRAMES.len();
    }

    /// 渲染完整的 TUI 界面
    ///
    /// 根据当前状态绘制整个终端界面，包括：
    /// - 标题栏（显示状态、提供者和模型）
    /// - 会话区域（消息历史）
    /// - 输入面板
    /// - 右侧面板（统计、文件列表等）
    ///
    /// 渲染后会更新内部状态缓存以支持后续的鼠标交互。
    ///
    /// # 参数
    ///
    /// - `transcript`: 会话消息列表
    /// - `input`: 当前用户输入文本
    /// - `cursor_idx`: 光标在输入中的位置
    /// - `busy`: Agent 是否正在处理
    /// - `awaiting_clear_confirm`: 是否等待确认清屏
    /// - `provider_name`: 模型提供者名称
    /// - `model_name`: 模型名称
    /// - `stats`: 会话统计信息
    /// - `workspace`: 工作区路径
    /// - `draft`: 草稿内容（思考过程的流式输出）
    /// - `session_title`: 会话标题
    /// - `modified_files`: 已修改文件列表
    /// - `files_collapsed`: 文件列表是否折叠
    /// - `scroll_back`: 滚动回退值（从底部向上的偏移）
    /// - `show_menu`: 是否显示菜单
    ///
    /// # 返回值
    ///
    /// 成功时返回 `Ok(())`，终端错误时返回 `Err`。
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub(crate) fn draw(
        &mut self,
        transcript: &[TranscriptEntry],
        input: &str,
        cursor_idx: usize,
        busy: bool,
        awaiting_clear_confirm: bool,
        provider_name: &str,
        model_name: &str,
        stats: &CliStats,
        workspace: &str,
        draft: &str,
        session_title: &str,
        modified_files: &[String],
        files_collapsed: bool,
        scroll_back: u16,
        show_menu: bool,
    ) -> Result<()> {
        // 计算当前状态哈希
        let render_hash = self.render_hash(
            transcript,
            input,
            cursor_idx,
            busy,
            awaiting_clear_confirm,
            provider_name,
            model_name,
            stats,
            workspace,
            draft,
            session_title,
            modified_files,
            files_collapsed,
            scroll_back,
            show_menu,
        );
        let now = Instant::now();
        let pwd_path = compact_cwd_path(workspace);

        // 用于收集渲染过程中的状态信息
        let mut frame_conversation_area: Option<ratatui::layout::Rect> = None;
        let mut frame_conversation_scroll: u16 = 0;
        let mut frame_conversation_lines: Vec<String> = Vec::new();
        let mut frame_conversation_think_map: Vec<Option<ThinkBlockMeta>> = Vec::new();

        self.terminal.draw(|f| {
            let area = f.area();
            // 清除整个区域
            f.render_widget(Clear, area);
            // 绘制扫描线背景效果
            render_scanline_background(f, area);
            // 计算呼吸灯颜色（周期性变化的霓虹色）
            let neon = neon_breath_color(self.spinner_idx);

            // 判断是否显示首页（无消息时）
            let show_home = stats.user_messages == 0 && stats.assistant_messages == 0;

            // 计算输入面板高度（根据输入行数调整）
            let input_lines = u16_from_usize_saturating(input.lines().count().max(1));
            let input_height = (input_lines + 2).clamp(7, 13);

            if show_home {
                // 渲染首页界面
                draw_home_screen(
                    f,
                    area,
                    input,
                    cursor_idx,
                    busy,
                    model_name,
                    workspace,
                    input_height,
                    neon,
                    self.spinner_idx,
                    show_menu,
                );
                return;
            }

            // 计算主界面布局
            let layout = layout::main_layout(area);
            let chunks = [
                layout.header_area(),
                layout.subheader_area(),
                layout.body_area(),
                layout.footer_area(),
            ];

            let (status_text, status_color) =
                if busy { ("● Running", WARNING) } else { ("● Ready", SUCCESS) };
            let header_columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(1)])
                .split(chunks[0]);
            let logo_lines = vec![
                Line::from(vec![
                    Span::styled(
                        "氛",
                        Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "围",
                        Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "视",
                        Style::default().fg(ACCENT_RED).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "窗",
                        Style::default().fg(ACCENT_CYAN).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(Span::styled("VibeWindow", Style::default().fg(TEXT_MUTED))),
            ];
            let header_logo_area = ratatui::layout::Rect {
                x: header_columns[0].x.saturating_add(1),
                y: header_columns[0].y,
                width: header_columns[0].width.saturating_sub(2),
                height: 2,
            };
            f.render_widget(Paragraph::new(Text::from(logo_lines)), header_logo_area);

            let info_area = ratatui::layout::Rect {
                x: header_columns[1].x,
                y: header_columns[1].y,
                width: header_columns[1].width.saturating_sub(2).max(1),
                height: header_columns[1].height,
            };
            let info_rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .split(info_area);
            let row1 = Line::from(vec![
                Span::styled(format!("v{CLI_TUI_VERSION}  "), Style::default().fg(TEXT_SUBTLE)),
                Span::styled(
                    status_text,
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]);
            let row2 = Line::from(vec![Span::styled(model_name, Style::default().fg(TEXT_MUTED))]);
            let row3 = Line::from(vec![
                Span::styled("路径 ", Style::default().fg(TEXT_SUBTLE)),
                Span::styled(&pwd_path, Style::default().fg(TEXT_MUTED)),
            ]);
            f.render_widget(Paragraph::new(row1).alignment(Alignment::Right), info_rows[0]);
            f.render_widget(Paragraph::new(row2).alignment(Alignment::Right), info_rows[1]);
            f.render_widget(Paragraph::new(row3).alignment(Alignment::Right), info_rows[2]);

            let body_columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
                .split(chunks[2]);

            let body_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Length(input_height.saturating_add(2)),
                    Constraint::Length(0),
                ])
                .split(body_columns[1]);

            let conversation_area = body_chunks[0];

            // 将会话内容转换为可显示的行
            let (lines, think_map) = transcript_to_lines(
                transcript,
                self.expand_tool_blocks,
                self.expand_think_all,
                &self.think_detail_overrides,
                draft,
            );

            // 计算滚动位置
            let visible_height = usize::from(conversation_area.height);
            let base_scroll = u16_from_usize_saturating(lines.len().saturating_sub(visible_height));
            // scroll_back 是从底部向上的偏移，effective_scroll 是实际滚动位置
            let effective_scroll = base_scroll.saturating_sub(scroll_back.min(base_scroll));

            // 保存状态以支持后续鼠标交互
            frame_conversation_area = Some(conversation_area);
            frame_conversation_scroll = effective_scroll;
            frame_conversation_lines = lines.iter().map(|l| l.to_string()).collect();
            frame_conversation_think_map = think_map;

            // 渲染会话区域
            let transcript_area = ratatui::layout::Rect {
                x: conversation_area.x,
                y: conversation_area.y,
                width: conversation_area.width.saturating_sub(2).max(1),
                height: conversation_area.height,
            };
            let transcript_widget = Paragraph::new(Text::from(lines))
                .wrap(wrap_trim_disabled())
                .scroll((effective_scroll, 0));
            f.render_widget(transcript_widget, transcript_area);

            // 渲染滚动条
            scroll::render_scrollbar(
                f,
                conversation_area,
                &frame_conversation_lines,
                effective_scroll,
            );

            // 渲染输入面板
            let input_panel_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(0), Constraint::Min(1), Constraint::Length(0)])
                .split(body_chunks[1]);
            input_panel::render_input_panel(
                f,
                input_panel_chunks[1],
                input,
                cursor_idx,
                busy,
                model_name,
                self.spinner_idx,
            );

            let mut footer = Vec::new();
            if awaiting_clear_confirm {
                footer.push(Span::styled(
                    "confirm clear: type y or yes",
                    Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
                ));
            }
            f.render_widget(Paragraph::new(Line::from(footer)), chunks[3]);

            if show_menu {
                let overlay = layout::centered_overlay_rect(area, 2, 5, 10);
                let lines = vec![Line::from(Span::styled(
                    "暂无命令",
                    Style::default().fg(TEXT_SUBTLE),
                ))];
                let popup = Paragraph::new(Text::from(lines))
                    .block(Block::default().borders(Borders::ALL).title("Commands"));
                f.render_widget(Clear, overlay);
                f.render_widget(popup, overlay);
            }
        })?;

        // 更新状态缓存
        self.last_conversation_area = frame_conversation_area;
        self.last_conversation_scroll = frame_conversation_scroll;
        self.last_conversation_lines = frame_conversation_lines;
        self.last_conversation_think_map = frame_conversation_think_map;
        self.last_render_hash = Some(render_hash);
        let _ = now;
        Ok(())
    }
}

/// TUI 清理实现
///
/// 当 `CliTui` 实例被销毁时，自动执行以下清理操作：
/// 1. 禁用终端 raw 模式
/// 2. 禁用鼠标事件捕获
/// 3. 离开备用屏幕缓冲区（恢复原屏幕内容）
/// 4. 显示光标
///
/// 所有错误都会被静默忽略，确保清理过程不会 panic。
impl Drop for CliTui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(PopKeyboardEnhancementFlags);
        let _ = self.terminal.backend_mut().execute(DisableMouseCapture);
        let _ = self.terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
