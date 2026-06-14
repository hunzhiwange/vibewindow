#[test]
fn task_1184_test_module_is_wired() {}

use super::*;
use crate::app::message::DesignMessage;

fn mesh() -> MeshFill {
    MeshFill {
        enabled: true,
        columns: 3,
        rows: 3,
        colors: (0..9).map(|i| format!("#{i:06x}")).collect(),
        points: (0..9).map(|i| vec![i as f64 / 10.0, i as f64 / 20.0]).collect(),
        handles: (0..9).map(|i| vec![i as f64; 8]).collect(),
        mirroring: Some("x".to_string()),
        outline: true,
        selected_point_index: Some(8),
    }
}

fn fills() -> Vec<FillItem> {
    vec![FillItem::Object(FillObject::Mesh(mesh()))]
}

fn updated_fills(message: Message) -> Vec<FillItem> {
    match message {
        Message::Design(DesignMessage::PropertyUpdate(id, key, value)) => {
            assert_eq!(id, "shape");
            assert_eq!(key, "fill");
            serde_json::from_value(value).expect("fill update should serialize")
        }
        _ => panic!("unexpected message"),
    }
}

fn updated_mesh(message: Message) -> MeshFill {
    match updated_fills(message).remove(0) {
        FillItem::Object(FillObject::Mesh(mesh)) => mesh,
        _ => panic!("expected mesh fill"),
    }
}

#[test]
fn update_mesh_grid_cells_clamps_and_preserves_overlap() {
    let mesh = updated_mesh(update_mesh_grid_cells("shape".to_string(), fills(), 0, 9, 0));

    assert_eq!(mesh.columns, 6);
    assert_eq!(mesh.rows, 2);
    assert_eq!(mesh.colors.len(), 12);
    assert_eq!(mesh.colors[0], "#000000");
    assert_eq!(mesh.colors[1], "#000001");
    assert_eq!(mesh.colors[6], "#000003");
    assert_eq!(mesh.points[6], vec![0.3, 0.15]);
    assert_eq!(mesh.handles[6], vec![3.0; 8]);
    assert_eq!(mesh.selected_point_index, Some(8));
}

#[test]
fn update_mesh_grid_cells_clears_selection_when_new_grid_is_smaller() {
    let mesh = updated_mesh(update_mesh_grid_cells("shape".to_string(), fills(), 0, 1, 1));

    assert_eq!((mesh.columns, mesh.rows), (2, 2));
    assert_eq!(mesh.colors.len(), 4);
    assert_eq!(mesh.selected_point_index, None);
}

#[test]
fn simple_mesh_flags_and_selection_update_only_mesh_targets() {
    let mesh = updated_mesh(update_mesh_outline("shape".to_string(), fills(), 0, false));
    assert!(!mesh.outline);

    let mesh =
        updated_mesh(update_mesh_mirroring("shape".to_string(), fills(), 0, Some("y".to_string())));
    assert_eq!(mesh.mirroring.as_deref(), Some("y"));

    let mesh = updated_mesh(update_mesh_selection("shape".to_string(), fills(), 0, 3));
    assert_eq!(mesh.selected_point_index, Some(3));

    let mesh = updated_mesh(clear_mesh_selection("shape".to_string(), fills(), 0));
    assert_eq!(mesh.selected_point_index, None);

    let solid = vec![FillItem::Color("#fff".to_string())];
    assert_eq!(
        updated_fills(update_mesh_outline("shape".to_string(), solid.clone(), 0, false)),
        solid
    );
}

#[test]
fn shuffle_and_regenerate_do_not_run_when_point_is_selected() {
    let original = fills();

    assert_eq!(
        updated_fills(shuffle_mesh_colors("shape".to_string(), original.clone(), 0)),
        original
    );
    assert_eq!(
        updated_fills(regenerate_mesh_colors("shape".to_string(), original.clone(), 0)),
        original
    );
}

#[test]
fn shuffle_and_regenerate_update_unselected_mesh_colors() {
    let mut mesh = mesh();
    mesh.selected_point_index = None;
    let input = vec![FillItem::Object(FillObject::Mesh(mesh.clone()))];

    let shuffled = updated_mesh(shuffle_mesh_colors("shape".to_string(), input.clone(), 0));
    assert_eq!(shuffled.colors.len(), mesh.colors.len());
    assert_eq!(shuffled.selected_point_index, None);

    let regenerated = updated_mesh(regenerate_mesh_colors("shape".to_string(), input, 0));
    assert_eq!(regenerated.colors.len(), mesh.columns * mesh.rows);
    assert_eq!(regenerated.selected_point_index, None);
}

#[test]
fn update_selected_color_and_apply_to_all_cover_valid_and_invalid_points() {
    let selected_mesh =
        updated_mesh(update_mesh_selected_color("shape".to_string(), fills(), 0, 2));
    assert_eq!(selected_mesh.colors.len(), 9);
    assert!(selected_mesh.colors[2].starts_with('#'));

    let all_color_mesh =
        updated_mesh(apply_mesh_color_to_all("shape".to_string(), fills(), 0, "#abcdef".into()));
    assert!(all_color_mesh.colors.iter().all(|color| color == "#abcdef"));

    let unchanged = updated_mesh(update_mesh_selected_color("shape".to_string(), fills(), 0, 99));
    assert_eq!(unchanged.colors, mesh().colors);
}

#[test]
fn reset_mesh_positions_restores_defaults_and_clears_selection() {
    let mesh = updated_mesh(reset_mesh_positions("shape".to_string(), fills(), 0));
    let (points, handles) = MeshFill::default_points_and_handles(3, 3);

    assert_eq!(mesh.points, points);
    assert_eq!(mesh.handles, handles);
    assert_eq!(mesh.selected_point_index, None);
}

#[test]
fn reset_selected_mesh_position_and_curve_handle_selected_and_none() {
    let reset_mesh = updated_mesh(reset_selected_mesh_position("shape".to_string(), fills(), 0));
    let (points, handles) = MeshFill::default_points_and_handles(3, 3);
    assert_eq!(reset_mesh.points[8], points[8]);
    assert_eq!(reset_mesh.handles[8], handles[8]);
    assert_eq!(reset_mesh.selected_point_index, Some(8));

    let curve_mesh = updated_mesh(reset_selected_mesh_curve("shape".to_string(), fills(), 0));
    assert_eq!(curve_mesh.handles[8], vec![0.8, 0.4, 0.8, 0.4, 0.8, 0.4, 0.8, 0.4]);

    let mut no_selection = mesh();
    no_selection.selected_point_index = None;
    let input = vec![FillItem::Object(FillObject::Mesh(no_selection.clone()))];
    assert_eq!(
        updated_fills(reset_selected_mesh_position("shape".to_string(), input.clone(), 0)),
        input
    );
    assert_eq!(
        updated_fills(reset_selected_mesh_curve("shape".to_string(), input.clone(), 0)),
        input
    );
}
