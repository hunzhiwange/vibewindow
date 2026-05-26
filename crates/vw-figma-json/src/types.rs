/// 基于 magic header 的 Figma 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 标准 Figma 文件 ("fig-kiwi")
    Figma,
    /// FigJam 文件 ("fig-jam.")
    FigJam,
}

/// 解析的 .fig 文件结构(包含版本和块)
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// 文件格式版本(uint32，little-endian)
    pub version: u32,
    /// 提取的块(第一个块是 schema，第二个块是数据，其余通常是图像)
    pub chunks: Vec<Vec<u8>>,
}

impl ParsedFile {
    /// 创建一个新的 ParsedFile
    pub fn new(version: u32, chunks: Vec<Vec<u8>>) -> Self {
        Self { version, chunks }
    }

    /// 获取schema 块(第一个块)
    pub fn schema_chunk(&self) -> Option<&[u8]> {
        self.chunks.first().map(|v| v.as_slice())
    }

    /// 获取数据块(第二个块)
    pub fn data_chunk(&self) -> Option<&[u8]> {
        self.chunks.get(1).map(|v| v.as_slice())
    }

    /// 获取图像块(前两个之后的所有块)
    pub fn image_chunks(&self) -> &[Vec<u8>] {
        if self.chunks.len() > 2 { &self.chunks[2..] } else { &[] }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
