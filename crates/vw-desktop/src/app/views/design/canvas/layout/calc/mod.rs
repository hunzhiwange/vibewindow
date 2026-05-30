use iced::{Size, Vector};

use crate::app::views::design::{
    canvas::{
        parse::{
            measure_text_width_with_font, parse_font_size, parse_line_height, parse_size,
            resolve_font_family, wrap_text_lines_with_font,
        },
        types::{AlignMode, ComputedLayout, LayoutDirection},
        utils::{resolve_ref_instance, theme_mode_for_element},
    },
    models::{DesignDoc, DesignElement},
};

use super::parse::{parse_align_mode, parse_gap, parse_layout, parse_padding};

fn is_fit_content(v: &Option<serde_json::Value>) -> bool {
    if let Some(serde_json::Value::String(s)) = v {
        s == "fit_content" || s.starts_with("fit_content(")
    } else {
        false
    }
}

fn is_fill_container(v: &Option<serde_json::Value>) -> bool {
    if let Some(serde_json::Value::String(s)) = v {
        s == "fill_container" || s.starts_with("fill_container")
    } else {
        false
    }
}

fn fill_container_weight(v: &Option<serde_json::Value>) -> Option<f32> {
    let Some(serde_json::Value::String(s)) = v else {
        return None;
    };
    if s == "fill_container" || s.starts_with("fill_container") { Some(1.0) } else { None }
}

fn fill_container_fallback_size(v: &Option<serde_json::Value>) -> Option<f32> {
    let Some(serde_json::Value::String(s)) = v else {
        return None;
    };
    let Some(inner) = s.strip_prefix("fill_container(") else {
        return None;
    };
    let Some(inner) = inner.strip_suffix(')') else {
        return None;
    };
    let inner = inner.trim().trim_end_matches("px").trim();
    inner.parse::<f32>().ok().filter(|v| *v > 0.0)
}

fn fit_content_min_size(v: &Option<serde_json::Value>) -> Option<f32> {
    let Some(serde_json::Value::String(s)) = v else {
        return None;
    };
    let Some(inner) = s.strip_prefix("fit_content(") else {
        return None;
    };
    let Some(inner) = inner.strip_suffix(')') else {
        return None;
    };
    let inner = inner.trim().trim_end_matches("px").trim();
    inner.parse::<f32>().ok().filter(|v| *v >= 0.0)
}

fn guess_direction_from_children(children: &[DesignElement]) -> LayoutDirection {
    if children.len() < 2 {
        return LayoutDirection::Vertical;
    }
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for c in children {
        min_x = min_x.min(c.x);
        max_x = max_x.max(c.x);
        min_y = min_y.min(c.y);
        max_y = max_y.max(c.y);
    }

    let range_x = (max_x - min_x).abs();
    let range_y = (max_y - min_y).abs();
    if range_x >= range_y { LayoutDirection::Horizontal } else { LayoutDirection::Vertical }
}

fn infer_container_layout_direction(element: &DesignElement) -> Option<LayoutDirection> {
    if element.layout.as_deref() == Some("none") {
        return None;
    }
    parse_layout(&element.layout).or_else(|| {
        if element.justify_content.is_some()
            || element.align_items.is_some()
            || element.gap.is_some()
        {
            Some(guess_direction_from_children(&element.children))
        } else {
            None
        }
    })
}

