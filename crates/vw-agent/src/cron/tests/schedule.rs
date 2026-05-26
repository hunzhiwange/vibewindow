//! 调度功能单元测试模块
//!
//! 本模块提供 cron 调度系统的集成测试，覆盖以下功能：
//! - 定时调度表达式解析与执行
//! - 调度时间计算与验证
//! - 时区处理与转换
//!
//! 测试范围包括：
//! - `Every` 模式：固定间隔循环调度
//! - `At` 模式：指定时间点一次性调度
//! - `Cron` 模式：cron 表达式调度（支持时区）

use crate::app::agent::cron::Schedule;
use crate::app::agent::cron::schedule::{
    next_run_for_schedule, normalize_expression, validate_schedule,
};
use chrono::{TimeZone, Utc};

/// 测试 `next_run_for_schedule` 函数对 `Every` 和 `At` 调度模式的支持
///
/// # 测试场景
/// - **Every 模式**：验证固定间隔调度能正确计算下一次执行时间
/// - **At 模式**：验证指定时间点调度能准确返回预设的执行时间
///
/// # 验证点
/// 1. Every 模式的下次执行时间必须晚于当前时间
/// 2. At 模式的返回时间必须精确等于设定的目标时间
#[test]
fn next_run_for_schedule_supports_every_and_at() {
    // 获取当前时间作为基准点
    let now = Utc::now();

    // 测试 Every 模式：每隔 60 秒执行一次
    let every = Schedule::Every { every_ms: 60_000 };
    let next = next_run_for_schedule(&every, now).unwrap();
    // 断言：下次执行时间必须在当前时间之后
    assert!(next > now);

    // 测试 At 模式：设定 10 分钟后执行
    let at = now + chrono::Duration::minutes(10);
    let at_schedule = Schedule::At { at };
    let next_at = next_run_for_schedule(&at_schedule, now).unwrap();
    // 断言：返回的执行时间必须精确匹配设定的时间点
    assert_eq!(next_at, at);
}

/// 测试 `next_run_for_schedule` 函数对时区的支持
///
/// # 测试场景
/// 验证 cron 表达式在指定时区下能正确计算 UTC 时间
///
/// # 测试细节
/// - 使用洛杉矶时区（America/Los_Angeles）
/// - cron 表达式：`0 9 * * *`（每天 9:00 AM）
/// - 基准时间：2026-02-16 00:00:00 UTC
///
/// # 预期结果
/// 洛杉矶时间 9:00 AM 对应 UTC 时间 17:00（洛杉矶 UTC-8）
/// 因此下次执行时间应为 2026-02-16 17:00:00 UTC
#[test]
fn next_run_for_schedule_supports_timezone() {
    // 设定基准时间：2026年2月16日 00:00:00 UTC
    let from = Utc.with_ymd_and_hms(2026, 2, 16, 0, 0, 0).unwrap();

    // 创建带时区的 cron 调度：每天洛杉矶时间 9:00 AM 执行
    let schedule =
        Schedule::Cron { expr: "0 9 * * *".into(), tz: Some("America/Los_Angeles".into()) };

    // 计算下次执行时间
    let next = next_run_for_schedule(&schedule, from).unwrap();

    // 断言：洛杉矶 9:00 AM = UTC 17:00（时差 8 小时）
    assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 16, 17, 0, 0).unwrap());
}
