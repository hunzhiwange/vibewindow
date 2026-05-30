use super::*;

/// 测试 `clear_sender_session_id_for_scope` 只移除目标范围键
///
/// 验证：
/// - 清除操作只影响指定的项目范围
/// - 其他项目范围的会话映射保持不变
#[test]
fn clear_sender_session_id_for_scope_removes_only_target_scope_key() {
    let project_scope_a = "scope-a";
    let project_scope_b = "scope-b";
    let sender_key = "telegram_user-1";
    let key_a = format!("{}::{}", project_scope_a, sender_key);
    let key_b = format!("{}::{}", project_scope_b, sender_key);

    // 准备测试数据
    let mut store = sender_session_store().lock().unwrap_or_else(|e| e.into_inner());
    store.insert(key_a.clone(), "ses_a".to_string());
    store.insert(key_b.clone(), "ses_b".to_string());
    drop(store);

    // 执行清除操作
    clear_sender_session_id_for_scope(project_scope_a, sender_key);

    // 验证结果
    let store = sender_session_store().lock().unwrap_or_else(|e| e.into_inner());
    assert!(!store.contains_key(&key_a));
    assert!(store.contains_key(&key_b));
}
