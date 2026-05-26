use super::EDITED;

#[test]
fn edited_event_type_is_stable() {
    assert_eq!(EDITED.r#type, "file.edited");
}
