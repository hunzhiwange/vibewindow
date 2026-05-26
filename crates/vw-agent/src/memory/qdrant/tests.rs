//! Qdrant 向量数据库内存后端测试模块
//!
//! 本模块提供针对 `QdrantMemory` 实现的单元测试，验证记忆存储与检索的核心功能：
//! - 记忆类别与字符串表示之间的双向转换
//! - 记忆负载（MemoryPayload）的 JSON 序列化行为
//!
//! # 测试范围
//!
//! 1. **类别映射测试**：验证 `MemoryCategory` 枚举与字符串之间的正确转换
//! 2. **序列化测试**：验证 `MemoryPayload` 结构体的 JSON 序列化输出
//!
//! # 依赖关系
//!
//! - 依赖父模块中的 `QdrantMemory`、`MemoryCategory` 和 `MemoryPayload` 类型
//! - 使用 `serde_json` 进行序列化验证

use super::*;

/// 测试 `category_to_str` 方法对已知类别的映射
///
/// # 测试场景
///
/// 验证 `QdrantMemory::category_to_str` 方法能够正确处理：
/// - 标准类别：`Core`、`Daily`、`Conversation`
/// - 自定义类别：`Custom(String)` 变体
///
/// # 预期行为
///
/// - `MemoryCategory::Core` 应映射为字符串 `"core"`
/// - `MemoryCategory::Daily` 应映射为字符串 `"daily"`
/// - `MemoryCategory::Conversation` 应映射为字符串 `"conversation"`
/// - `MemoryCategory::Custom("notes")` 应映射为字符串 `"notes"`
#[test]
fn category_to_str_maps_known_categories() {
    // 验证标准类别的字符串映射
    assert_eq!(QdrantMemory::category_to_str(&MemoryCategory::Core), "core");
    assert_eq!(QdrantMemory::category_to_str(&MemoryCategory::Daily), "daily");
    assert_eq!(QdrantMemory::category_to_str(&MemoryCategory::Conversation), "conversation");

    // 验证自定义类别的字符串映射（提取内部值）
    assert_eq!(QdrantMemory::category_to_str(&MemoryCategory::Custom("notes".into())), "notes");
}

/// 测试 `parse_category` 方法对已知和自定义值的解析
///
/// # 测试场景
///
/// 验证 `QdrantMemory::parse_category` 方法能够正确将字符串反向转换为 `MemoryCategory`：
/// - 标准字符串：`"core"`、`"daily"`、`"conversation"`
/// - 自定义字符串：任何非标准值应转换为 `Custom` 变体
///
/// # 预期行为
///
/// - 字符串 `"core"` 应解析为 `MemoryCategory::Core`
/// - 字符串 `"daily"` 应解析为 `MemoryCategory::Daily`
/// - 字符串 `"conversation"` 应解析为 `MemoryCategory::Conversation`
/// - 字符串 `"custom_notes"` 应解析为 `MemoryCategory::Custom("custom_notes")`
#[test]
fn parse_category_maps_known_and_custom_values() {
    // 验证标准字符串的解析
    assert_eq!(QdrantMemory::parse_category("core"), MemoryCategory::Core);
    assert_eq!(QdrantMemory::parse_category("daily"), MemoryCategory::Daily);
    assert_eq!(QdrantMemory::parse_category("conversation"), MemoryCategory::Conversation);

    // 验证自定义字符串的解析（包装为 Custom 变体）
    assert_eq!(
        QdrantMemory::parse_category("custom_notes"),
        MemoryCategory::Custom("custom_notes".into())
    );
}

/// 测试 `MemoryPayload` 结构体的完整序列化
///
/// # 测试场景
///
/// 验证包含所有字段的 `MemoryPayload` 实例能够正确序列化为 JSON：
/// - `key`: 记忆项的唯一标识
/// - `content`: 记忆内容文本
/// - `category`: 记忆类别字符串
/// - `timestamp`: 时间戳（ISO 8601 格式）
/// - `session_id`: 可选的会话 ID（此处为 `Some` 值）
///
/// # 预期行为
///
/// 序列化后的 JSON 字符串应包含所有字段的值，包括 `session_id` 字段
#[test]
fn memory_payload_serializes_correctly() {
    // 构建测试用的记忆负载实例（包含所有字段）
    let payload = MemoryPayload {
        key: "test_key".into(),
        content: "test content".into(),
        category: "core".into(),
        timestamp: "2026-02-20T00:00:00Z".into(),
        session_id: Some("session-1".into()), // 会话 ID 存在
    };

    // 执行 JSON 序列化
    let json = serde_json::to_string(&payload).unwrap();

    // 验证关键字段在 JSON 输出中存在
    assert!(json.contains("test_key"));
    assert!(json.contains("test content"));
    assert!(json.contains("session-1")); // 包含会话 ID
}

/// 测试 `MemoryPayload` 在 `session_id` 为 `None` 时的序列化行为
///
/// # 测试场景
///
/// 验证当 `session_id` 字段为 `None` 时，序列化后的 JSON 应跳过该字段，
/// 而不是输出 `"session_id": null`。这依赖于 `#[serde(skip_serializing_if = "Option::is_none")]` 属性。
///
/// # 预期行为
///
/// - 当 `session_id` 为 `None` 时，JSON 输出不应包含 `session_id` 字段
/// - 其他字段应正常序列化
///
/// # 技术要点
///
/// 此行为确保了：
/// 1. JSON 输出更加简洁，避免冗余的 `null` 值
/// 2. 与 Qdrant 向量数据库的 payload 格式保持一致
#[test]
fn memory_payload_skips_none_session_id() {
    // 构建测试用的记忆负载实例（session_id 为 None）
    let payload = MemoryPayload {
        key: "test_key".into(),
        content: "test content".into(),
        category: "core".into(),
        timestamp: "2026-02-20T00:00:00Z".into(),
        session_id: None, // 会话 ID 不存在
    };

    // 执行 JSON 序列化
    let json = serde_json::to_string(&payload).unwrap();

    // 验证 session_id 字段在 JSON 输出中不存在
    assert!(!json.contains("session_id"));
}
