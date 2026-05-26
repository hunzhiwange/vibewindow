use super::*;
use std::path::Path;

#[test]
fn walk_boundary_accepts_descendants_only() {
    let boundary = Path::new("/tmp/project");

    assert!(is_within_walk_boundary(boundary, Path::new("/tmp/project/src")));
    assert!(is_within_walk_boundary(boundary, Path::new("/tmp/project")));
    assert!(!is_within_walk_boundary(boundary, Path::new("/tmp/project-other")));
}
