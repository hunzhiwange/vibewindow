//! 定时任务调度模块的单元测试
//!
//! 本模块包含对 cron 表达式解析、时区处理以及下次运行时间计算等功能的测试用例。
//! 主要验证：
//! - 不同时区下的 cron 表达式解析
//! - 下次运行时间的正确计算
//! - 时区转换的准确性

use super::*;

/// 定时任务调度测试模块
///
/// 包含对 Schedule 相关功能的单元测试，验证调度器在各种场景下的行为。
#[allow(dead_code)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    /// 测试 `next_run_for_schedule` 函数对时区的支持
    ///
    /// 验证点：
    /// - cron 表达式能够正确解析指定的时区
    /// - 下次运行时间能够正确转换为目标时区的时间
    /// - UTC 时间与指定时区时间的转换准确无误
    ///
    /// 测试场景：
    /// - 从 UTC 时间 `2026-02-16 00:00:00` 开始
    /// - cron 表达式 `0 9 * * *`（每天上午 9 点执行）
    /// - 时区设置为 `America/Los_Angeles`（洛杉矶时区，UTC-8）
    /// - 期望下次运行时间为 UTC `2026-02-16 17:00:00`
    ///
    /// 时区转换说明：
    /// 洛杉矶时间 09:00 对应 UTC 时间 17:00（洛杉矶为 UTC-8）
    #[test]
    fn next_run_for_schedule_supports_timezone() {
        // 设置起始时间点：2026年2月16日 00:00:00 UTC
        let from = Utc.with_ymd_and_hms(2026, 2, 16, 0, 0, 0).unwrap();

        // 创建定时计划：每天洛杉矶时间 09:00 执行
        let schedule =
            Schedule::Cron { expr: "0 9 * * *".into(), tz: Some("America/Los_Angeles".into()) };

        // 计算下次运行时间
        let next = next_run_for_schedule(&schedule, from).unwrap();

        // 验证：洛杉矶时间 09:00 = UTC 17:00
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 16, 17, 0, 0).unwrap());
    }
}
