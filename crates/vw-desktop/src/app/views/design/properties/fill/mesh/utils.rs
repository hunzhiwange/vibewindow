//! 网格填充工具模块
//!
//! 本模块提供了用于网格填充图案生成的实用工具函数，包括随机数种子计算和镜像配置解析。
//!
//! # 主要功能
//!
//! - **随机数生成**：使用 Xorshift 算法生成伪随机数种子
//! - **镜像配置**：解析和生成网格填充的镜像标志（X轴、Y轴或双向镜像）

/// 使用 Xorshift 算法生成下一个随机数种子
///
/// 该函数实现了 Xorshift64 随机数生成算法，通过位异或和移位操作来生成伪随机数。
/// 这是一种快速且高质量的伪随机数生成器，适用于图形学和游戏开发场景。
///
/// # 参数
///
/// - `x`: 当前的种子值（64位无符号整数）
///
/// # 返回值
///
/// 返回基于输入种子生成的新的64位无符号整数种子
///
/// # 算法说明
///
/// 使用 Xorshift64 算法的标准参数：
/// 1. `x ^= x << 13` - 左移13位后异或
/// 2. `x ^= x >> 7`  - 右移7位后异或
/// 3. `x ^= x << 17` - 左移17位后异或
///
/// 这些特定的移位值经过数学验证，能产生良好的随机性和周期性。
///
/// # 示例
///
/// ```ignore
/// let seed: u64 = 12345;
/// let next = next_seed(seed);
/// // next 是一个基于 seed 生成的新随机数
/// ```
pub(super) fn next_seed(mut x: u64) -> u64 {
    // 左移13位后与原值异或，引入高位的不确定性
    x ^= x << 13;
    // 右移7位后与原值异或，混合高位和低位
    x ^= x >> 7;
    // 左移17位后与原值异或，完成一轮混合
    x ^= x << 17;
    x
}

/// 解析镜像标志字符串，返回X轴和Y轴的镜像状态
///
/// 该函数将字符串形式的镜像配置解析为布尔标志对。支持的格式包括：
/// - `"none"` 或空字符串：不镜像
/// - `"x"`：仅X轴镜像
/// - `"y"`：仅Y轴镜像
/// - `"xy"`：双向镜像
///
/// # 参数
///
/// - `m`: 可选的镜像配置字符串引用，格式不区分大小写
///
/// # 返回值
///
/// 返回一个元组 `(mirror_x, mirror_y)`：
/// - 第一个元素表示是否在X轴镜像
/// - 第二个元素表示是否在Y轴镜像
///
/// # 解析规则
///
/// - 输入为 `None` 时，默认不镜像
/// - 字符串会被转换为小写并去除首尾空格
/// - 检查字符串中是否包含 'x' 和 'y' 字符来确定镜像方向
/// - 空字符串或 "none" 被视为无镜像
///
/// # 示例
///
/// ```ignore
/// // 不镜像
/// assert_eq!(mirroring_flags(None), (false, false));
/// assert_eq!(mirroring_flags(Some("none")), (false, false));
///
/// // X轴镜像
/// assert_eq!(mirroring_flags(Some("x")), (true, false));
///
/// // Y轴镜像
/// assert_eq!(mirroring_flags(Some("y")), (false, true));
///
/// // 双向镜像
/// assert_eq!(mirroring_flags(Some("xy")), (true, true));
/// assert_eq!(mirroring_flags(Some("XY")), (true, true)); // 不区分大小写
/// ```
pub(super) fn mirroring_flags(m: Option<&str>) -> (bool, bool) {
    // 获取字符串或空字符串，去除首尾空格并转换为小写
    let s = m.unwrap_or("").trim().to_ascii_lowercase();

    // 空字符串或 "none" 表示不镜像
    if s.is_empty() || s == "none" {
        return (false, false);
    }

    // 检查是否包含 'x' 和 'y' 来确定镜像方向
    let has_x = s.contains('x');
    let has_y = s.contains('y');

    (has_x, has_y)
}

/// 将布尔镜像标志转换为镜像配置字符串
///
/// 该函数执行 `mirroring_flags` 的逆操作，将布尔标志对转换回字符串表示。
///
/// # 参数
///
/// - `x`: 是否在X轴镜像
/// - `y`: 是否在Y轴镜像
///
/// # 返回值
///
/// 返回一个 `Option<String>`：
/// - `None` - 不镜像（两个参数都为 false）
/// - `Some("x")` - 仅X轴镜像
/// - `Some("y")` - 仅Y轴镜像
/// - `Some("xy")` - 双向镜像
///
/// # 示例
///
/// ```ignore
/// // 不镜像
/// assert_eq!(mirroring_value(false, false), None);
///
/// // X轴镜像
/// assert_eq!(mirroring_value(true, false), Some("x".to_string()));
///
/// // Y轴镜像
/// assert_eq!(mirroring_value(false, true), Some("y".to_string()));
///
/// // 双向镜像
/// assert_eq!(mirroring_value(true, true), Some("xy".to_string()));
/// ```
#[allow(dead_code)]
pub(super) fn mirroring_value(x: bool, y: bool) -> Option<String> {
    match (x, y) {
        // 两个方向都不镜像，返回 None
        (false, false) => None,
        // 仅X轴镜像
        (true, false) => Some("x".to_string()),
        // 仅Y轴镜像
        (false, true) => Some("y".to_string()),
        // X轴和Y轴都镜像
        (true, true) => Some("xy".to_string()),
    }
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod utils_tests;
