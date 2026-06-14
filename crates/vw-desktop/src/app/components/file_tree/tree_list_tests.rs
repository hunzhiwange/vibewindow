use super::tree_list::build_file_tree_list;
use crate::app::App;
use crate::app::components::file_tree::model::build_file_tree_model;

#[test]
fn tree_list_builds_for_basic_project_tree() {
    let mut app = App::new().0;
    app.project_path = Some("/tmp/demo".to_string());
    let files = vec!["/tmp/demo/src/main.rs".to_string(), "/tmp/demo/README.md".to_string()];
    app.file_index_cache.insert("/tmp/demo".to_string(), files.clone());
    app.file_tree_model_cache
        .insert("/tmp/demo".to_string(), build_file_tree_model("/tmp/demo", &files));
    app.file_tree_expanded = vec!["src".to_string()];
    app.file_tree_expanded_set.insert("src".to_string());

    let _ = build_file_tree_list(&app);
}
