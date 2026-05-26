use super::*;
use std::path::Path;

#[test]
fn snapshots_match_only_identical_text() {
    let snapshot = FileSnapshot::from_text("hello");
    assert!(snapshot.matches_text("hello"));
    assert!(!snapshot.matches_text("hello!"));
    assert_eq!(snapshot.size_bytes, 5);
}

#[test]
fn cache_tracks_reads_and_invalidates_by_normalized_path() {
    let root = Path::new("/tmp/workspace");
    let mut cache = FileReadStateCache::new(4, 100);
    let normalized = cache.note_read(Some(root), "src/../file.txt", 7, false, None, None, None);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.total_bytes(), 7);
    assert_eq!(cache.get(Some(root), "file.txt").unwrap().path, normalized);
    assert!(cache.invalidate(Some(root), "file.txt").is_some());
    assert!(cache.is_empty());
}

#[test]
fn cache_evicts_oldest_when_limits_are_exceeded() {
    let mut cache = FileReadStateCache::new(1, 10);
    cache.note_read(None, "a.txt", 4, false, None, None, None);
    cache.note_read(None, "b.txt", 4, false, None, None, None);
    assert!(cache.get(None, "a.txt").is_none());
    assert!(cache.get(None, "b.txt").is_some());
}
