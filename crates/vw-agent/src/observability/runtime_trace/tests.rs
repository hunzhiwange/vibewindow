//! 运行时追踪模块的单元测试
//!
//! 本模块包含对 `runtime_trace` 模块功能的测试用例，主要验证：
//! - 配置解析与路径解析逻辑
//! - 存储模式（None/Rolling/Full）的解析
//! - 滚动模式下的事件保留策略
//! - 事件 ID 查找功能
//!
//! # 测试覆盖
//!
//! | 功能 | 测试用例 |
//! |------|----------|
//! | 路径解析 | `resolve_trace_path_relative_joins_workspace` |
//! | 存储模式解析 | `storage_mode_parses_known_values` |
//! | 滚动保留 | `rolling_mode_keeps_latest_entries` |
//! | ID 查找 | `find_event_by_id_returns_match` |

use super::*;

/// 创建用于测试的观测配置实例
///
/// 返回一个预设了测试默认值的 `ObservabilityConfig`：
/// - `backend`: "none"（禁用外部后端）
/// - `runtime_trace_mode`: "rolling"（滚动模式）
/// - `runtime_trace_max_entries`: 3（最大保留 3 条记录）
fn test_observability_config() -> ObservabilityConfig {
    ObservabilityConfig {
        backend: "none".to_string(),
        otel_endpoint: None,
        otel_service_name: None,
        runtime_trace_mode: "rolling".to_string(),
        runtime_trace_path: "state/runtime-trace.jsonl".to_string(),
        runtime_trace_max_entries: 3,
    }
}

/// 测试：相对路径应与工作区目录拼接
///
/// # 验证内容
///
/// 当配置中的 `runtime_trace_path` 为相对路径时，
/// `resolve_trace_path` 函数应将其与工作区目录拼接，
/// 生成完整的绝对路径。
///
/// # 预期结果
///
/// 输入相对路径 "state/runtime-trace.jsonl" 时，
/// 输出应为 `<workspace>/state/runtime-trace.jsonl`。
#[test]
fn resolve_trace_path_relative_joins_workspace() {
    let cfg = test_observability_config();
    let workspace = tempfile::tempdir().unwrap();
    let path = resolve_trace_path(&cfg, workspace.path());
    assert_eq!(path, workspace.path().join("state/runtime-trace.jsonl"));
}

/// 测试：存储模式应正确解析已知值
///
/// # 验证内容
///
/// `storage_mode_from_config` 函数应将配置中的字符串值
/// 正确转换为 `RuntimeTraceStorageMode` 枚举：
///
/// | 配置值 | 枚举值 |
/// |--------|--------|
/// | "none" | `None` |
/// | "rolling" | `Rolling` |
/// | "full" | `Full` |
#[test]
fn storage_mode_parses_known_values() {
    let mut cfg = test_observability_config();

    // 测试 "none" 模式
    cfg.runtime_trace_mode = "none".into();
    assert_eq!(storage_mode_from_config(&cfg), RuntimeTraceStorageMode::None);

    // 测试 "rolling" 模式
    cfg.runtime_trace_mode = "rolling".into();
    assert_eq!(storage_mode_from_config(&cfg), RuntimeTraceStorageMode::Rolling);

    // 测试 "full" 模式
    cfg.runtime_trace_mode = "full".into();
    assert_eq!(storage_mode_from_config(&cfg), RuntimeTraceStorageMode::Full);
}

/// 测试：滚动模式应仅保留最新的事件条目
///
/// # 验证内容
///
/// 在 `Rolling` 存储模式下，当日志条目数超过 `max_entries` 限制时，
/// 系统应自动删除最旧的条目，仅保留最新的 N 条记录。
///
/// # 测试场景
///
/// 1. 创建最大容量为 2 的滚动日志器
/// 2. 依次写入 5 条事件（id-0 到 id-4）
/// 3. 读取日志文件内容
///
/// # 预期结果
///
/// 仅保留最新的 2 条记录：
/// - `event-4`（最新）
/// - `event-3`（次新）
///
/// 较旧的 `event-0`、`event-1`、`event-2` 应被自动清理。
#[test]
fn rolling_mode_keeps_latest_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("trace.jsonl");
    // 创建容量为 2 的滚动模式日志器
    let logger = RuntimeTraceLogger::new(RuntimeTraceStorageMode::Rolling, 2, path.clone());

    // 写入 5 条事件，超过容量限制
    for i in 0..5 {
        let event = RuntimeTraceEvent {
            id: format!("id-{i}"),
            timestamp: Utc::now().to_rfc3339(),
            event_type: "test".into(),
            channel: None,
            provider: None,
            model: None,
            turn_id: None,
            success: None,
            message: Some(format!("event-{i}")),
            payload: serde_json::json!({ "i": i }),
        };
        logger.append(&event).unwrap();
    }

    // 读取日志，验证仅保留最新的 2 条
    let events = load_events(&path, 10, None, None).unwrap();
    assert_eq!(events.len(), 2);
    // 验证顺序：最新的在前
    assert_eq!(events[0].message.as_deref(), Some("event-4"));
    assert_eq!(events[1].message.as_deref(), Some("event-3"));
}

/// 测试：按 ID 查找事件应返回匹配结果
///
/// # 验证内容
///
/// `find_event_by_id` 函数应能根据事件 ID 在日志文件中
/// 精确查找并返回匹配的事件记录。
///
/// # 测试场景
///
/// 1. 创建 Full 模式的日志器（保留所有记录）
/// 2. 写入一条具有特定 ID 的事件
/// 3. 使用该 ID 进行查找
///
/// # 预期结果
///
/// 查找结果应为 `Some(event)`，且事件的 ID 与目标 ID 一致。
#[test]
fn find_event_by_id_returns_match() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("trace.jsonl");
    // 创建 Full 模式日志器，保留所有记录
    let logger = RuntimeTraceLogger::new(RuntimeTraceStorageMode::Full, 100, path.clone());

    // 创建并写入目标事件
    let target_id = "target-event";
    let event = RuntimeTraceEvent {
        id: target_id.into(),
        timestamp: Utc::now().to_rfc3339(),
        event_type: "tool_call_result".into(),
        channel: Some("telegram".into()),
        provider: Some("openrouter".into()),
        model: Some("x".into()),
        turn_id: Some("turn-1".into()),
        success: Some(false),
        message: Some("boom".into()),
        payload: serde_json::json!({ "error": "boom" }),
    };
    logger.append(&event).unwrap();

    // 按 ID 查找并验证
    let found = find_event_by_id(&path, target_id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, target_id);
}
