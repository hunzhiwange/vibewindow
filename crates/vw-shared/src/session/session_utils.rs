use once_cell::sync::Lazy;

use crate::time::now_ms;

/// 生成短随机 slug，失败时回退为当前时间戳。
pub fn create_slug() -> String {
    const CHARS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut bytes = [0u8; 8];
    if getrandom::getrandom(&mut bytes).is_err() {
        return format!("{:x}", now_ms());
    }
    let mut out = String::with_capacity(bytes.len());
    for b in bytes {
        out.push(CHARS[(b as usize) % 62] as char);
    }
    out
}

/// 判断标题是否仍是系统生成的默认标题。
pub fn is_default_title(title: &str) -> bool {
    static RE: Lazy<regex::Regex> = Lazy::new(|| {
        regex::Regex::new(
            r"^(New session - |Child session - )\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$",
        )
        .unwrap()
    });
    RE.is_match(title)
}

#[cfg(test)]
#[path = "session_utils_tests.rs"]
mod session_utils_tests;
