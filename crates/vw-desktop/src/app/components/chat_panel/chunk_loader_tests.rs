//! 验证聊天面板分块加载器。
//! 测试覆盖窗口扩展和边界条件，确保长对话渲染稳定。

use std::collections::HashSet;

use super::chunk_loader::{eviction_chunk_starts, heavy_cache_keep_bounds, retained_chunk_starts};

#[test]
fn retained_chunk_starts_keeps_visible_neighbors_and_protected_chunks() {
    let retained = retained_chunk_starts(260, 70, 125, &[192]);

    assert!(retained.contains(&64));
    assert!(retained.contains(&128));
    assert!(retained.contains(&192));
}

#[test]
fn eviction_chunk_starts_returns_sorted_unretained_chunks() {
    let prepared = HashSet::from([0, 64, 128, 192]);
    let retained = HashSet::from([64, 128]);

    assert_eq!(eviction_chunk_starts(&prepared, &retained), vec![0, 192]);
}

#[test]
fn heavy_cache_keep_bounds_expands_for_protected_chunk() {
    let (keep_start, keep_end) = heavy_cache_keep_bounds(260, 120, 140, 24, &[192]);

    assert_eq!(keep_start, 96);
    assert_eq!(keep_end, 248);
}
