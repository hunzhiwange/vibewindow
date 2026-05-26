//! fullscreen layout slots 计算。
//!
//! 当前阶段只为 tui_v2 提供稳定的五段式宿主，不掺杂 message row、virtual list 或
//! overlay manager 的更细语义：
//! - `header`
//! - `scrollable`
//! - `bottom_float`
//! - `bottom`
//! - `modal`

use ratatui::layout::Rect;

/// fullscreen 布局的稳定 slots。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct FullscreenLayoutSlots {
    pub(crate) header: Rect,
    pub(crate) scrollable: Rect,
    pub(crate) project_context: Option<Rect>,
    pub(crate) modified_files: Option<Rect>,
    pub(crate) bottom: Rect,
    pub(crate) bottom_float: Option<Rect>,
    pub(crate) modal: Option<Rect>,
}

impl FullscreenLayoutSlots {
    /// 按当前 scrollable 容器可用高度粗略估算消息视口容量。
    ///
    /// 当前 fullscreen skeleton 仍按“每条消息至少占一行”近似，
    /// 后续虚拟列表和高度缓存会在 Phase 4 继续替换这层估算。
    pub(crate) fn message_viewport_capacity(&self) -> usize {
        self.scrollable.height.saturating_sub(2) as usize
    }
}

/// fullscreen 布局的静态参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FullscreenLayoutConfig {
    pub(crate) header_height: u16,
    pub(crate) bottom_height: u16,
    pub(crate) bottom_float_height: u16,
    pub(crate) modal_width_pct: u16,
    pub(crate) modal_height_pct: u16,
}

impl Default for FullscreenLayoutConfig {
    fn default() -> Self {
        Self {
            header_height: 3,
            bottom_height: 7,
            bottom_float_height: 3,
            modal_width_pct: 70,
            modal_height_pct: 45,
        }
    }
}

/// 计算 fullscreen slots。
pub(crate) fn compute_fullscreen_layout(
    area: Rect,
    has_bottom_float: bool,
    has_modal: bool,
    prompt_extra_rows: u16,
) -> FullscreenLayoutSlots {
    let config = FullscreenLayoutConfig::default();
    if area.width == 0 || area.height == 0 {
        return FullscreenLayoutSlots::default();
    }

    let (header_height, scroll_height, bottom_float_height, bottom_height) =
        resolve_section_heights(area.height, has_bottom_float, config, prompt_extra_rows);

    let header = Rect::new(area.x, area.y, area.width, header_height);
    let scrollable_area = Rect::new(
        area.x,
        area.y.saturating_add(header_height),
        area.width,
        scroll_height,
    );
    let (scrollable, project_context, modified_files) = split_scrollable_area(scrollable_area);

    let bottom_float = if bottom_float_height == 0 {
        None
    } else {
        Some(Rect::new(
            area.x,
            scrollable.y.saturating_add(scrollable.height),
            area.width,
            bottom_float_height,
        ))
    };

    let bottom_y = bottom_float
        .map(|rect| rect.y.saturating_add(rect.height))
        .unwrap_or_else(|| scrollable.y.saturating_add(scrollable.height));
    let bottom = Rect::new(area.x, bottom_y, area.width, bottom_height);

    FullscreenLayoutSlots {
        header,
        scrollable,
        project_context,
        modified_files,
        bottom,
        bottom_float,
        modal: has_modal.then(|| centered_rect(area, config.modal_width_pct, config.modal_height_pct, 28, 8)),
    }
}

fn split_scrollable_area(area: Rect) -> (Rect, Option<Rect>, Option<Rect>) {
    let Some(sidebar_width) = resolve_sidebar_width(area.width) else {
        return (area, None, None);
    };

    // 侧栏需要同时容纳两个 panel；高度偏矮时优先把空间还给 transcript。
    if area.height < 16 {
        return (area, None, None);
    }

    let transcript_width = area.width.saturating_sub(sidebar_width);
    if transcript_width < 32 {
        return (area, None, None);
    }

    let scrollable = Rect::new(area.x, area.y, transcript_width, area.height);
    let sidebar = Rect::new(
        area.x.saturating_add(transcript_width),
        area.y,
        sidebar_width,
        area.height,
    );
    let project_height = resolve_project_context_height(sidebar.height);
    let project_context = Rect::new(sidebar.x, sidebar.y, sidebar.width, project_height);
    let modified_files_height = sidebar.height.saturating_sub(project_height);
    let modified_files = (modified_files_height > 0).then(|| {
        Rect::new(
            sidebar.x,
            sidebar.y.saturating_add(project_height),
            sidebar.width,
            modified_files_height,
        )
    });

    (scrollable, Some(project_context), modified_files)
}

