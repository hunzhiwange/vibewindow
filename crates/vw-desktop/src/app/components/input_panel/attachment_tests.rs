//! attachment_tests.rs 测试模块。
//!
//! 这些测试固定相邻解析器、视图辅助函数或状态计算的行为，防止后续 UI 重排时破坏边界契约。

use super::truncate_attachment_name_middle;

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
