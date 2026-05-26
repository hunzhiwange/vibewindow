use iced::{Color, Point, Size, Vector, widget::canvas::Cache};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    RotateTopLeft,
    RotateTopRight,
    RotateBottomLeft,
    RotateBottomRight,
}

#[derive(Default)]
pub struct DesignCanvasState {
    pub is_panning: bool,
    pub last_cursor_pos: Option<Point>,
    pub hovered_id: Option<String>,
    pub hovered_tailwind_selection: Option<(String, Vec<usize>)>,
    pub brush_points_world: Vec<Point>,
    pub brush_erasing: bool,
    pub brush_erase_dirty: bool,
    pub tool_preview_start: Option<Point>,
    pub tool_preview_current: Option<Point>,
    pub tool_preview_parent_id: Option<String>,
    pub resizing: Option<(String, Handle, iced::Rectangle)>,
    pub rotating: Option<(String, f32, f32)>,
    pub drag_start: Option<Point>,
    pub last_click: Option<(Instant, Point)>,
    // For box selection
    pub selection_box_start: Option<Point>,
    // For moving multiple elements: IDs, initial top-left positions, start cursor, moved flag
    pub moving_elements: Option<(Vec<(String, Point)>, Point, bool)>,
    // Current candidate frame id for reparenting while dragging
    pub drop_target_frame_id: Option<String>,
    pub mesh_drag: Option<MeshDragState>,
    pub selected_mesh_handle: Option<SelectedMeshHandle>,
    pub overlay_cache: Cache,
}

#[derive(Debug, Clone)]
pub struct SelectedMeshHandle {
    pub element_id: String,
    pub fill_index: usize,
    pub point_index: usize,
    pub handle_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshDragKind {
    Point,
    Handle(usize),
}

#[derive(Debug, Clone)]
pub struct MeshDragState {
    pub element_id: String,
    pub fill_index: usize,
    pub point_index: usize,
    pub kind: MeshDragKind,
    pub has_moved: bool,
    pub start_cursor_u: f64,
    pub start_cursor_v: f64,
    pub start_point_x: f64,
    pub start_point_y: f64,
    pub start_handles: [f64; 8],
}

#[derive(Clone, Copy)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Copy)]
pub struct ShadowSpec {
    pub color: Color,
    pub offset: Vector,
    pub blur: f32,
    pub spread: f32,
}

#[derive(Clone, Copy)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub enum AlignMode {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

pub struct ComputedLayout {
    pub offset: Vector,
    pub size: Size,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
