use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理鼠标左键按下事件
    pub(in super::super) fn handle_left_pressed(
        &self,
        state: &mut DesignCanvasState,
        cursor_pos: Point,
        cursor_in_root_selected: bool,
    ) -> Option<Action<Message>> {
        let now = Instant::now();

        if self.active_tool == DesignTool::Move
            && let Some((last_time, last_pos)) = state.last_click
            && now.duration_since(last_time).as_millis() < 300
            && last_pos.distance(cursor_pos) < 5.0
            && !cursor_in_root_selected
        {
            let world_x = (cursor_pos.x - self.pan.x) / self.zoom;
            let world_y = (cursor_pos.y - self.pan.y) / self.zoom;

            if let Some(hit_id) = hit_test(&self.doc.children, self.doc.as_ref(), world_x, world_y)
                && let Some(el) = find_element_by_id(&self.doc.children, &hit_id)
                && (el.kind == "Typography"
                    || el.kind == "text"
                    || el.kind.eq_ignore_ascii_case("sticky_note"))
            {
                return Some(Action::publish(Message::Design(DesignMessage::EditStart(
                    hit_id,
                    el.content.clone().unwrap_or_default(),
                ))));
            }
        }
        state.last_click = Some((now, cursor_pos));

        if self.active_tool == DesignTool::Pen {
            state.brush_points_world.clear();
            state.brush_points_world.push(Point::new(
                (cursor_pos.x - self.pan.x) / self.zoom,
                (cursor_pos.y - self.pan.y) / self.zoom,
            ));
            state.brush_erasing = false;
            state.brush_erase_dirty = false;
            return Some(Action::request_redraw());
        }

        if self.active_tool == DesignTool::Eraser {
            state.brush_points_world.clear();
            state.brush_erasing = true;
            state.brush_erase_dirty = true;
            return Some(Action::publish(Message::Design(DesignMessage::EraseBrushAt(
                Point::new(
                    (cursor_pos.x - self.pan.x) / self.zoom,
                    (cursor_pos.y - self.pan.y) / self.zoom,
                ),
                ERASER_RADIUS_PX / self.zoom.max(0.0001),
            ))));
        }

        if let Some(sel_id) = self.selected_id
            && let Some(rect) =
                get_element_screen_bounds(self.doc.as_ref(), sel_id, self.pan, self.zoom)
        {
            let mut rotation = 0.0;
            if let Some(el) = find_element_by_id(&self.doc.children, sel_id) {
                rotation = el.rotation.unwrap_or(0.0);
            }

            let (check_x, check_y) = if rotation != 0.0 {
                let cx = rect.x + rect.width / 2.0;
                let cy = rect.y + rect.height / 2.0;
                rotate_point(cursor_pos.x, cursor_pos.y, cx, cy, -rotation.to_radians())
            } else {
                (cursor_pos.x, cursor_pos.y)
            };

            if self.active_tool == DesignTool::Move
                && let Some(el) = find_element_by_id(&self.doc.children, sel_id)
            {
                let mut fills = mesh::parse_fill_items(&el.fill);
                if let Some(fill_index) =
                    mesh::choose_mesh_fill_index(&fills, self.selected_fill_index)
                    && let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get_mut(fill_index)
                {
                    m.normalize();
                    if let Some((point_index, kind)) =
                        mesh::hit_test_mesh(m, rect, check_x, check_y)
                    {
                        m.selected_point_index = Some(point_index);
                        if matches!(kind, MeshDragKind::Handle(_)) {
                            m.materialize_effective_handles(point_index);
                        }
                        let (u, v) = mesh::cursor_to_uv(check_x, check_y, rect);
                        let (start_point_x, start_point_y) = m
                            .points
                            .get(point_index)
                            .map(|p| {
                                (
                                    p.first().copied().unwrap_or(0.0),
                                    p.get(1).copied().unwrap_or(0.0),
                                )
                            })
                            .unwrap_or((0.0, 0.0));
                        let start_handles = mesh::mesh_point_handles(m, point_index);

                        state.selected_mesh_handle = match kind {
                            MeshDragKind::Handle(handle_index) => Some(SelectedMeshHandle {
                                element_id: sel_id.to_string(),
                                fill_index,
                                point_index,
                                handle_index,
                            }),
                            MeshDragKind::Point => None,
                        };

                        state.mesh_drag = Some(MeshDragState {
                            element_id: sel_id.to_string(),
                            fill_index,
                            point_index,
                            kind,
                            has_moved: false,
                            start_cursor_u: u,
                            start_cursor_v: v,
                            start_point_x,
                            start_point_y,
                            start_handles,
                        });

                        return Some(Action::publish(Message::Design(
                            DesignMessage::PropertyUpdateTransient(
                                sel_id.to_string(),
                                "fill".to_string(),
                                serde_json::json!(fills),
                            ),
                        )));
                    }
                }
            }

            if let Some(handle) = hit_test_handle(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                check_x,
                check_y,
                self.zoom,
            ) {
                match handle {
                    Handle::RotateTopLeft
                    | Handle::RotateTopRight
                    | Handle::RotateBottomLeft
                    | Handle::RotateBottomRight => {
                        let center_x = rect.x + rect.width / 2.0;
                        let center_y = rect.y + rect.height / 2.0;
                        let start_angle = (cursor_pos.y - center_y).atan2(cursor_pos.x - center_x);

                        let mut current_rot = 0.0;
                        if let Some(element) = find_element_by_id(&self.doc.children, sel_id) {
                            current_rot = element.rotation.unwrap_or(0.0);
                        }

                        state.rotating = Some((sel_id.to_string(), current_rot, start_angle));
                        state.drag_start = Some(cursor_pos);
                    }
                    _ => {
                        if let Some(el) = find_element_by_id(&self.doc.children, sel_id) {
                            let local_rect = Rectangle::new(
                                Point::new(el.x, el.y),
                                Size::new(rect.width / self.zoom, rect.height / self.zoom),
                            );
                            state.resizing = Some((sel_id.to_string(), handle, local_rect));
                            state.drag_start = Some(cursor_pos);
                        }
                    }
                }
                return Some(Action::request_redraw());
            }
        }

        if self.active_tool == DesignTool::Hand {
            state.is_panning = true;
            state.last_cursor_pos = Some(cursor_pos);
            return Some(Action::request_redraw());
        }

        if matches!(self.active_tool, DesignTool::Text | DesignTool::Icon)
            && let Some((element, parent_id, start_editing)) =
                build_created_element(self, state, cursor_pos)
        {
            return Some(Action::publish(Message::Design(DesignMessage::CreateElement {
                element,
                parent_id,
                start_editing,
            })));
        }

        if tool_supports_drag_preview(self.active_tool) {
            state.tool_preview_start = Some(cursor_pos);
            state.tool_preview_current = Some(cursor_pos);
            state.tool_preview_parent_id = root_frame_at_cursor(self, cursor_pos)
                .filter(|_| self.active_tool != DesignTool::Frame)
                .map(ToString::to_string);
            return Some(Action::request_redraw());
        }

        if !state.is_panning {
            if cursor_in_root_selected {
                if self.active_tool == DesignTool::Move {
                    let mut initial_positions = Vec::new();
                    for id in self.selected_ids {
                        if let Some(el) = find_element_by_id(&self.doc.children, id) {
                            initial_positions.push((id.clone(), Point::new(el.x, el.y)));
                        }
                    }
                    if !initial_positions.is_empty() {
                        state.moving_elements = Some((initial_positions, cursor_pos, false));
                    }
                }
                return Some(Action::request_redraw());
            }

            if let Some(hit) = self.frame_header_hit(cursor_pos) {
                return Some(match hit {
                    FrameHeaderHit::Fit { id, .. } => Action::publish(Message::Design(
                        DesignMessage::FitToElement(id.to_string()),
                    )),
                    FrameHeaderHit::Title { id, .. } => Action::publish(Message::Design(
                        DesignMessage::ElementSelected(id.to_string()),
                    )),
                });
            }

            let world_x = (cursor_pos.x - self.pan.x) / self.zoom;
            let world_y = (cursor_pos.y - self.pan.y) / self.zoom;

            if let Some(hit_id) = hit_test(&self.doc.children, self.doc.as_ref(), world_x, world_y)
            {
                if let Some(real_id) = hit_id.strip_suffix(":fit") {
                    return Some(Action::publish(Message::Design(DesignMessage::FitToElement(
                        real_id.to_string(),
                    ))));
                }

                if let Some(el) = find_element_by_id(&self.doc.children, &hit_id)
                    && el.kind.eq_ignore_ascii_case("tailwind")
                    && let Some(content) = &el.content
                    && let Some(rect) =
                        get_element_screen_bounds(self.doc.as_ref(), &hit_id, self.pan, self.zoom)
                {
                    let nodes = crate::app::views::design::canvas::tailwind::parse_html(content);
                    if let Some(path) =
                        crate::app::views::design::canvas::tailwind::renderer::hit_test_path(
                            &nodes,
                            Rectangle {
                                x: rect.x,
                                y: rect.y,
                                width: rect.width,
                                height: rect.height,
                            },
                            self.zoom,
                            cursor_pos,
                        )
                    {
                        if self.active_tool == DesignTool::Move {
                            let is_selected = self.selected_ids.contains(&hit_id);
                            if !is_selected {
                                let mut initial_positions = Vec::new();
                                initial_positions.push((hit_id.clone(), Point::new(el.x, el.y)));
                                state.moving_elements =
                                    Some((initial_positions, cursor_pos, false));
                            } else {
                                let mut initial_positions = Vec::new();
                                for id in self.selected_ids {
                                    if let Some(el) = find_element_by_id(&self.doc.children, id) {
                                        initial_positions
                                            .push((id.clone(), Point::new(el.x, el.y)));
                                    }
                                }
                                state.moving_elements =
                                    Some((initial_positions, cursor_pos, false));
                            }
                        }
                        return Some(Action::publish(Message::Design(
                            DesignMessage::SelectTailwindNode(hit_id, path),
                        )));
                    }
                }

                if self.active_tool == DesignTool::Move {
                    let is_selected = self.selected_ids.contains(&hit_id);

                    if !is_selected {
                        let mut initial_positions = Vec::new();
                        if let Some(el) = find_element_by_id(&self.doc.children, &hit_id) {
                            initial_positions.push((hit_id.clone(), Point::new(el.x, el.y)));
                        }
                        state.moving_elements = Some((initial_positions, cursor_pos, false));
                        return Some(Action::publish(Message::Design(
                            DesignMessage::ElementSelected(hit_id),
                        )));
                    } else {
                        let mut initial_positions = Vec::new();
                        for id in self.selected_ids {
                            if let Some(el) = find_element_by_id(&self.doc.children, id) {
                                initial_positions.push((id.clone(), Point::new(el.x, el.y)));
                            }
                        }
                        state.moving_elements = Some((initial_positions, cursor_pos, false));
                        return Some(Action::request_redraw());
                    }
                } else {
                    return Some(Action::publish(Message::Design(DesignMessage::ElementSelected(
                        hit_id,
                    ))));
                }
            } else {
                if self.active_tool == DesignTool::Move {
                    state.selection_box_start = Some(cursor_pos);
                    return Some(Action::request_redraw());
                }
                return Some(Action::publish(Message::Design(DesignMessage::EditSubmit)));
            }
        }

        None
    }
}

#[cfg(test)]
#[path = "pressed_tests.rs"]
mod pressed_tests;
