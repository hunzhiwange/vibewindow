//! 进制转换工具消息处理模块
//!
//! 本模块提供进制转换器的用户交互消息定义与状态更新逻辑。
//! 支持在 2-36 进制之间进行数值转换，包括输入验证、实时转换、
//! 进制交换、结果复制等功能。

use crate::app::{App, Message};
use iced::Task;

/// 进制转换工具的消息类型枚举
///
/// 定义用户在进制转换界面中可能触发的所有操作消息。
#[derive(Debug, Clone)]
pub enum BaseToolMessage {
    /// 选择源进制（从哪个进制转换）
    ///
    /// # 参数
    /// * `u32` - 用户选择的进制值（2-36）
    SelectFrom(u32),

    /// 选择目标进制（转换到哪个进制）
    ///
    /// # 参数
    /// * `u32` - 用户选择的进制值（2-36）
    SelectTo(u32),

    /// 输入内容变更
    ///
    /// # 参数
    /// * `String` - 用户输入的新文本
    InputChanged(String),

    /// 交换源进制与目标进制
    ///
    /// 同时交换输入与输出内容，实现双向转换
    Swap,

    /// 复制输出结果到剪贴板
    CopyOutput,

    /// 清除通知消息
    ///
    /// 在复制操作后自动触发，用于清除"已复制"提示
    ClearNotification,
}

/// 处理进制转换工具的消息更新
///
/// 根据不同的消息类型更新应用状态，执行进制转换逻辑。
/// 所有转换操作都会实时更新输出结果，错误信息会显示在通知区域。
///
/// # 参数
/// * `app` - 应用状态的可变引用
/// * `message` - 待处理的消息
///
/// # 返回值
/// 返回可能需要执行的 Iced 任务（如剪贴板操作、定时任务等）
pub fn update(app: &mut App, message: BaseToolMessage) -> Task<Message> {
    match message {
        // 选择源进制：更新进制值、净化输入、重新计算输出
        BaseToolMessage::SelectFrom(b) => {
            app.base_from = sanitize_base(b);
            app.base_input = sanitize_input(&app.base_input, app.base_from);
            refresh_output(app);
            Task::none()
        }
        // 选择目标进制：更新进制值、重新计算输出
        BaseToolMessage::SelectTo(b) => {
            app.base_to = sanitize_base(b);
            refresh_output(app);
            Task::none()
        }
        // 输入变更：净化输入、重新计算输出
        BaseToolMessage::InputChanged(s) => {
            app.base_input = sanitize_input(&s, app.base_from);
            refresh_output(app);
            Task::none()
        }
        // 交换源/目标进制及输入/输出内容
        BaseToolMessage::Swap => {
            std::mem::swap(&mut app.base_from, &mut app.base_to);
            std::mem::swap(&mut app.base_input, &mut app.base_output);
            // 交换后需重新净化输入（确保符合新进制要求）
            app.base_input = sanitize_input(&app.base_input, app.base_from);
            refresh_output(app);
            Task::none()
        }
        // 复制输出：写入剪贴板并显示通知，2秒后自动清除
        BaseToolMessage::CopyOutput => {
            let text = app.base_output.clone();
            app.base_notification = Some("已复制结果".to_string());
            Task::batch(vec![
                iced::clipboard::write(text),
                crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::BaseTool(BaseToolMessage::ClearNotification),
                ),
            ])
        }
        // 清除通知消息
        BaseToolMessage::ClearNotification => {
            if app.base_notification.as_deref() == Some("已复制结果") {
                app.base_notification = None;
            }
            Task::none()
        }
    }
}

fn refresh_output(app: &mut App) {
    match convert(&app.base_input, app.base_from, app.base_to) {
        Ok(output) => {
            app.base_output = output;
            app.base_notification = None;
        }
        Err(error) => {
            app.base_output.clear();
            app.base_notification = Some(error);
        }
    }
}

/// 净化并验证进制值
///
/// 确保进制值在有效范围 [2, 36] 内，否则回退到十进制。
///
/// # 参数
/// * `b` - 原始进制值
///
/// # 返回值
/// 返回有效的进制值（2-36），无效时返回 10
fn sanitize_base(b: u32) -> u32 {
    if (2..=36).contains(&b) { b } else { 10 }
}

/// 净化输入字符串，过滤掉不符合当前进制的字符
///
/// 保留前导负号，移除所有在指定进制下无效的数字/字母字符。
///
/// # 参数
/// * `raw` - 原始输入字符串
/// * `base` - 当前进制（用于判断字符有效性）
///
/// # 返回值
/// 返回净化后的字符串，仅包含符合当前进制的有效字符
fn sanitize_input(raw: &str, base: u32) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut first = true;
    for ch in raw.trim().chars() {
        // 处理前导负号
        if first && ch == '-' {
            out.push(ch);
            first = false;
            continue;
        }
        first = false;
        // 仅保留小于当前进制的有效数字字符
        let ok = if let Some(v) = digit_value(ch) { v < base } else { false };
        if ok {
            out.push(ch);
        }
    }
    out
}