fn resolve_sidebar_width(total_width: u16) -> Option<u16> {
    if total_width < 88 {
        return None;
    }

    let desired = u16_from_u32_saturating((u32::from(total_width) * 28) / 100);
    let sidebar_width = desired.clamp(28, 36);
    Some(sidebar_width.min(total_width.saturating_sub(32)))
}

fn resolve_project_context_height(sidebar_height: u16) -> u16 {
    let desired = u16_from_u32_saturating((u32::from(sidebar_height) * 2) / 5);
    desired.max(8).min(sidebar_height.saturating_sub(4).max(1))
}

fn resolve_section_heights(
    area_height: u16,
    has_bottom_float: bool,
    config: FullscreenLayoutConfig,
    prompt_extra_rows: u16,
) -> (u16, u16, u16, u16) {
    // 24 行附近优先保证输入区的可编辑高度，状态 footer 让位给主消息区与 prompt host。
    let is_compact_height = area_height <= 24;
    let mut header = if is_compact_height {
        config.header_height.saturating_sub(1).max(2)
    } else {
        config.header_height
    }
    .min(area_height.max(1));
    let mut bottom = if is_compact_height {
        config.bottom_height.max(8)
    } else {
        config.bottom_height
    }
    .saturating_add(prompt_extra_rows)
    .min(area_height.max(1));
    let mut bottom_float = if has_bottom_float && !is_compact_height {
        config.bottom_float_height.min(area_height.max(1))
    } else {
        0
    };
    let min_scroll = 1;

    let mut overflow = header
        .saturating_add(bottom)
        .saturating_add(bottom_float)
        .saturating_add(min_scroll)
        .saturating_sub(area_height);

    if overflow > 0 {
        let reduce_bottom = overflow.min(bottom.saturating_sub(3));
        bottom = bottom.saturating_sub(reduce_bottom);
        overflow = overflow.saturating_sub(reduce_bottom);
    }

    if overflow > 0 {
        let reduce_header = overflow.min(header.saturating_sub(2));
        header = header.saturating_sub(reduce_header);
        overflow = overflow.saturating_sub(reduce_header);
    }

    if overflow > 0 {
        let reduce_float = overflow.min(bottom_float);
        bottom_float = bottom_float.saturating_sub(reduce_float);
        overflow = overflow.saturating_sub(reduce_float);
    }

    if overflow > 0 {
        let reduce_bottom_tail = overflow.min(bottom.saturating_sub(1));
        bottom = bottom.saturating_sub(reduce_bottom_tail);
        overflow = overflow.saturating_sub(reduce_bottom_tail);
    }

    if overflow > 0 {
        header = header.saturating_sub(overflow.min(header.saturating_sub(1)));
    }

    let scroll = area_height.saturating_sub(header.saturating_add(bottom).saturating_add(bottom_float));
    (header, scroll.max(1), bottom_float, bottom)
}

fn centered_rect(
    area: Rect,
    width_pct: u16,
    height_pct: u16,
    min_width: u16,
    min_height: u16,
) -> Rect {
    if area.width == 0 || area.height == 0 {
        return Rect::default();
    }

    let target_width = u16_from_u32_saturating((u32::from(area.width) * u32::from(width_pct)) / 100);
    let target_height =
        u16_from_u32_saturating((u32::from(area.height) * u32::from(height_pct)) / 100);
    let width = target_width.max(min_width).min(area.width.saturating_sub(2).max(1));
    let height = target_height.max(min_height).min(area.height.saturating_sub(2).max(1));
    let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
    let y = area.y.saturating_add(area.height.saturating_sub(height) / 2);

    Rect::new(x, y, width, height)
}

fn u16_from_u32_saturating(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
