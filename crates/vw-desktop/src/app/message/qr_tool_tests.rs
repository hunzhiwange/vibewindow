//! 覆盖二维码工具消息处理行为，验证输入校验、生成和保存状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{QrEcLevel, QrRenderRequest, parse_qr_size_input, render_qr_png};

#[test]
fn parse_qr_size_input_accepts_valid_value() {
    assert_eq!(parse_qr_size_input("256").unwrap(), 256);
}

#[test]
fn parse_qr_size_input_rejects_out_of_range_value() {
    let error = parse_qr_size_input("32").unwrap_err();
    assert!(error.contains("64-2048"));
}

#[test]
fn render_qr_png_returns_png_bytes() {
    let request = QrRenderRequest {
        data: "https://example.com".to_string(),
        size: 256,
        level: QrEcLevel::M,
        color_hex: "#000000".to_string(),
        icon_bytes: None,
    };

    let png = render_qr_png(&request).unwrap();

    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    assert!(png.len() > 32);
}
