use iced::Element;

use super::empty::render;
use super::*;

fn keep_element(element: Element<'static, Message>) {
    let _ = std::hint::black_box(element);
}

#[test]
fn render_builds_empty_state_element() {
    keep_element(render());
}
