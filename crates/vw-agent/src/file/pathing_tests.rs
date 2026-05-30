use super::pathing::{
    contains_path, image_mime_type, is_binary_by_extension, is_image_by_extension,
};

#[test]
fn extension_helpers_are_case_insensitive() {
    assert!(is_image_by_extension("cover.PNG"));
    assert_eq!(image_mime_type("cover.jpeg"), "image/jpeg");
    assert!(is_binary_by_extension("archive.ZIP"));
    assert!(!is_binary_by_extension("src/lib.rs"));
}

#[test]
fn contains_path_rejects_parent_escape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("root");
    std::fs::create_dir(&root).expect("create root");

    assert!(contains_path(&root, &root.join("child.txt")));
    assert!(!contains_path(&root, &temp.path().join("outside.txt")));
}
