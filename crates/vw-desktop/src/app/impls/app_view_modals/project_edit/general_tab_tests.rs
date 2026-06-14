use super::*;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

#[test]
fn general_tab_builds_with_default_text_preview() {
    let mut app = test_app();
    app.project_edit_name = "Project".to_string();
    app.project_edit_icon = String::new();
    app.project_edit_icon_color = "#60a5fa".to_string();

    let element = general_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn general_tab_builds_with_icon_text_preview_and_color_fallbacks() {
    let mut app = test_app();
    app.project_edit_name = String::new();
    app.project_edit_icon = "  Zed  ".to_string();
    app.project_edit_icon_color = "bad-color".to_string();

    let element = general_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn general_tab_builds_image_preview_and_hover_clear_overlay() {
    let path = std::env::temp_dir()
        .join(format!("vibe-window-project-edit-general-{}.png", std::process::id()));
    std::fs::write(&path, b"not a real png but enough for a path handle").unwrap();

    let mut app = test_app();
    app.project_edit_name = "Image".to_string();
    app.project_edit_icon = path.to_string_lossy().into_owned();
    app.project_edit_icon_hovered = true;
    app.project_edit_icon_color = "rgb(96, 165, 250)".to_string();

    let element = general_tab(&app);

    std::hint::black_box(element);
    let _ = std::fs::remove_file(path);
}
