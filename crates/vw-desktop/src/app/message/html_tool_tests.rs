//! 覆盖 HTML 工具消息处理的行为，确保输入、预览和结果状态符合预期。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::html_tool::{beautify_html, compress_html};

#[test]
fn beautify_html_preserves_textarea_whitespace() {
    let input = "<textarea>  line 1\n    line 2\n</textarea>";
    let expected = "<textarea>\n  line 1\n    line 2\n</textarea>\n";

    assert_eq!(beautify_html(input).as_deref(), Some(expected));
}

#[test]
fn beautify_html_indents_script_content() {
    let input = "<script>const answer = 42;\nconsole.log(answer);</script>";
    let expected = "<script>\n    const answer = 42;\n    console.log(answer);\n</script>\n";

    assert_eq!(beautify_html(input).as_deref(), Some(expected));
}

#[test]
fn compress_html_preserves_pre_content() {
    let input = "<pre>  keep\n    spacing</pre>";
    let expected = "<pre>  keep\n    spacing</pre>";

    assert_eq!(compress_html(input).as_deref(), Some(expected));
}
