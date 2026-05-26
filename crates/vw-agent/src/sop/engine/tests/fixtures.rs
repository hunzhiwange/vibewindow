use super::*;

/// 创建手动触发事件
///
/// 用于测试手动触发的 SOP 执行流程。
/// 手动事件不包含主题和载荷信息。
pub(super) fn manual_event() -> SopEvent {
    SopEvent {
        source: SopTriggerSource::Manual,
        topic: None,
        payload: None,
        timestamp: now_iso8601(),
    }
}

/// 创建 MQTT 触发事件
///
/// # 参数
/// - `topic`: MQTT 主题字符串
/// - `payload`: MQTT 消息载荷
///
/// 用于测试基于 MQTT 消息的 SOP 触发逻辑。
pub(super) fn mqtt_event(topic: &str, payload: &str) -> SopEvent {
    SopEvent {
        source: SopTriggerSource::Mqtt,
        topic: Some(topic.into()),
        payload: Some(payload.into()),
        timestamp: now_iso8601(),
    }
}

/// 构建测试用 SOP 对象
///
/// # 参数
/// - `name`: SOP 名称标识
/// - `mode`: 执行模式（自动/监督/逐步/基于优先级）
/// - `priority`: 优先级级别
///
/// # 返回
/// 返回包含两个测试步骤的 SOP 配置，用于验证引擎行为。
pub(super) fn test_sop(name: &str, mode: SopExecutionMode, priority: SopPriority) -> Sop {
    Sop {
        name: name.into(),
        description: format!("Test SOP: {name}"),
        version: "1.0.0".into(),
        priority,
        execution_mode: mode,
        triggers: vec![SopTrigger::Manual],
        steps: vec![
            SopStep {
                number: 1,
                title: "Step one".into(),
                body: "Do step one".into(),
                suggested_tools: vec!["shell".into()],
                requires_confirmation: false,
            },
            SopStep {
                number: 2,
                title: "Step two".into(),
                body: "Do step two".into(),
                suggested_tools: vec![],
                requires_confirmation: false,
            },
        ],
        cooldown_secs: 0,
        max_concurrent: 1,
        location: None,
    }
}

/// 创建预加载 SOP 的引擎实例
///
/// # 参数
/// - `sops`: 要注册到引擎的 SOP 列表
///
/// # 返回
/// 返回已配置指定 SOP 的引擎实例。
pub(super) fn engine_with_sops(sops: Vec<Sop>) -> SopEngine {
    let mut engine = SopEngine::new(SopConfig::default());
    engine.sops = sops;
    engine
}

/// 从任意 SopRunAction 变体中提取运行 ID
///
/// # 参数
/// - `action`: SOP 运行动作枚举
///
/// # 返回
/// 返回运行 ID 字符串的引用，无论动作类型为何。
pub(super) fn extract_run_id(action: &SopRunAction) -> &str {
    match action {
        SopRunAction::ExecuteStep { run_id, .. }
        | SopRunAction::WaitApproval { run_id, .. }
        | SopRunAction::Completed { run_id, .. }
        | SopRunAction::Failed { run_id, .. } => run_id,
    }
}