use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理滚轮滚动事件
    pub(in super::super) fn handle_wheel_scrolled(
        &self,
        cursor_pos: Point,
        delta: &mouse::ScrollDelta,
    ) -> Option<Action<Message>> {
        if self.mouse_wheel_zoom_enabled {
            let scroll_y = match delta {
                mouse::ScrollDelta::Lines { y, .. } => *y,
                mouse::ScrollDelta::Pixels { y, .. } => *y,
            };

            if scroll_y != 0.0 {
                let factor = if scroll_y > 0.0 { 1.1 } else { 0.9 };
                return Some(Action::publish(Message::Design(DesignMessage::Zoom(
                    factor,
                    Some(cursor_pos),
                ))));
            }

            return None;
        }

        let (dx, dy) = match delta {
            mouse::ScrollDelta::Lines { x, y } => ((*x) * 60.0, (*y) * 60.0),
            mouse::ScrollDelta::Pixels { x, y } => (*x, *y),
        };

        if dx == 0.0 && dy == 0.0 {
            return None;
        }

        let new_pan = self.pan + Vector::new(dx, dy);
        Some(Action::publish(Message::Design(DesignMessage::Pan(new_pan))))
    }
}

#[cfg(test)]
#[path = "wheel_tests.rs"]
mod wheel_tests;
