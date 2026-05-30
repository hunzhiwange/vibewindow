use crate::app::{App, Message};
use iced::Task;
use std::fmt::Display;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsUnit {
    Seconds,
    Milliseconds,
}

impl TsUnit {
    pub fn all() -> [TsUnit; 2] {
        [TsUnit::Seconds, TsUnit::Milliseconds]
    }
}

impl Display for TsUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsUnit::Seconds => write!(f, "秒(s)"),
            TsUnit::Milliseconds => write!(f, "毫秒(ms)"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimestampToolMessage {
    ToggleAuto(bool),
    RefreshNow,
    InputTsChanged(String),
    UnitSelected(TsUnit),
    ConvertFromTs,
    CopyTimeOutput,
    InputTimeChanged(String),
    ConvertFromTime,
    CopyTsOutput,
    ClearNotification,
}

pub fn update(app: &mut App, message: TimestampToolMessage) -> Task<Message> {
    match message {
        TimestampToolMessage::ToggleAuto(b) => {
            app.ts_auto = b;
            if b {
                return crate::app::message::after(
                    std::time::Duration::from_secs(1),
                    Message::TimestampTool(TimestampToolMessage::RefreshNow),
                );
            }
            Task::none()
        }
        TimestampToolMessage::RefreshNow => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            let secs = now.as_secs() as i64;
            let ms = now.as_millis();
            app.ts_now_unix_sec = secs.to_string();
            app.ts_now_unix_ms = ms.to_string();
            app.ts_now_utc_str = format_utc(secs);
            if app.ts_auto {
                crate::app::message::after(
                    std::time::Duration::from_secs(1),
                    Message::TimestampTool(TimestampToolMessage::RefreshNow),
                )
            } else {
                Task::none()
            }
        }
        TimestampToolMessage::InputTsChanged(s) => {
            app.ts_input_ts = s;
            Task::none()
        }
        TimestampToolMessage::UnitSelected(u) => {
            app.ts_unit = u;
            Task::none()
        }
        TimestampToolMessage::ConvertFromTs => {
            let raw = app.ts_input_ts.trim();
            if raw.is_empty() {
                app.ts_time_output.clear();
                return Task::none();
            }
            let val = raw.parse::<i128>().unwrap_or(0);
            let secs = match app.ts_unit {
                TsUnit::Seconds => val as i64,
                TsUnit::Milliseconds => (val / 1000) as i64,
            };
            app.ts_time_output = format_utc(secs);
            app.ts_notification = Some("转换成功".to_string());
            crate::app::message::after(
                std::time::Duration::from_secs(2),
                Message::TimestampTool(TimestampToolMessage::ClearNotification),
            )
        }
        TimestampToolMessage::CopyTimeOutput => {
            let text = app.ts_time_output.clone();
            app.ts_notification = Some("已复制".to_string());
            Task::batch(vec![
                iced::clipboard::write(text),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::TimestampTool(TimestampToolMessage::ClearNotification),
                ),
            ])
        }
        TimestampToolMessage::InputTimeChanged(s) => {
            app.ts_time_input = s;
            Task::none()
        }
        TimestampToolMessage::ConvertFromTime => {
            if let Some((secs, ms)) = parse_utc_time_to_unix(&app.ts_time_input) {
                app.ts_ts_output_sec = secs.to_string();
                app.ts_ts_output_ms = ms.to_string();
                app.ts_notification = Some("转换成功".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::TimestampTool(TimestampToolMessage::ClearNotification),
                )
            } else {
                app.ts_notification = Some("格式错误".to_string());
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::TimestampTool(TimestampToolMessage::ClearNotification),
                )
            }
        }
        TimestampToolMessage::CopyTsOutput => {
            let text = format!("{} / {}", app.ts_ts_output_sec, app.ts_ts_output_ms);
            app.ts_notification = Some("已复制".to_string());
            Task::batch(vec![
                iced::clipboard::write(text),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::TimestampTool(TimestampToolMessage::ClearNotification),
                ),
            ])
        }
        TimestampToolMessage::ClearNotification => {
            app.ts_notification = None;
            Task::none()
        }
    }
}

pub fn format_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (sod / 3600) as i32;
    let minute = ((sod % 3600) / 60) as i32;
    let second = (sod % 60) as i32;
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, hour, minute, second)
}

fn civil_from_days(mut z: i64) -> (i32, i32, i32) {
    z += 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100 + yoe / 400);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as i32;
    let m = (mp + if mp < 10 { 3 } else { -9 }) as i32;
    y += if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

fn days_from_civil(mut y: i32, m: i32, d: i32) -> i64 {
    y -= if m <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + yoe / 400 + doy;
    (era as i64) * 146097 + (doe as i64) - 719468
}

fn parse_utc_time_to_unix(s: &str) -> Option<(i64, i128)> {
    // Accept formats like "YYYY-MM-DD HH:MM:SS" or with ".ms"
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (date_part, time_part) = if let Some(space_idx) = s.find(' ') {
        (&s[..space_idx], &s[space_idx + 1..])
    } else {
        (s, "")
    };
    let mut date_iter = date_part.split('-');
    let y = date_iter.next()?.parse::<i32>().ok()?;
    let m = date_iter.next()?.parse::<i32>().ok()?;
    let d = date_iter.next()?.parse::<i32>().ok()?;

    let (h, mi, se, ms) = if time_part.is_empty() {
        (0_i32, 0_i32, 0_i32, 0_i32)
    } else {
        let mut t_and_ms = time_part.split('.');
        let hms = t_and_ms.next().unwrap_or("");
        let ms_str = t_and_ms.next();
        let mut hms_iter = hms.split(':');
        let h = hms_iter.next().unwrap_or("0").parse::<i32>().ok()?;
        let mi = hms_iter.next().unwrap_or("0").parse::<i32>().ok()?;
        let se_val = hms_iter.next().unwrap_or("0").parse::<i32>().ok()?;
        let ms = ms_str.unwrap_or("0").parse::<i32>().unwrap_or(0);
        (h, mi, se_val, ms)
    };

    let days = days_from_civil(y, m, d);
    let secs = days * 86_400 + (h as i64) * 3600 + (mi as i64) * 60 + (se as i64);
    let millis: i128 = (secs as i128) * 1000 + (ms as i128);
    Some((secs, millis))
}
#[cfg(test)]
#[path = "timestamp_tool_tests.rs"]
mod timestamp_tool_tests;
