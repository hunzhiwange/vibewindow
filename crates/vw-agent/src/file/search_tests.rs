use super::SearchInput;
use super::search::{fuzzy_score, is_hidden, search};
use std::fs;

#[test]
fn fuzzy_score_prefers_substring_matches_and_filters_hidden_when_query_is_hidden() {
    assert_eq!(fuzzy_score("lib", "src/lib.rs"), 6);
    assert!(is_hidden("src/.secret/file"));
    assert_eq!(fuzzy_score(".secret", "src/visible.rs"), i64::MAX);
}

#[test]
fn fuzzy_score_handles_hidden_path_queries_and_levenshtein_fallback() {
    assert_eq!(fuzzy_score("src/.env", "src/env"), i64::MAX);
    assert!(fuzzy_score(".secret", "src/.secret/file") < i64::MAX);
    assert_eq!(fuzzy_score("abc", "xyz"), 3);
    assert!(is_hidden(r"src\.cache\file"));
    assert!(is_hidden("src/.cache/"));
    assert!(!is_hidden("src/cache/file"));
}

#[test]
fn search_empty_query_defaults_to_files_and_truncates_sorted_results() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("dir")).expect("create dir");
    fs::write(temp.path().join("b.rs"), "b").expect("write b");
    fs::write(temp.path().join("a.rs"), "a").expect("write a");
    fs::write(temp.path().join("dir/nested.rs"), "nested").expect("write nested");

    let result = search(
        temp.path(),
        SearchInput { query: "  ".to_string(), limit: 2, dirs: false, r#type: None },
    );

    assert_eq!(result, vec!["a.rs".to_string(), "b.rs".to_string()]);
}

#[test]
fn search_can_return_directories_or_all_items() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("create src");
    fs::create_dir_all(temp.path().join("docs")).expect("create docs");
    fs::write(temp.path().join("src/main.rs"), "main").expect("write main");
    fs::write(temp.path().join("docs/readme.md"), "docs").expect("write docs");

    let dirs = search(
        temp.path(),
        SearchInput {
            query: "".to_string(),
            limit: 10,
            dirs: false,
            r#type: Some("directory".to_string()),
        },
    );
    let all_from_dirs_flag = search(
        temp.path(),
        SearchInput { query: "".to_string(), limit: 10, dirs: true, r#type: None },
    );
    let all_from_unknown_type = search(
        temp.path(),
        SearchInput {
            query: "".to_string(),
            limit: 10,
            dirs: false,
            r#type: Some("unknown".to_string()),
        },
    );

    assert_eq!(dirs, vec!["docs/".to_string(), "src/".to_string()]);
    assert_eq!(
        all_from_dirs_flag,
        vec![
            "docs/".to_string(),
            "docs/readme.md".to_string(),
            "src/".to_string(),
            "src/main.rs".to_string(),
        ]
    );
    assert_eq!(all_from_unknown_type, all_from_dirs_flag);
}

#[test]
fn search_trims_query_scores_results_and_filters_visible_files_for_hidden_queries() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("create src");
    fs::create_dir_all(temp.path().join(".secret")).expect("create secret");
    fs::write(temp.path().join("src/lib.rs"), "lib").expect("write lib");
    fs::write(temp.path().join("src/main.rs"), "main").expect("write main");
    fs::write(temp.path().join(".secret/token"), "token").expect("write token");
    fs::write(temp.path().join("visible.txt"), "visible").expect("write visible");

    let best_match = search(
        temp.path(),
        SearchInput { query: " lib ".to_string(), limit: 1, dirs: false, r#type: None },
    );
    let hidden_match = search(
        temp.path(),
        SearchInput { query: ".secret".to_string(), limit: 10, dirs: false, r#type: None },
    );

    assert_eq!(best_match, vec!["src/lib.rs".to_string()]);
    assert_eq!(hidden_match, vec![".secret/token".to_string()]);
}

#[test]
fn search_missing_root_returns_empty_results() {
    let temp = tempfile::tempdir().expect("tempdir");

    let result = search(
        temp.path().join("missing"),
        SearchInput { query: "".to_string(), limit: 10, dirs: true, r#type: None },
    );

    assert!(result.is_empty());
}
