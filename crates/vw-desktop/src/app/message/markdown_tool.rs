//! Markdown 工具消息处理模块
//!
//! 本模块负责处理 Markdown 编辑器工具的所有交互消息，包括：
//! - 文本编辑操作（粗体、斜体、标题等格式化）
//! - 图片插入与管理（本地文件选择、远程 URL 加载）
//! - 流式预览功能（模拟打字机效果）
//! - HTML 转 Markdown 功能
//! - 剪贴板操作
//!
//! # 模块结构
//!
//! - [`MarkdownToolMessage`]: 定义所有可能的消息类型
//! - [`update`]: 主更新函数，处理消息并更新应用状态
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::app::message::markdown_tool::{MarkdownToolMessage, update};
//!
//! // 处理编辑器动作
//! let task = update(&mut app, MarkdownToolMessage::InsertBold);
//! ```

use crate::app::components::markdown_editor::MarkdownViewMode;
use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::{markdown, text_editor};
use std::path::PathBuf;

/// Markdown 工具消息枚举
///
/// 定义了 Markdown 编辑器工具支持的所有消息类型，
/// 用于在用户界面和应用状态之间传递操作指令。
///
/// # 消息分类
///
/// - **编辑操作**: `EditorAction`, `InsertBold`, `InsertItalic` 等
/// - **视图控制**: `SetViewMode`, `ToggleStream`
/// - **图片管理**: `InsertImage`, `PickImageFile`, `RemoteImageLoaded` 等
/// - **HTML 转换**: `OpenHtml2Md`, `ConvertHtmlToMarkdown`
/// - **其他操作**: `Copy`, `Clear`, `ClearNotification`
#[derive(Debug, Clone)]
pub enum MarkdownToolMessage {
    /// 文本编辑器动作
    EditorAction(text_editor::Action),

    OpenContextMenu {
        x: f32,
        y: f32,
    },
    CloseContextMenu,
    ContextMenuCopy,
    ContextMenuCut,
    ContextMenuPaste,
    ContextMenuDelete,
    EditorWheelScrolled {
        delta: mouse::ScrollDelta,
        viewport_height: f32,
    },
    ScrollbarChanged {
        top_line: f32,
        viewport_height: f32,
    },

    /// 设置视图模式
    ///
    /// 切换编辑器的显示模式（编辑模式、预览模式、分屏模式）。
    SetViewMode(MarkdownViewMode),

    /// 清空编辑器内容
    ///
    /// 重置编辑器为空白状态，同时清除预览内容。
    Clear,

    /// 复制到剪贴板
    ///
    /// 将当前编辑器的全部内容复制到系统剪贴板，
    /// 并显示"已复制"的通知。
    Copy,

    /// 插入粗体格式
    ///
    /// 在光标位置插入 `**粗体**` 文本片段。
    InsertBold,

    /// 插入斜体格式
    ///
    /// 在光标位置插入 `*斜体*` 文本片段。
    InsertItalic,

    /// 插入删除线格式
    ///
    /// 在光标位置插入 `~~删除线~~` 文本片段。
    InsertStrike,

    /// 插入标题
    ///
    /// 在光标位置插入三级标题 `### 标题` 格式。
    InsertHeading,

    /// 插入引用块
    ///
    /// 在光标位置插入引用块 `> 引用` 格式。
    InsertQuote,

    /// 插入代码块
    ///
    /// 在光标位置插入围栏代码块格式：
    /// ```text
    /// ```text
    /// 代码块
    /// ```ignore
    /// ```
    InsertCodeBlock,

    /// 插入链接
    ///
    /// 在光标位置插入链接格式 `[链接文本](https://example.com)`。
    InsertLink,

    /// 插入表格
    ///
    /// 在光标位置插入一个两列表格模板，便于快速编辑 Markdown 表格。
    InsertTable,

    /// 打开图片插入对话框
    ///
    /// 显示图片输入界面，允许用户输入 URL 或选择本地文件。
    InsertImage,

    /// 关闭图片插入对话框
    ///
    /// 隐藏图片输入界面。
    CloseImage,

    /// 图片 URL 输入框内容变化
    ///
    /// 当用户在 URL 输入框中输入时触发，更新应用状态中的输入值。
    ImageUrlChanged(String),

