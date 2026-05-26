use super::filesystem::{contains, exists, find_up, glob_up, is_dir, normalize_path, overlaps, up};

#[tokio::test]
async fn path_helpers_detect_boundaries_and_walk_upwards() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();
    let child = root.join("a").join("b");
    std::fs::create_dir_all(&child).expect("mkdir");
    std::fs::write(root.join("Cargo.toml"), "x").expect("write");
    std::fs::write(child.join("local.txt"), "x").expect("write");

    assert!(exists(root.join("Cargo.toml")));
    assert!(is_dir(&child));
    assert_eq!(normalize_path(&child), child);
    assert!(contains(root, &child));
    assert!(overlaps(root, &child));
    assert_eq!(find_up("Cargo.toml", &child, None::<&std::path::Path>).await.len(), 1);
    assert_eq!(up(&["Cargo.toml"], &child, None::<&std::path::Path>).len(), 1);
    assert_eq!(glob_up("*.txt", &child, Some(root)).len(), 1);
}
