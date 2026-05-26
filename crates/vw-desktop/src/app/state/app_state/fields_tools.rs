    /// 是否显示设计快捷键面板
    pub show_design_shortcuts: bool,
    /// 是否显示设计变量面板
    pub show_design_variables: bool,
    /// 是否显示缩放菜单
    pub show_zoom_menu: bool,
    /// 是否显示图层面板
    pub show_layer_panel: bool,
    /// 图层面板宽度
    pub layer_panel_width: f32,
    /// 是否正在拖动图层面板
    pub dragging_layer_panel: bool,
    /// 图层面板拖动起始 X 坐标
    pub layer_panel_drag_anchor_x: Option<f32>,
    /// 图层面板拖动起始宽度
    pub layer_panel_start_width: f32,
    /// 是否折叠 AI 生成操作面板
    pub show_design_planner_panel: bool,
    /// AI 生成操作面板宽度
    pub design_planner_panel_width: f32,
    /// AI 生成面板吸附角落
    pub design_planner_corner: DesignPlannerCorner,
    /// 是否正在拖动 AI 生成操作面板
    pub dragging_design_planner_panel: bool,
    /// AI 生成操作面板拖动起始 X 坐标
    pub design_planner_panel_drag_anchor_x: Option<f32>,
    /// AI 生成操作面板拖动起始宽度
    pub design_planner_panel_start_width: f32,
    /// 是否显示属性面板
    pub show_properties_panel: bool,
    /// 属性面板宽度
    pub properties_panel_width: f32,
    /// 是否正在拖动属性面板
    pub dragging_properties_panel: bool,
    /// 属性面板拖动起始 X 坐标
    pub properties_panel_drag_anchor_x: Option<f32>,
    /// 属性面板拖动起始宽度
    pub properties_panel_start_width: f32,
    /// JSON 工具编辑器内容
    pub json_tool_editor: text_editor::Content,
    /// JSON 工具是否记住选择
    pub json_tool_remember: bool,
    /// JSON 工具是否正在加载
    pub json_tool_loading: bool,
    /// JSON 工具通知消息
    pub json_tool_notification: Option<String>,
    /// JSON 工具编辑器右键菜单是否打开
    pub json_tool_context_menu_open: bool,
    /// JSON 工具编辑器右键菜单位置
    pub json_tool_context_menu_pos: Option<(f32, f32)>,
    /// JSON 工具当前顶部滚动行号
    pub json_tool_scroll_top_line: f32,
    /// JSON 工具滚轮累积的小数行偏移
    pub json_tool_scroll_remainder: f32,
    /// JSON 工具编辑器视口高度
    pub json_tool_viewport_height: f32,
    /// JSON/YAML 转换工具左侧编辑器
    pub json_yaml_left_editor: text_editor::Content,
    /// JSON/YAML 转换工具右侧编辑器
    pub json_yaml_right_editor: text_editor::Content,
    /// JSON/YAML 转换是否正在加载
    pub json_yaml_loading: bool,
    /// JSON/YAML 转换通知消息
    pub json_yaml_notification: Option<String>,
    /// JSON/YAML 左侧编辑器右键菜单是否打开
    pub json_yaml_left_context_menu_open: bool,
    /// JSON/YAML 左侧编辑器右键菜单位置
    pub json_yaml_left_context_menu_pos: Option<(f32, f32)>,
    /// JSON/YAML 左侧编辑器当前顶部滚动行号
    pub json_yaml_left_scroll_top_line: f32,
    /// JSON/YAML 左侧编辑器滚轮累积的小数行偏移
    pub json_yaml_left_scroll_remainder: f32,
    /// JSON/YAML 左侧编辑器视口高度
    pub json_yaml_left_viewport_height: f32,
    /// JSON/YAML 右侧编辑器右键菜单是否打开
    pub json_yaml_right_context_menu_open: bool,
    /// JSON/YAML 右侧编辑器右键菜单位置
    pub json_yaml_right_context_menu_pos: Option<(f32, f32)>,
    /// JSON/YAML 右侧编辑器当前顶部滚动行号
    pub json_yaml_right_scroll_top_line: f32,
    /// JSON/YAML 右侧编辑器滚轮累积的小数行偏移
    pub json_yaml_right_scroll_remainder: f32,
    /// JSON/YAML 右侧编辑器视口高度
    pub json_yaml_right_viewport_height: f32,
    /// SQL 工具编辑器内容
    pub sql_tool_editor: text_editor::Content,
    /// SQL 工具是否记住选择
    pub sql_tool_remember: bool,
    /// SQL 工具是否正在加载
    pub sql_tool_loading: bool,
    /// SQL 工具通知消息
    pub sql_tool_notification: Option<String>,
    pub sql_tool_context_menu_open: bool,
    pub sql_tool_context_menu_pos: Option<(f32, f32)>,
    /// SQL 工具当前顶部滚动行号
    pub sql_tool_scroll_top_line: f32,
    /// SQL 工具滚轮累积的小数行偏移
    pub sql_tool_scroll_remainder: f32,
    /// SQL 工具编辑器视口高度
    pub sql_tool_viewport_height: f32,
    /// Redis 客户端工具状态
    pub redis_tool: RedisToolUiState,
    /// HTML 工具编辑器内容
    pub html_tool_editor: text_editor::Content,
    /// HTML 工具是否记住选择
    pub html_tool_remember: bool,
    /// HTML 工具是否正在加载
    pub html_tool_loading: bool,
    /// HTML 工具通知消息
    pub html_tool_notification: Option<String>,
    /// HTML 工具编辑器右键菜单是否打开
    pub html_tool_context_menu_open: bool,
    /// HTML 工具编辑器右键菜单位置
    pub html_tool_context_menu_pos: Option<(f32, f32)>,
    /// HTML 工具当前顶部滚动行号
    pub html_tool_scroll_top_line: f32,
    /// HTML 工具滚轮累积的小数行偏移
    pub html_tool_scroll_remainder: f32,
    /// HTML 工具编辑器视口高度
    pub html_tool_viewport_height: f32,
    /// JSON 差异对比左侧编辑器
    pub json_diff_left_editor: text_editor::Content,
    /// JSON 差异对比右侧编辑器
    pub json_diff_right_editor: text_editor::Content,
    /// JSON 差异对比结果列表
    pub json_diff_results: Vec<crate::app::message::json_diff_tool::JsonDiffEntry>,
    /// JSON 差异对比通知消息
    pub json_diff_notification: Option<String>,
    /// JSON 差异对比通知是否为错误态
    pub json_diff_notification_is_error: bool,
    /// JSON 差异对比是否正在加载
    pub json_diff_loading: bool,
    /// JSON 差异对比左侧编辑器右键菜单是否打开
    pub json_diff_left_context_menu_open: bool,
    /// JSON 差异对比左侧编辑器右键菜单位置
    pub json_diff_left_context_menu_pos: Option<(f32, f32)>,
    /// JSON 差异对比左侧编辑器当前顶部滚动行号
    pub json_diff_left_scroll_top_line: f32,
    /// JSON 差异对比左侧编辑器滚轮累积的小数行偏移
    pub json_diff_left_scroll_remainder: f32,
    /// JSON 差异对比左侧编辑器视口高度
    pub json_diff_left_viewport_height: f32,
    /// JSON 差异对比右侧编辑器右键菜单是否打开
    pub json_diff_right_context_menu_open: bool,
    /// JSON 差异对比右侧编辑器右键菜单位置
    pub json_diff_right_context_menu_pos: Option<(f32, f32)>,
    /// JSON 差异对比右侧编辑器当前顶部滚动行号
    pub json_diff_right_scroll_top_line: f32,
    /// JSON 差异对比右侧编辑器滚轮累积的小数行偏移
    pub json_diff_right_scroll_remainder: f32,
    /// JSON 差异对比右侧编辑器视口高度
    pub json_diff_right_viewport_height: f32,
    /// Markdown 工具编辑器内容
    pub markdown_tool_editor: text_editor::Content,
    /// Markdown 工具编辑器右键菜单是否打开
    pub markdown_tool_context_menu_open: bool,
    /// Markdown 工具编辑器右键菜单位置
    pub markdown_tool_context_menu_pos: Option<(f32, f32)>,
    /// Markdown 工具编辑器当前顶部滚动行号
    pub markdown_tool_scroll_top_line: f32,
    /// Markdown 工具编辑器滚轮累积的小数行偏移
    pub markdown_tool_scroll_remainder: f32,
    /// Markdown 工具编辑器视口高度
    pub markdown_tool_viewport_height: f32,
    /// Markdown 工具渲染内容
    pub markdown_tool_content: iced::widget::markdown::Content,
    /// Markdown 工具视图模式
    pub markdown_tool_view_mode: MarkdownViewMode,
    /// Markdown 工具通知消息
    pub markdown_tool_notification: Option<String>,
    /// Markdown 工具是否显示 HTML 转 Markdown 功能
    pub markdown_tool_show_html2md: bool,
    /// Markdown 工具 HTML 输入编辑器
    pub markdown_tool_html_editor: text_editor::Content,
    /// Markdown 工具是否显示图片功能
    pub markdown_tool_show_image: bool,
    /// Markdown 工具图片 URL 输入
    pub markdown_tool_image_url_input: String,
    /// Markdown 工具远程图片缓存
    pub markdown_tool_remote_images: HashMap<String, iced::widget::image::Handle>,
    /// Markdown 工具正在加载的远程图片 URL 集合
    pub markdown_tool_remote_images_loading: HashSet<String>,
    /// Markdown 工具是否启用流式渲染
    pub markdown_tool_stream_enabled: bool,
    /// Markdown 工具流式渲染字符数
    pub markdown_tool_stream_chars: usize,
    /// 思维导图标签页列表
    pub mindmap_tabs: Vec<crate::apps::mindmap::state::MindMapTab>,
    /// 当前活跃的思维导图标签页 ID
    pub mindmap_active_tab_id: Option<String>,
    /// Dify 工作流无限画布状态
    pub workflow_state: crate::apps::workflow::state::WorkflowState,
    /// 密码生成器是否包含数字
    pub pwd_digits: bool,
    /// 密码生成器是否包含小写字母
    pub pwd_lowercase: bool,
    /// 密码生成器是否包含大写字母
    pub pwd_uppercase: bool,
    /// 密码生成器是否包含特殊字符
    pub pwd_special: bool,
    /// 密码生成器长度输入
    pub pwd_length_input: String,
    /// 密码生成器数量输入
    pub pwd_count_input: String,
    /// 密码生成器输出编辑器
    pub pwd_output_editor: text_editor::Content,
    /// 密码生成器通知消息
    pub pwd_notification: Option<String>,
    /// 密码生成器通知是否为错误状态
    pub pwd_notification_is_error: bool,
    /// 密码生成器编辑器右键菜单是否打开
    pub pwd_context_menu_open: bool,
    /// 密码生成器编辑器右键菜单位置
    pub pwd_context_menu_pos: Option<(f32, f32)>,
    /// 密码生成器编辑器当前顶部滚动行号
    pub pwd_scroll_top_line: f32,
    /// 密码生成器编辑器滚轮累积的小数行偏移
    pub pwd_scroll_remainder: f32,
    /// 密码生成器编辑器视口高度
    pub pwd_viewport_height: f32,
    /// 进制转换源进制
    pub base_from: u32,
    /// 进制转换目标进制
    pub base_to: u32,
    /// 进制转换输入值
    pub base_input: String,
    /// 进制转换输出值
    pub base_output: String,
    /// 进制转换通知消息
    pub base_notification: Option<String>,
    /// 时间戳工具是否自动更新
    pub ts_auto: bool,
    /// 时间戳工具当前 Unix 秒级时间戳
    pub ts_now_unix_sec: String,
    /// 时间戳工具当前 Unix 毫秒级时间戳
    pub ts_now_unix_ms: String,
    /// 时间戳工具当前 UTC 时间字符串
    pub ts_now_utc_str: String,
    /// 时间戳工具输入时间戳
    pub ts_input_ts: String,
    /// 时间戳工具单位选择
    pub ts_unit: message::timestamp_tool::TsUnit,
    /// 时间戳工具时间输出
    pub ts_time_output: String,
    /// 时间戳工具时间输入
    pub ts_time_input: String,
    /// 时间戳工具转换后的秒级输出
    pub ts_ts_output_sec: String,
    /// 时间戳工具转换后的毫秒级输出
    pub ts_ts_output_ms: String,
    /// 时间戳工具通知消息
    pub ts_notification: Option<String>,
    /// QR 码生成器输入文本
    pub qr_input: String,
    /// QR 码尺寸
    pub qr_size: u32,
    /// QR 码尺寸输入
    pub qr_size_input: String,
    /// QR 码纠错级别
    pub qr_level: message::qr_tool::QrEcLevel,
    /// QR 码图片句柄
    pub qr_image: Option<iced::widget::image::Handle>,
    /// QR 码工具是否正在处理
    pub qr_loading: bool,
    /// QR 码生成器通知消息
    pub qr_notification: Option<String>,
    /// QR 码通知是否为错误态
    pub qr_notification_is_error: bool,
    /// QR 码颜色十六进制值
    pub qr_color_hex: String,
    /// QR 码颜色格式
    pub qr_color_format: ColorFormat,
    /// QR 码图标模式
    pub qr_icon_mode: message::qr_tool::QrIconMode,
    /// QR 码图标字节数据
    pub qr_icon_bytes: Option<Vec<u8>>,
    /// QR 码编辑器内容
    pub qr_editor: text_editor::Content,
    /// QR 码编辑器当前顶部滚动行号
    pub qr_scroll_top_line: f32,
    /// QR 码编辑器滚轮累积的小数行偏移
    pub qr_scroll_remainder: f32,
    /// QR 码编辑器视口高度
    pub qr_viewport_height: f32,
    /// 是否显示 QR 码颜色选择器
    pub show_qr_color_picker: bool,
    /// 颜色工具当前颜色
    pub color_tool_color: Color,
    /// 颜色工具颜色格式
    pub color_tool_format: ColorFormat,
    /// 颜色工具十六进制输入
    pub color_hex_input: String,
    /// 颜色工具 RGB 输入
    pub color_rgb_input: String,
    /// 颜色工具 HSL 输入
    pub color_hsl_input: String,
    /// 颜色工具 HSV 输入
    pub color_hsv_input: String,
    /// 颜色工具通知消息
    pub color_notification: Option<String>,
    /// 清理工具是否清理系统临时文件
    pub cleaner_clear_system_temp: bool,
    /// 清理工具是否清理应用缓存
    pub cleaner_clear_app_cache: bool,
    /// 清理工具是否清理日志
    pub cleaner_clear_logs: bool,
    /// 清理工具是否清理包缓存
    pub cleaner_clear_package_cache: bool,
    /// 清理工具是否清理下载目录
    pub cleaner_clear_downloads: bool,
    /// 清理工具是否清空回收站
    pub cleaner_empty_trash: bool,
    /// 清理工具是否清理安装包
    pub cleaner_clear_installers: bool,
    /// 清理工具是否清理其他常用应用缓存
    pub cleaner_clear_other_apps: bool,
    /// 清理工具是否清理企业微信缓存
    pub cleaner_clear_wechat_work: bool,
    /// 清理工具是否清理微信缓存
    pub cleaner_clear_wechat: bool,
    /// 清理工具是否清理 QQ 缓存
    pub cleaner_clear_qq: bool,
    /// 清理工具是否清理钉钉缓存
    pub cleaner_clear_dingtalk: bool,
    /// 清理工具是否清理飞书缓存
    pub cleaner_clear_feishu: bool,
    /// 清理工具是否清理 Safari 上网缓存
    pub cleaner_clear_safari: bool,
    /// 清理工具是否清理 Chrome 上网缓存
    pub cleaner_clear_chrome: bool,
    /// 清理工具是否清理 Edge 上网缓存
    pub cleaner_clear_edge: bool,
    /// 清理工具是否清理 Firefox 上网缓存
    pub cleaner_clear_firefox: bool,
    /// 清理工具是否清理 Mail 上网缓存
    pub cleaner_clear_mail: bool,
    /// 清理工具是否正在运行
    pub cleaner_running: bool,
    /// 清理工具是否正在取消
    pub cleaner_cancelling: bool,
    /// 清理工具是否正在扫描
    pub cleaner_scanning: bool,
    /// 清理工具是否已完成扫描
    pub cleaner_scanned: bool,
    /// 清理工具动画帧索引
    pub cleaner_animation_frame: usize,
    /// 清理工具扫描报告
    pub cleaner_scan_report: Option<CleanerScanReport>,
    /// 清理工具树形展开节点
    pub cleaner_tree_expanded: HashSet<String>,
    /// 清理工具是否显示预览模式
    pub cleaner_preview_mode: bool,
    /// 清理工具输出编辑器
    pub cleaner_output_editor: text_editor::Content,
    /// 清理工具通知消息
    pub cleaner_notification: Option<String>,
    /// 清理工具最近一次是否已成功完成
    pub cleaner_last_run_completed: bool,
    /// 清理工具取消标志
    pub cleaner_cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// 大文件查找工具扫描根目录
    pub large_file_root: String,
    /// 大文件查找工具是否正在扫描
    pub large_file_scanning: bool,
    /// 大文件查找工具是否已完成扫描
    pub large_file_scanned: bool,
    /// 大文件查找工具动画帧索引
    pub large_file_animation_frame: usize,
    /// 大文件查找工具当前筛选标签
    pub large_file_active_filter: String,
    /// 大文件查找工具扫描报告
    pub large_file_report: Option<LargeFileScanReport>,
    /// 大文件查找工具通知消息
    pub large_file_notification: Option<String>,
    /// 大文件查找工具当前扫描阶段标签
    pub large_file_progress_label: String,
    /// 大文件查找工具当前扫描路径
    pub large_file_current_path: String,
    /// 大文件查找工具当前进度
    pub large_file_progress_value: f32,
    /// 大文件查找工具已处理文件数
    pub large_file_processed_files: usize,
    /// 大文件查找工具总文件数
    pub large_file_total_files: usize,
    /// 大文件查找工具已选择的文件路径
    pub large_file_selected_entries: HashSet<String>,
    /// 大文件查找工具是否正在删除
    pub large_file_deleting: bool,
    /// 大文件查找工具扫描取消标志
    pub large_file_cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// 大文件查找工具共享扫描进度
    pub large_file_progress_state: std::sync::Arc<std::sync::Mutex<LargeFileScanProgress>>,
