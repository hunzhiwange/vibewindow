//! 提供代理侧本地化与时间展示辅助函数。
//! 共享文本格式化由 vw_shared 负责，本模块只保留与当前时间语义相关的轻量封装。

pub use vw_shared::util::duration;
/// 重新导出共享实现，保持当前模块的公开访问路径稳定。
pub use vw_shared::util::number;
/// 重新导出共享实现，保持当前模块的公开访问路径稳定。
pub use vw_shared::util::pluralize;
/// 重新导出共享实现，保持当前模块的公开访问路径稳定。
pub use vw_shared::util::titlecase;
/// 重新导出共享实现，保持当前模块的公开访问路径稳定。
pub use vw_shared::util::truncate;
/// 重新导出共享实现，保持当前模块的公开访问路径稳定。
pub use vw_shared::util::truncate_middle;

use time::{OffsetDateTime, UtcOffset};

fn datetime_from_millis_utc(ms: i64) -> OffsetDateTime {
    let nanos = i128::from(ms) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(UtcOffset::UTC)
}

/// 执行 time_short 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub fn time_short(ms: i64) -> String {
    let dt = datetime_from_millis_utc(ms);
    let hour = dt.hour();
    let minute = dt.minute();
    format!("{:02}:{:02}", hour, minute)
}

/// 执行 datetime 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub fn datetime(ms: i64) -> String {
    let dt = datetime_from_millis_utc(ms);
    let t = time_short(ms);
    let d = format!("{:04}-{:02}-{:02}", dt.year(), u8::from(dt.month()), dt.day());
    format!("{} · {}", t, d)
}

/// 执行 today_time_or_datetime 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub fn today_time_or_datetime(ms: i64) -> String {
    let dt = datetime_from_millis_utc(ms);
    let now_ms = web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let now = datetime_from_millis_utc(i64::try_from(now_ms).unwrap_or(i64::MAX));
    let is_today = dt.year() == now.year() && dt.month() == now.month() && dt.day() == now.day();
    if is_today { time_short(ms) } else { datetime(ms) }
}
