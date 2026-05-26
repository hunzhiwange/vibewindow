use crate::id::{ProjectId, SessionId};

#[test]
fn string_ids_convert_and_serialize_as_plain_strings() {
    let project = ProjectId::from("project-1");
    let session = SessionId::from(String::from("session-1"));

    assert_eq!(project.as_ref(), "project-1");
    assert_eq!(session.as_ref(), "session-1");
    assert_eq!(serde_json::to_string(&project).expect("serialize"), "\"project-1\"");
}
