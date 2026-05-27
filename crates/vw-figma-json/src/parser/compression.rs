use crate::error::{FigError, Result};
use flate2::read::DeflateDecoder;
use std::io::Read;

/// PNG魔法签名(前两个字节：137、80)
const PNG_MAGIC: &[u8; 2] = &[137, 80];

/// JPEG 魔术签名(前两个字节：255、216)
const JPEG_MAGIC: &[u8; 2] = &[255, 216];

/// 检查数据是否已压缩(PNG 或 JPEG 图像)
///
/// 图像已经被压缩，所以我们不应该尝试解压缩它们。
///
/// # 参数
/// * `bytes` - 要检查的数据
///
/// # 返回值
/// * `true` - 如果数据以 PNG 或 JPEG 魔术字节开头
/// * `false` - 否则
///
/// # 示例
/// ```
/// use fig2json::parser::compression::is_already_compressed;
///
/// // PNG 图像
/// let png_data = &[137, 80, 78, 71, 13, 10, 26, 10];
/// assert!(is_already_compressed(png_data));
///
/// // JPEG 图像
/// let jpeg_data = &[255, 216, 255, 224];
/// assert!(is_already_compressed(jpeg_data));
///
/// // 常规压缩数据
/// let other_data = &[120, 156, 1, 2, 3];
/// assert!(!is_already_compressed(other_data));
/// ```
pub fn is_already_compressed(bytes: &[u8]) -> bool {
    if bytes.len() < 2 {
        return false;
    }

    let magic = &bytes[0..2];

    // 检查 PNG：[137, 80]
    if magic == PNG_MAGIC {
        return true;
    }

    // 检查 JPEG：[255, 216]
    if magic == JPEG_MAGIC {
        return true;
    }

    false
}

/// 使用 DEFLATE 或 Zstandard 解压缩块数据
///
/// Figma 使用两种压缩格式：
/// - DEFLATE (zlib) - 更常见，用于较旧的文件
/// - Zstandard - 用于较新的文件
///
/// 该函数首先尝试 DEFLATE，如果 DEFLATE 失败，则返回到 Zstandard。
/// 如果数据已压缩 (PNG/JPEG)，则按原样返回数据。
///
/// # 参数
/// * `bytes` - 压缩块数据
///
/// # 返回值
/// * `Ok(Vec<u8>)` - 解压缩数据
/// * `Err(FigError)` - 如果两种解压方法都失败
///
/// # 示例
/// ```no_run
/// use fig2json::parser::compression::decompress_chunk;
///
/// let compressed_data = vec![120, 156, 75, 76, 28, 5, 0, 1, 153, 0, 206];
/// let decompressed = decompress_chunk(&compressed_data).unwrap();
/// ```
pub fn decompress_chunk(bytes: &[u8]) -> Result<Vec<u8>> {
    // 跳过已压缩图像的解压缩
    if is_already_compressed(bytes) {
        return Ok(bytes.to_vec());
    }

    // 首先尝试 DEFLATE (zlib) - 更常见
    match decompress_deflate(bytes) {
        Ok(data) => Ok(data),
        Err(_) => {
            // DEFLATE 失败，尝试 Zstandard
            match decompress_zstd(bytes) {
                Ok(data) => Ok(data),
                Err(e) => Err(FigError::ZipError(format!(
                    "Failed to decompress chunk (tried both DEFLATE and Zstandard): {}",
                    e
                ))),
            }
        }
    }
}

/// 使用 DEFLATE 解压缩数据(原始，没有 zlib 包装器)
fn decompress_deflate(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| FigError::ZipError(format!("DEFLATE decompression failed: {}", e)))?;
    Ok(decompressed)
}

/// 使用 Zstandard 解压缩数据
fn decompress_zstd(bytes: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(bytes)
        .map_err(|e| FigError::ZipError(format!("Zstandard decompression failed: {}", e)))
}

#[cfg(test)]
#[path = "compression_tests.rs"]
mod compression_tests;
