    /// 项目失败重试分钟数输入
    pub(crate) project_edit_failed_retry_minutes_input: String,
    /// 项目运行超时分钟数输入
    pub(crate) project_edit_running_timeout_minutes_input: String,
    /// 项目 PR 提交停滞超时秒数输入
    pub(crate) project_edit_pr_submitted_stall_timeout_seconds_input: String,
    /// 是否显示差异视图
    pub(crate) show_diff: bool,
    /// 是否显示 Git 差异摘要
    pub(crate) show_git_diff_summary: bool,
    /// 是否显示 Git 差异高亮
    pub(crate) show_git_diff_highlight: bool,
    /// 终端状态（主终端）
    pub terminal: TerminalState,
    /// 按项目分组的终端状态映射
    pub(crate) terminals_by_project: HashMap<String, TerminalState>,
    /// 差异视图主题
    pub(crate) diff_theme: DiffTheme,
    /// 应用主题
    pub(crate) app_theme: Theme,
    /// 编辑器是否跟随系统主题
    pub(crate) editor_follow_system_theme: bool,
    /// 编辑器主题
    pub(crate) editor_theme: Theme,
    /// 展开的差异块列表（文件路径，块索引）
    pub(crate) expanded_hunks: Vec<(String, usize)>,
    /// 上下文扩展映射（文件路径，块索引） -> （前扩展行数，后扩展行数）
    pub(crate) context_expansions: HashMap<(String, usize), (usize, usize)>,
    /// Git 提交消息输入
    pub(crate) git_commit_message: String,
    /// Git 提交类型（约定式提交）
    pub(crate) git_commit_type: Option<ConventionalCommitType>,
    /// Git 提交作用域输入
    pub(crate) git_commit_scope: String,
    /// Git 提交描述输入
    pub(crate) git_commit_description: String,
    /// Git 提交描述编辑器内容
    pub(crate) git_commit_description_editor: text_editor::Content,
    /// Git 提交是否进行中
    pub(crate) git_commit_in_progress: bool,
    /// 已暂存的文件列表
    pub(crate) staged_files_selected: Vec<String>,
    /// 已暂存的差异块列表（文件路径，块索引）
    pub(crate) staged_hunks_selected: Vec<(String, usize)>,
    /// 已暂存的行列表（文件路径，行号）
    pub(crate) staged_lines_selected: Vec<(String, usize)>,
    /// 已暂存的旧行列表（文件路径，行号）
    pub(crate) staged_old_lines_selected: Vec<(String, usize)>,
    /// Git 差异中选中的行列表
    pub(crate) git_diff_selected_lines: Vec<GitDiffSelectedLine>,
    /// Git 差异拖动选择的行范围
    pub(crate) git_diff_drag_range: Option<GitDiffLineRange>,
    /// Git 差异已选中的连续行范围
    pub(crate) git_diff_selected_range: Option<GitDiffLineRange>,
    /// 是否正在拖动选择 Git 差异行
    pub(crate) git_diff_dragging: bool,
    /// Git 差异拖动起始文本
    pub(crate) git_diff_drag_start_text: Option<String>,
    /// Git 差异上次点击信息（文件，行号，是否旧版本，时间）
    pub(crate) git_diff_last_click: Option<(String, usize, bool, web_time::Instant)>,
    /// Git 差异悬停行信息（文件，行号，是否旧版本）
    pub(crate) git_diff_hovered_line: Option<(String, usize, bool)>,
    /// Git 差异评论草稿
    pub(crate) git_diff_comment_draft: Option<GitDiffCommentDraft>,
    /// Git 差异右键菜单状态
    pub(crate) git_diff_context_menu: Option<GitDiffContextMenuState>,
    /// Git 差异文件操作菜单状态
    pub(crate) git_diff_file_menu: Option<GitDiffFileMenuState>,
    /// 是否显示 Git 复制对话框
    pub(crate) show_git_copy_modal: bool,
    /// Git 复制对话框编辑器内容
    pub(crate) git_copy_modal_editor: text_editor::Content,
    /// Git 复制对话框是否使用颜色
    pub(crate) git_copy_modal_use_color: bool,
    /// Git 复制对话框代码编辑器
    pub(crate) git_copy_modal_code_editor: iced_code_editor::CodeEditor,
    /// 是否显示自定义差异对话框
    pub(crate) show_git_custom_diff_modal: bool,
    /// 自定义差异对话框是否隐藏输入框
    pub(crate) git_custom_diff_hide_inputs: bool,
    /// 自定义差异标题
    pub(crate) git_custom_diff_title: String,
    /// 自定义差异前内容编辑器
    pub(crate) git_custom_diff_before_editor: text_editor::Content,
    /// 自定义差异后内容编辑器
    pub(crate) git_custom_diff_after_editor: text_editor::Content,
    /// 聊天中显示的文本差异
    pub(crate) chat_text_diff: Option<ChatTextDiff>,
    /// 是否显示 Git 过滤选项
    pub(crate) show_git_filter_options: bool,
    /// Git 过滤查询输入
    pub(crate) git_filter_query: String,
    /// Git 过滤是否包含指定路径
    pub(crate) git_filter_included: bool,
    /// Git 过滤是否排除指定路径
    pub(crate) git_filter_excluded: bool,
    /// Git 过滤是否包含新文件
    pub(crate) git_filter_new: bool,
    /// Git 过滤是否包含修改文件
    pub(crate) git_filter_modified: bool,
    /// Git 过滤是否包含删除文件
    pub(crate) git_filter_deleted: bool,
    /// Git 聚焦的文件路径
    pub(crate) git_focused_file: Option<String>,
    /// Git 悬停的文件头部路径
    pub(crate) git_hovered_file_header: Option<String>,
    /// Git 面板头部是否被悬停
    pub(crate) git_panel_header_hovered: bool,
    /// Git 变更文件列表
    pub(crate) git_changed_files: Vec<String>,
    /// Git 变更文件是否正在加载
    pub(crate) git_changed_files_loading: bool,
    /// Git diff 文件元数据缓存
    pub(crate) git_diff_file_metas: Vec<crate::app::components::git_panel::DiffFileMeta>,
    /// Git diff 文件元数据是否正在加载
    pub(crate) git_diff_file_metas_loading: bool,
    /// Git diff 文件元数据当前对应的仓库路径
    pub(crate) git_diff_file_metas_repo_path: Option<String>,
    pub(crate) git_diff_contents: HashMap<String, (String, String)>,
    pub(crate) git_diff_contents_loading: HashSet<String>,
    pub(crate) git_diff_scroll_offset_y: f32,
    pub(crate) git_diff_scroll_viewport_h: f32,
    /// 最近复制的代码哈希
    pub(crate) last_copied_code_hash: Option<u64>,
    /// 最近复制时间
    pub(crate) last_copy_time: Option<web_time::SystemTime>,
    /// 聊天是否自动滚动
    pub(crate) chat_auto_scroll: bool,
    /// 聊天滚动容器 ID
    pub(crate) chat_scroll_id: Id,
    /// 聊天滚动相对偏移（0.0-1.0）
    pub(crate) chat_scroll_offset_y: f32,
    /// 聊天滚动视口高度
    pub(crate) chat_scroll_viewport_h: f32,
    /// 流式场景下上次自动贴底触发时间（毫秒）
    pub(crate) chat_stream_autoscroll_last_ms: u64,
    /// 程序化贴底后的保护窗口截止时间（毫秒）
    pub(crate) chat_autoscroll_hold_until_ms: u64,
    /// 聊天面板是否全屏占据右侧区域
    pub(crate) chat_panel_fullscreen: bool,
    /// 聊天面板是否半全屏占据主内容区域但保留文件树和终端
    pub(crate) chat_panel_half_fullscreen: bool,
    /// 是否显示聊天右上角全屏浮动控件
    pub(crate) show_chat_fullscreen_overlay: bool,
    /// 输入编辑器 ID
    pub(crate) input_editor_id: Id,
    pub(crate) json_tool_editor_id: Id,
    pub(crate) sql_tool_editor_id: Id,
    pub(crate) html_tool_editor_id: Id,
    pub(crate) json_diff_left_editor_id: Id,
    pub(crate) json_diff_right_editor_id: Id,
    pub(crate) json_yaml_left_editor_id: Id,
    pub(crate) json_yaml_right_editor_id: Id,
    pub(crate) markdown_tool_editor_id: Id,
    pub(crate) pwd_editor_id: Id,
    /// 文件搜索滚动容器 ID
    pub(crate) file_search_scroll_id: Id,
    /// 预览滚动容器 ID
    pub(crate) preview_scroll_id: Id,
    /// 预览标签页滚动容器 ID
    pub(crate) preview_tabs_scroll_id: Id,
    /// Git 差异滚动容器 ID
    pub(crate) git_diff_scroll_id: Id,
    /// Git Diff 是否全屏占据右侧区域
    pub(crate) git_diff_fullscreen: bool,
    /// Git Diff 是否半全屏占据主内容区域但保留文件树和终端
    pub(crate) git_diff_half_fullscreen: bool,
    /// 主页应用栏滚动 X 偏移
    pub(crate) home_apps_bar_scroll_x: f32,
    /// 主页应用栏滚动容器 ID
    pub(crate) home_apps_bar_scroll_id: Id,