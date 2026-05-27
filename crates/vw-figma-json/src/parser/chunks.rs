use crate::error::{FigError, Result};
use crate::types::ParsedFile;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

/// 有效 .fig 文件的最小文件大小(8 字节标头 + 4 字节版本)
const MIN_FILE_SIZE: usize = 12;

/// 最小块头大小(长度为 4 个字节)
const CHUNK_HEADER_SIZE: usize = 4;

/// 从 ZIP 存档中提取 canvas.fig
///
/// 一些 .fig 文件(尤其是较大的文件)存储为 ZIP 存档
/// 包含 `canvas.fig` 文件。该函数提取该文件。
///
/// # 参数
/// * `bytes` - 原始 ZIP 文件字节
///
/// # 返回值
/// * `Ok(Vec<u8>)` - 提取的 canvas.fig 文件内容
/// * `Err(FigError)` - 如果 ZIP 提取失败或找不到 canvas.fig
///
/// # 示例
/// ```no_run
/// use fig2json::parser::extract_from_zip;
///
/// let zip_bytes = std::fs::read("example.fig").unwrap();
/// let canvas_bytes = extract_from_zip(&zip_bytes).unwrap();
/// ```
pub fn extract_from_zip(bytes: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // 查找 "canvas.fig" 条目
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name == "canvas.fig" {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            return Ok(contents);
        }
    }

    Err(FigError::CanvasNotFoundInZip)
}

/// 将整个 ZIP 存档解压到一个目录
///
/// 将 ZIP 存档中的所有文件提取到指定目录，
/// 保留 ZIP 文件的目录结构。
///
/// # 参数
/// * `bytes` - 原始 ZIP 文件字节
/// * `target_dir` - 将文件解压到的目录(不得存在)
///
/// # 返回值
/// * `Ok(())` - 如果提取成功
/// * `Err(FigError)` - 如果提取失败
///
/// # 示例
/// ```no_run
/// use fig2json::parser::extract_zip_to_directory;
/// use std::path::Path;
///
/// let zip_bytes = std::fs::read("example.zip").unwrap();
/// extract_zip_to_directory(&zip_bytes, Path::new("output")).unwrap();
/// ```
pub fn extract_zip_to_directory(bytes: &[u8], target_dir: &Path) -> Result<()> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // 创建目标目录
    fs::create_dir_all(target_dir)?;

    // 提取每个文件
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue, // Skip files with unsafe names
        };

        let output_path = target_dir.join(&file_path);

        if file.is_dir() {
            // 创建目录
            fs::create_dir_all(&output_path)?;
        } else {
            // 如果需要创建父目录
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // 提取文件
            let mut output_file = fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
        }
    }

    Ok(())
}

/// 从 .fig 文件中提取块(版本格式)
///
/// 解析 Evan Wallace 方法使用的版本格式：
/// ```text
/// [8 bytes] Magic header: "fig-kiwi" or "fig-jam."
/// [4 bytes] Version (uint32, little-endian)
/// [4 bytes] Chunk 0 length (uint32, little-endian)
/// [N bytes] Compressed chunk 0 (schema)
/// [4 bytes] Chunk 1 length (uint32, little-endian)
/// [N bytes] Compressed chunk 1 (data)
/// [...] Additional chunks (images, etc.)
/// ```
///
/// # 参数
/// * `bytes` - 原始 .fig 文件字节(魔术头验证后)
///
/// # 返回值
/// * `Ok(ParsedFile)` - 带有版本和块的解析文件
/// * `Err(FigError)` - 如果解析失败
///
/// # 示例
/// ```no_run
/// use fig2json::parser::extract_chunks;
///
/// let bytes = std::fs::read("example.canvas.fig").unwrap();
/// let parsed = extract_chunks(&bytes).unwrap();
/// println!("Version: {}", parsed.version);
/// println!("Chunks: {}", parsed.chunks.len());
/// ```
pub fn extract_chunks(bytes: &[u8]) -> Result<ParsedFile> {
    // 验证最小文件大小
    if bytes.len() < MIN_FILE_SIZE {
        return Err(FigError::FileTooSmall { expected: MIN_FILE_SIZE, actual: bytes.len() });
    }

    // 读取偏移量 8 处的版本(在 8 字节魔术头之后)
    let version = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    // 提取从偏移量 12 开始的所有块
    let mut chunks = Vec::new();
    let mut offset = 12;

    while offset < bytes.len() {
        // 检查我们是否有足够的字节用于块头
        if offset + CHUNK_HEADER_SIZE > bytes.len() {
            // 不再有完整的块，我们完成了
            break;
        }

        // 读取块长度(4 字节，小端)
        let chunk_length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += CHUNK_HEADER_SIZE;

        // 验证我们有足够的字节用于块数据
        if offset + chunk_length > bytes.len() {
            return Err(FigError::IncompleteChunk {
                offset: offset - CHUNK_HEADER_SIZE,
                expected: chunk_length,
                actual: bytes.len() - offset,
            });
        }

        // 提取块数据
        let chunk_data = bytes[offset..offset + chunk_length].to_vec();
        chunks.push(chunk_data);
        offset += chunk_length;
    }

    // 验证我们至少有 2 个块(模式 + 数据)
    if chunks.len() < 2 {
        return Err(FigError::NotEnoughChunks { expected: 2, actual: chunks.len() });
    }

    Ok(ParsedFile::new(version, chunks))
}

#[cfg(test)]
#[path = "chunks_tests.rs"]
mod chunks_tests;
