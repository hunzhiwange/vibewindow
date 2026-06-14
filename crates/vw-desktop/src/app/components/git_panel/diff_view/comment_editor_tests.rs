use super::comment_editor::diff_comment_editor;
use crate::app::App;
use crate::app::state::{GitDiffCommentDraft, GitDiffLineRange};
use iced::widget::text_editor;

fn app() -> App {
    App::new().0
}

#[test]
fn diff_comment_editor_test_module_is_linked() {
    assert_eq!("diff_comment_editor", "diff_comment_editor");
}

#[test]
fn comment_editor_is_present_only_when_draft_exists() {
    let mut app = app();
    assert!(diff_comment_editor(&app).is_none());

    app.git_diff_comment_draft = Some(GitDiffCommentDraft {
        range: GitDiffLineRange {
            file: "src/very_long_file_name_for_comment_editor.rs".to_string(),
            start: 1,
            end: 3,
            is_old: false,
        },
        editor: text_editor::Content::with_text("Looks good"),
    });
    assert!(diff_comment_editor(&app).is_some());

    app.git_diff_comment_draft = Some(GitDiffCommentDraft {
        range: GitDiffLineRange { file: "old.rs".to_string(), start: 4, end: 4, is_old: true },
        editor: text_editor::Content::new(),
    });
    assert!(diff_comment_editor(&app).is_some());
}
