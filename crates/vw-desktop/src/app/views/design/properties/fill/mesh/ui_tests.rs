#[test]
fn task_1185_test_module_is_wired() {}

use super::*;

fn keep_element(element: Element<'static, Message>) {
    std::hint::black_box(element);
}

fn mesh(selected_point_index: Option<usize>, mirroring: Option<&str>, outline: bool) -> MeshFill {
    let mut mesh = MeshFill::new_random(3, 3);
    mesh.colors = vec![
        "#000000".to_string(),
        "#111111".to_string(),
        "#222222".to_string(),
        "#333333".to_string(),
        "#444444".to_string(),
        "#555555".to_string(),
        "#666666".to_string(),
        "#777777".to_string(),
        "#888888".to_string(),
    ];
    mesh.selected_point_index = selected_point_index;
    mesh.mirroring = mirroring.map(str::to_string);
    mesh.outline = outline;
    mesh
}

fn fills(mesh: MeshFill) -> Vec<FillItem> {
    vec![FillItem::Object(crate::app::views::design::properties::fill::types::FillObject::Mesh(
        mesh,
    ))]
}

#[test]
fn render_covers_selected_unselected_and_mirroring_variants() {
    for (selected, mirroring, outline) in [
        (Some(4), Some("x"), true),
        (Some(99), Some("y"), false),
        (None, Some("xy"), true),
        (None, None, false),
    ] {
        let mesh = mesh(selected, mirroring, outline);
        keep_element(render(
            mesh.clone(),
            0,
            fills(mesh),
            "shape".to_string(),
            iced::Vector::new(12.0, -8.0),
            1.25,
        ));
    }
}

#[test]
fn render_handles_sparse_color_arrays() {
    let mut mesh = mesh(None, Some("x"), true);
    mesh.columns = 6;
    mesh.rows = 6;
    mesh.colors = vec!["not-a-color".to_string()];

    keep_element(render(
        mesh.clone(),
        0,
        fills(mesh),
        "shape".to_string(),
        iced::Vector::new(0.0, 0.0),
        0.5,
    ));
}

#[test]
fn grid_size_matrix_builds_for_min_and_max_active_cells() {
    let mesh = mesh(None, Some("x"), true);
    let fills = fills(mesh);

    keep_element(grid_size_matrix(1, 1, "shape".to_string(), fills.clone(), 0));
    keep_element(grid_size_matrix(5, 5, "shape".to_string(), fills, 0));
}
