use super::render_scrollbar;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn renders_full_track_when_no_scroll_needed() {
    let backend = TestBackend::new(4, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    let lines = vec!["one".to_string(), "two".to_string()];

    terminal
        .draw(|frame| {
            render_scrollbar(frame, Rect::new(0, 0, 4, 4), &lines, 0);
        })
        .unwrap();

    assert!(buffer_text(&terminal).contains("█"));
}

#[test]
fn renders_thumb_when_scroll_is_active() {
    let backend = TestBackend::new(4, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    let lines = (0..20).map(|idx| format!("line {idx}")).collect::<Vec<_>>();

    terminal
        .draw(|frame| {
            render_scrollbar(frame, Rect::new(0, 0, 4, 4), &lines, 5);
        })
        .unwrap();

    let text = buffer_text(&terminal);
    assert!(text.contains("█"));
    assert!(text.contains("│"));
}
