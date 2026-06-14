use super::menu::build_file_tree_menu;
use crate::app::App;
use crate::app::state::{FileTreeClipboard, FileTreeClipboardMode};

#[test]
fn file_tree_menu_builds_with_and_without_clipboard() {
    let app = App::new().0;
    let _ = build_file_tree_menu(&app, true);

    let mut app = App::new().0;
    app.file_tree_clipboard = Some(FileTreeClipboard {
        mode: FileTreeClipboardMode::Copy,
        src_path: "/tmp/demo".to_string(),
    });
    let _ = build_file_tree_menu(&app, false);
}