    /// 从 URL 插入图片
    ///
    /// 使用当前输入的 URL 生成 Markdown 图片语法并插入到编辑器中。
    InsertImageFromUrl,

    /// 打开图片文件选择器
    ///
    /// 启动系统文件选择对话框，允许用户选择图片文件。
    /// 仅在非 WASM 目标平台可用。
    PickImageFile,

    /// 图片文件选择完成
    ///
    /// 用户选择文件后的回调，包含选中的文件路径。
    /// 如果用户取消选择，则参数为 `None`。
    ImagePicked(Option<PathBuf>),

    /// 获取远程图片
    ///
    /// 手动触发从指定 URL 下载图片的操作。
    FetchRemoteImage(String),

    /// 远程图片加载完成
    ///
    /// 异步下载图片完成后的回调，包含图片 URL 和下载结果。
    /// 成功时包含图片的字节数据，失败时包含错误信息。
    RemoteImageLoaded(String, Result<Vec<u8>, String>),

    /// 切换流式显示模式
    ///
    /// 启用或禁用流式预览效果，模拟打字机逐字显示。
    /// 参数为 `true` 时启用，`false` 时禁用。
    ToggleStream(bool),

    /// 流式显示时钟滴答
    ///
    /// 定时触发，每次增加显示的字符数，实现流式动画效果。
    StreamTick,

    /// 打开 HTML 转 Markdown 对话框
    ///
    /// 显示 HTML 输入界面，允许用户粘贴 HTML 内容进行转换。
    OpenHtml2Md,

    /// 关闭 HTML 转 Markdown 对话框
    ///
    /// 隐藏 HTML 输入界面。
    CloseHtml2Md,

    /// HTML 编辑器动作
    ///
    /// HTML 输入框的编辑操作，由 iced 的 `text_editor` 组件产生。
    HtmlEditorAction(text_editor::Action),

    /// 执行 HTML 到 Markdown 的转换
    ///
    /// 解析 HTML 输入框中的内容，转换为 Markdown 格式，
    /// 并替换主编辑器的内容。
    ConvertHtmlToMarkdown,

    /// 清除通知消息
    ///
    /// 移除当前显示的通知提示（如"已复制"、"已插入图片"等）。
    ClearNotification,
}

/// 流式显示时每次增加的字符数
///
/// 用于控制流式预览效果的动画速度，值越大显示越快。
const STREAM_STEP_CHARS: usize = 60;

/// 从 Markdown 文本中提取远程图片 URL
///
/// 解析 Markdown 内容，识别所有图片链接，
/// 并筛选出以 `http://` 或 `https://` 开头的远程 URL。
///
/// # 参数
///
/// - `md`: Markdown 文本内容
///
/// # 返回值
///
/// 返回去重排序后的远程图片 URL 列表
///
/// # 示例
///
/// ```ignore
/// let urls = extract_remote_image_urls("![alt](https://example.com/img.png)");
/// assert_eq!(urls, vec!["https://example.com/img.png"]);
/// ```
#[allow(dead_code)]
fn extract_remote_image_urls(md: &str) -> Vec<String> {
    use pulldown_cmark::{Event, Options, Parser, Tag};

    let mut urls = Vec::new();
    // 使用 pulldown_cmark 解析 Markdown，启用表格和任务列表扩展
    let parser = Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS);

    // 遍历解析事件，提取图片标签中的 URL
    for ev in parser {
        if let Event::Start(Tag::Image(_, url, _)) = ev {
            let s = url.to_string();
            // 仅保留 HTTP/HTTPS 协议的远程 URL
            if s.starts_with("http://") || s.starts_with("https://") {
                urls.push(s);
            }
        }
    }

    // 排序并去重，确保每个 URL 只出现一次
    urls.sort();
    urls.dedup();
    urls
}

