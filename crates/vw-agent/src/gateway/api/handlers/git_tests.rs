use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn normalize_path_trims_slashes_and_current_directory() {
    assert_eq!(normalize_path("./src/main.rs"), "src/main.rs");
    assert_eq!(normalize_path("/src/main.rs"), "src/main.rs");
}

#[test]
fn validate_repo_relative_path_rejects_escape_segments() {
    assert!(validate_repo_relative_path("src/lib.rs").is_ok());
    assert!(validate_repo_relative_path("../secret").is_err());
    assert!(validate_repo_relative_path("/absolute").is_err());
}

#[test]
fn final_line_ending_only_change_matches_missing_newline() {
    assert!(is_final_line_ending_only_change("123\n", "123"));
    assert!(is_final_line_ending_only_change("123", "123\n"));
    assert!(is_final_line_ending_only_change("123\r\n", "123"));
}

#[test]
fn final_line_ending_only_change_rejects_content_changes() {
    assert!(!is_final_line_ending_only_change("123\n", "124"));
    assert!(!is_final_line_ending_only_change("123\n", "123\n"));
    assert!(!is_final_line_ending_only_change("123\n456\n", "123\n457"));
}

#[test]
fn selected_file_content_replaces_selected_hunk() {
    let mut selection = GitFileSelection::default();
    selection.hunks.insert(0);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("hunk should build selected content");

    assert_eq!(content, "alpha\nnew\nomega\n");
}

#[test]
fn selected_file_content_keeps_unselected_delete_side() {
    let mut selection = GitFileSelection::default();
    selection.new_lines.insert(1);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nnew\nold\nomega\n");
}

#[test]
fn selected_file_content_keeps_unselected_insert_side() {
    let mut selection = GitFileSelection::default();
    selection.old_lines.insert(1);

    let content =
        build_selected_file_content("alpha\nold\nomega\n", "alpha\nnew\nomega\n", &selection)
            .expect("line selection should build selected content");

    assert_eq!(content, "alpha\nomega\n");
}
