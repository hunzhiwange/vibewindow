use super::{FileType, ParsedFile};

#[test]
fn file_type_is_copyable_and_comparable() {
    let figma = FileType::Figma;
    let copied = figma;

    assert_eq!(copied, FileType::Figma);
    assert_ne!(copied, FileType::FigJam);
}

#[test]
fn parsed_file_exposes_schema_data_and_image_chunks() {
    let parsed = ParsedFile::new(
        42,
        vec![b"schema".to_vec(), b"data".to_vec(), b"image-a".to_vec(), b"image-b".to_vec()],
    );

    assert_eq!(parsed.version, 42);
    assert_eq!(parsed.schema_chunk(), Some(&b"schema"[..]));
    assert_eq!(parsed.data_chunk(), Some(&b"data"[..]));
    assert_eq!(parsed.image_chunks(), &[b"image-a".to_vec(), b"image-b".to_vec()]);
}

#[test]
fn parsed_file_handles_missing_optional_chunks() {
    let empty = ParsedFile::new(1, Vec::new());
    assert_eq!(empty.schema_chunk(), None);
    assert_eq!(empty.data_chunk(), None);
    assert!(empty.image_chunks().is_empty());

    let schema_only = ParsedFile::new(2, vec![b"schema".to_vec()]);
    assert_eq!(schema_only.schema_chunk(), Some(&b"schema"[..]));
    assert_eq!(schema_only.data_chunk(), None);
    assert!(schema_only.image_chunks().is_empty());

    let data_only = ParsedFile::new(3, vec![b"schema".to_vec(), b"data".to_vec()]);
    assert_eq!(data_only.data_chunk(), Some(&b"data"[..]));
    assert!(data_only.image_chunks().is_empty());
}
