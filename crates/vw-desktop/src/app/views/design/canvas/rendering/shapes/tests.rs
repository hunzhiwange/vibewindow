//! # Shapes 模块单元测试
//!
//! 本模块包含对 `shapes` 模块中核心功能的单元测试。
//!
//! ## 主要测试内容
//!
//! - **槽位展开测试**：验证 `expand_slot_children` 函数能够正确地将设计元素中的
//!   槽位（slot）字段展开为引用类型（ref）的子元素集合
//!
//! ## 测试覆盖范围
//!
//! 当前测试套件覆盖以下场景：
//! - 非空槽位数组的展开
//!
//! ## 相关模块
//!
//! - [`super`] (父模块)：包含被测试的核心实现，如 `DesignElement` 和 `expand_slot_children`

use super::helpers::{clamp_child_size_to_content, expand_slot_children};
use crate::app::views::design::models::DesignElement;

/// Shapes 模块的单元测试集
///
/// 本测试模块使用 `#[allow(dead_code)]` 属性，因为测试函数可能不会被
/// 非测试构建直接调用，但仍需保留以供 `cargo test` 使用。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试：将非空槽位展开为引用类型的子元素
    ///
    /// ## 测试目的
    ///
    /// 验证 `expand_slot_children` 函数能够正确处理包含多个引用 ID 的非空槽位，
    /// 并将其转换为对应的引用类型（ref）子元素数组。
    ///
    /// ## 测试场景
    ///
    /// - **输入**：一个 `DesignElement` 实例，包含：
    ///   - `kind`: "frame" - 元素类型为框架
    ///   - `id`: "root" - 根元素标识符
    ///   - `slot`: JSON 数组 `["A", "B"]` - 包含两个引用 ID
    ///
    /// - **预期输出**：包含两个元素的数组，每个元素都是引用类型（ref），
    ///   分别指向 "A" 和 "B"
    ///
    /// ## 验证点
    ///
    /// 1. 输出数组长度应为 2
    /// 2. 第一个子元素的 `kind` 应为 "ref"
    /// 3. 第一个子元素的 `reference` 应为 "A"
    /// 4. 第二个子元素的 `reference` 应为 "B"
    ///
    /// ## 示例
    ///
    /// ```ignore
    /// // 输入的 DesignElement
    /// {
    ///     kind: "frame",
    ///     id: "root",
    ///     slot: ["A", "B"]
    /// }
    ///
    /// // 预期输出
    /// [
    ///     { kind: "ref", reference: "A" },
    ///     { kind: "ref", reference: "B" }
    /// ]
    /// ```
    #[test]
    fn expands_non_empty_slot_into_ref_children() {
        // 创建测试用的设计元素实例
        // - 类型为 "frame"，表示一个框架容器
        // - ID 为 "root"，标识根元素
        // - slot 包含 JSON 数组 ["A", "B"]，代表两个子元素的引用
        let el = DesignElement {
            kind: "frame".to_string(),
            id: "root".to_string(),
            slot: Some(serde_json::json!(["A", "B"])),
            ..Default::default()
        };

        // 执行槽位展开函数，将 slot 中的引用 ID 转换为引用类型子元素
        let out = expand_slot_children(&el).unwrap();

        // 验证：输出数组长度应为 2（对应 slot 中的两个元素）
        assert_eq!(out.len(), 2);

        // 验证：第一个子元素的类型应为 "ref"（引用类型）
        assert_eq!(out[0].kind, "ref");

        // 验证：第一个子元素的引用目标应为 "A"
        assert_eq!(out[0].reference.as_deref(), Some("A"));

        // 验证：第二个子元素的引用目标应为 "B"
        assert_eq!(out[1].reference.as_deref(), Some("B"));
    }

    #[test]
    fn clamp_child_size_to_content_limits_overflowing_size() {
        let clipped = clamp_child_size_to_content(
            iced::Size::new(204.0, 32.0),
            12.0,
            5.0,
            iced::Size::new(392.0, 22.0),
        );
        assert_eq!(clipped.width, 192.0);
        assert_eq!(clipped.height, 22.0);
    }

    #[test]
    fn clamp_child_size_to_content_returns_zero_when_child_starts_outside() {
        let clipped = clamp_child_size_to_content(
            iced::Size::new(204.0, 32.0),
            240.0,
            40.0,
            iced::Size::new(30.0, 20.0),
        );
        assert_eq!(clipped.width, 0.0);
        assert_eq!(clipped.height, 0.0);
    }
}