pub fn resolve_element_size(
    element: &DesignElement,
    _parent_size: Option<Size>,
    doc: &DesignDoc,
    inherited_theme_mode: Option<&str>,
) -> Size {
    let theme_mode = theme_mode_for_element(doc, element, inherited_theme_mode);
    let w_opt = parse_size(&element.width, &doc.variables, theme_mode);
    let h_opt = parse_size(&element.height, &doc.variables, theme_mode);

    let is_fill_w = is_fill_container(&element.width) || element.fill_width == Some(true);
    let is_fill_h = is_fill_container(&element.height) || element.fill_height == Some(true);
    let is_fit_w = is_fit_content(&element.width) || element.hug_width == Some(true);
    let is_fit_h = is_fit_content(&element.height) || element.hug_height == Some(true);
    let fit_w_min = fit_content_min_size(&element.width);
    let fit_h_min = fit_content_min_size(&element.height);

    let mut width = w_opt.unwrap_or_else(|| {
        if is_fill_w && let Some(parent) = _parent_size {
            if let Some(max_w) = fill_container_fallback_size(&element.width) {
                return parent.width.min(max_w);
            }
            return parent.width;
        }
        if is_fill_w
            && _parent_size.is_none()
            && let Some(fallback) = fill_container_fallback_size(&element.width)
        {
            return fallback;
        }
        if is_fit_w {
            return fit_w_min.unwrap_or(0.0);
        }
        100.0
    });
    let mut height = h_opt.unwrap_or_else(|| {
        if is_fill_h && let Some(parent) = _parent_size {
            if let Some(max_h) = fill_container_fallback_size(&element.height) {
                return parent.height.min(max_h);
            }
            return parent.height;
        }
        if is_fill_h
            && _parent_size.is_none()
            && let Some(fallback) = fill_container_fallback_size(&element.height)
        {
            return fallback;
        }
        if is_fit_h {
            return fit_h_min.unwrap_or(0.0);
        }
        100.0
    });

    if element.kind == "ref"
        && let Some(inst) = resolve_ref_instance(element, &doc.children, None)
    {
        let inst_size = resolve_element_size(&inst, _parent_size, doc, theme_mode);
        let width_specified = element.width.is_some() || is_fill_w;
        let height_specified = element.height.is_some() || is_fill_h;
        if is_fit_w {
            width = inst_size.width;
        } else if !width_specified {
            width = inst_size.width;
        }
        if is_fit_h {
            height = inst_size.height;
        } else if !height_specified {
            height = inst_size.height;
        }
        if let Some(min_w) = fit_w_min {
            width = width.max(min_w);
        }
        if let Some(min_h) = fit_h_min {
            height = height.max(min_h);
        }
        return Size::new(width, height);
    }

    // Handle fit_content for children (containers)
    if (is_fit_w || is_fit_h) && !element.children.is_empty() {
        let visible_children: Vec<&DesignElement> = element
            .children
            .iter()
            .filter(|c| c.enabled != Some(false) && c.visible != Some(false))
            .collect();
        if visible_children.is_empty() {
            return Size::new(width, height);
        }

        let layout = parse_layout(&element.layout)
            .or_else(|| {
                if let Some(l) = &element.layout
                    && l == "none"
                {
                    return None;
                }
                if element.justify_content.is_some()
                    || element.align_items.is_some()
                    || element.gap.is_some()
                {
                    Some(LayoutDirection::Horizontal)
                } else {
                    None
                }
            })
            .unwrap_or(LayoutDirection::Vertical);

        let gap = parse_gap(&element.gap, &doc.variables, theme_mode);
        let padding = parse_padding(&element.padding, &doc.variables, theme_mode);

        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        let mut sum_w: f32 = 0.0;
        let mut sum_h: f32 = 0.0;
        let count = visible_children.len();

        for child in &visible_children {
            let child_size = resolve_element_size(child, _parent_size, doc, theme_mode);
            max_w = max_w.max(child_size.width);
            max_h = max_h.max(child_size.height);
            sum_w += child_size.width;
            sum_h += child_size.height;
        }

        let total_gap = if count > 0 { gap * ((count as i32 - 1).max(0) as f32) } else { 0.0 };

        if is_fit_w {
            width = match layout {
                LayoutDirection::Horizontal => sum_w + total_gap,
                LayoutDirection::Vertical => max_w,
            } + padding.left
                + padding.right;
        }

        if is_fit_h {
            height = match layout {
                LayoutDirection::Horizontal => max_h,
                LayoutDirection::Vertical => sum_h + total_gap,
            } + padding.top
                + padding.bottom;
        }
    }

    if !element.children.is_empty()
        && (w_opt.is_none() || h_opt.is_none())
        && let Some(layout) = infer_container_layout_direction(element)
    {
        let visible_children: Vec<&DesignElement> = element
            .children
            .iter()
            .filter(|c| c.enabled != Some(false) && c.visible != Some(false))
            .collect();
        if visible_children.is_empty() {
            return Size::new(width, height);
        }

        let can_hug_w = w_opt.is_none() && !is_fill_w && !is_fit_w;
        let can_hug_h = h_opt.is_none() && !is_fill_h && !is_fit_h;

        let gap = parse_gap(&element.gap, &doc.variables, theme_mode);
        let padding = parse_padding(&element.padding, &doc.variables, theme_mode);
        let child_parent_size = Some(Size::new(
            (width - padding.left - padding.right).max(0.0),
            (height - padding.top - padding.bottom).max(0.0),
        ));
        let count = visible_children.len();
        let total_gap = if count > 0 { gap * ((count as i32 - 1).max(0) as f32) } else { 0.0 };

        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        let mut sum_w: f32 = 0.0;
        let mut sum_h: f32 = 0.0;

        let mut any_child_fill_w = false;
        let mut any_child_fill_h = false;

        for child in &visible_children {
            any_child_fill_w |= is_fill_container(&child.width) || child.fill_width == Some(true);
            any_child_fill_h |= is_fill_container(&child.height) || child.fill_height == Some(true);

            let child_size = resolve_element_size(child, child_parent_size, doc, theme_mode);
            max_w = max_w.max(child_size.width);
            max_h = max_h.max(child_size.height);
            sum_w += child_size.width;
            sum_h += child_size.height;
        }

        if can_hug_w {
            let hug_w_ok = match layout {
                LayoutDirection::Horizontal => !any_child_fill_w,
                LayoutDirection::Vertical => !any_child_fill_w,
            };
            if hug_w_ok {
                width = match layout {
                    LayoutDirection::Horizontal => sum_w + total_gap,
                    LayoutDirection::Vertical => max_w,
                } + padding.left
                    + padding.right;
            }
        }

        if can_hug_h {
            let hug_h_ok = match layout {
                LayoutDirection::Horizontal => !any_child_fill_h,
                LayoutDirection::Vertical => !any_child_fill_h,
            };
            if hug_h_ok {
                height = match layout {
                    LayoutDirection::Horizontal => max_h,
                    LayoutDirection::Vertical => sum_h + total_gap,
                } + padding.top
                    + padding.bottom;
            }
        }
    }

    // Handle content (text) measurement - covers fit_content on text too
    if element.content.is_some()
        && (w_opt.is_none() || h_opt.is_none() || is_fit_w || is_fit_h)
        && let Some(content) = &element.content
    {
        let font_size = parse_font_size(&element.font_size, &doc.variables, theme_mode);
        let line_height_val =
            parse_line_height(&element.line_height, font_size, &doc.variables, theme_mode);
        let padding = parse_padding(&element.padding, &doc.variables, theme_mode);
        let font_family = resolve_font_family(&element.font_family, &doc.variables, theme_mode);

        let text_growth =
            element.text_growth.as_deref().unwrap_or("").to_lowercase().replace('_', "-");
        let can_wrap = text_growth.starts_with("fixed-width");
        let has_reliable_wrap_width = w_opt.is_some() || (is_fill_w && _parent_size.is_some());
        let wrap_width = if can_wrap && !is_fit_w && has_reliable_wrap_width {
            (width - padding.left - padding.right).max(0.0)
        } else {
            0.0
        };

        let mut max_width: f32 = 0.0;
        let visual_line_count: usize = if wrap_width > 0.0 {
            let lines =
                wrap_text_lines_with_font(content, wrap_width, &font_family, font_size, 0.0);
            for line in &lines {
                let w = measure_text_width_with_font(line, &font_family, font_size, 0.0);
                max_width = max_width.max(w);
            }
            if lines.is_empty() && !content.is_empty() { 1 } else { lines.len().max(1) }
        } else {
            let mut count = 0usize;
            for line in content.lines() {
                count += 1;
                let w = measure_text_width_with_font(line, &font_family, font_size, 0.0);
                max_width = max_width.max(w);
            }
            if count == 0 && !content.is_empty() { 1 } else { count.max(1) }
        };

        if (w_opt.is_none() || is_fit_w) && !is_fill_w {
            width = max_width + padding.left + padding.right;
        }
        if (h_opt.is_none() || is_fit_h) && !is_fill_h {
            height = (visual_line_count as f32) * line_height_val + padding.top + padding.bottom;
        }
    }

    if let Some(min_w) = fit_w_min {
        width = width.max(min_w);
    }
    if let Some(min_h) = fit_h_min {
        height = height.max(min_h);
    }

    Size::new(width, height)
}

