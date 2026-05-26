//! # CLI 交互式循环核心
//!
//! 本模块实现了 CLI 交互式会话的主循环逻辑，负责处理用户输入、
//! 事件分发和界面更新。这是 VibeWindow 交互式命令行界面的核心调度器。
//!
//! ## 主要职责
//!
//! - 初始化 TUI（终端用户界面）环境
//! - 管理会话状态和历史记录
//! - 处理终端事件（键盘、鼠标、窗口大小调整）
//! - 协调各子系统间的交互（会话管理、统计、转录等）
//! - 维护应用程序状态机
//!
//! ## 事件流
//!
//! ```text
//! [终端事件] -> [事件轮询] -> [事件分发] -> [处理器执行] -> [界面重绘]
//! ```
//!
//! ## 模块关系
//!
//! - [`super::event_handlers`] - 处理各类终端事件
//! - [`super::input_submit`] - 处理用户输入提交和结果处理
//! - [`super::super::session`] - 会话创建和管理
//! - [`super::super::tui`] - 终端 UI 渲染

use super::event_handlers::{
    handle_key_press, handle_mouse_event, handle_resize_event, poll_and_tick,
};
use super::input_submit::{SubmitOutcome as FlowSubmitOutcome, handle_submit_result};
use crate::app::agent::config::Config;
use crate::session::ui_types as models;
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};

use super::super::session::{build_project_info, collect_modified_files, create_cli_session};
use super::super::setup::CliSetup;
use super::super::stats::{CliStats, build_session_title};
use super::super::stdio::StdIoRedirectGuard;
use super::super::transcript::{TranscriptEntry, TranscriptRole};
use super::super::tui::CliTui;
use crate::app::agent::session;

