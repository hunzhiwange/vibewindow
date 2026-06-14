use super::{view, view_file_manager};
use crate::app::App;

#[test]
fn file_tree_views_handle_empty_and_loaded_projects() {
    let app = App::new().0;
    let _ = view(&app);
    let _ = view_file_manager(&app);

    let mut app = App::new().0;
    app.project_path = Some("/tmp/demo".to_string());
    let _ = view(&app);
    let _ = view_file_manager(&app);
}
