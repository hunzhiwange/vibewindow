//! # VibeWindow WebView 应用入口
//!
//! 本模块是 VibeWindow WebView 应用的主入口点，提供跨平台的 WebView 窗口功能。

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
use std::borrow::Cow;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{self, Component, Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use wry::http::{Request, Response, StatusCode};

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_TARGET: &str = "https://example.com/";
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_TITLE: &str = "Vibe Window WebView";
#[cfg(not(target_arch = "wasm32"))]
const SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3 Safari/605.1.15";

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq, Eq)]
enum UserEvent {
    Resize(u32, u32),
    Move(i32, i32),
    OpenUrl(String),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq, Eq)]
enum StartTarget {
    Url(String),
    LocalDir { root: PathBuf, entry: String },
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq, Eq)]
struct LaunchOptions {
    mode: String,
    window_title: String,
    width: Option<i32>,
    height: Option<i32>,
    pos_x: Option<i32>,
    pos_y: Option<i32>,
    cookies_json: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            mode: "window".to_string(),
            window_title: DEFAULT_TITLE.to_string(),
            width: None,
            height: None,
            pos_x: None,
            pos_y: None,
            cookies_json: "[]".to_string(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_url(s: &str) -> String {
    let mut t = s.trim();
    loop {
        let before = t;
        for q in ['`', '"', '\''] {
            if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
                t = &t[1..t.len() - 1];
                t = t.trim();
            }
        }
        if t == before {
            break;
        }
    }
    t.to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn percent_encode_url_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 8);
    for b in path.as_bytes() {
        let c = *b as char;
        let is_unreserved = matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~');
        if is_unreserved || c == '/' {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn percent_decode(s: &str) -> String {
    fn from_hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h1), Some(h2)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2]))
        {
            out.push((h1 << 4) | h2);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn is_safe_relative_path(p: &Path) -> bool {
    !p.components()
        .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
}

#[cfg(not(target_arch = "wasm32"))]
fn guess_mime(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn local_protocol_response(root: &Path, request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let raw_path = request.uri().path();
    let req_path = if raw_path.is_empty() { "/" } else { raw_path };
    let rel = percent_decode(req_path.trim_start_matches('/'));
    let rel_path = Path::new(&rel);

    if !rel.is_empty() && !is_safe_relative_path(rel_path) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Cow::Owned(Vec::new()))
            .unwrap();
    }

    let mut file_path = if rel.is_empty() { root.join("index.html") } else { root.join(rel_path) };
    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    let (status, body, mime_path) = if let Ok(bytes) = std::fs::read(&file_path) {
        (StatusCode::OK, bytes, file_path)
    } else {
        let fallback_index = root.join("index.html");
        let should_fallback = path::Path::new(req_path)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| !s.contains('.'))
            .unwrap_or(true);
        if should_fallback {
            match std::fs::read(&fallback_index) {
                Ok(bytes) => (StatusCode::OK, bytes, fallback_index),
                Err(_) => (StatusCode::NOT_FOUND, Vec::new(), file_path),
            }
        } else {
            (StatusCode::NOT_FOUND, Vec::new(), file_path)
        }
    };

    Response::builder()
        .status(status)
        .header("Content-Type", guess_mime(&mime_path))
        .body(Cow::Owned(body))
        .unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let s = url.strip_prefix("file://")?;
    let s = s.strip_prefix("localhost").unwrap_or(s);
    let s = s.strip_prefix('/').unwrap_or(s);
    let decoded = percent_decode(s);

    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(decoded.replace('/', "\\")))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Some(PathBuf::from(format!("/{decoded}")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_target(target: &str) -> StartTarget {
    let lower = target.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return StartTarget::Url(target.to_string());
    }

    let as_path = if lower.starts_with("file://") {
        file_url_to_path(target).unwrap_or_else(|| PathBuf::from(target))
    } else {
        PathBuf::from(target)
    };

    let candidate = match std::fs::metadata(&as_path) {
        Ok(m) if m.is_dir() => {
            let index = as_path.join("index.html");
            if index.is_file() { Some((as_path, index)) } else { None }
        }
        Ok(m) if m.is_file() => as_path.parent().map(|root| (root.to_path_buf(), as_path.clone())),
        _ => None,
    };

    if let Some((root, entry_file)) = candidate
        && let (Ok(root), Ok(entry_file)) =
            (std::fs::canonicalize(&root), std::fs::canonicalize(&entry_file))
        && let Some(name) = entry_file.file_name().and_then(|s| s.to_str())
    {
        return StartTarget::LocalDir { root, entry: format!("/{name}") };
    }

    StartTarget::Url(target.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_launch_options(args: &[String]) -> LaunchOptions {
    let mut options = LaunchOptions::default();
    for a in args {
        if let Some(v) = a.strip_prefix("--mode=") {
            options.mode = v.to_string();
        } else if let Some(v) = a.strip_prefix("--cookies=") {
            options.cookies_json = v.to_string();
        } else if let Some(v) = a.strip_prefix("--width=") {
            options.width = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--height=") {
            options.height = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--x=") {
            options.pos_x = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--y=") {
            options.pos_y = v.parse::<i32>().ok();
        } else if let Some(v) = a.strip_prefix("--title=") {
            options.window_title = v.to_string();
        }
    }
    options
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_stdin_event(line: &str) -> Option<UserEvent> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["resize", w, h, ..] => Some(UserEvent::Resize(w.parse().ok()?, h.parse().ok()?)),
        ["move", x, y, ..] => Some(UserEvent::Move(x.parse().ok()?, y.parse().ok()?)),
        _ => None,
    }
}

#[cfg(not(any(target_arch = "wasm32", coverage)))]
fn webview_data_dir() -> PathBuf {
    if let Some(dirs) = directories::ProjectDirs::from("dev", "VibeWindow", "vibe-window") {
        #[cfg(target_os = "windows")]
        {
            let base = dirs.config_dir().parent().unwrap_or_else(|| dirs.config_dir());
            return base.join("webview_data");
        }

        #[cfg(not(target_os = "windows"))]
        {
            return dirs.data_dir().join("webview_data");
        }
    }

    fallback_webview_data_dir()
}

#[cfg(all(not(target_arch = "wasm32"), coverage))]
fn webview_data_dir() -> PathBuf {
    fallback_webview_data_dir()
}

#[cfg(not(target_arch = "wasm32"))]
fn fallback_webview_data_dir() -> PathBuf {
    let base_data_dir = directories::BaseDirs::new().map(|dirs| dirs.data_dir().to_path_buf());
    fallback_webview_data_dir_from(base_data_dir.as_deref())
}

#[cfg(not(target_arch = "wasm32"))]
fn fallback_webview_data_dir_from(base_data_dir: Option<&Path>) -> PathBuf {
    base_data_dir
        .map(|path| path.join("vibe-window").join("webview_data"))
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("webview_data"))
}

#[cfg(not(target_arch = "wasm32"))]
fn append_log(log_path: &Path, s: &str) {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut buf = String::with_capacity(s.len() + 1);
    buf.push_str(s);
    buf.push('\n');
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, buf.as_bytes()));
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn build_init_script(
    data_path: &Path,
    cookies_json: &str,
    user_agent: &str,
) -> Result<String, serde_json::Error> {
    let data_path_js = serde_json::to_string(&data_path.to_string_lossy())?;
    let ua_js = serde_json::to_string(user_agent)?;
    Ok(format!(
        r#"
        (function() {{
            console.log('%c [VibeWindow] Init Script Starting...', 'color: green; font-weight: bold');
            const dataPath = {data_path_js};
            const userAgent = {ua_js};
            const cookieConfigs = {cookies_json};
            console.log('Data Path:', dataPath);

            const DEFAULT_DOMAIN = '';

            function escapeRegExp(s) {{
                return String(s).replace(/[.*+?^${{}}()|[\]\\]/g, '\\$&');
            }}

            function normalizeDomain(domain) {{
                if (!domain) return null;
                const d = String(domain).trim();
                if (!d) return null;
                return d.startsWith('.') ? d : `.${{d}}`;
            }}

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

            function getCookie(name) {{
                if (!name) return null;
                const safeName = escapeRegExp(name);
                const match = document.cookie.match(new RegExp('(?:^|;\\s*)' + safeName + '=([^;]+)'));
                return match ? match[1] : null;
            }}

            function initSessionSync() {{
                try {{
                    let configs = cookieConfigs || [];

                    function getBackupName(name) {{
                        return 'vibe_window_webview_backup_' + name;
                    }}

                    configs.forEach(config => {{
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

                    let lastCookie = document.cookie;
                    setInterval(() => {{
                        const currentCookie = document.cookie;
                        if (currentCookie !== lastCookie) {{
                            configs.forEach(config => {{
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

            if (document.readyState === 'loading') {{
                document.addEventListener('DOMContentLoaded', initSessionSync);
            }} else {{
                initSessionSync();
            }}
        }})();
        "#
    ))
}

#[cfg(not(any(target_arch = "wasm32", coverage)))]
fn main() {
    use std::env;
    use std::io::{self, BufRead};
    use std::thread;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::WindowBuilder;
    use wry::{NewWindowResponse, WebContext, WebViewBuilder};

    let args = env::args().skip(1).collect::<Vec<_>>();
    let target = normalize_url(args.first().map(String::as_str).unwrap_or(DEFAULT_TARGET));
    let options = parse_launch_options(&args[1..]);
    let (url, local_root) = match resolve_target(&target) {
        StartTarget::Url(url) => (url, None),
        StartTarget::LocalDir { root, entry } => {
            let entry = percent_encode_url_path(&entry);
            (format!("vibe://localhost{entry}"), Some(root))
        }
    };

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    {
        let stdin_proxy = proxy.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines().map_while(Result::ok) {
                if let Some(event) = parse_stdin_event(&line) {
                    let _ = stdin_proxy.send_event(event);
                }
            }
        });
    }

    let mut wb = WindowBuilder::new().with_title(options.window_title.clone());
    if options.mode == "embedded" {
        wb = wb.with_decorations(false).with_always_on_top(true);
    }

    let size_w = options.width.unwrap_or(1280) as f64;
    let size_h = options.height.unwrap_or(800) as f64;
    wb = wb.with_inner_size(tao::dpi::LogicalSize::new(size_w, size_h));
    if let (Some(px), Some(py)) = (options.pos_x, options.pos_y) {
        wb = wb.with_position(tao::dpi::LogicalPosition::new(px as f64, py as f64));
    }

    let data_path = webview_data_dir();
    let log_path = data_path.join("vw-webview.log");
    {
        let log_path_for_panic = log_path.clone();
        std::panic::set_hook(Box::new(move |info| {
            append_log(&log_path_for_panic, &format!("panic: {info}"));
        }));
    }

    if let Err(e) = std::fs::create_dir_all(&data_path) {
        eprintln!("Failed to create data directory: {e}");
        append_log(&log_path, &format!("Failed to create data directory: {e}"));
    } else {
        println!("WebView data path: {data_path:?}");
        append_log(&log_path, &format!("WebView data path: {data_path:?}"));
    }
    append_log(&log_path, &format!("Args: {:?}", env::args().collect::<Vec<_>>()));
    append_log(&log_path, &format!("Normalized url: {url:?}"));

    let window = match wb.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            let msg = format!("Failed to create window: {e:?}\nLog: {log_path:?}");
            append_log(&log_path, &msg);
            show_error_dialog("VibeWindow WebView", &msg);
            return;
        }
    };

    let init_script = match build_init_script(&data_path, &options.cookies_json, SAFARI_USER_AGENT)
    {
        Ok(script) => script,
        Err(e) => {
            let msg = format!("Failed to build initialization script: {e}\nLog: {log_path:?}");
            append_log(&log_path, &msg);
            show_error_dialog("VibeWindow WebView", &msg);
            return;
        }
    };

    let mut web_context = WebContext::new(Some(data_path.clone()));

    #[cfg(target_os = "macos")]
    let mut builder = {
        WebViewBuilder::new_with_web_context(&mut web_context)
            .with_url(url.as_str())
            .with_user_agent(SAFARI_USER_AGENT)
            .with_devtools(true)
            .with_initialization_script(&init_script)
            .with_new_window_req_handler({
                let newwin_proxy = proxy.clone();
                move |req_url, _features| {
                    let _ = newwin_proxy.send_event(UserEvent::OpenUrl(req_url.to_string()));
                    NewWindowResponse::Deny
                }
            })
    };

    #[cfg(not(target_os = "macos"))]
    let mut builder = {
        WebViewBuilder::new_with_web_context(&mut web_context)
            .with_url(url.as_str())
            .with_devtools(true)
            .with_initialization_script(&init_script)
            .with_new_window_req_handler({
                let newwin_proxy = proxy.clone();
                move |req_url, _features| {
                    let _ = newwin_proxy.send_event(UserEvent::OpenUrl(req_url.to_string()));
                    NewWindowResponse::Deny
                }
            })
    };

    if let Some(root) = local_root.clone() {
        builder = builder.with_custom_protocol("vibe".into(), move |_webview_id, request| {
            local_protocol_response(&root, request)
        });
    }

    let webview = match builder.build(&window) {
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

    event_loop.run(move |event, _, control_flow| {
        if *control_flow != ControlFlow::Exit {
            *control_flow = ControlFlow::Wait;
        }

        match event {
            Event::UserEvent(UserEvent::Resize(w, h)) => {
                window.set_inner_size(tao::dpi::LogicalSize::new(w as f64, h as f64));
            }
            Event::UserEvent(UserEvent::Move(x, y)) => {
                window.set_outer_position(tao::dpi::LogicalPosition::new(x as f64, y as f64));
            }
            Event::UserEvent(UserEvent::OpenUrl(u)) => {
                let js = format!("location.href = {}", serde_json::to_string(&u).unwrap());
                let _ = webview.evaluate_script(&js);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

#[cfg(any(target_arch = "wasm32", coverage))]
fn main() {}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