/// 执行进制转换
///
/// 将字符串形式的数值从源进制转换到目标进制。
/// 支持正负数，使用 u128 作为中间表示，最大支持 39 位十进制数。
///
/// # 参数
/// * `s` - 输入字符串（已净化）
/// * `from` - 源进制（2-36）
/// * `to` - 目标进制（2-36）
///
/// # 返回值
/// * `Ok(String)` - 转换成功，返回目标进制下的字符串表示
/// * `Err(String)` - 转换失败，返回错误信息
///
/// # 错误情况
/// - 进制值不在 2-36 范围内
/// - 数值超出 u128 表示范围
/// - 包含非法字符
fn convert(s: &str, from: u32, to: u32) -> Result<String, String> {
    // 空字符串直接返回空
    if s.trim().is_empty() {
        return Ok(String::new());
    }
    // 验证进制范围
    if !(2..=36).contains(&from) || !(2..=36).contains(&to) {
        return Err("仅支持 2-36 进制".to_string());
    }
    // 检测并处理负数
    let negative = s.trim_start().starts_with('-');
    let s = s.trim().trim_start_matches('+').trim_start_matches('-');
    // 解析并转换
    match parse_to_u128(s, from) {
        Some(n) => {
            let mut out = if to == 10 { n.to_string() } else { to_base(n, to) };
            // 添加负号
            if negative && !out.is_empty() {
                out.insert(0, '-');
            }
            Ok(out)
        }
        None => Err("非法数字或超出范围".to_string()),
    }
}

/// 将字符串解析为 u128 数值
///
/// 按指定进制解析字符串，使用溢出检查避免数值超出范围。
///
/// # 参数
/// * `s` - 待解析的字符串（不包含符号）
/// * `radix` - 进制（2-36）
///
/// # 返回值
/// * `Some(u128)` - 解析成功
/// * `None` - 解析失败（非法字符或溢出）
fn parse_to_u128(s: &str, radix: u32) -> Option<u128> {
    let mut val: u128 = 0;
    for ch in s.chars() {
        let d = digit_value(ch)?;
        // 字符数值必须小于当前进制
        if d >= radix {
            return None;
        }
        // 使用 checked 运算检测溢出
        val = val.checked_mul(radix as u128)?.checked_add(d as u128)?;
    }
    Some(val)
}

/// 获取字符对应的数值
///
/// 将数字字符或字母字符转换为对应的数值：
/// - '0'-'9' -> 0-9
/// - 'a'-'z' -> 10-35
/// - 'A'-'Z' -> 10-35
///
/// # 参数
/// * `ch` - 待转换的字符
///
/// # 返回值
/// * `Some(u32)` - 有效数字/字母字符
/// * `None` - 无效字符
fn digit_value(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some(ch as u32 - '0' as u32),
        'a'..='z' => Some(ch as u32 - 'a' as u32 + 10),
        'A'..='Z' => Some(ch as u32 - 'A' as u32 + 10),
        _ => None,
    }
}

/// 将 u128 数值转换为指定进制的字符串表示
///
/// 使用除余法将数值转换为任意进制（2-36）的字符串。
///
/// # 参数
/// * `n` - 待转换的数值
/// * `radix` - 目标进制（2-36）
///
/// # 返回值
/// 返回目标进制下的字符串表示（不含符号）
fn to_base(mut n: u128, radix: u32) -> String {
    // 特殊情况：零
    if n == 0 {
        return "0".to_string();
    }
    let mut digits: Vec<char> = Vec::new();
    // 重复除以进制，收集余数
    while n > 0 {
        let rem = (n % radix as u128) as u32;
        digits.push(digit_char(rem));
        n /= radix as u128;
    }
    // 余数序列需反转得到正确顺序
    digits.iter().rev().collect()
}

/// 将数值转换为对应的数字字符
///
/// 将 0-35 的数值转换为对应的字符：
/// - 0-9 -> '0'-'9'
/// - 10-35 -> 'A'-'Z'
///
/// # 参数
/// * `d` - 数值（0-35）
///
/// # 返回值
/// 返回对应的数字字符，超出范围返回 '?'
fn digit_char(d: u32) -> char {
    match d {
        0..=9 => char::from_digit(d, 10).unwrap(),
        10..=35 => (b'A' + (d - 10) as u8) as char,
        _ => '?',
    }
}
#[cfg(test)]
#[path = "base_tool_tests.rs"]
mod base_tool_tests;
