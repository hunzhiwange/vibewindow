use super::fs_ops::{list, publish_edited, read};
use super::{ContentType, Error, NodeType};
use std::fs;

#[test]
fn read_text_file_trims_content() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("note.txt"), " hello \n").expect("write text");

    let content = read(temp.path(), "note.txt").expect("read text");

    assert!(matches!(content.r#type, ContentType::Text));
    assert_eq!(content.content, "hello");
    assert_eq!(content.encoding, None);
    assert_eq!(content.mime_type, None);
}

#[test]
fn read_missing_text_file_returns_empty_text() {
    let temp = tempfile::tempdir().expect("tempdir");

    let content = read(temp.path(), "missing.txt").expect("missing text");

    assert!(matches!(content.r#type, ContentType::Text));
    assert_eq!(content.content, "");
}

#[test]
fn read_image_encodes_bytes_as_base64_with_mime_type() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("pixel.png"), [0_u8, 1, 2, 3]).expect("write image");

    let content = read(temp.path(), "pixel.png").expect("read image");

    assert!(matches!(content.r#type, ContentType::Text));
    assert_eq!(content.content, "AAECAw==");
    assert_eq!(content.encoding.as_deref(), Some("base64"));
    assert_eq!(content.mime_type.as_deref(), Some("image/png"));
}

#[test]
fn read_missing_image_returns_empty_text_without_metadata() {
    let temp = tempfile::tempdir().expect("tempdir");

    let content = read(temp.path(), "missing.png").expect("missing image");

    assert!(matches!(content.r#type, ContentType::Text));
    assert_eq!(content.content, "");
    assert_eq!(content.encoding, None);
    assert_eq!(content.mime_type, None);
}

#[test]
fn read_binary_extension_returns_binary_content() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("archive.zip"), [0_u8, 159, 146, 150]).expect("write binary");

    let content = read(temp.path(), "archive.zip").expect("read binary");

    assert!(matches!(content.r#type, ContentType::Binary));
    assert_eq!(content.content, "");
}

#[test]
fn read_rejects_paths_outside_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let outside = temp.path().join("outside.txt");
    fs::write(&outside, "secret").expect("write outside");
    let root = temp.path().join("root");
    fs::create_dir(&root).expect("create root");

    let err = read(&root, "../outside.txt").expect_err("escape should fail");

    assert!(matches!(err, Error::AccessDenied(_)));
}

#[test]
fn list_sorts_directories_before_files_and_skips_ignored_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir(temp.path().join("z_dir")).expect("create z dir");
    fs::create_dir(temp.path().join("a_dir")).expect("create a dir");
    fs::create_dir(temp.path().join(".git")).expect("create git");
    fs::write(temp.path().join("b.txt"), "b").expect("write b");
    fs::write(temp.path().join("a.txt"), "a").expect("write a");
    fs::write(temp.path().join(".DS_Store"), "").expect("write ds store");

    let nodes = list(temp.path(), None).expect("list root");
    let names = nodes.iter().map(|node| node.name.as_str()).collect::<Vec<_>>();

    assert_eq!(names, vec!["a_dir", "z_dir", "a.txt", "b.txt"]);
    assert!(nodes[0].absolute.ends_with("a_dir"));
    assert!(matches!(nodes[0].r#type, NodeType::Directory));
    assert!(matches!(nodes[2].r#type, NodeType::File));
    assert!(nodes.iter().all(|node| !node.ignored));
}

#[test]
fn list_nested_directory_uses_forward_slash_relative_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src/nested")).expect("create nested");
    fs::write(temp.path().join("src/nested/lib.rs"), "lib").expect("write lib");

    let nodes = list(temp.path(), Some("src")).expect("list src");

    assert!(nodes.iter().any(|node| {
        node.name == "nested"
            && node.path == "src/nested"
            && matches!(node.r#type, NodeType::Directory)
    }));
}

#[test]
fn list_rejects_paths_outside_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("root");
    fs::create_dir(&root).expect("create root");

    let err = list(&root, Some("..")).expect_err("escape should fail");

    assert!(matches!(err, Error::AccessDenied(_)));
}

#[test]
fn list_propagates_read_dir_errors() {
    let temp = tempfile::tempdir().expect("tempdir");

    let err = list(temp.path(), Some("missing")).expect_err("missing dir should fail");

    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn publish_edited_accepts_file_path() {
    publish_edited("src/lib.rs");
}
