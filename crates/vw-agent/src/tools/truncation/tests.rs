//! 文本截断功能测试模块
//!
//! 本模块提供了针对 `truncation` 模块中 `compute_preview` 函数的单元测试。
//! 主要测试以下场景：
//! - 从文本头部按行数截断
//! - 从文本头部按字节数截断
//! - 从文本尾部按行数截断
//!
//! # 测试覆盖
//!
//! - `Direction::Head` 策略下的行截断和字节截断
//! - `Direction::Tail` 策略下的行截断
//! - 截断结果的正确性验证（文本内容、行数、字节数、是否命中字节限制）

use super::super::truncation::{Direction, Options, compute_preview};

/// 测试从头部按行数截断文本
///
/// # 测试场景
///
/// - 输入：包含 4 行的文本（"a", "b", "c", "d"）
/// - 配置：最大 2 行，最大 1000 字节，从头部截断
/// - 预期输出：前 2 行内容（"a\nb"）
///
/// # 验证点
///
/// - 返回的文本内容应仅包含前 2 行
/// - `hit_bytes` 标志应为 false（未达到字节限制）
/// - `line_count` 应准确反映返回的行数
#[test]
fn head_truncates_by_lines() {
    // 准备测试数据：4 行文本，每行一个字母
    let text = ["a", "b", "c", "d"].join("\n");

    // 配置截断选项：最多 2 行，字节限制宽松（1000 字节），从头部截取
    let opts = Options { max_lines: 2, max_bytes: 1000, direction: Direction::Head };

    // 执行预览计算
    let p = compute_preview(&text, &opts);

    // 验证：结果文本应为前 2 行
    assert_eq!(p.text, "a\nb");
    // 验证：未触发字节限制
    assert!(!p.hit_bytes);
    // 验证：行数计数正确
    assert_eq!(p.line_count, 2);
}

/// 测试从头部按字节数截断文本
///
/// # 测试场景
///
/// - 输入：3 行文本，总字节数为 17（5+1+5+1+5）
/// - 配置：最大 2000 行，最大 7 字节，从头部截断
/// - 预期输出：第 1 行内容（"12345"），因为 7 字节限制会截断到第一个换行符前
///
/// # 验证点
///
/// - 返回的文本应只包含第 1 行（5 字节）
/// - `hit_bytes` 标志应为 true（达到了字节限制）
/// - `bytes` 字段应准确反映返回的字节数
#[test]
fn head_truncates_by_bytes() {
    // 准备测试数据：3 行，每行 5 字符，总大小 17 字节（含换行符）
    let text = ["12345", "67890", "abcde"].join("\n");

    // 配置截断选项：行数限制宽松，字节限制为 7 字节，从头部截取
    let opts = Options { max_lines: 2000, max_bytes: 7, direction: Direction::Head };

    // 执行预览计算
    let p = compute_preview(&text, &opts);

    // 验证：结果文本应为第 1 行（5 字节），因为 7 字节限制会导致在第 1 行后截断
    assert_eq!(p.text, "12345");
    // 验证：触发了字节限制
    assert!(p.hit_bytes);
    // 验证：字节数计数正确（5 字节）
    assert_eq!(p.bytes, 5);
}

/// 测试从尾部按行数截断文本
///
/// # 测试场景
///
/// - 输入：包含 4 行的文本（"a", "b", "c", "d"）
/// - 配置：最大 2 行，最大 1000 字节，从尾部截断
/// - 预期输出：最后 2 行内容（"c\nd"）
///
/// # 验证点
///
/// - 返回的文本内容应仅包含最后 2 行
/// - `hit_bytes` 标志应为 false（未达到字节限制）
/// - `line_count` 应准确反映返回的行数
#[test]
fn tail_truncates_from_end() {
    // 准备测试数据：4 行文本，每行一个字母
    let text = ["a", "b", "c", "d"].join("\n");

    // 配置截断选项：最多 2 行，字节限制宽松（1000 字节），从尾部截取
    let opts = Options { max_lines: 2, max_bytes: 1000, direction: Direction::Tail };

    // 执行预览计算
    let p = compute_preview(&text, &opts);

    // 验证：结果文本应为最后 2 行
    assert_eq!(p.text, "c\nd");
    // 验证：未触发字节限制
    assert!(!p.hit_bytes);
    // 验证：行数计数正确
    assert_eq!(p.line_count, 2);
}
