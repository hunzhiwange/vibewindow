use super::{adaptive_logo_scale, draw_home_screen, draw_main_screen};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
};

#[test]
fn adaptive_logo_scale_is_clamped_by_space() {
    assert_eq!(adaptive_logo_scale(Rect::new(0, 0, 20, 5), 2), 1);
    assert!(adaptive_logo_scale(Rect::new(0, 0, 200, 80), 3) <= 8);
}

#[test]
fn draw_home_screen_renders_with_menu_overlay() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            draw_home_screen(
                frame,
                Rect::new(0, 0, 100, 30),
                "hello",
                99,
                true,
                "gpt-5",
                "/tmp/workspace",
                3,
                Color::Cyan,
                2,
                true,
            );
        })
        .unwrap();
}

#[test]
fn draw_main_screen_handles_overflow_and_footer_warning() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let right_chunks = vec![Rect::new(70, 0, 30, 6), Rect::new(70, 6, 30, 8)];
    let modified = (0..12).map(|idx| format!("src/file_{idx}.rs")).collect::<Vec<_>>();

    terminal
        .draw(|frame| {
            draw_main_screen(
                frame,
                right_chunks.clone(),
                "Session title",
                &modified,
                false,
                Style::default(),
                "/tmp/workspace",
                true,
                true,
            );
        })
        .unwrap();
}
