use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn normalize_path_removes_current_directory_prefix() {
    assert_eq!(normalize_path("./crates/vw-agent"), "crates/vw-agent");
    assert_eq!(normalize_path("/crates/vw-agent"), "crates/vw-agent");
}

#[test]
fn worktree_id_round_trips_directory() {
    let directory = "/tmp/vibe window/project";
    let id = worktree_id_from_directory(directory);

    assert_eq!(directory_from_worktree_id(&id).expect("decode"), directory);
}
