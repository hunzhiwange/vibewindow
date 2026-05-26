#[test]
fn bases_vec_contains_supported_range() {
    let bases = super::bases_vec();
    assert_eq!(bases.first().copied(), Some(2));
    assert_eq!(bases.last().copied(), Some(36));
    assert_eq!(bases.len(), 35);
}
