//! Figma JSON 错误模块，定义导入解析过程中可向上传递的统一错误类型。

use thiserror::Error;

/// FigError 枚举描述该模块支持的 FigError 取值集合。
///
/// 每个变体代表一个明确分支，调用方应通过显式匹配处理新增状态。
#[derive(Error, Debug)]
pub enum FigError {
    #[error("Invalid magic header: expected 'fig-kiwi' or 'fig-jam.', found {0:?}")]
    InvalidMagicHeader(Vec<u8>),

    #[error("File too small: expected at least {expected} bytes, found {actual}")]
    FileTooSmall { expected: usize, actual: usize },

    #[error("Incomplete chunk at offset {offset}: expected {expected} bytes, found {actual}")]
    IncompleteChunk { offset: usize, expected: usize, actual: usize },

    #[error("Not enough chunks: expected at least {expected}, found {actual}")]
    NotEnoughChunks { expected: usize, actual: usize },

    #[error("ZIP extraction failed: {0}")]
    ZipError(String),

    #[error("Canvas file not found in ZIP archive")]
    CanvasNotFoundInZip,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("ZIP library error: {0}")]
    ZipLibraryError(#[from] zip::result::ZipError),
}

/// Result 是该模块共享结果类型的别名。
pub type Result<T> = std::result::Result<T, FigError>;

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