/// 触发远程图片的异步加载
///
/// 扫描编辑器内容中的远程图片 URL，
/// 对尚未加载且未在加载中的图片发起异步下载请求。
///
/// # 参数
///
/// - `app`: 可变引用应用状态
///
/// # 返回值
///
/// 返回包含所有新启动的下载任务的 `Task`
///
/// # 平台差异
///
/// - WASM 目标: 返回空任务（不支持远程图片加载）
/// - Native 目标: 执行实际的 HTTP 请求
fn trigger_remote_fetches(app: &mut App) -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        // WASM 平台暂不支持远程图片加载
        let _ = app;
        Task::none()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 提取编辑器中的所有远程图片 URL
        let urls = extract_remote_image_urls(&app.markdown_tool_editor.text());
        let mut tasks: Vec<Task<Message>> = Vec::new();

        for url in urls {
            // 跳过已加载完成的图片
            if app.markdown_tool_remote_images.contains_key(&url) {
                continue;
            }
            // 跳过正在加载中的图片
            if app.markdown_tool_remote_images_loading.contains(&url) {
                continue;
            }

            // 标记该 URL 为加载中状态
            app.markdown_tool_remote_images_loading.insert(url.clone());

            // 创建异步下载任务
            tasks.push(Task::perform(
                {
                    let url = url.clone();
                    async move {
                        // 发起 HTTP GET 请求
                        let resp = reqwest::get(&url)
                            .await
                            .map_err(|e| format!("request failed: {}", e))?;
                        // 读取响应体为字节数组
                        let bytes =
                            resp.bytes().await.map_err(|e| format!("read body failed: {}", e))?;
                        Ok(bytes.to_vec())
                    }
                },
                move |res| {
                    // 将下载结果封装为消息
                    Message::MarkdownTool(MarkdownToolMessage::RemoteImageLoaded(url.clone(), res))
                },
            ));
        }

        // 批量执行所有下载任务
        Task::batch(tasks)
    }
}

/// 刷新 Markdown 预览内容
///
/// 根据编辑器的当前内容和流式显示设置，
/// 更新预览区域的 Markdown 解析结果。
///
/// # 参数
///
/// - `app`: 可变引用应用状态
///
/// # 行为说明
///
/// - 流式模式启用时：仅解析当前已显示字符数对应的内容
/// - 流式模式禁用时：解析编辑器的完整内容
fn refresh_preview(app: &mut App) {
    let full = app.markdown_tool_editor.text();

    if app.markdown_tool_stream_enabled {
        // 流式模式：仅解析部分内容
        let total_chars = full.chars().count();
        // 确保当前显示字符数不超过总字符数
        app.markdown_tool_stream_chars = app.markdown_tool_stream_chars.min(total_chars);

        // 计算字符索引对应的字节位置
        let byte_idx = full
            .char_indices()
            .nth(app.markdown_tool_stream_chars)
            .map(|(i, _)| i)
            .unwrap_or(full.len());

        // 解析部分内容用于预览
        app.markdown_tool_content = markdown::Content::parse(&full[..byte_idx]);
    } else {
        // 非流式模式：解析完整内容
        app.markdown_tool_content = markdown::Content::parse(&full);
    }
}

/// 替换编辑器的全部内容
///
/// 使用新文本完全替换当前编辑器内容，并刷新预览。
///
/// # 参数
///
/// - `app`: 可变引用应用状态
/// - `text`: 新的 Markdown 文本内容
fn replace_markdown(app: &mut App, text: String) {
    // 创建包含新文本的编辑器内容
    app.markdown_tool_editor = text_editor::Content::with_text(&text);
    // 刷新预览以反映更改
    refresh_preview(app);
}

/// 在光标位置粘贴文本片段
///
/// 将指定的文本片段插入到编辑器当前光标位置，
/// 并刷新预览内容。
///
/// # 参数
///
/// - `app`: 可变引用应用状态
/// - `snippet`: 要插入的文本片段
fn paste_snippet(app: &mut App, snippet: &str) {
    // 使用粘贴操作插入文本片段
    app.markdown_tool_editor.perform(text_editor::Action::Edit(text_editor::Edit::Paste(
        std::sync::Arc::new(snippet.to_string()),
    )));
    // 刷新预览
    refresh_preview(app);
}

