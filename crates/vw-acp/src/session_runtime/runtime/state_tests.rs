use super::*;

#[test]
fn discard_output_formatter_accepts_empty_lifecycle() {
    let mut formatter = DiscardOutputFormatter;

    formatter.set_context(OutputFormatterContext { session_id: "session-1".to_string() });
    formatter.flush();
}
