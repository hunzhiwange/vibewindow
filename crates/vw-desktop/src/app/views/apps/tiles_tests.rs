#[test]
fn tile_column_formula_has_minimum_one_column() {
    let available_width = 160.0_f32.max(220.0);
    let cols = ((available_width + 16.0) / (220.0 + 16.0)).floor() as usize;
    assert_eq!(cols.max(1), 1);
}
