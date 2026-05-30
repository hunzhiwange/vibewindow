use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理光标移动事件
    pub(in super::super) fn handle_cursor_moved(
        &self,
        state: &mut DesignCanvasState,
        cursor_pos: Point,
    ) -> Option<Action<Message>> {
        if self.hover_disabled {
            if state.hovered_id.is_some() || state.hovered_tailwind_selection.is_some() {
                state.hovered_id = None;
                state.hovered_tailwind_selection = None;
                return Some(Action::request_redraw());
            }
            return None;
        }

        if self.active_tool == DesignTool::Pen && !state.brush_points_world.is_empty() {
            let point = Point::new(
                (cursor_pos.x - self.pan.x) / self.zoom,
                (cursor_pos.y - self.pan.y) / self.zoom,
            );
            let should_add = state
                .brush_points_world
                .last()
                .copied()
                .map(|last| {
                    let dx = point.x - last.x;
                    let dy = point.y - last.y;
                    dx * dx + dy * dy > (1.0 / self.zoom.max(0.0001)).powi(2)
                })
                .unwrap_or(true);
            if should_add {
                state.brush_points_world.push(point);
                return Some(Action::request_redraw());
            }
            return None;
        }

        if self.active_tool == DesignTool::Eraser {
            if state.brush_erasing {
                state.brush_erase_dirty = true;
                return Some(Action::publish(Message::Design(DesignMessage::EraseBrushAt(
                    Point::new(
                        (cursor_pos.x - self.pan.x) / self.zoom,
                        (cursor_pos.y - self.pan.y) / self.zoom,
                    ),
                    ERASER_RADIUS_PX / self.zoom.max(0.0001),
                ))));
            }

            if state.hovered_id.is_some() || state.hovered_tailwind_selection.is_some() {
                state.hovered_id = None;
                state.hovered_tailwind_selection = None;
            }
            return Some(Action::request_redraw());
        }

        if tool_supports_drag_preview(self.active_tool) && state.tool_preview_start.is_some() {
            state.tool_preview_current = Some(cursor_pos);
            state.tool_preview_parent_id = root_frame_at_cursor(self, cursor_pos)
                .filter(|_| self.active_tool != DesignTool::Frame)
                .map(ToString::to_string);
            return Some(Action::request_redraw());
        }

        if let Some(drag) = state.mesh_drag.as_mut() {
            let element_id = drag.element_id.clone();
            let fill_index = drag.fill_index;
            if let Some(el) = find_element_by_id(&self.doc.children, &element_id) {
                let Some(rect) =
                    get_element_screen_bounds(self.doc.as_ref(), &element_id, self.pan, self.zoom)
                else {
                    return None;
                };

                let rotation = el.rotation.unwrap_or(0.0);
                let (check_x, check_y) = if rotation != 0.0 {
                    let cx = rect.x + rect.width / 2.0;
                    let cy = rect.y + rect.height / 2.0;
                    rotate_point(cursor_pos.x, cursor_pos.y, cx, cy, -rotation.to_radians())
                } else {
                    (cursor_pos.x, cursor_pos.y)
                };

                let mut fills = mesh::parse_fill_items(&el.fill);
                if let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get_mut(fill_index) {
                    m.normalize();
                    if mesh::update_mesh_drag(m, drag, rect, check_x, check_y) {
                        drag.has_moved = true;
                        return Some(Action::publish(Message::Design(
                            DesignMessage::PropertyUpdateTransient(
                                element_id,
                                "fill".to_string(),
                                serde_json::json!(fills),
                            ),
                        )));
                    }
                }
            }
        }

        if state.moving_elements.is_some() {
            let (items, start) = {
                let Some((items, start, has_moved)) = state.moving_elements.as_mut() else {
                    return None;
                };
                if !*has_moved {
                    if cursor_pos.distance(*start) < 2.0 {
                        return None;
                    }
                    *has_moved = true;
                }
                (items.clone(), *start)
            };
            let dx = (cursor_pos.x - start.x) / self.zoom;
            let dy = (cursor_pos.y - start.y) / self.zoom;

            let mut updates = Vec::new();
            for (id, initial_pos) in &items {
                let nx = initial_pos.x + dx;
                let ny = initial_pos.y + dy;
                updates.push((
                    id.to_string(),
                    vec![
                        (
                            "x".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(nx as f64).unwrap(),
                            ),
                        ),
                        (
                            "y".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(ny as f64).unwrap(),
                            ),
                        ),
                    ],
                ));
            }

            fn collect_drop_target_frames(
                elements: &[DesignElement],
                doc: &crate::app::views::design::models::DesignDoc,
                pan: Vector,
                zoom: f32,
                cursor_pos: Point,
                moving_ids: &[String],
                depth: usize,
                candidates: &mut Vec<(String, usize, f32)>,
            ) {
                for element in elements {
                    if moving_ids.contains(&element.id) {
                        continue;
                    }

                    if element.kind.eq_ignore_ascii_case("frame")
                        && let Some(rect) = get_element_screen_bounds(doc, &element.id, pan, zoom)
                        && rect.contains(cursor_pos)
                    {
                        let area = rect.width * rect.height;
                        candidates.push((element.id.clone(), depth, area));
                    }

                    collect_drop_target_frames(
                        &element.children,
                        doc,
                        pan,
                        zoom,
                        cursor_pos,
                        moving_ids,
                        depth + 1,
                        candidates,
                    );
                }
            }

            let moving_ids: Vec<String> = items.iter().map(|(id, _)| id.clone()).collect();
            let mut candidates: Vec<(String, usize, f32)> = Vec::new();
            collect_drop_target_frames(
                &self.doc.children,
                self.doc.as_ref(),
                self.pan,
                self.zoom,
                cursor_pos,
                &moving_ids,
                0,
                &mut candidates,
            );
            let candidate_frame = candidates
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.total_cmp(&a.2)));
            state.drop_target_frame_id = candidate_frame.map(|(id, _, _)| id);

            return Some(Action::publish(Message::Design(
                DesignMessage::BatchPropertiesUpdateTransient(updates),
            )));
        }

        if let Some((id, initial_rot, start_angle)) = &state.rotating
            && let Some(rect) =
                get_element_screen_bounds(self.doc.as_ref(), id, self.pan, self.zoom)
        {
            let center_x = rect.x + rect.width / 2.0;
            let center_y = rect.y + rect.height / 2.0;
            let current_angle = (cursor_pos.y - center_y).atan2(cursor_pos.x - center_x);
            let delta = current_angle - start_angle;
            let delta_deg = delta.to_degrees();
            let new_rot = initial_rot + delta_deg;

            return Some(Action::publish(Message::Design(DesignMessage::PropertyUpdateTransient(
                id.clone(),
                "rotation".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(new_rot as f64).unwrap()),
            ))));
        }

        if let Some((id, handle, initial_bounds)) = &state.resizing
            && let Some(start) = state.drag_start
        {
            let dx = (cursor_pos.x - start.x) / self.zoom;
            let dy = (cursor_pos.y - start.y) / self.zoom;

            let (ix, iy, iw, ih) =
                (initial_bounds.x, initial_bounds.y, initial_bounds.width, initial_bounds.height);

            let (nx, ny, nw, nh) = match handle {
                Handle::Right => (ix, iy, iw + dx, ih),
                Handle::Bottom => (ix, iy, iw, ih + dy),
                Handle::BottomRight => (ix, iy, iw + dx, ih + dy),
                Handle::Left => (ix + dx, iy, iw - dx, ih),
                Handle::Top => (ix, iy + dy, iw, ih - dy),
                Handle::TopLeft => (ix + dx, iy + dy, iw - dx, ih - dy),
                Handle::TopRight => (ix, iy + dy, iw + dx, ih - dy),
                Handle::BottomLeft => (ix + dx, iy, iw - dx, ih + dy),
                _ => (ix, iy, iw, ih),
            };

            let nw = nw.max(1.0);
            let nh = nh.max(1.0);

            let mut updates = Vec::new();
            updates.push((
                "x".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(nx as f64).unwrap()),
            ));
            updates.push((
                "y".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(ny as f64).unwrap()),
            ));
            updates.push((
                "width".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(nw as f64).unwrap()),
            ));
            updates.push((
                "height".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(nh as f64).unwrap()),
            ));

            return Some(Action::publish(Message::Design(
                DesignMessage::PropertiesUpdateTransient(id.clone(), updates),
            )));
        }

        if state.is_panning
            && let Some(last_pos) = state.last_cursor_pos
        {
            let delta = cursor_pos - last_pos;
            let new_pan = self.pan + delta;
            state.last_cursor_pos = Some(cursor_pos);
            return Some(Action::publish(Message::Design(DesignMessage::Pan(new_pan))));
        }

        if state.selection_box_start.is_some() {
            return Some(Action::request_redraw());
        }

        if state.resizing.is_some()
            || state.rotating.is_some()
            || state.mesh_drag.is_some()
            || state.moving_elements.is_some()
        {
            if state.hovered_id.is_some() || state.hovered_tailwind_selection.is_some() {
                state.hovered_id = None;
                state.hovered_tailwind_selection = None;
                return Some(Action::request_redraw());
            }
            return None;
        }

        let world_x = (cursor_pos.x - self.pan.x) / self.zoom;
        let world_y = (cursor_pos.y - self.pan.y) / self.zoom;
        let raw_hit = hit_test(&self.doc.children, self.doc.as_ref(), world_x, world_y);

        let (hovered_id, hovered_tailwind_selection) = if let Some(hit_id) = raw_hit {
            let base = if let Some(real) = hit_id.strip_suffix(":fit") { real } else { &hit_id };

            let hovered_tailwind_selection = if let Some(el) =
                find_element_by_id(&self.doc.children, base)
                && el.kind.eq_ignore_ascii_case("tailwind")
                && let Some(content) = &el.content
                && let Some(rect) =
                    get_element_screen_bounds(self.doc.as_ref(), base, self.pan, self.zoom)
            {
                let nodes = crate::app::views::design::canvas::tailwind::parse_html(content);
                crate::app::views::design::canvas::tailwind::renderer::hit_test_path(
                    &nodes,
                    Rectangle { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
                    self.zoom,
                    cursor_pos,
                )
                .map(|path| (base.to_string(), path))
            } else {
                None
            };

            let is_reusable =
                self.doc.find_element(base).is_some_and(|el| el.reusable == Some(true));
            if is_reusable {
                (Some(base.to_string()), hovered_tailwind_selection)
            } else if let Some(path) = self.doc.find_path_to_element(base) {
                let mut mapped = base.to_string();
                for ancestor in path.iter().rev() {
                    if self.doc.find_element(ancestor).is_some_and(|el| el.reusable == Some(true)) {
                        mapped = ancestor.clone();
                        break;
                    }
                }
                (Some(mapped), hovered_tailwind_selection)
            } else {
                (Some(base.to_string()), hovered_tailwind_selection)
            }
        } else {
            (None, None)
        };

        if state.hovered_id != hovered_id
            || state.hovered_tailwind_selection != hovered_tailwind_selection
        {
            state.hovered_id = hovered_id;
            state.hovered_tailwind_selection = hovered_tailwind_selection;
            return Some(Action::request_redraw());
        }

        None
    }
}

#[cfg(test)]
#[path = "moved_tests.rs"]
mod moved_tests;