/// 处理 Markdown 工具消息的主更新函数
///
/// 根据接收到的消息类型执行相应的操作，
/// 更新应用状态并返回需要执行的异步任务。
///
/// # 参数
///
/// - `app`: 可变引用应用状态，包含所有 Markdown 工具相关的状态字段
/// - `message`: 要处理的消息
///
/// # 返回值
///
/// 返回可能需要执行的异步任务，如剪贴板写入、文件选择、图片下载等
///
/// # 消息处理说明
///
/// 该函数处理以下类别的消息：
///
/// ## 视图控制
/// - `SetViewMode`: 切换编辑/预览/分屏模式
/// - `ToggleStream`: 启用/禁用流式显示
/// - `StreamTick`: 推进流式动画
///
/// ## 编辑操作
/// - `EditorAction`: 处理原始编辑器动作
/// - `Clear`: 清空内容
/// - `Copy`: 复制到剪贴板
/// - 格式化操作: `InsertBold`, `InsertItalic` 等
///
/// ## 图片管理
/// - `InsertImage`/`CloseImage`: 打开/关闭图片对话框
/// - `ImageUrlChanged`: 更新 URL 输入
/// - `InsertImageFromUrl`: 从 URL 插入
/// - `PickImageFile`/`ImagePicked`: 选择本地文件
/// - `FetchRemoteImage`/`RemoteImageLoaded`: 加载远程图片
///
/// ## HTML 转换
/// - `OpenHtml2Md`/`CloseHtml2Md`: 打开/关闭转换对话框
/// - `HtmlEditorAction`: HTML 编辑器操作
/// - `ConvertHtmlToMarkdown`: 执行转换
///
/// ## 其他
/// - `ClearNotification`: 清除通知消息
pub fn update(app: &mut App, message: MarkdownToolMessage) -> Task<Message> {
    match message {
        MarkdownToolMessage::OpenContextMenu { x, y } => {
            app.markdown_tool_context_menu_open = true;
            app.markdown_tool_context_menu_pos = Some((x, y));
            Task::none()
        }
        MarkdownToolMessage::CloseContextMenu => {
            close_context_menu(app);
            focus_editor_task(&app.markdown_tool_editor_id)
        }
        MarkdownToolMessage::ContextMenuCopy => {
            close_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.markdown_tool_editor, &app.markdown_tool_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                app.markdown_tool_notification = Some("已复制".to_string());
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        MarkdownToolMessage::ContextMenuCut => {
            close_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.markdown_tool_editor, &app.markdown_tool_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                app.markdown_tool_notification = Some("已剪切".to_string());
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        MarkdownToolMessage::ContextMenuPaste => {
            close_context_menu(app);
            paste_task(&app.markdown_tool_editor_id, |content| {
                Message::MarkdownTool(MarkdownToolMessage::EditorAction(paste_action(content)))
            })
        }
        MarkdownToolMessage::ContextMenuDelete => {
            close_context_menu(app);
            let (_outcome, task) =
                selection_delete_task(&mut app.markdown_tool_editor, &app.markdown_tool_editor_id);
            task
        }
        MarkdownToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            close_context_menu(app);
            app.markdown_tool_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.markdown_tool_scroll_remainder += delta_lines;

            let whole_lines = if app.markdown_tool_scroll_remainder >= 0.0 {
                app.markdown_tool_scroll_remainder.floor() as i32
            } else {
                app.markdown_tool_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.markdown_tool_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.markdown_tool_editor
                    .perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        MarkdownToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            close_context_menu(app);
            app.markdown_tool_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.markdown_tool_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.markdown_tool_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        MarkdownToolMessage::SetViewMode(mode) => {
            app.markdown_tool_view_mode = mode;
            Task::none()
        }

        // 处理编辑器动作
        MarkdownToolMessage::EditorAction(action) => {
            close_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            let is_edit = action.is_edit();
            app.markdown_tool_editor.perform(action);
            if is_edit {
                refresh_preview(app);
                return trigger_remote_fetches(app);
            }
            Task::none()
        }

        // 清空编辑器内容
        MarkdownToolMessage::Clear => {
            app.markdown_tool_editor = text_editor::Content::new();
            refresh_preview(app);
            Task::none()
        }

        // 复制内容到剪贴板
        MarkdownToolMessage::Copy => {
            let text = app.markdown_tool_editor.text();
            app.markdown_tool_notification = Some("已复制".to_string());
            // 返回剪贴板写入任务和延迟清除通知的任务
            Task::batch(vec![
                iced::clipboard::write(text),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::MarkdownTool(MarkdownToolMessage::ClearNotification),
                ),
            ])
        }

        // 插入粗体格式
        MarkdownToolMessage::InsertBold => {
            paste_snippet(app, "**粗体**");
            trigger_remote_fetches(app)
        }

        // 插入斜体格式
        MarkdownToolMessage::InsertItalic => {
            paste_snippet(app, "*斜体*");
            trigger_remote_fetches(app)
        }

        // 插入删除线格式
        MarkdownToolMessage::InsertStrike => {
            paste_snippet(app, "~~删除线~~");
            trigger_remote_fetches(app)
        }

        // 插入标题格式
        MarkdownToolMessage::InsertHeading => {
            paste_snippet(app, "### 标题");
            trigger_remote_fetches(app)
        }

        // 插入引用块格式
        MarkdownToolMessage::InsertQuote => {
            paste_snippet(app, "> 引用");
            trigger_remote_fetches(app)
        }

        // 插入代码块格式
        MarkdownToolMessage::InsertCodeBlock => {
            paste_snippet(app, "```text\n代码块\n```");
            trigger_remote_fetches(app)
        }

        // 插入链接格式
        MarkdownToolMessage::InsertLink => {
            paste_snippet(app, "[链接文本](https://example.com)");
            trigger_remote_fetches(app)
        }

        // 打开图片插入对话框
        MarkdownToolMessage::InsertImage => {
            app.markdown_tool_show_image = true;
            Task::none()
        }

        // 关闭图片插入对话框
        MarkdownToolMessage::CloseImage => {
            app.markdown_tool_show_image = false;
            Task::none()
        }

        // 更新图片 URL 输入框内容
        MarkdownToolMessage::ImageUrlChanged(v) => {
            app.markdown_tool_image_url_input = v;
            Task::none()
        }

        // 从 URL 插入图片
        MarkdownToolMessage::InsertImageFromUrl => {
            let url = app.markdown_tool_image_url_input.trim();
            if !url.is_empty() {
                // 生成 Markdown 图片语法
                let snippet = format!("![]({})", url);
                paste_snippet(app, &snippet);
                app.markdown_tool_show_image = false;
                app.markdown_tool_notification = Some("已插入图片".to_string());
                // 返回图片加载任务和延迟清除通知的任务
                return Task::batch(vec![
                    trigger_remote_fetches(app),
                    crate::app::message::after(
                        std::time::Duration::from_secs(2),
                        Message::MarkdownTool(MarkdownToolMessage::ClearNotification),
                    ),
                ]);
            }
            Task::none()
        }

        // 打开文件选择器选择图片
        MarkdownToolMessage::PickImageFile => Task::perform(
            async {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // 在原生平台上打开文件选择对话框
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                        .pick_file()
                        .await;
                    handle.map(|f| f.path().to_path_buf())
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // WASM 平台暂不支持文件选择
                    None
                }
            },
            |opt| Message::MarkdownTool(MarkdownToolMessage::ImagePicked(opt)),
        ),

        // 处理选中的图片文件
        MarkdownToolMessage::ImagePicked(opt) => {
            if let Some(path) = opt {
                // 使用文件名（不含扩展名）作为替代文本
                let alt = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                let p = path.to_string_lossy();
                // 生成 Markdown 图片语法，使用尖括号包裹路径以处理包含空格的路径
                let snippet = format!("![{}](<{}>)", alt, p);
                paste_snippet(app, &snippet);
                app.markdown_tool_show_image = false;
                app.markdown_tool_notification = Some("已插入图片".to_string());
                return Task::batch(vec![
                    trigger_remote_fetches(app),
                    crate::app::message::after(
                        std::time::Duration::from_secs(2),
                        Message::MarkdownTool(MarkdownToolMessage::ClearNotification),
                    ),
                ]);
            }
            Task::none()
        }

        // 插入表格格式
        MarkdownToolMessage::InsertTable => {
            paste_snippet(app, "| header1 | header2 |\n| --- | --- |\n| cell1 | cell2 |");
            trigger_remote_fetches(app)
        }

        // 手动触发远程图片加载
        MarkdownToolMessage::FetchRemoteImage(url) => {
            #[cfg(target_arch = "wasm32")]
            {
                let _ = url;
                Task::none()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // 检查图片是否已加载或正在加载
                if app.markdown_tool_remote_images.contains_key(&url)
                    || app.markdown_tool_remote_images_loading.contains(&url)
                {
                    return Task::none();
                }
                // 标记为加载中
                app.markdown_tool_remote_images_loading.insert(url.clone());
                // 创建异步下载任务
                Task::perform(
                    {
                        let url = url.clone();
                        async move {
                            let resp = reqwest::get(&url)
                                .await
                                .map_err(|e| format!("request failed: {}", e))?;
                            let bytes = resp
                                .bytes()
                                .await
                                .map_err(|e| format!("read body failed: {}", e))?;
                            Ok(bytes.to_vec())
                        }
                    },
                    move |res| {
                        Message::MarkdownTool(MarkdownToolMessage::RemoteImageLoaded(
                            url.clone(),
                            res,
                        ))
                    },
                )
            }
        }

        // 远程图片加载完成
        MarkdownToolMessage::RemoteImageLoaded(url, res) => {
            // 从加载中集合移除
            app.markdown_tool_remote_images_loading.remove(&url);
            // 成功时将图片数据存入缓存
            if let Ok(bytes) = res {
                app.markdown_tool_remote_images
                    .insert(url, iced::widget::image::Handle::from_bytes(bytes));
            }
            Task::none()
        }

        // 切换流式显示模式
        MarkdownToolMessage::ToggleStream(enabled) => {
            app.markdown_tool_stream_enabled = enabled;
            // 启用时从 0 开始，禁用时设置为最大值以立即显示全部内容
            app.markdown_tool_stream_chars = if enabled { 0 } else { usize::MAX };
            refresh_preview(app);
            Task::none()
        }

        // 流式显示时钟滴答
        MarkdownToolMessage::StreamTick => {
            if app.markdown_tool_stream_enabled {
                let full = app.markdown_tool_editor.text();
                let total_chars = full.chars().count();
                // 如果还有未显示的字符，增加显示数量
                if app.markdown_tool_stream_chars < total_chars {
                    app.markdown_tool_stream_chars =
                        (app.markdown_tool_stream_chars + STREAM_STEP_CHARS).min(total_chars);
                    refresh_preview(app);
                }
            }
            Task::none()
        }

        // 打开 HTML 转 Markdown 对话框
        MarkdownToolMessage::OpenHtml2Md => {
            app.markdown_tool_show_html2md = true;
            Task::none()
        }

        // 关闭 HTML 转 Markdown 对话框
        MarkdownToolMessage::CloseHtml2Md => {
            app.markdown_tool_show_html2md = false;
            Task::none()
        }

        // HTML 编辑器动作
        MarkdownToolMessage::HtmlEditorAction(action) => {
            app.markdown_tool_html_editor.perform(action);
            Task::none()
        }

        // 执行 HTML 到 Markdown 的转换
        MarkdownToolMessage::ConvertHtmlToMarkdown => {
            let html = app.markdown_tool_html_editor.text();
            // 使用 html2md 库解析 HTML 并转换为 Markdown
            let md = html2md::parse_html(&html);
            replace_markdown(app, md);
            app.markdown_tool_show_html2md = false;
            app.markdown_tool_notification = Some("已转换".to_string());
            Task::batch(vec![
                trigger_remote_fetches(app),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::MarkdownTool(MarkdownToolMessage::ClearNotification),
                ),
            ])
        }

        // 清除通知消息
        MarkdownToolMessage::ClearNotification => {
            app.markdown_tool_notification = None;
            Task::none()
        }
    }
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.markdown_tool_viewport_height / line_height).floor().max(1.0)
}

fn close_context_menu(app: &mut App) {
    app.markdown_tool_context_menu_open = false;
    app.markdown_tool_context_menu_pos = None;
}

fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.markdown_tool_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }
    let max_scroll = max_scroll_top_line(app);
    app.markdown_tool_scroll_top_line =
        (app.markdown_tool_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::MarkdownTool(MarkdownToolMessage::ClearNotification),
    )
}

#[cfg(test)]
#[path = "markdown_tool_tests.rs"]
mod markdown_tool_tests;
