//! 覆盖预览面板相关行为的回归测试。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use std::io::Write;

use super::{preview_open_error, safe_preview};

fn write_temp_file(name: &str, bytes: &[u8]) -> tempfile::NamedTempFile {
    let mut file = tempfile::Builder::new().prefix(name).tempfile().expect("create temp file");
    file.write_all(bytes).expect("write temp file");
    file.as_file().sync_all().expect("sync temp file");
    file
}

#[test]
fn preview_open_error_allows_utf8_text_files() {
    let file = write_temp_file("preview_text", b"fn main() {}\n");
    let path = file.path().to_string_lossy().to_string();

    assert_eq!(preview_open_error(&path), None);
}

#[test]
fn preview_open_error_blocks_binary_files() {
    let file = write_temp_file("preview_binary", &[0x50, 0x4B, 0x03, 0x04, 0x00, 0xFF, 0xAA, 0x00]);
    let path = file.path().to_string_lossy().to_string();

    assert!(preview_open_error(&path).is_some());
}

#[test]
fn safe_preview_reports_binary_files_instead_of_raw_bytes() {
    let file = write_temp_file("preview_binary", &[0x50, 0x4B, 0x03, 0x04, 0x00, 0xFF, 0xAA, 0x00]);
    let path = file.path().to_string_lossy().to_string();

    let (content, truncated) = safe_preview(&path);

    assert!(!truncated);
    assert!(content.contains("二进制文件"));
    assert!(content.contains("不支持文本预览"));
}
