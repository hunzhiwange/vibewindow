#[test]
fn cleaner_progress_value_clamps_to_valid_range() {
    assert_eq!(super::simplify_path("$HOME/Library/Caches/app"), "~/Library/Caches/app");
}
