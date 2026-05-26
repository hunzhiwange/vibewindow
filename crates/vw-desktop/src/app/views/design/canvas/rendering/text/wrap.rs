//! 文本换行模块
//!
//! 提供文本自动换行功能，根据指定的最大宽度和字体大小，
//! 将长文本分割成多个适合显示的行。

/// 将文本内容按照指定的最大宽度进行换行处理
///
/// 该函数会遍历文本中的每一行，并根据字符宽度和最大宽度限制，
/// 将过长的行自动分割成多行。对于非 ASCII 字符（如中文、日文等），
/// 使用全角宽度计算；对于 ASCII 字符，使用半角宽度计算。
///
/// # 参数
///
/// * `content` - 需要进行换行处理的原始文本内容
/// * `max_width` - 单行文本的最大宽度（像素值）
/// * `font_size` - 字体大小（像素值），用于计算字符宽度
///
/// # 返回值
///
/// 返回一个字符串向量，每个元素代表一行文本。如果输入参数无效
/// （`max_width <= 0` 或 `font_size <= 0`），返回空向量。
///
/// # 示例
///
/// ```ignore
/// let text = "这是一个很长的文本需要进行换行处理";
/// let lines = wrap_text_lines(text, 100.0, 16.0);
/// // lines 包含根据宽度限制分割后的多行文本
/// ```
///
/// # 宽度计算规则
///
/// - ASCII 字符（码点 <= 127）：宽度 = font_size * 0.6
/// - 非 ASCII 字符（码点 > 127）：宽度 = font_size
#[allow(dead_code)]
pub(super) fn wrap_text_lines(content: &str, max_width: f32, font_size: f32) -> Vec<String> {
    // 参数有效性检查：最大宽度和字体大小必须大于 0
    if max_width <= 0.0 || font_size <= 0.0 {
        return Vec::new();
    }

    // 存储换行后的所有文本行
    let mut out = Vec::new();

    // 遍历原始文本中的每一行
    for line in content.lines() {
        // 如果是空行，直接添加空字符串并继续处理下一行
        if line.is_empty() {
            out.push(String::new());
            continue;
        }

        // 当前正在构建的文本行
        let mut current = String::new();
        // 当前行的累计宽度
        let mut current_width = 0.0;

        // 遍历当前行中的每个字符
        for ch in line.chars() {
            // 根据字符类型计算字符宽度
            // 非 ASCII 字符（如中文）使用全角宽度，ASCII 字符使用半角宽度
            let ch_width = if ch as u32 > 127 { font_size } else { font_size * 0.6 };

            // 如果添加当前字符会超过最大宽度，且当前行不为空，则换行
            if current_width + ch_width > max_width && !current.is_empty() {
                // 将当前行添加到输出中
                out.push(current);
                // 重置当前行和宽度计数
                current = String::new();
                current_width = 0.0;
            }

            // 将当前字符添加到当前行
            current.push(ch);
            // 更新当前行的累计宽度
            current_width += ch_width;
        }

        // 将最后一行（如果有内容）添加到输出中
        if !current.is_empty() {
            out.push(current);
        }
    }

    out
}

#[cfg(test)]
#[path = "wrap_tests.rs"]
mod wrap_tests;
