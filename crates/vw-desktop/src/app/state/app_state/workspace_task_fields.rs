    /// 活跃的图层菜单 ID
    pub active_layer_menu: Option<String>,
    /// 图层菜单锚点位置
    pub layer_menu_anchor: Option<Point>,
    /// 正在拖动的图层 ID
    pub dragging_layer: Option<String>,
    /// 拖动目标图层 ID
    pub drag_target_layer: Option<String>,
    /// 悬停的图层 ID
    pub hovered_layer_id: Option<String>,
    /// 错误消息
    pub error_message: Option<String>,
    /// 活跃的颜色选择器
    pub active_color_picker: Option<ActiveColorPicker>,
    /// 活跃的填充选择器
    pub active_fill_picker: Option<ActiveFillPicker>,
    /// 活跃的效果选择器
    pub active_effect_picker: Option<ActiveEffectPicker>,
    /// 活跃的字体选择器
    pub active_font_picker: Option<ActiveFontPicker>,
    /// 活跃的图标选择器
    pub active_icon_picker: Option<ActiveIconPicker>,
    /// 活跃的 Tailwind 类选择器
    pub active_tailwind_class_picker: Option<ActiveTailwindClassPicker>,
    /// 鼠标光标位置
    pub cursor_position: Point,
    /// 是否显示元素 HTML 预览
    pub show_element_html_preview: bool,
    /// 元素 HTML 预览编辑器
    pub element_html_preview_editor: text_editor::Content,
    /// 设计帮助文本
    pub design_help_text: Option<String>,
    /// Tailwind 类过滤器查询
    pub tailwind_filter_query: String,
    /// 字体过滤器查询
    pub(crate) font_filter_query: String,
    /// 图标过滤器查询
    pub(crate) icon_picker_filter_query: String,
    /// 图标选择器当前图标库
    pub(crate) icon_picker_family_tab: String,
    /// 是否显示预览设置
    pub(crate) show_preview_settings: bool,
    /// 是否显示预览右上角全屏浮动控件
    pub(crate) show_preview_fullscreen_overlay: bool,
    /// 当前字体大小
    pub(crate) current_font_size: f32,
    /// 当前行高
    pub(crate) current_line_height: f32,
    /// 是否自动调整行高
    pub(crate) auto_adjust_line_height: bool,
    /// 预览编辑器自动保存模式
    pub(crate) preview_auto_save_mode: crate::app::PreviewAutoSaveMode,
    /// 当前语言设置
    pub(crate) current_language: iced_code_editor::i18n::Language,
    /// 待丢弃的文件路径
    pub(crate) file_to_discard: Option<String>,
    /// 当前活跃的标签页 ID
    pub(crate) active_tab_id: Option<String>,
    /// 悬停的标签页 ID
    pub(crate) hovered_tab_id: Option<String>,
    /// 悬停的最近项目路径
    pub(crate) hovered_recent_project: Option<String>,
    /// 打开的标签页列表
    pub(crate) open_tabs: Vec<AppTab>,
    /// 应用搜索查询
    pub apps_search_query: String,
    /// 网页书签列表
    pub web_bookmarks: Vec<WebBookmark>,
    /// 是否显示网页链接菜单
    pub show_web_links_menu: bool,
    /// 网页书签标题输入
    pub web_bookmark_title_input: String,
    /// 网页书签 URL 输入
    pub web_bookmark_url_input: String,
    /// 网页书签宽度输入
    pub web_bookmark_width_input: String,
    /// 网页书签高度输入
    pub web_bookmark_height_input: String,
    /// 正在编辑的网页书签索引
    pub editing_web_bookmark: Option<usize>,
    /// 编辑网页书签标题输入
    pub edit_web_bookmark_title_input: String,
    /// 编辑网页书签 URL 输入
    pub edit_web_bookmark_url_input: String,
    /// 编辑网页书签宽度输入
    pub edit_web_bookmark_width_input: String,
    /// 编辑网页书签高度输入
    pub edit_web_bookmark_height_input: String,
    /// 编辑网页书签 Cookie 配置编辑器
    pub edit_web_bookmark_cookie_configs_editor: text_editor::Content,
    /// 独立 WebView 子进程列表（仅非 wasm32 平台）
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) independent_webview_children: Vec<std::process::Child>,
    /// 是否显示插槽内容
    pub show_slot_content: bool,
    /// 是否显示插槽溢出
    pub show_slot_overflow: bool,
    /// 加载动画帧索引
    pub spinner_frame: usize,
    /// 通知列表
    pub(crate) notifications: Vec<Notification>,
    /// 通知面板是否展开
    pub(crate) notifications_expanded: bool,
    /// 下一个通知 ID
    pub(crate) next_notification_id: usize,
    /// 通知滚动容器 ID
    pub(crate) notifications_scroll_id: Id,
    /// 通知消息编辑器映射（通知 ID -> 编辑器内容）
    pub(crate) notification_editors: HashMap<usize, text_editor::Content>,
    /// 最近点击复制的通知 ID
    pub(crate) copied_notification_id: Option<usize>,
    /// 当前显示的轻量提示
    pub(crate) active_toast: Option<Toast>,
    /// 下一个轻量提示 ID
    pub(crate) next_toast_id: usize,
    /// 是否显示任务看板
    pub show_task_board: bool,
    /// 任务看板是否正在加载
    pub task_board_loading: bool,
    /// 任务看板任务列表
    pub task_board_tasks: Vec<crate::app::task::Task>,
    /// 任务看板创建对话框是否打开
    pub task_board_create_modal_open: bool,
    /// 任务看板草稿
    pub task_board_draft: crate::app::task::TaskDraft,
    /// 任务看板上次使用的模型
    pub task_board_last_model: String,
    /// 任务看板上次使用的 ACP 智能体
    pub task_board_last_acp_agent: Option<String>,
    /// 任务看板批量选中的任务 ID 集合
    pub task_board_selected_tasks: HashSet<String>,
    /// 当前启用批量操作的任务状态列
    pub task_board_bulk_active_status: Option<crate::app::task::TaskStatus>,
    /// 任务看板批量设置优先级输入
    pub task_board_bulk_priority_input: String,
    /// 任务看板批量设置模型输入
    pub task_board_bulk_model_input: String,
    /// 任务看板批量设置 ACP 智能体
    pub task_board_bulk_acp_agent: Option<String>,
    /// 任务看板选中的任务 ID
    pub task_board_selected_task: Option<String>,
    /// 任务看板正在查看日志的任务
    pub task_board_viewing_logs: Option<crate::app::task::Task>,
    /// 任务看板按任务 ID 缓存的内存日志
    pub task_board_log_cache: HashMap<String, Vec<crate::app::task::TaskLogEntry>>,
    /// 任务看板正在编辑的任务 ID
    pub task_board_editing_task_id: Option<String>,
    /// 任务看板拖动中的任务（任务 ID，状态）
    pub task_board_dragging: Option<(String, crate::app::task::TaskStatus)>,
    /// 任务看板待放置的任务（任务 ID，目标状态，位置）
    pub task_board_drag_pending: Option<(String, crate::app::task::TaskStatus, iced::Point)>,
    /// 任务看板状态过滤器
    pub task_board_filter_status: Option<crate::app::task::TaskStatus>,
    /// 任务看板优先级过滤器（最小值，最大值）
    pub task_board_filter_priority: Option<(u32, u32)>,
    /// 任务看板是否按优先级排序
    pub task_board_sort_by_priority: bool,
    /// 任务看板是否升序排序
    pub task_board_sort_ascending: bool,
    /// 任务看板设置
    pub task_board_settings: crate::app::task::TaskBoardSettings,
    /// 任务看板执行器状态
    pub task_board_executor: crate::app::task::TaskExecutorState,
    /// 任务看板执行器是否正在运行
    pub task_board_executor_running: bool,
    /// 任务看板下次刷新时间（毫秒时间戳）
    pub task_board_next_refresh_at_ms: u64,
    /// 任务看板下次调度器触发时间（毫秒时间戳）
    pub task_board_next_scheduler_tick_at_ms: u64,
    /// 任务看板下次自动审核触发时间（毫秒时间戳）
    pub task_board_next_auto_review_tick_at_ms: u64,
    /// 任务看板下次自动晋升触发时间（毫秒时间戳）
    pub task_board_next_auto_promote_tick_at_ms: u64,
    /// 任务看板上次日志刷新时间（毫秒时间戳）
    pub task_board_last_log_flush_at_ms: u64,
    /// 任务看板日志扫描游标
    pub task_board_log_scan_cursor: usize,
    /// 任务看板超时扫描游标
    pub task_board_timeout_scan_cursor: usize,
    /// 任务看板调度扫描游标
    pub task_board_schedule_scan_cursor: usize,
    /// 任务看板执行器弹出框是否打开
    pub task_board_executor_popover: bool,
    /// 任务看板批量执行器弹出框是否打开
    pub task_board_bulk_executor_popover: bool,
    /// 任务看板批量模型弹出框是否打开
    pub task_board_bulk_model_popover: bool,
    /// 任务看板 worktree 维护是否进行中
    pub task_board_worktree_maintenance_in_flight: bool,
    /// 任务看板 worktree 快照是否正在加载
    pub task_board_worktree_snapshot_loading: bool,
    /// 任务看板手动 worktree 操作类型（cleanup / merge）
    pub task_board_worktree_manual_action_kind: Option<&'static str>,
    /// 任务看板手动 worktree 二次确认操作类型（cleanup / merge）
    pub task_board_worktree_manual_confirm_kind: Option<&'static str>,
    /// 任务看板手动 worktree 操作日志
    pub task_board_worktree_action_logs: Vec<String>,
    /// 任务看板手动 worktree 操作日志自动隐藏截止时间（毫秒时间戳）
    pub task_board_worktree_action_logs_visible_until_ms: Option<u64>,
    /// 任务看板手动 worktree 操作日志接收器
    pub task_board_worktree_action_log_rx:
        Option<std::sync::mpsc::Receiver<crate::app::task::executor::TaskLogStream>>,
    /// 任务看板 worktree 池快照
    pub task_board_worktree_snapshot: Option<crate::app::task::WorktreePoolSnapshot>,
    /// 任务看板上次 worktree 快照时间（毫秒时间戳）
    pub task_board_last_worktree_snapshot_at_ms: u64,
    /// 任务看板 worktree 面板是否展开
    pub task_board_worktree_panel_expanded: bool,
    /// 任务看板是否显示像素办公室
    pub task_board_worktree_pixel_office: bool,
    /// 任务看板上下文菜单（任务 ID，X，Y）
    pub task_board_context_menu: Option<(String, f32, f32)>,
    /// 任务看板新建子任务内容
    pub task_board_new_subtask_content: String,
    /// 任务看板描述编辑器
    pub task_board_desc_editor: text_editor::Content,
    /// 任务看板提示词编辑器
    pub task_board_prompt_editor: text_editor::Content,
    /// 任务看板日志查看器（只读，可选择复制）
    pub task_board_logs_editor: text_editor::Content,
    /// 任务看板日志编辑器 ID
    pub task_board_logs_editor_id: Id,
    /// 任务看板日志是否自动跟随到底部
    pub task_board_logs_auto_scroll: bool,
    /// 任务看板日志右键菜单是否打开
    pub task_board_logs_context_menu_open: bool,
    /// 任务看板日志右键菜单位置
    pub task_board_logs_context_menu_pos: Option<(f32, f32)>,
    /// 任务看板日志当前顶部滚动行号
    pub task_board_logs_scroll_top_line: f32,
    /// 任务看板日志滚轮累积的小数行偏移
    pub task_board_logs_scroll_remainder: f32,
    /// 任务看板日志编辑器视口高度
    pub task_board_logs_viewport_height: f32,
    /// 任务看板模型选择弹出框是否打开
    pub task_board_model_popover: bool,
    /// 任务看板创建后是否清除提示词
    pub task_board_clear_prompt_after_create: bool,
    /// 任务看板创建后是否关闭对话框
    pub task_board_close_after_create: bool,
    /// 任务看板编辑后是否关闭详情面板
    pub task_board_close_after_edit: bool,
    /// 任务看板创建是否成功
    pub task_board_create_submit_success: bool,
    /// 任务看板编辑是否成功
    pub task_board_edit_submit_success: bool,
    /// 任务看板设置对话框是否打开
    pub task_board_settings_modal_open: bool,
    /// 任务看板设置对话框当前标签页
    pub task_board_settings_modal_tab: TaskBoardSettingsModalTab,
    /// 任务看板是否为导入模式
    pub task_board_is_import_mode: bool,
    /// 任务看板导入编辑器
    pub task_board_import_editor: text_editor::Content,
    /// 任务看板导入提示词格式
    pub task_board_import_prompt_format: crate::app::task::TaskImportPromptFormat,
    /// 任务看板导入提示词面板是否折叠
    pub task_board_import_prompt_collapsed: bool,
    /// 任务看板每个状态列是否显示纵向滚动条
    pub task_board_column_has_vertical_scrollbar: HashMap<crate::app::task::TaskStatus, bool>,