use super::*;

#[test]
fn parse_info_entries_ignores_comments_and_blank_lines() {
    let entries = parse_info_entries("# Server\nredis_version:7.2\n\nconnected_clients:3\n");

    assert_eq!(info_value(&entries, "redis_version").as_deref(), Some("7.2"));
    assert_eq!(parse_info_u64(&entries, "connected_clients"), 3);
}

#[test]
fn parse_keyspace_stats_extracts_db_counts() {
    let entries = parse_info_entries("db0:keys=2,expires=1,avg_ttl=10\n");
    let stats = parse_keyspace_stats(&entries);

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].db, "db0");
    assert_eq!(stats[0].keys, 2);
    assert_eq!(stats[0].expires, 1);
}

#[test]
fn build_preview_command_quotes_arguments() {
    let command = build_preview_command(&Some("GET"), &["key with space".to_string()]);

    assert_eq!(command, "GET 'key with space'");
}
