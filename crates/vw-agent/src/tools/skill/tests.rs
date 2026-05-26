//! SkillTool 工具的单元测试模块
//!
//! 本模块包含针对 `SkillTool` 的测试用例，主要验证工具的 JSON Schema 定义
//! 是否符合预期的结构和必需字段要求。
//!
//! # 测试覆盖
//!
//! - Schema 结构验证：确保返回的 schema 是有效的 JSON 对象格式
//! - 必需字段验证：确保 `name` 字段被正确标记为必需字段

use super::super::*;
use serde_json::json;

/// 测试 SkillTool 的 schema 包含所有必需字段
///
/// 此测试验证 `SkillTool::schema()` 返回的 JSON Schema 符合以下要求：
///
/// # 验证项
///
/// 1. **类型验证**：schema 的顶层 `type` 字段应为 `"object"`
/// 2. **属性验证**：`properties` 中必须包含 `name` 字段，且其为对象类型
/// 3. **必需字段验证**：`required` 数组应包含 `"name"` 字符串
///
/// # 示例
///
/// 期望的 schema 结构示例：
/// ```json
/// {
///     "type": "object",
///     "properties": {
///         "name": { ... }
///     },
///     "required": ["name"]
/// }
/// ```
#[test]
fn skill_schema_has_required_fields() {
    // 获取 SkillTool 的 JSON Schema 定义
    let schema = SkillTool::schema();

    // 验证 schema 类型为对象
    assert_eq!(schema["type"], "object");

    // 验证 properties 中存在 name 字段，且其为对象类型
    assert!(schema["properties"]["name"].is_object());

    // 验证 name 字段被标记为必需字段
    assert_eq!(schema["required"], json!(["name"]));
}
