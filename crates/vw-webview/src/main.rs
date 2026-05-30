//! # VibeWindow WebView 应用入口
//!
//! 本模块是 VibeWindow WebView 应用的主入口点，提供跨平台的 WebView 窗口功能。
//!
//! ## 主要功能
//!
//! - 创建和管理原生 WebView 窗口
//! - 支持加载远程 URL 或本地文件
//! - 提供 JavaScript 注入能力，用于会话同步和 Cookie 管理
//! - 支持通过标准输入进行窗口控制（调整大小、移动）
//! - 支持自定义协议（`vibe://`）用于本地资源访问
//!
//! ## 支持的运行模式
//!
//! - `window`: 标准窗口模式，带有标题栏和边框
//! - `embedded`: 嵌入模式，无边框且始终置顶
//!
//! ## 使用示例
//!
//! ```bash
//! # 加载远程 URL
//! vw_webview https://example.com
//!
//! # 加载本地目录
//! vw_webview /path/to/local/site
//!
//! # 使用自定义参数
//! vw_webview https://example.com --width=1024 --height=768 --title="My App"
//! ```

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::borrow::Cow;
    use std::env;
    use std::io::{self, BufRead};
    use std::path::{self, Component, Path, PathBuf};
    use std::thread;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::WindowBuilder;
    use wry::http::{Request, Response, StatusCode};
    use wry::{NewWindowResponse, WebContext, WebViewBuilder};

    /// 用户自定义事件枚举
    ///
    /// 用于从外部线程（如标准输入读取线程）向主事件循环发送控制指令。
    #[derive(Debug)]
    enum UserEvent {
        /// 调整窗口大小
        /// 参数：(宽度, 高度)
        Resize(u32, u32),

        /// 移动窗口位置
        /// 参数：(X 坐标, Y 坐标)
        Move(i32, i32),

        /// 在 WebView 中打开指定 URL
        OpenUrl(String),
    }

    // ==================== 参数解析 ====================

    // 收集命令行参数，跳过程序名
    let args = env::args().skip(1).collect::<Vec<_>>();

    // 第一个参数作为目标地址（URL 或本地路径），默认加载示例站点
    let target = args.first().cloned().unwrap_or_else(|| "https://example.com/".to_string());

    /// 规范化 URL 字符串
    ///
    /// 移除字符串首尾的空白字符和引号（支持反引号、双引号、单引号），
    /// 确保输入的 URL 或路径格式正确。
    ///
    /// # 参数
    ///
    /// - `s`: 待规范化的字符串
    ///
    /// # 返回值
    ///
    /// 去除首尾引号和空白后的字符串
    fn normalize_url(s: &str) -> String {
        let mut t = s.trim();
        loop {
            let before = t;
            // 依次尝试移除各种类型的引号
            for q in ['`', '"', '\''] {
                if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
                    t = &t[1..t.len() - 1];
                    t = t.trim();
                }
            }
            // 如果字符串没有变化，说明已经没有可移除的引号
            if t == before {
                break;
            }
        }
        t.to_string()
    }
    let target = normalize_url(&target);

    // ==================== URL 编解码工具函数 ====================

    /// 对 URL 路径进行百分号编码
    ///
    /// 根据 RFC 3986 规范，对 URL 路径中的特殊字符进行编码。
    /// 保留字符（字母、数字、`-`、`.`、`_`、`~`、`/`）不进行编码。
    ///
    /// # 参数
    ///
    /// - `path`: 待编码的路径字符串
    ///
    /// # 返回值
    ///
    /// 编码后的路径字符串
    fn percent_encode_url_path(path: &str) -> String {
        let mut out = String::with_capacity(path.len() + 8);
        for b in path.as_bytes() {
            let c = *b as char;
            // 判断是否为非保留字符（RFC 3986）
            let is_unreserved =
                matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~');
            // 斜杠是路径分隔符，需要保留
            let is_allowed = is_unreserved || c == '/';
            if is_allowed {
                out.push(c);
            } else {
                // 非保留字符编码为 %XX 格式
                out.push_str(&format!("%{:02X}", b));
            }
        }
        out
    }

    /// 对字符串进行百分号解码
    ///
    /// 将 `%XX` 格式的编码序列解码为原始字符。
    ///
    /// # 参数
    ///
    /// - `s`: 待解码的字符串
    ///
    /// # 返回值
    ///
    /// 解码后的字符串，使用 UTF-8 损失替换策略处理无效字节
    fn percent_decode(s: &str) -> String {
        /// 将十六进制字节转换为数值
        ///
        /// # 参数
        ///
        /// - `b`: ASCII 字节（'0'-'9', 'a'-'f', 'A'-'F'）
        ///
        /// # 返回值
        ///
        /// 转换后的数值（0-15），无效输入返回 None
        fn from_hex(b: u8) -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        }

        let bytes = s.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            // 检测 %XX 编码序列
            if bytes[i] == b'%'
                && i + 2 < bytes.len()
                && let (Some(h1), Some(h2)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2]))
            {
                // 合并高低位字节
                out.push((h1 << 4) | h2);
                i += 3;
                continue;
            }
            out.push(bytes[i]);
            i += 1;
        }
        // 使用损失替换策略处理可能的无效 UTF-8
        String::from_utf8_lossy(&out).to_string()
    }

    // ==================== 安全性检查 ====================

    /// 检查路径是否为安全的相对路径
    ///
    /// 防止路径遍历攻击，确保路径不包含：
    /// - 父目录引用（`..`）
    /// - Windows 路径前缀（如 `C:`）
    /// - 根目录引用（`/`）
    ///
    /// # 参数
    ///
    /// - `p`: 待检查的路径
    ///
    /// # 返回值
    ///
    /// 如果路径安全返回 `true`，否则返回 `false`
    fn is_safe_relative_path(p: &Path) -> bool {
        !p.components()
            .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
    }

    // ==================== MIME 类型推断 ====================

    /// 根据文件扩展名推断 MIME 类型
    ///
    /// 为本地文件提供正确的 Content-Type 响应头。
    ///
    /// # 参数
    ///
    /// - `path`: 文件路径
    ///
    /// # 返回值
    ///
    /// MIME 类型字符串，未知类型返回 `application/octet-stream`
    fn guess_mime(path: &Path) -> &'static str {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        match ext.as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "js" => "text/javascript; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "wasm" => "application/wasm",
            "txt" => "text/plain; charset=utf-8",
            "map" => "application/json; charset=utf-8",
            _ => "application/octet-stream",
        }
    }

    // ==================== 本地协议处理器 ====================

    /// 处理 vibe:// 自定义协议的请求
    ///
    /// 将自定义协议请求映射到本地文件系统，提供静态文件服务。
    /// 实现了 SPA（单页应用）回退支持，对于无扩展名的路径返回 index.html。
    ///
    /// # 参数
    ///
    /// - `root`: 本地文件根目录
    /// - `request`: HTTP 请求对象
    ///
    /// # 返回值
    ///
    /// HTTP 响应对象，包含文件内容和正确的 Content-Type
    fn local_protocol_response(
        root: &Path,
        request: Request<Vec<u8>>,
    ) -> Response<Cow<'static, [u8]>> {
        let raw_path = request.uri().path();
        let mut req_path = raw_path;
        // 空路径默认为根路径
        if req_path.is_empty() {
            req_path = "/";
        }

        // 解码并验证路径安全性
        let rel = percent_decode(req_path.trim_start_matches('/'));
        let rel_path = Path::new(&rel);
        if !rel.is_empty() && !is_safe_relative_path(rel_path) {
            // 拒绝不安全的路径访问
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Cow::Owned(Vec::new()))
                .unwrap();
        }

        // 构建实际文件路径
        let mut file_path =
            if rel.is_empty() { root.join("index.html") } else { root.join(rel_path) };

        // 如果是目录，尝试返回 index.html
        if file_path.is_dir() {
            file_path = file_path.join("index.html");
        }

        // 尝试读取文件
        let file_bytes = std::fs::read(&file_path).ok();
        let (status, body, mime_path) = if let Some(bytes) = file_bytes {
            (StatusCode::OK, bytes, file_path)
        } else {
            // SPA 回退：对于无扩展名的路径，返回 index.html
            let fallback_index = root.join("index.html");
            let should_fallback = path::Path::new(req_path)
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| !s.contains('.'))
                .unwrap_or(true);
            if should_fallback {
                if let Ok(bytes) = std::fs::read(&fallback_index) {
                    (StatusCode::OK, bytes, fallback_index)
                } else {
                    (StatusCode::NOT_FOUND, Vec::new(), file_path)
                }
            } else {
                (StatusCode::NOT_FOUND, Vec::new(), file_path)
            }
        };

        // 构建响应
        Response::builder()
            .status(status)
            .header("Content-Type", guess_mime(&mime_path))
            .body(Cow::Owned(body))
            .unwrap()
    }

    // ==================== 目标类型定义 ====================

    /// 启动目标类型枚举
    ///
    /// 区分远程 URL 和本地目录两种启动模式。
    enum StartTarget {
        /// 远程 URL 地址
        Url(String),

        /// 本地目录模式
        /// 包含根目录路径和入口文件名
        LocalDir { root: PathBuf, entry: String },
    }

    /// 解析目标字符串为启动目标
    ///
    /// 根据输入字符串判断是远程 URL 还是本地路径：
    /// - 以 `http://` 或 `https://` 开头：远程 URL
    /// - 以 `file://` 开头或存在本地文件：本地目录模式
    ///
    /// # 参数
    ///
    /// - `target`: 目标字符串（URL 或路径）
    ///
    /// # 返回值
    ///
    /// 解析后的启动目标
    fn resolve_target(target: &str) -> StartTarget {
        let lower = target.to_ascii_lowercase();
        // 检测 HTTP/HTTPS URL
        if lower.starts_with("http://") || lower.starts_with("https://") {
            return StartTarget::Url(target.to_string());
        }

        /// 将 file:// URL 转换为本地路径
        ///
        /// # 参数
        ///
        /// - `url`: file:// 格式的 URL
        ///
        /// # 返回值
        ///
        /// 转换后的本地路径，无效格式返回 None
        fn file_url_to_path(url: &str) -> Option<PathBuf> {
            let s = url.strip_prefix("file://")?;
            // 移除可选的 localhost 前缀
            let s = s.strip_prefix("localhost").unwrap_or(s);
            // 移除开头的斜杠（稍后根据平台处理）
            let s = s.strip_prefix('/').unwrap_or(s);
            let decoded = percent_decode(s);

            // Windows 路径需要特殊处理
            #[cfg(target_os = "windows")]
            {
                let decoded = decoded.replace('/', "\\");
                Some(PathBuf::from(decoded))
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Unix 系统需要恢复开头的斜杠
                Some(PathBuf::from(format!("/{decoded}")))
            }
        }

        // 解析路径
        let as_path = if lower.starts_with("file://") {
            file_url_to_path(target).unwrap_or_else(|| PathBuf::from(target))
        } else {
            PathBuf::from(target)
        };

        // 检测是否为有效的本地路径
        let candidate: Option<(PathBuf, PathBuf)> = match std::fs::metadata(&as_path) {
            Ok(m) if m.is_dir() => {
                // 如果是目录，查找 index.html
                let index = as_path.join("index.html");
                if index.is_file() { Some((as_path, index)) } else { None }
            }
            Ok(m) if m.is_file() => {
                // 如果是文件，使用其父目录作为根目录
                let root = as_path.parent().map(|p| p.to_path_buf());
                root.map(|r| (r, as_path))
            }
            _ => None,
        };

        // 如果找到有效的本地路径，返回 LocalDir 模式
        if let Some((root, entry_file)) = candidate
            && let (Ok(root), Ok(entry_file)) =
                (std::fs::canonicalize(&root), std::fs::canonicalize(&entry_file))
            && let Some(name) = entry_file.file_name().and_then(|s| s.to_str())
        {
            return StartTarget::LocalDir { root, entry: format!("/{name}") };
        }

        // 默认作为 URL 处理
        StartTarget::Url(target.to_string())
    }

    // ==================== URL 和参数处理 ====================

    // 解析目标，确定加载方式
    let (url, local_root) = match resolve_target(&target) {
        StartTarget::Url(url) => (url, None),
        StartTarget::LocalDir { root, entry } => {
            // 本地文件使用自定义 vibe:// 协议
            let entry = percent_encode_url_path(&entry);
            (format!("vibe://localhost{entry}"), Some(root))
        }
    };

    // 初始化窗口参数，设置默认值
    let mut mode = "window".to_string();
    let mut window_title = "Vibe Window WebView".to_string();
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;
    let mut pos_x: Option<i32> = None;
    let mut pos_y: Option<i32> = None;
    let mut cookies_json = "[]".to_string();

    // 解析命令行参数（从第二个参数开始，第一个是目标地址）
    for a in args.iter().skip(1) {
        if let Some(v) = a.strip_prefix("--mode=") {
            mode = v.to_string();
        } else if let Some(v) = a.strip_prefix("--cookies=") {
            cookies_json = v.to_string();
        } else if let Some(v) = a.strip_prefix("--width=") {
            width = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--height=") {
            height = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--x=") {
            pos_x = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--y=") {
            pos_y = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--title=") {
            window_title = v.to_string();
        }
    }

    // ==================== 事件循环初始化 ====================

    // 创建支持用户自定义事件的事件循环
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // 启动标准输入读取线程，用于接收外部控制命令
    {
        let stdin_proxy = proxy.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            // 持续读取标准输入的每一行
            for line in stdin.lock().lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                match parts[0] {
                    // resize 命令：调整窗口大小
                    "resize" if parts.len() >= 3 => {
                        if let (Ok(w), Ok(h)) = (parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                            let _ = stdin_proxy.send_event(UserEvent::Resize(w, h));
                        }
                    }
                    // move 命令：移动窗口位置
                    "move" if parts.len() >= 3 => {
                        if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                            let _ = stdin_proxy.send_event(UserEvent::Move(x, y));
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    // ==================== 窗口创建 ====================

    // 创建窗口构建器，设置窗口标题
    let mut wb = WindowBuilder::new().with_title(window_title.clone());

    // 嵌入模式：移除装饰并始终置顶
    if mode == "embedded" {
        wb = wb.with_decorations(false).with_always_on_top(true);
    }

    // 设置窗口尺寸，默认 1280x800
    let size_w = width.unwrap_or(1280) as f64;
    let size_h = height.unwrap_or(800) as f64;
    wb = wb.with_inner_size(tao::dpi::LogicalSize::new(size_w, size_h));

    // 设置窗口位置（如果指定）
    if let (Some(px), Some(py)) = (pos_x, pos_y) {
        wb = wb.with_position(tao::dpi::LogicalPosition::new(px as f64, py as f64));
    }

    // ==================== 数据目录管理 ====================

    /// 获取 WebView 数据存储目录
    ///
    /// 根据操作系统选择合适的数据存储位置：
    /// - Windows: `%APPDATA%/webview_data`
    /// - macOS/Linux: `~/.local/share/vibe-window/webview_data`
    ///
    /// # 返回值
    ///
    /// 数据目录路径
    fn webview_data_dir() -> std::path::PathBuf {
        if let Some(dirs) = directories::ProjectDirs::from("dev", "VibeWindow", "vibe-window") {
            #[cfg(target_os = "windows")]
            {
                // Windows: 使用 AppData 目录
                let base = dirs.config_dir().parent().unwrap_or_else(|| dirs.config_dir());
                return base.join("webview_data");
            }

            #[cfg(not(target_os = "windows"))]
            {
                // macOS/Linux: 使用标准数据目录
                return dirs.data_dir().join("webview_data");
            }
        }

        // 备选方案：使用用户数据目录或当前目录
        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.data_dir().join("vibe-window").join("webview_data")
        } else {
            std::env::current_dir().unwrap().join("webview_data")
        }
    }

    let data_path = webview_data_dir();
    let log_path = data_path.join("vw-webview.log");

    /// 追加日志到文件
    ///
    /// 将日志消息写入指定的日志文件，自动创建父目录。
    ///
    /// # 参数
    ///
    /// - `log_path`: 日志文件路径
    /// - `s`: 日志内容
    fn append_log(log_path: &std::path::Path, s: &str) {
        // 确保日志目录存在
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut buf = String::new();
        buf.push_str(s);
        buf.push('\n');
        // 追加写入日志文件
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, buf.as_bytes()));
    }

    /// 显示错误对话框
    ///
    /// 在 Windows 上显示原生对话框，在其他平台输出到标准错误。
    ///
    /// # 参数
    ///
    /// - `title`: 对话框标题
    /// - `description`: 错误描述
    fn show_error_dialog(title: &str, description: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = rfd::MessageDialog::new()
                .set_title(title)
                .set_description(description)
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
        #[cfg(not(target_os = "windows"))]
        {
            eprintln!("{title}: {description}");
        }
    }

    // 设置 panic 钩子，记录崩溃信息到日志
    {
        let log_path_for_panic = log_path.clone();
        std::panic::set_hook(Box::new(move |info| {
            append_log(&log_path_for_panic, &format!("panic: {info}"));
        }));
    }

    // 确保数据目录存在
    if let Err(e) = std::fs::create_dir_all(&data_path) {
        eprintln!("Failed to create data directory: {}", e);
        append_log(&log_path, &format!("Failed to create data directory: {e}"));
    } else {
        println!("WebView data path: {:?}", data_path);
        append_log(&log_path, &format!("WebView data path: {:?}", data_path));
    }

    // 记录启动信息
    append_log(&log_path, &format!("Args: {:?}", env::args().collect::<Vec<_>>()));
    append_log(&log_path, &format!("Normalized url: {url:?}"));

    // 构建窗口
    let window = match wb.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            let msg = format!("Failed to create window: {e:?}\nLog: {log_path:?}");
            append_log(&log_path, &msg);
            show_error_dialog("VibeWindow WebView", &msg);
            return;
        }
    };

    // ==================== JavaScript 初始化脚本 ====================

    // 使用 JSON 序列化确保字符串在 JS 中是安全的
    let data_path_js = serde_json::to_string(&data_path.to_string_lossy())
        .unwrap_or_else(|_| "'unknown'".to_string());
    let ua_js = serde_json::to_string("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3 Safari/605.1.15").unwrap();

    // 构建初始化脚本，包含 Cookie 同步功能
    let init_script = format!(
        r#"
        (function() {{
            console.log('%c [VibeWindow] Init Script Starting...', 'color: green; font-weight: bold');
            const dataPath = {0};
            const userAgent = {1};
            const cookieConfigs = {2};
            console.log('Data Path:', dataPath);

            const DEFAULT_DOMAIN = '';

            // 转义正则表达式特殊字符
            function escapeRegExp(s) {{
                return String(s).replace(/[.*+?^${{}}()|[\]\\]/g, '\\$&');
            }}

            // 规范化域名格式（添加前导点）
            function normalizeDomain(domain) {{
                if (!domain) return null;
                const d = String(domain).trim();
                if (!d) return null;
                return d.startsWith('.') ? d : `.${{d}}`;
            }}

            // 设置 Cookie 的辅助函数
            function setCookie(name, value, days, domain) {{
                if (!name) return;
                if (value === undefined || value === null) return;

                const isHttps = window.location && window.location.protocol === 'https:';
                const isHostCookie = String(name).startsWith('__Host-');
                const isSecureCookie = String(name).startsWith('__Secure-') || isHostCookie;

                let maxAge = "";
                if (typeof days === 'number' && isFinite(days) && days > 0) {{
                    maxAge = "; max-age=" + Math.floor(days * 24 * 60 * 60);
                }}

                let cookieVal = `${{name}}=${{value}}${{maxAge}}; path=/`;

                if (!isHostCookie) {{
                    const d = normalizeDomain(domain || DEFAULT_DOMAIN);
                    if (d) {{
                        cookieVal += `; domain=${{d}}`;
                    }}
                }}

                if (isHttps || isSecureCookie) {{
                    cookieVal += "; Secure";
                }}

                document.cookie = cookieVal;
                const after = getCookie(name);
                if (after !== String(value)) {{
                    console.warn('Set Cookie failed or was rejected:', cookieVal);
                }} else {{
                    console.log('Set Cookie:', cookieVal);
                }}
            }}

            // 获取 Cookie 的辅助函数
            function getCookie(name) {{
                if (!name) return null;
                const safeName = escapeRegExp(name);
                const match = document.cookie.match(new RegExp('(?:^|;\\s*)' + safeName + '=([^;]+)'));
                return match ? match[1] : null;
            }}

            // 初始化会话同步功能
            function initSessionSync() {{
                try {{
                    let configs = cookieConfigs || [];

                    // 获取备份 Cookie 名称
                    function getBackupName(name) {{
                        return 'vibe_window_webview_backup_' + name;
                    }}

                    // 1. 从备份恢复 Cookie
                    configs.forEach(config => {{
                        // 按 URL 过滤配置
                        if (config.url_filter && window.location.href.indexOf(config.url_filter) === -1) {{
                            return;
                        }}

                        const name = config.name;
                        const bName = getBackupName(name);
                        const days = config.days || 365;
                        const domain = config.domain || DEFAULT_DOMAIN;

                        const backupVal = getCookie(bName);
                        const currentVal = getCookie(name);

                        if (backupVal) {{
                            if (!currentVal) {{
                                console.log('Restoring ' + name + ' from ' + bName);
                                setCookie(name, backupVal, days, domain);
                            }} else if (currentVal !== backupVal) {{
                                console.log('Updating ' + name + ' from ' + bName + ' (mismatch detected)...');
                                setCookie(name, backupVal, days, domain);
                            }}
                        }}
                    }});

                    // 2. 启动 Cookie 监控循环
                    let lastCookie = document.cookie;
                    setInterval(() => {{
                        const currentCookie = document.cookie;
                        if (currentCookie !== lastCookie) {{
                            configs.forEach(config => {{
                                // 按 URL 过滤配置
                                if (config.url_filter && window.location.href.indexOf(config.url_filter) === -1) {{
                                    return;
                                }}

                                const name = config.name;
                                const bName = getBackupName(name);
                                const days = config.days || 365;
                                const domain = config.domain || DEFAULT_DOMAIN;

                                const session = getCookie(name);
                                if (session) {{
                                    const currentBackup = getCookie(bName);
                                    if (session !== currentBackup) {{
                                        console.log('New value for ' + name + ', backing up to ' + bName);
                                        setCookie(bName, session, days, domain);
                                        setCookie(name, session, days, domain);
                                    }}
                                }}
                            }});
                            lastCookie = currentCookie;
                        }}
                    }}, 1000);
                }} catch (e) {{
                    console.error('Session Sync Error:', e);
                }}
            }}

            // 确保 DOM 就绪后执行
            if (document.readyState === 'loading') {{
                document.addEventListener('DOMContentLoaded', initSessionSync);
            }} else {{
                initSessionSync();
            }}
        }})();
        "#,
        data_path_js, ua_js, cookies_json
    );

    // ==================== 平台特定的 WebView 创建 ====================

    // macOS 平台分支
    #[cfg(target_os = "macos")]
    {
        // 创建 WebContext，用于管理 WebView 的持久化数据
        let mut web_context = WebContext::new(Some(data_path.clone()));
        let safari_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3 Safari/605.1.15";

        // 构建 WebView
        let mut builder = WebViewBuilder::new_with_web_context(&mut web_context)
            .with_url(url.as_str())
            .with_user_agent(safari_ua)
            .with_devtools(true)
            .with_initialization_script(&init_script)
            .with_new_window_req_handler({
                let newwin_proxy = proxy.clone();
                move |req_url, _features| {
                    // 新窗口请求通过事件循环处理
                    let _ = newwin_proxy.send_event(UserEvent::OpenUrl(req_url.to_string()));
                    NewWindowResponse::Deny
                }
            });

        // 注册自定义协议处理器（用于本地文件）
        if let Some(root) = local_root.clone() {
            builder = builder.with_custom_protocol("vibe".into(), move |_webview_id, request| {
                local_protocol_response(&root, request)
            });
        }

        // 构建 WebView 实例
        let _webview = match builder.build(&window) {
            Ok(w) => w,
            Err(e) => {
                let msg = format!(
                    "WebView build error: {e:?}\nLog: {log_path:?}\nHint: install Microsoft Edge WebView2 Runtime (Evergreen x64)."
                );
                append_log(&log_path, &msg);
                show_error_dialog("VibeWindow WebView", &msg);
                return;
            }
        };

        // 运行事件循环
        event_loop.run(move |event, _, control_flow| {
            if *control_flow != ControlFlow::Exit {
                *control_flow = ControlFlow::Wait;
            }

            match event {
                // 处理窗口大小调整事件
                Event::UserEvent(UserEvent::Resize(w, h)) => {
                    window.set_inner_size(tao::dpi::LogicalSize::new(w as f64, h as f64));
                }
                // 处理窗口移动事件
                Event::UserEvent(UserEvent::Move(x, y)) => {
                    window.set_outer_position(tao::dpi::LogicalPosition::new(x as f64, y as f64));
                }
                // 处理打开 URL 事件
                Event::UserEvent(UserEvent::OpenUrl(u)) => {
                    let js = format!("location.href = {}", serde_json::to_string(&u).unwrap());
                    let _ = _webview.evaluate_script(&js);
                }
                // 处理窗口关闭请求
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            }
        });
    }

    // 非 macOS 平台分支（Windows/Linux）
    #[cfg(not(target_os = "macos"))]
    {
        // 创建 WebContext，用于管理 WebView 的持久化数据
        let mut web_context = WebContext::new(Some(data_path.clone()));

        // 构建 WebView（非 macOS 不设置 User-Agent）
        let mut builder = WebViewBuilder::new_with_web_context(&mut web_context)
            .with_url(url.as_str())
            .with_devtools(true)
            .with_initialization_script(&init_script)
            .with_new_window_req_handler({
                let newwin_proxy = proxy.clone();
                move |req_url, _features| {
                    // 新窗口请求通过事件循环处理
                    let _ = newwin_proxy.send_event(UserEvent::OpenUrl(req_url.to_string()));
                    NewWindowResponse::Deny
                }
            });

        // 注册自定义协议处理器（用于本地文件）
        if let Some(root) = local_root.clone() {
            builder = builder.with_custom_protocol("vibe".into(), move |_webview_id, request| {
                local_protocol_response(&root, request)
            });
        }

        // 构建 WebView 实例
        let _webview = match builder.build(&window) {
            Ok(w) => w,
            Err(e) => {
                let msg = format!(
                    "WebView build error: {e:?}\nLog: {log_path:?}\nHint: install Microsoft Edge WebView2 Runtime (Evergreen x64)."
                );
                append_log(&log_path, &msg);
                show_error_dialog("VibeWindow WebView", &msg);
                return;
            }
        };

        // 运行事件循环
        event_loop.run(move |event, _, control_flow| {
            if *control_flow != ControlFlow::Exit {
                *control_flow = ControlFlow::Wait;
            }

            match event {
                // 处理窗口大小调整事件
                Event::UserEvent(UserEvent::Resize(w, h)) => {
                    window.set_inner_size(tao::dpi::LogicalSize::new(w as f64, h as f64));
                }
                // 处理窗口移动事件
                Event::UserEvent(UserEvent::Move(x, y)) => {
                    window.set_outer_position(tao::dpi::LogicalPosition::new(x as f64, y as f64));
                }
                // 处理打开 URL 事件
                Event::UserEvent(UserEvent::OpenUrl(u)) => {
                    let js = format!("location.href = {}", serde_json::to_string(&u).unwrap());
                    let _ = _webview.evaluate_script(&js);
                }
                // 处理窗口关闭请求
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            }
        });
    }
}

/// WebAssembly 目标的空入口点
///
/// 由于 WebView 功能在 WebAssembly 环境中不可用，此函数为空实现。
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
