use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理指针事件（鼠标事件）
    ///
    /// 这是鼠标事件的主入口方法，根据事件类型分发到对应的处理函数：
    /// - 左键按下：调用 `handle_left_pressed`
    /// - 左键释放：调用 `handle_left_released`
    /// - 光标移动：调用 `handle_cursor_moved`
    /// - 滚轮滚动：调用 `handle_wheel_scrolled`
    /// - 中键按下/释放：切换平移模式
    ///
    /// # 参数
    ///
    /// - `state`：可变的画布状态引用
    /// - `event`：画布事件
    /// - `bounds`：画布边界矩形
    /// - `cursor`：鼠标光标状态
    ///
    /// # 返回值
    ///
    /// 返回 `Some(Action<Message>)` 表示需要执行的操作，
    /// 返回 `None` 表示事件未处理。
    ///
    /// # 特殊处理
    ///
    /// - 如果光标不在画布内且存在悬停元素，清除悬停状态
    /// - 如果处于颜色拾取模式，左键点击会触发颜色拾取消息
    pub(in super::super) fn update_pointer_event(
        &self,
        state: &mut DesignCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let Some(cursor_pos) = cursor.position_in(bounds) else {
            if state.hovered_id.is_some() {
                state.hovered_id = None;
                return Some(Action::request_redraw());
            }
            if state.tool_preview_start.is_some() {
                state.tool_preview_current = None;
                state.tool_preview_parent_id = None;
                return Some(Action::request_redraw());
            }
            if !state.brush_points_world.is_empty() || state.brush_erasing {
                return Some(Action::request_redraw());
            }
            return None;
        };

        if self.color_picking
            && let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
        {
            return Some(Action::publish(Message::Design(DesignMessage::PickColor(
                cursor_pos,
            ))));
        }

        let root_selected_id = self.selected_id.and_then(|id| {
            self.doc.children.iter().find(|el| el.id == id && el.kind == "frame").map(|_| id)
        });
        let root_selected_bounds = root_selected_id
            .and_then(|id| get_element_screen_bounds(self.doc.as_ref(), id, self.pan, self.zoom));
        let cursor_in_root_selected =
            root_selected_bounds.map(|rect| rect.contains(cursor_pos)).unwrap_or(false);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                return self.handle_left_pressed(state, cursor_pos, cursor_in_root_selected);
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let world_x = (cursor_pos.x - self.pan.x) / self.zoom;
                let world_y = (cursor_pos.y - self.pan.y) / self.zoom;
                let hit_id = hit_test(&self.doc.children, self.doc.as_ref(), world_x, world_y);
                return Some(Action::publish(Message::Design(
                    DesignMessage::CanvasContextMenuOpen(cursor_pos, hit_id),
                )));
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                return self.handle_left_released(state, cursor_pos);
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                return self.handle_cursor_moved(state, cursor_pos);
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                return self.handle_wheel_scrolled(cursor_pos, delta);
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.is_panning = true;
                state.last_cursor_pos = Some(cursor_pos);
                return Some(Action::request_redraw());
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                state.is_panning = false;
                state.last_cursor_pos = None;
                return Some(Action::request_redraw());
            }
            _ => {}
        }

        None
    }
}

#[cfg(test)]
#[path = "pointer_tests.rs"]
mod pointer_tests;
