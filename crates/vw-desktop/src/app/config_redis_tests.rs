// Tests for plan6 task 842.
const SOURCE: &str = include_str!("config_redis.rs");

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn config_redis_tests_keeps_planned_coverage_targets() {
    for name in [
        "RedisToolGatewaySnapshot",
        "load_redis_tool_state",
        "load_redis_tool_state_async",
        "load_redis_tool_snapshot_async",
        "redis_settings_update_async",
        "redis_connection_create_async",
        "redis_connection_update_async",
        "redis_connection_delete_async",
        "redis_connection_activate_async",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}
