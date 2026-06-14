//! attachment_tests.rs 测试模块。
//!
//! 这些测试固定相邻解析器、视图辅助函数或状态计算的行为，防止后续 UI 重排时破坏边界契约。

use super::{
    AttachmentDisplayItem, AttachmentDisplayKind, attachment_preview_strip,
    is_supported_image_attachment, parse_attachment_markers, split_attachment_name_extension,
    truncate_attachment_name_middle,
};

/// 验证 truncate attachment name middle preserves short name 这一行为，确保对应解析或视图契约稳定。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn truncate_attachment_name_middle_preserves_short_name() {
    assert_eq!(truncate_attachment_name_middle("short-name.png", 20), "short-name.png");
}

/// 验证 truncate attachment name middle preserves extension and total length 这一行为，确保对应解析或视图契约稳定。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn truncate_attachment_name_middle_preserves_extension_and_total_length() {
    let result = truncate_attachment_name_middle("1234567890abcdefghidef.png", 20);

    assert_eq!(result, "1234567890...def.png");
    assert_eq!(result.chars().count(), 20);
}

/// 验证 truncate attachment name middle falls back without extension 这一行为，确保对应解析或视图契约稳定。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn truncate_attachment_name_middle_falls_back_without_extension() {
    let result = truncate_attachment_name_middle("1234567890123456789012345", 20);

    assert_eq!(result, "123456789...89012345");
    assert_eq!(result.chars().count(), 20);
}

#[test]
fn image_attachment_detection_is_case_insensitive_and_safe_for_missing_extensions() {
    for path in ["a.png", "b.JPG", "c.Jpeg", "d.webp", "e.GIF", "f.bmp"] {
        assert!(is_supported_image_attachment(path));
    }

    for path in ["README", "archive.zip", ".png", "image.svg", "folder/name"] {
        assert!(!is_supported_image_attachment(path));
    }
}

#[test]
fn attachment_display_item_classifies_local_paths() {
    let image = AttachmentDisplayItem::from_local_path("/tmp/screenshot.PNG");
    assert_eq!(image.kind, AttachmentDisplayKind::Image);
    assert_eq!(image.path, "/tmp/screenshot.PNG");
    assert_eq!(image.display_name, None);

    let document = AttachmentDisplayItem::from_local_path("/tmp/report.pdf");
    assert_eq!(document.kind, AttachmentDisplayKind::Document);
}

#[test]
fn split_attachment_name_extension_rejects_edge_dots() {
    assert_eq!(split_attachment_name_extension("report.final.pdf"), Some(("report.final", ".pdf")));
    assert_eq!(split_attachment_name_extension(".env"), None);
    assert_eq!(split_attachment_name_extension("trailing."), None);
    assert_eq!(split_attachment_name_extension("README"), None);
}

#[test]
fn truncate_attachment_name_middle_handles_tiny_limits_and_unicode() {
    assert_eq!(truncate_attachment_name_middle("abcdef", 3), "abc");
    assert_eq!(truncate_attachment_name_middle("abcdef", 5), "a...f");

    let result = truncate_attachment_name_middle("项目截图长文件名.png", 8);
    assert_eq!(result.chars().count(), 8);
    assert!(result.contains("..."));
}

#[test]
fn parse_attachment_markers_extracts_known_marker_forms_and_cleans_text() {
    let input =
        "before [IMAGE: /tmp/a.png]\n[DOCUMENT: /tmp/b.pdf]\n[Document: spec] /tmp/c.txt\n after ";
    let (cleaned, attachments) = parse_attachment_markers(input);

    assert_eq!(cleaned, "before\n\n\n after");
    assert_eq!(attachments.len(), 3);
    assert_eq!(attachments[0].path, "/tmp/a.png");
    assert_eq!(attachments[0].kind, AttachmentDisplayKind::Image);
    assert_eq!(attachments[1].kind, AttachmentDisplayKind::Document);
    assert_eq!(attachments[2].display_name.as_deref(), Some("spec"));
    assert_eq!(attachments[2].path, "/tmp/c.txt");
}

#[test]
fn parse_attachment_markers_preserves_unknown_empty_and_unclosed_markers() {
    let (cleaned, attachments) =
        parse_attachment_markers("keep [IMAGE: ] and [UNKNOWN: value] plus [DOCUMENT: /x");

    assert!(attachments.is_empty());
    assert_eq!(cleaned, "keep [IMAGE: ] and [UNKNOWN: value] plus [DOCUMENT: /x");
}

#[test]
fn attachment_preview_strip_builds_empty_and_non_empty_elements() {
    let _ = attachment_preview_strip(vec![]);
    let _ = attachment_preview_strip(vec![
        AttachmentDisplayItem::from_local_path("/tmp/missing.png"),
        AttachmentDisplayItem::from_local_path("/tmp/readme.txt"),
    ]);
}
