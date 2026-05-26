use super::model::default_doc;

#[test]
fn default_doc_starts_with_single_center_topic() {
    let doc = default_doc();

    assert_eq!(doc.text, "中心主题");
    assert!(doc.children.is_empty());
}
