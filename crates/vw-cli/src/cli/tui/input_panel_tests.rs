use super::{render_input_box, render_input_panel};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

#[test]
fn render_input_box_returns_inner_text_area() {
    let backend = TestBackend::new(40, 8);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut rendered = Rect::default();

    terminal
        .draw(|frame| {
            rendered = render_input_box(frame, Rect::new(2, 1, 20, 4), "");
        })
        .unwrap();

    assert_eq!(rendered, Rect::new(3, 1, 18, 3));
}

#[test]
fn render_input_panel_draws_without_panic_for_busy_state() {
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render_input_panel(frame, Rect::new(0, 0, 80, 10), "hello\nworld", 50, true, "gpt", 1);
        })
        .unwrap();
}
