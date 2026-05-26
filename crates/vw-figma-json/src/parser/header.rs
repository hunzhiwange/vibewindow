use crate::error::{FigError, Result};
use crate::types::FileType;

/// 标准 Figma 文件的魔术头
const FIGMA_MAGIC: &[u8; 8] = b"fig-kiwi";

/// FigJam 文件的神奇标头
const FIGJAM_MAGIC: &[u8; 8] = b"fig-jam.";

/// ZIP 魔术签名(前两个字节)
const ZIP_MAGIC: &[u8; 2] = b"PK";

/// 根据魔术头检测文件类型
///
/// # 参数
/// * `bytes` - 要分析的原始文件字节
///
/// # 返回值
/// * `Ok(FileType)` - 检测到的文件类型(Figma 或 FigJam)
/// * `Err(FigError)` - 如果文件太小或标题无效
///
/// # 示例
/// ```
/// use fig2json::parser::detect_file_type;
///
/// let bytes = b"fig-kiwi\x00\x00\x00\x00...";
/// let file_type = detect_file_type(bytes).unwrap();
/// ```
pub fn detect_file_type(bytes: &[u8]) -> Result<FileType> {
    if bytes.len() < 8 {
        return Err(FigError::FileTooSmall { expected: 8, actual: bytes.len() });
    }

    let header = &bytes[0..8];

    // 检查 "fig-kiwi" (标准 Figma)
    if header == FIGMA_MAGIC {
        return Ok(FileType::Figma);
    }

    // 检查 "fig-jam." (FigJam)
    if header == FIGJAM_MAGIC {
        return Ok(FileType::FigJam);
    }

    // 无效标头
    Err(FigError::InvalidMagicHeader(header.to_vec()))
}

/// 检查文件是否是 ZIP 容器
///
/// 一些 .fig 文件是 ZIP 存档，其中包含 `canvas.fig` 文件。
/// 该函数检查 ZIP 魔术签名 "PK" (0x50 0x4B)。
///
/// # 参数
/// * `bytes` - 要分析的原始文件字节
///
/// # 返回值
/// * `true` - 如果文件以 ZIP 签名开头
/// * `false` - 否则
///
/// # 示例
/// ```
/// use fig2json::parser::is_zip_container;
///
/// let zip_bytes = b"PK\x03\x04...";
/// assert!(is_zip_container(zip_bytes));
///
/// let fig_bytes = b"fig-kiwi...";
/// assert!(!is_zip_container(fig_bytes));
/// ```
pub fn is_zip_container(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && &bytes[0..2] == ZIP_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_figma_header() {
        let bytes = b"fig-kiwi\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::Figma);
    }

    #[test]
    fn test_detect_figjam_header() {
        let bytes = b"fig-jam.\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::FigJam);
    }

    #[test]
    fn test_invalid_header() {
        let bytes = b"invalid!";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_too_small() {
        let bytes = b"fig";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::FileTooSmall { expected, actual }) => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_is_zip_container() {
        // 有效的 ZIP 签名
        let zip_bytes = b"PK\x03\x04";
        assert!(is_zip_container(zip_bytes));

        // 不是邮政编码
        let fig_bytes = b"fig-kiwi";
        assert!(!is_zip_container(fig_bytes));

        // 太小
        let small_bytes = b"P";
        assert!(!is_zip_container(small_bytes));
    }
}
