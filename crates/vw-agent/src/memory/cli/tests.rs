//! 内存 CLI 模块的单元测试
//!
//! 本模块包含对 `parse_category` 和 `truncate_content` 等辅助函数的测试用例。
//! 这些测试验证了内存分类解析和内容截断的正确性。

use super::*;

/// 测试 `parse_category` 函数对已知分类变体的解析
///
/// # 验证点
/// - 能够正确解析标准分类：core、daily、conversation
/// - 解析不区分大小写（CORE 应解析为 Core）
/// - 解析能够忽略首尾空白字符（"  Daily  " 应解析为 Daily）
#[test]
fn parse_category_known_variants() {
    assert_eq!(parse_category("core"), MemoryCategory::Core);
    assert_eq!(parse_category("daily"), MemoryCategory::Daily);
    assert_eq!(parse_category("conversation"), MemoryCategory::Conversation);
    assert_eq!(parse_category("CORE"), MemoryCategory::Core);
    assert_eq!(parse_category("  Daily  "), MemoryCategory::Daily);
}

/// 测试 `parse_category` 函数对自定义分类的回退处理
///
/// # 验证点
/// - 无法识别的分类字符串应作为自定义分类（Custom）返回
/// - 自定义分类保留原始字符串值
#[test]
fn parse_category_custom_fallback() {
    assert_eq!(parse_category("project_notes"), MemoryCategory::Custom("project_notes".into()));
}

/// 测试 `truncate_content` 函数对短文本的处理
///
/// # 验证点
/// - 当文本长度小于最大长度时，应保持原样返回
#[test]
fn truncate_content_short_text_unchanged() {
    assert_eq!(truncate_content("hello", 10), "hello");
}

/// 测试 `truncate_content` 函数对长文本的截断处理
///
/// # 验证点
/// - 当文本长度超过最大长度时，应进行截断
/// - 截断后的文本应以 "..." 结尾
/// - 截断后的总字符数不应超过指定的最大长度
#[test]
fn truncate_content_long_text_truncated() {
    let result = truncate_content("this is a very long string", 10);
    assert!(result.ends_with("..."));
    assert!(result.chars().count() <= 10);
}

/// 测试 `truncate_content` 函数对多行文本的处理
///
/// # 验证点
/// - 多行文本应只使用第一行内容
/// - 第一行提取后再应用截断规则
#[test]
fn truncate_content_multiline_uses_first_line() {
    assert_eq!(truncate_content("first\nsecond", 20), "first");
}

/// 测试 `truncate_content` 函数对空字符串的处理
///
/// # 验证点
/// - 空字符串应原样返回，不发生任何变化
#[test]
fn truncate_content_empty_string() {
    assert_eq!(truncate_content("", 10), "");
}