pub fn compute_layout(
    direction: LayoutDirection,
    children: &[DesignElement],
    container_size: Size,
    _parent: &DesignElement,
    doc: &DesignDoc,
    parent_theme_mode: Option<&str>,
) -> Vec<ComputedLayout> {
    let mut sizes: Vec<Size> = children
        .iter()
        .map(|c| resolve_element_size(c, Some(container_size), doc, parent_theme_mode))
        .collect();

    let justify = parse_align_mode(&_parent.justify_content);
    let align = parse_align_mode(&_parent.align_items);
    let gap = parse_gap(&_parent.gap, &doc.variables, parent_theme_mode);

    let count = children.len();
    let mut offsets: Vec<Vector> = vec![Vector::new(0.0, 0.0); count];
    let mut active_indices: Vec<usize> = Vec::new();
    for (i, child) in children.iter().enumerate() {
        if child.enabled == Some(false) || child.visible == Some(false) {
            sizes[i] = Size::new(0.0, 0.0);
            continue;
        }
        active_indices.push(i);
    }
    let active_count = active_indices.len();

    let (container_main, container_cross) = match direction {
        LayoutDirection::Horizontal => {
            (container_size.width.max(0.0), container_size.height.max(0.0))
        }
        LayoutDirection::Vertical => {
            (container_size.height.max(0.0), container_size.width.max(0.0))
        }
    };

    // 1. Identify fill children on Main Axis
    let mut main_fill_indices: Vec<(usize, f32)> = Vec::new();
    let mut used_main = 0.0;

    for &i in &active_indices {
        let child = &children[i];
        let fill_weight = match direction {
            LayoutDirection::Horizontal => fill_container_weight(&child.width),
            LayoutDirection::Vertical => fill_container_weight(&child.height),
        }
        .or_else(|| match direction {
            LayoutDirection::Horizontal => {
                child.fill_width.and_then(|v| if v { Some(1.0) } else { None })
            }
            LayoutDirection::Vertical => {
                child.fill_height.and_then(|v| if v { Some(1.0) } else { None })
            }
        });

        if let Some(w) = fill_weight {
            main_fill_indices.push((i, w));
        } else {
            used_main += match direction {
                LayoutDirection::Horizontal => sizes[i].width,
                LayoutDirection::Vertical => sizes[i].height,
            };
        }
    }

    let total_gap =
        if active_count > 0 { gap * ((active_count as i32 - 1).max(0) as f32) } else { 0.0 };
    used_main += total_gap;

    // 2. Distribute remaining space to fill children
    if !main_fill_indices.is_empty() {
        let remaining = (container_main - used_main).max(0.0);
        let total_weight: f32 = main_fill_indices.iter().map(|(_, w)| *w).sum::<f32>().max(1.0);
        for &(i, w) in &main_fill_indices {
            let share = remaining * (w / total_weight);
            match direction {
                LayoutDirection::Horizontal => sizes[i].width = share,
                LayoutDirection::Vertical => sizes[i].height = share,
            }
        }
    }

    // 3. Handle Cross Axis Fill (override resolve_element_size default which ignores padding)
    for &i in &active_indices {
        let child = &children[i];
        let is_fill_cross = match direction {
            LayoutDirection::Horizontal => {
                is_fill_container(&child.height) || child.fill_height == Some(true)
            }
            LayoutDirection::Vertical => {
                is_fill_container(&child.width) || child.fill_width == Some(true)
            }
        };
        if is_fill_cross {
            match direction {
                LayoutDirection::Horizontal => sizes[i].height = container_cross,
                LayoutDirection::Vertical => sizes[i].width = container_cross,
            }
        }
    }

    if let Some(AlignMode::Stretch) = align {
        for &i in &active_indices {
            let s = &mut sizes[i];
            match direction {
                LayoutDirection::Horizontal => {
                    // Only stretch if not explicitly fixed? usually stretch overrides auto, but fill_container is explicit.
                    // If element is fixed height, stretch might be ignored or respected depending on CSS impl.
                    // Here we respect stretch if not handled by fill_cross logic?
                    // Actually fill_cross logic above sets it to container_cross.
                    // If align is stretch, we also want to set it to container_cross.
                    // But if child has explicit size, stretch usually overrides it in Flexbox unless it has max/min.
                    // Let's assume AlignMode::Stretch applies to everyone unless they have specific behavior?
                    // Existing logic was applying it blindly.
                    // But we should prioritize fill logic if it exists.
                    // If fill logic ran, s.height is already container_cross.
                    s.height = container_cross;
                }
                LayoutDirection::Vertical => {
                    s.width = container_cross;
                }
            }
        }
    }

    let total_children_main: f32 = active_indices
        .iter()
        .map(|&i| match direction {
            LayoutDirection::Horizontal => sizes[i].width,
            LayoutDirection::Vertical => sizes[i].height,
        })
        .sum();
    let remaining = (container_main - total_children_main - total_gap).max(0.0);

    let (start_offset, between_gap, leading_gap) = match justify.unwrap_or(AlignMode::Start) {
        AlignMode::Start => (0.0, gap, 0.0),
        AlignMode::Center => (remaining / 2.0, gap, 0.0),
        AlignMode::End => (remaining, gap, 0.0),
        AlignMode::SpaceBetween => {
            let between = if active_count > 1 {
                (container_main - total_children_main) / ((active_count - 1) as f32)
            } else {
                0.0
            };
            (0.0, between.max(0.0), 0.0)
        }
        AlignMode::SpaceAround => {
            let slots = active_count as f32;
            let around =
                if slots > 0.0 { (container_main - total_children_main) / slots } else { 0.0 };
            (around / 2.0, around, around / 2.0)
        }
        AlignMode::SpaceEvenly => {
            let slots = (active_count + 1) as f32;
            let evenly =
                if slots > 0.0 { (container_main - total_children_main) / slots } else { 0.0 };
            (evenly, evenly, evenly)
        }
        AlignMode::Stretch => (0.0, gap, 0.0),
    };

    let mut cursor_main = start_offset + leading_gap;

    for &i in &active_indices {
        let s = &sizes[i];
        let cross_offset = match align.unwrap_or(AlignMode::Start) {
            AlignMode::Start | AlignMode::Stretch => 0.0,
            AlignMode::Center => {
                let child_cross = match direction {
                    LayoutDirection::Horizontal => s.height,
                    LayoutDirection::Vertical => s.width,
                };
                (container_cross - child_cross).max(0.0) / 2.0
            }
            AlignMode::End => {
                container_cross
                    - match direction {
                        LayoutDirection::Horizontal => s.height,
                        LayoutDirection::Vertical => s.width,
                    }
            }
            _ => 0.0,
        };

        offsets[i] = match direction {
            LayoutDirection::Horizontal => Vector::new(cursor_main, cross_offset),
            LayoutDirection::Vertical => Vector::new(cross_offset, cursor_main),
        };
        cursor_main += match direction {
            LayoutDirection::Horizontal => s.width,
            LayoutDirection::Vertical => s.height,
        } + between_gap;
    }

    children
        .iter()
        .zip(sizes)
        .zip(offsets)
        .map(|((_, size), off)| ComputedLayout { offset: off, size })
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
