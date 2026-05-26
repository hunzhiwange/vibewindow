//! NoneMemory 存储后端的单元测试模块
//!
//! 本模块提供了针对 `NoneMemory` 的测试用例，验证其作为空操作（no-op）
//! 内存后端的行为是否符合预期。NoneMemory 是一个特殊的内存实现，
//! 它不会实际存储任何数据，所有操作都是空操作。

use super::*;

/// 测试 NoneMemory 的所有操作都是空操作（no-op）
///
/// # 测试内容
///
/// 该测试验证 NoneMemory 的以下行为：
/// - **存储操作**：调用 `store` 方法不会实际存储数据
/// - **获取操作**：`get` 方法始终返回 `None`
/// - **召回操作**：`recall` 方法始终返回空列表
/// - **列表操作**：`list` 方法始终返回空列表
/// - **删除操作**：`forget` 方法始终返回 `false`（表示没有删除任何内容）
/// - **计数操作**：`count` 方法始终返回 `0`
/// - **健康检查**：`health_check` 方法始终返回 `true`
///
/// # 预期行为
///
/// NoneMemory 是一个特殊的内存实现，它的设计目的是：
/// 1. 提供一个不会产生任何副作用的内存后端
/// 2. 在不需要持久化记忆的场景下作为占位符使用
/// 3. 在测试或特殊配置中禁用记忆功能
///
/// # 示例
///
/// ```ignore
/// let memory = NoneMemory::new();
/// // 无论执行什么操作，NoneMemory 都不会存储任何数据
/// memory.store("key", "value", MemoryCategory::Core, None).await.unwrap();
/// assert!(memory.get("key").await.unwrap().is_none());
/// ```
#[tokio::test]
async fn none_memory_is_noop() {
    // 创建一个新的 NoneMemory 实例
    let memory = NoneMemory::new();

    // 尝试存储键值对
    // 即使调用成功，数据也不会被实际存储
    memory.store("k", "v", MemoryCategory::Core, None).await.unwrap();

    // 验证获取操作返回 None（数据未被存储）
    assert!(memory.get("k").await.unwrap().is_none());

    // 验证召回操作返回空列表（没有匹配的记录）
    assert!(memory.recall("k", 10, None).await.unwrap().is_empty());

    // 验证列表操作返回空列表（没有任何存储的记录）
    assert!(memory.list(None, None).await.unwrap().is_empty());

    // 验证删除操作返回 false（表示没有删除任何内容，因为本来就没有存储）
    assert!(!memory.forget("k").await.unwrap());

    // 验证计数操作返回 0（没有存储任何记录）
    assert_eq!(memory.count().await.unwrap(), 0);

    // 验证健康检查始终返回 true
    // 因为 NoneMemory 不依赖任何外部资源，总是处于健康状态
    assert!(memory.health_check().await);
}
