//! Text chunking for local knowledge ingestion.

const DEFAULT_CHUNK_CHARS: usize = 700;
const DEFAULT_OVERLAP_CHARS: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextChunk {
    pub ordinal: usize,
    pub content: String,
}

pub(crate) fn chunk_text(text: &str) -> Vec<TextChunk> {
    chunk_text_with_limits(text, DEFAULT_CHUNK_CHARS, DEFAULT_OVERLAP_CHARS)
}

pub(crate) fn chunk_text_with_limits(
    text: &str,
    chunk_chars: usize,
    overlap_chars: usize,
) -> Vec<TextChunk> {
    let normalized = normalize_text(text);
    if normalized.is_empty() {
        return Vec::new();
    }
    let chars = normalized.chars().collect::<Vec<_>>();
    let chunk_chars = chunk_chars.max(1);
    let overlap_chars = overlap_chars.min(chunk_chars.saturating_sub(1));
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let mut end = (start + chunk_chars).min(chars.len());
        if end < chars.len() && remaining_fits_tail(&chars, start, chunk_chars) {
            end = chars.len();
        }
        if end < chars.len() {
            end = prefer_boundary(&chars, start, end).max(start + 1);
        }
        let content = chars[start..end].iter().collect::<String>().trim().to_string();
        if !content.is_empty() {
            chunks.push(TextChunk { ordinal: chunks.len(), content });
        }
        if end >= chars.len() {
            break;
        }
        start = end.saturating_sub(overlap_chars);
    }

    chunks
}

fn remaining_fits_tail(chars: &[char], start: usize, chunk_chars: usize) -> bool {
    let remaining = chars.len().saturating_sub(start);
    let tail_slack = (chunk_chars / 4).max(1);
    remaining <= chunk_chars.saturating_add(tail_slack)
}

fn normalize_text(text: &str) -> String {
    text.lines().map(str::trim).collect::<Vec<_>>().join("\n").trim().to_string()
}

fn prefer_boundary(chars: &[char], start: usize, end: usize) -> usize {
    let min = start + ((end - start) / 2);
    for index in (min..end).rev() {
        if matches!(chars[index], '\n' | '。' | '！' | '？' | '.' | '!' | '?') {
            return index + 1;
        }
    }
    end
}

#[cfg(test)]
#[path = "chunker_tests.rs"]
mod chunker_tests;