/// 运行 CLI 交互式主循环
///
/// 这是交互式 CLI 模式的入口点和主事件循环。该函数会阻塞执行，
/// 直到用户退出交互式会话（通过 `/exit` 命令或 Ctrl+C）。
///
/// ## 功能概述
///
/// 1. **初始化阶段**
///    - 设置标准 I/O 重定向到日志文件
///    - 创建 TUI 实例
///    - 初始化会话状态（历史、ID、转录记录等）
///    - 收集工作区信息
///
/// 2. **事件循环阶段**
///    - 轮询终端事件
///    - 分发事件到对应的处理器
///    - 处理用户输入提交
///    - 更新界面显示
///
/// 3. **清理阶段**
///    - 在退出时设置最终输出（如果需要）
///
/// ## 参数
///
/// - `config` - 代理配置，包含工作区目录、模型设置等
/// - `setup` - CLI 设置信息，包含 provider 名称、模型名称等
/// - `final_output` - 输出参数，用于存储会话结束时的最终输出内容
///
/// ## 返回值
///
/// - `Ok(())` - 用户正常退出交互式会话
/// - `Err(...)` - 发生错误（如终端初始化失败、事件处理错误等）
///
/// ## 示例
///
/// ```ignore
/// use crate::app::agent::config::Config;
/// use crate::app::agent::agent::loop_::cli::setup::CliSetup;
///
/// let config = Config::load()?;
/// let setup = CliSetup::new(&config)?;
/// let mut final_output = String::new();
///
/// // 运行交互式循环（阻塞直到用户退出）
/// run_interactive_loop(&config, &setup, &mut final_output).await?;
///
/// if !final_output.is_empty() {
///     println!("最终输出: {}", final_output);
/// }
/// ```
///
/// ## 状态管理
///
/// 函数内部维护以下状态：
///
/// | 状态变量 | 类型 | 用途 |
/// |---------|------|------|
/// | `session_history` | `Vec<ChatMessage>` | 会话消息历史 |
/// | `session_id` | `String` | 当前会话标识符 |
/// | `stream_id` | `u64` | 流标识符（毫秒时间戳） |
/// | `transcript` | `Vec<TranscriptEntry>` | 界面显示的转录记录 |
/// | `input` | `String` | 用户当前输入缓冲区 |
/// | `cursor_idx` | `usize` | 光标在输入中的位置 |
/// | `busy` | `bool` | 是否正在处理请求 |
/// | `awaiting_clear_confirm` | `bool` | 是否等待清屏确认 |
/// | `stats` | `CliStats` | 会话统计信息 |
/// | `modified_files` | `Vec<...>` | 已修改文件列表 |
/// | `files_collapsed` | `bool` | 文件列表是否折叠显示 |
/// | `draft` | `String` | 草稿内容 |
/// | `scroll_back` | `u16` | 滚动偏移量 |
/// | `show_menu` | `bool` | 是否显示菜单 |
///
/// ## 错误处理
///
/// 以下情况会导致错误返回：
/// - TUI 初始化失败（终端不支持、权限问题等）
/// - 标准输出重定向失败
/// - 事件读取错误
/// - 界面绘制错误
///
/// ## 线程安全性
///
/// 该函数不是线程安全的，应在单个异步任务中运行。
/// 所有状态都是局部可变变量，不涉及共享状态。
///
/// ## 资源管理
///
/// - `_stdio_redirect_guard` 会在函数返回时自动恢复标准 I/O
/// - TUI 资源通过 RAII 模式管理
///
/// ## 内部流程
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                      初始化阶段                              │
/// ├─────────────────────────────────────────────────────────────┤
/// │ 1. 重定向 stdio 到日志文件                                   │
/// │ 2. 创建 TUI 实例                                            │
/// │ 3. 初始化会话状态（ID、历史、转录、统计等）                    │
/// │ 4. 收集工作区信息和已修改文件                                 │
/// │ 5. 首次绘制界面                                              │
/// └─────────────────────────────────────────────────────────────┘
///                           ↓
/// ┌─────────────────────────────────────────────────────────────┐
/// │                      主事件循环                              │
/// │  ┌──────────────────────────────────────────────────────┐   │
/// │  │  轮询事件 (poll_and_tick)                             │   │
/// │  │   - 更新 TUI tick                                    │   │
/// │  │   - 处理待处理事件                                    │   │
/// │  └──────────────────────────────────────────────────────┘   │
/// │                          ↓                                   │
/// │  ┌──────────────────────────────────────────────────────┐   │
/// │  │  读取终端事件 (event::read)                           │   │
/// │  └──────────────────────────────────────────────────────┘   │
/// │                          ↓                                   │
/// │  ┌──────────────────────────────────────────────────────┐   │
/// │  │  事件类型分发                                         │   │
/// │  │   - Resize → handle_resize_event                     │   │
/// │  │   - Mouse  → handle_mouse_event                      │   │
/// │  │   - Key    → handle_key_press → handle_submit_result │   │
/// │  └──────────────────────────────────────────────────────┘   │
/// │                          ↓                                   │
/// │        [如果 submit 结果为 Exit，则退出循环]                  │
/// └─────────────────────────────────────────────────────────────┘
/// ```
pub(crate) async fn run_interactive_loop(
    config: &Config,
    setup: &CliSetup,
    final_output: &mut String,
) -> Result<()> {
    // =========================================================================
    // 初始化阶段
    // =========================================================================

    // 构建交互式日志文件路径
    // 日志保存在工作区 .vibewindow/logs 目录下
    let interactive_log_path =
        config.workspace_dir.join(".vibewindow").join("logs").join("interactive-terminal.log");

    // 重定向标准 I/O 到日志文件
    // 这确保后台输出不会干扰 TUI 显示
    // _stdio_redirect_guard 实现了 Drop，在作用域结束时自动恢复标准 I/O
    let _stdio_redirect_guard = StdIoRedirectGuard::redirect_to_file(&interactive_log_path)?;

    // 初始化终端用户界面
    // 这会设置终端为 raw 模式并准备备用屏幕缓冲区
    let mut tui = CliTui::new()?;

    // 获取当前工作目录作为请求根目录
    // 如果获取失败，则使用配置中的工作区目录
    let request_root_dir = std::env::current_dir().unwrap_or_else(|_| config.workspace_dir.clone());

    // =========================================================================
    // 会话状态初始化
    // =========================================================================

    // 会话消息历史：存储完整的对话消息（用于与 AI 模型交互）
    let mut session_history: Vec<models::ChatMessage> = Vec::new();

    // 当前会话 ID：唯一标识此次交互式会话
    let mut session_id = create_cli_session(&request_root_dir, None).await;

    // 流 ID：用于标识消息流，基于当前毫秒时间戳
    let mut stream_id = session::session::now_ms();

    // 会话标题是否已刷新：控制标题的动态更新
    let mut session_title_refreshed = false;

    // 转录记录：用于 TUI 显示的消息列表
    // 初始化时添加欢迎信息
    let mut transcript: Vec<TranscriptEntry> = vec![TranscriptEntry::new(
        TranscriptRole::System,
        "欢迎来到 VibeWindow 交互式模式，输入 /help 查看命令",
    )];

    // 用户输入缓冲区：存储当前正在编辑的文本
    let mut input = String::new();

    // 光标索引：光标在输入缓冲区中的位置（字节偏移）
    let mut cursor_idx: usize = 0;

    // 忙碌状态：标记是否正在处理 AI 请求
    // true 时会显示加载指示器并禁用部分交互
    let mut busy = false;

    // 等待清屏确认：标记是否在等待用户确认清屏操作
    // 用于防止意外清空会话历史
    let mut awaiting_clear_confirm = false;

    // CLI 统计信息：记录 token 使用量、请求次数等
    let mut stats = CliStats::default();

    // 工作区项目信息：包含文件结构、依赖等元数据
    let workspace = build_project_info(&config.workspace_dir);

    // 已修改文件列表：跟踪工作区中已变更的文件
    let mut modified_files = collect_modified_files(&config.workspace_dir);

    // 文件列表折叠状态：控制修改文件列表的展开/折叠
    let mut files_collapsed = true;

    // 草稿内容：存储临时编辑的内容
    let mut draft = String::new();

    // 滚动回退：记录向上滚动的行数
    let mut scroll_back: u16 = 0;

    // 显示菜单：控制菜单的可见性
    let mut show_menu = false;
    let mut exit_confirm_armed = false;

    // =========================================================================
    // 初始界面渲染
    // =========================================================================

    // 构建会话标题（包含 provider 和模型信息）
    let session_title = build_session_title(&stats, &setup.provider_name, &setup.model_name);

    // 首次绘制 TUI 界面
    // 这会在终端上渲染初始状态
    tui.draw(
        &transcript,
        &input,
        cursor_idx,
        busy,
        awaiting_clear_confirm,
        &setup.provider_name,
        &setup.model_name,
        &stats,
        &workspace,
        &draft,
        &session_title,
        &modified_files,
        files_collapsed,
        scroll_back,
        show_menu,
    )?;

    // =========================================================================
    // 主事件循环
    // =========================================================================

    loop {
        // -----------------------------------------------------------------
        // 步骤 1: 轮询事件并更新 TUI tick
        // -----------------------------------------------------------------
        // poll_and_tick 负责：
        // - 处理 TUI 动画帧更新（如光标闪烁）
        // - 检查是否有待处理的事件
        // - 如果返回 false，表示没有事件需要处理，继续循环
        if !poll_and_tick(
            &mut tui,
            busy,
            &transcript,
            &input,
            cursor_idx,
            awaiting_clear_confirm,
            &stats,
            &workspace,
            &draft,
            &setup.provider_name,
            &setup.model_name,
            &modified_files,
            files_collapsed,
            scroll_back,
            show_menu,
        )? {
            continue;
        }

        // -----------------------------------------------------------------
        // 步骤 2: 阻塞读取下一个终端事件
        // -----------------------------------------------------------------
        // 这会等待直到有事件发生（按键、鼠标、窗口调整等）
        let evt = event::read()?;

        // -----------------------------------------------------------------
        // 步骤 3: 处理窗口大小调整事件
        // -----------------------------------------------------------------
        // 当终端窗口大小改变时触发，需要重新计算布局并重绘
        if let CrosstermEvent::Resize(_, _) = evt {
            handle_resize_event(
                &mut tui,
                &transcript,
                &input,
                cursor_idx,
                busy,
                awaiting_clear_confirm,
                &stats,
                &workspace,
                &draft,
                &setup.provider_name,
                &setup.model_name,
                &modified_files,
                files_collapsed,
                scroll_back,
                show_menu,
            )?;
            continue;
        }

        // -----------------------------------------------------------------
        // 步骤 4: 处理鼠标事件
        // -----------------------------------------------------------------
        // 包括鼠标点击、滚动等交互
        if let CrosstermEvent::Mouse(mouse) = evt {
            handle_mouse_event(
                &mut tui,
                mouse,
                &mut scroll_back,
                &transcript,
                &input,
                cursor_idx,
                busy,
                awaiting_clear_confirm,
                &stats,
                &workspace,
                &draft,
                &setup.provider_name,
                &setup.model_name,
                &modified_files,
                files_collapsed,
                show_menu,
            )?;
            continue;
        }

        // -----------------------------------------------------------------
        // 步骤 5: 过滤非键盘事件
        // -----------------------------------------------------------------
        // 只处理键盘事件，忽略其他类型的事件
        let CrosstermEvent::Key(key) = evt else {
            continue;
        };

        // -----------------------------------------------------------------
        // 步骤 6: 过滤按键释放和重复事件
        // -----------------------------------------------------------------
        // 只处理实际的按键按下事件，避免重复触发
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // -----------------------------------------------------------------
        // 步骤 7: 处理键盘按键
        // -----------------------------------------------------------------
        // handle_key_press 返回 Option<String>:
        // - Some(user_input): 用户提交了输入（按下了 Enter）
        // - None: 只是普通按键（字符输入、光标移动等）
        let submit = handle_key_press(
            &mut tui,
            key,
            &mut transcript,
            &mut input,
            &mut cursor_idx,
            &mut scroll_back,
            &mut show_menu,
            &mut exit_confirm_armed,
        );

        // -----------------------------------------------------------------
        // 步骤 8: 处理用户输入提交
        // -----------------------------------------------------------------
        if let Some(user_input) = submit {
            // 执行用户提交的命令或消息
            // 这会：
            // 1. 解析命令（如 /help, /exit 等）
            // 2. 发送消息到 AI 模型
            // 3. 更新会话历史和转录
            // 4. 更新统计信息
            // 5. 处理文件修改
            let result = handle_submit_result(
                config,
                setup,
                user_input,
                &mut tui,
                &mut transcript,
                &mut session_history,
                &mut session_id,
                &mut stream_id,
                &mut session_title_refreshed,
                &input,
                &mut cursor_idx,
                &mut busy,
                &mut awaiting_clear_confirm,
                &mut stats,
                &workspace,
                &mut modified_files,
                &mut files_collapsed,
                &mut draft,
                &mut scroll_back,
                &mut show_menu,
                final_output,
            )
            .await?;

            // 如果返回 Exit，则退出主循环
            // 这通常发生在用户输入 /exit 命令时
            if result == FlowSubmitOutcome::Exit {
                break;
            }
        }
    }

    // 函数返回 Ok(()) 表示正常退出
    // 此时 _stdio_redirect_guard 会自动恢复标准 I/O
    Ok(())
}

