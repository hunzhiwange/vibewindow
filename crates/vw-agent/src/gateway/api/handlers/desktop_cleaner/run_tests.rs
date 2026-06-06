#[test]
fn cleanup_stats_ignores_targets_covered_by_existing_directory() {
    let mut stats = super::CleanupStats::default();
    stats.track_directory("/tmp");
    stats.track_matching_files("/tmp", &["log"]);

    assert_eq!(stats.targets.len(), 1);
    assert_eq!(stats.targets[0].kind, super::ScanDetailKind::Directory);
}

#[test]
fn cleanup_stats_summary_lines_use_formatted_totals() {
    let stats = super::CleanupStats::default();

    assert_eq!(stats.summary_line(), "本次预计清理垃圾数据：0 B");
    assert_eq!(stats.actual_removed_line(), "本次实际删除垃圾数据：0 B");
}