/// 用户提交结果枚举
///
/// 表示用户输入提交后应该采取的后续行动。
/// 这个枚举用于控制主循环的流程。
///
/// ## 变体说明
///
/// - `Continue` - 继续主循环，等待下一个事件
/// - `Exit` - 退出交互式循环，结束会话
///
/// ## 使用场景
///
/// | 变体 | 触发条件 |
/// |------|----------|
/// | `Continue` | 普通消息、大部分命令、AI 响应完成 |
/// | `Exit` | 用户输入 `/exit` 命令或 `/quit` 命令 |
///
/// ## 示例
///
/// ```ignore
/// match submit_result {
///     SubmitOutcome::Continue => {
///         // 继续处理下一个事件
///     }
///     SubmitOutcome::Exit => {
///         // 清理资源并退出
///         break;
///     }
/// }
/// ```
///
/// ## 设计说明
///
/// 这个枚举是 `pub(crate)` 可见性，仅在本 crate 内部使用。
/// 它实现了 `PartialEq` 和 `Eq` trait，支持相等性比较。
#[derive(PartialEq, Eq)]
pub(crate) enum SubmitOutcome {
    /// 继续执行主循环
    ///
    /// 表示当前操作已完成，应继续等待下一个用户输入或事件。
    /// 这是绝大多数操作的默认结果。
    Continue,

    /// 退出交互式循环
    ///
    /// 表示用户请求退出，应在处理完当前状态后跳出主循环。
    /// 这会触发 `run_interactive_loop` 函数返回 `Ok(())`。
    Exit,
}
