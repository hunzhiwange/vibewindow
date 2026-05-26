use iced::widget::Id;
use iced::widget::canvas::Cache;
use iced::widget::text_editor::Content;
use iced::{Point, Vector};
use std::collections::HashSet;
use std::path::PathBuf;

use super::canvas::creation::{DEFAULT_BRUSH_COLOR_HEX, DEFAULT_BRUSH_WIDTH_PX};
use super::models::{DesignDoc, DesignTool, StickyNoteKind, compute_tree_metrics};
use crate::app::task::{TASK_MODEL_AUTO, TaskExecutorBackend};
use crate::app::views::design::properties::color_picker::ActiveColorPicker;

const DEFAULT_DESIGN_GENERATION_PARALLEL_PAGES: usize = 2;
const MAX_DESIGN_GENERATION_PARALLEL_PAGES: usize = 16;

pub fn sanitize_design_generation_parallel_pages(value: usize) -> usize {
    value.clamp(1, MAX_DESIGN_GENERATION_PARALLEL_PAGES)
}

fn load_design_generation_parallel_pages() -> usize {
    crate::app::config::load_app_config()
        .get("design_generation_parallel_pages")
        .and_then(|value| value.as_u64())
        .map(|value| sanitize_design_generation_parallel_pages(value as usize))
        .unwrap_or(DEFAULT_DESIGN_GENERATION_PARALLEL_PAGES)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignGenerationTheme {
    Shadcn,
    Nitro,
    Halo,
    Lunaris,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignGenerationDevice {
    #[default]
    Auto,
    DesktopWeb,
    MobileApp,
    Tablet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageImportTarget {
    Element,
    Fill { element_id: String, fill_index: usize },
}

impl DesignGenerationDevice {
    pub const ALL: [Self; 4] = [Self::Auto, Self::DesktopWeb, Self::MobileApp, Self::Tablet];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "自动识别",
            Self::DesktopWeb => "网站 / PC",
            Self::MobileApp => "移动端 / APP",
            Self::Tablet => "平板",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Auto => "根据需求文本自动判断端类型，并保留响应式约束。",
            Self::DesktopWeb => "按桌面网站宽度生成，适合官网与后台。",
            Self::MobileApp => "按移动端宽度生成，适合 APP / H5 / 小程序。",
            Self::Tablet => "按平板宽度生成，适合中屏交互布局。",
        }
    }
}

impl DesignGenerationTheme {
    pub const ALL: [Self; 4] = [Self::Shadcn, Self::Nitro, Self::Halo, Self::Lunaris];

    pub fn label(self) -> &'static str {
        match self {
            Self::Shadcn => "Shadcn 中性",
            Self::Nitro => "Nitro 企业",
            Self::Halo => "Halo 亲和",
            Self::Lunaris => "Lunaris 科技",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Shadcn => "中性、极简，适合大多数产品官网与内容站。",
            Self::Nitro => "理性、专业，适合企业官网与 B 端产品介绍。",
            Self::Halo => "明亮、圆润，适合偏消费品与转化导向页面。",
            Self::Lunaris => "暗色、科技感，适合 AI、开发者与工具产品。",
        }
    }
}

impl std::fmt::Display for DesignGenerationTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct DesignChatMessage {
    pub role: DesignChatRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct DesignChatSession {
    pub id: usize,
    pub title: String,
    pub messages: Vec<DesignChatMessage>,
    pub input: Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignGenerationStatus {
    Planned,
    Placeholder,
    Queued,
    Running,
    Generated,
    Filled,
    Failed,
    Aggregated,
}

impl DesignGenerationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "待生成",
            Self::Placeholder => "占位预览",
            Self::Queued => "排队中",
            Self::Running => "生成中",
            Self::Generated => "已生成",
            Self::Filled => "已回填",
            Self::Failed => "失败",
            Self::Aggregated => "已汇总",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignStyle {
    #[default]
    Default,
    Minimalist,
    Modern,
    Business,
    Creative,
    Retro,
    Tech,
    Elegant,
    Vibrant,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignPlannerTab {
    #[default]
    Chat,
    Tools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignSettingsTab {
    #[default]
    General,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignPlannerCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigmaProgressStage {
    Importing,
    Parsing,
}

impl FigmaProgressStage {
    pub fn title(self) -> &'static str {
        match self {
            Self::Importing => "导入 Figma",
            Self::Parsing => "解析 Figma",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FigmaProgressState {
    pub stage: FigmaProgressStage,
    pub progress: f32,
    pub current: usize,
    pub total: usize,
    pub detail: String,
}

impl FigmaProgressState {
    pub fn new(
        stage: FigmaProgressStage,
        current: usize,
        total: usize,
        detail: impl Into<String>,
    ) -> Self {
        let progress = if total == 0 { 0.0 } else { current as f32 / total as f32 }.clamp(0.0, 1.0);
        Self { stage, progress, current, total, detail: detail.into() }
    }

    pub fn percentage(&self) -> u8 {
        (self.progress * 100.0).round().clamp(0.0, 100.0) as u8
    }
}

impl DesignPlannerCorner {
    pub fn config_key(self) -> &'static str {
        match self {
            Self::TopLeft => "top_left",
            Self::TopRight => "top_right",
            Self::BottomLeft => "bottom_left",
            Self::BottomRight => "bottom_right",
        }
    }
}

impl DesignStyle {
    pub fn all() -> [DesignStyle; 10] {
        [
            DesignStyle::Default,
            DesignStyle::Minimalist,
            DesignStyle::Modern,
            DesignStyle::Business,
            DesignStyle::Creative,
            DesignStyle::Retro,
            DesignStyle::Tech,
            DesignStyle::Elegant,
            DesignStyle::Vibrant,
            DesignStyle::Dark,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DesignStyle::Default => "默认",
            DesignStyle::Minimalist => "极简",
            DesignStyle::Modern => "现代",
            DesignStyle::Business => "商务",
            DesignStyle::Creative => "创意",
            DesignStyle::Retro => "复古",
            DesignStyle::Tech => "科技",
            DesignStyle::Elegant => "优雅",
            DesignStyle::Vibrant => "活力",
            DesignStyle::Dark => "暗黑",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesignGenerationModule {
    pub module_id: String,
    pub title: String,
    pub description: String,
    pub status: DesignGenerationStatus,
    pub target_frame_id: String,
    pub target_frame_options: Vec<String>,
    pub generated_doc: Option<DesignDoc>,
    pub is_generating: bool,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DesignGenerationPage {
    pub frame_id: String,
    pub title: String,
    pub objective: String,
    pub status: DesignGenerationStatus,
    pub modules: Vec<DesignGenerationModule>,
}

#[derive(Debug, Clone)]
pub struct DesignGenerationPlan {
    pub summary: Option<String>,
    pub pages: Vec<DesignGenerationPage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPopoverType {
    Shape,
    Fill,
    Border,
    TextColor,
    ToolbarBrush,
    ToolbarShape,
    ToolbarIcon,
}

pub struct DesignState {
    pub context_popover: Option<ContextPopoverType>,
    pub canvas_context_menu_anchor: Option<Point>,
    pub paste_anchor: Option<Point>,
    pub doc: DesignDoc,
    pub file_path: Option<PathBuf>,
    pub layer_tree_metrics: (usize, u16),
    pub canvas_cache: Cache,
    pub selected_element_id: Option<String>,
    pub selected_element_ids: HashSet<String>,
    pub pan: Vector,
    pub zoom: f32,
    pub active_group_id: u32,
    pub expanded_nodes: HashSet<String>,
    pub active_tool: DesignTool,
    pub brush_color_hex: String,
    pub brush_width_px: f32,

    // Editing
    pub editing_id: Option<String>,
    pub editing_content: String,
    pub editing_editor: Content,

    // Context Editor
    pub context_editor: Content,
    pub context_element_id: Option<String>,
    pub context_expanded: bool,

    // Content Editor
    pub content_editor: Content,

    // Tailwind Editors
    pub tailwind_html_editor: Content,
    pub tailwind_node_class_editor: Content,
    pub tailwind_node_text_editor: Content,
    pub tailwind_inspector_collapsed: bool,
    pub tailwind_tree_collapsed: HashSet<String>,
    pub tailwind_tree_scroll_id: Id,

    // Selection details
    pub selected_fill_index: Option<usize>,
    pub selected_effect_index: Option<usize>,

    // UI State
    pub show_zoom_menu: bool,
    pub image_import_target: Option<ImageImportTarget>,
    pub image_import_input: String,
    pub image_import_error: Option<String>,
    pub image_import_loading: bool,
    pub sticky_note_dialog_open: bool,
    pub sticky_note_dialog_default_kind: StickyNoteKind,

    pub active_layer_menu: Option<String>,
    pub active_page_menu: Option<u32>,
    pub page_menu_anchor: Option<Point>,
    pub renaming_page_id: Option<u32>,
    pub renaming_page_name: String,
    pub current_variable_collection: Option<String>,
    pub active_variable_collection_menu: Option<String>,
    pub renaming_variable_collection: Option<String>,
    pub variable_collection_rename_value: String,
    pub confirm_delete_variable_collection: Option<String>,
    pub active_variable_theme_menu: Option<String>,
    pub renaming_variable_theme: Option<String>,
    pub variable_theme_rename_value: String,
    pub confirm_delete_variable_theme: Option<String>,
    pub active_variable_menu: Option<String>,
    pub variable_move_target_picker: Option<String>,
    pub renaming_variable: Option<String>,
    pub variable_rename_value: String,
    pub confirm_delete_variable: Option<String>,
    pub show_add_variable_menu: bool,
    pub context_shape_group_hover: Option<String>,
    pub icon_filter_query: String,
    pub toolbar_icon_family: String,
    pub toolbar_icon_name: String,
    pub toolbar_icon_family_tab: String,
    pub dragging_layer: Option<String>,
    pub drag_target_layer: Option<String>,
    pub hovered_layer_id: Option<String>,
    pub active_color_picker: Option<ActiveColorPicker>,
    pub show_element_html_preview: bool,
    pub element_html_preview_editor: Content,
    pub design_help_text: Option<String>,
    pub tailwind_filter_query: String,
    pub font_filter_query: String,
    pub new_group_name: String,
    pub tailwind_class_input: String,
    pub tailwind_node_class_input: String,
    pub tailwind_node_class_dropdown_open: bool,
    pub tailwind_inspector_hovered: bool,

    // AI design planning
    pub design_planner_active_tab: DesignPlannerTab,
    pub design_chat_sessions: Vec<DesignChatSession>,
    pub design_chat_active_session: usize,
    pub design_chat_session_seed: usize,
    pub design_chat_selected_message: Option<usize>,
    pub design_chat_messages: Vec<DesignChatMessage>,
    pub design_chat_input: Content,
    pub design_generation_executor: TaskExecutorBackend,
    pub design_generation_executor_popover: bool,
    pub design_generation_theme_popover: bool,
    pub design_generation_device_popover: bool,
    pub design_generation_style_popover: bool,
    pub design_generation_style: DesignStyle,
    pub design_generation_model: String,
    pub design_generation_model_popover: bool,
    pub design_generation_theme: DesignGenerationTheme,
    pub design_generation_device: DesignGenerationDevice,
    pub design_generation_parallel_pages: usize,
    pub design_generation_parallel_pages_input: String,
    pub design_generation_loading: bool,
    pub design_generation_brief: String,
    pub design_generation_summary: Option<String>,
    pub design_generation_pages: Vec<DesignGenerationPage>,
    pub design_generation_logs: Vec<String>,
    pub design_generation_log_editor: Content,
    pub design_generation_stream_rx:
        Option<std::sync::mpsc::Receiver<crate::app::task::TaskLogStream>>,
    pub design_generation_stream_cursor: usize,
    pub design_generation_anim_frame: u8,
    pub design_generation_log_files: Vec<String>,
    pub design_generation_current_log_file: Option<String>,
    pub design_generation_show_all_logs: bool,
    pub design_planner_quick_menu_open: bool,

    // History
    pub history: Vec<DesignDoc>,
    pub history_index: usize,

    pub figma_progress: Option<FigmaProgressState>,
    pub figma_progress_rx: Option<std::sync::mpsc::Receiver<FigmaProgressState>>,
}

impl DesignState {
    pub fn has_single_empty_page(&self) -> bool {
        self.doc.children.is_empty() && self.doc.groups.len() == 1
    }

    pub fn ensure_valid_group(&mut self) {
        self.doc.normalize_groups();
        if !self.doc.groups.iter().any(|group| group.id == self.active_group_id) {
            self.active_group_id = self.doc.first_group_id();
        }
    }

    pub fn focus_first_element_in_active_group(&mut self) -> Option<String> {
        let target_id = self
            .doc
            .first_top_level_in_group(self.active_group_id)
            .map(|element| element.id.clone());
        self.selected_fill_index = None;
        self.selected_effect_index = None;
        self.selected_element_ids.clear();

        if let Some(id) = target_id.clone() {
            self.selected_element_id = Some(id.clone());
            self.selected_element_ids.insert(id);
        } else {
            self.selected_element_id = None;
        }

        self.canvas_cache.clear();
        target_id
    }

    pub fn selection_in_active_group(&self) -> bool {
        self.selected_element_id
            .as_deref()
            .and_then(|id| self.doc.group_id_for_element(id))
            .is_some_and(|group_id| group_id == self.active_group_id)
    }

    fn default_design_chat_messages() -> Vec<DesignChatMessage> {
        vec![DesignChatMessage {
            role: DesignChatRole::System,
            content: "描述你想生成的网站需求，我会在一次任务中生成页面骨架和模块内容。".to_string(),
        }]
    }

    pub fn sync_active_chat_session_from_legacy(&mut self) {
        let active =
            self.design_chat_active_session.min(self.design_chat_sessions.len().saturating_sub(1));
        if let Some(session) = self.design_chat_sessions.get_mut(active) {
            session.messages = self.design_chat_messages.clone();
            session.input = self.design_chat_input.clone();
            if let Some(prompt) = self
                .design_chat_messages
                .iter()
                .rev()
                .find(|message| matches!(message.role, DesignChatRole::User))
                .map(|message| message.content.trim())
                .filter(|value| !value.is_empty())
            {
                session.title = prompt.chars().take(20).collect::<String>();
            }
        }
    }

    pub fn sync_legacy_from_active_chat_session(&mut self) {
        let active =
            self.design_chat_active_session.min(self.design_chat_sessions.len().saturating_sub(1));
        if let Some(session) = self.design_chat_sessions.get(active) {
            self.design_chat_messages = session.messages.clone();
            self.design_chat_input = session.input.clone();
            self.design_chat_selected_message = None;
            return;
        }
        self.design_chat_messages = Self::default_design_chat_messages();
        self.design_chat_input = Content::new();
        self.design_chat_selected_message = None;
    }

    pub fn select_design_chat_session(&mut self, index: usize) {
        if index >= self.design_chat_sessions.len() {
            return;
        }
        self.sync_active_chat_session_from_legacy();
        self.design_chat_active_session = index;
        self.sync_legacy_from_active_chat_session();
    }

    pub fn create_design_chat_session(&mut self) {
        self.sync_active_chat_session_from_legacy();
        self.design_chat_session_seed = self.design_chat_session_seed.saturating_add(1);
        let title = format!("New Chat {}", self.design_chat_session_seed);
        self.design_chat_sessions.push(DesignChatSession {
            id: self.design_chat_session_seed,
            title,
            messages: Self::default_design_chat_messages(),
            input: Content::new(),
        });
        self.design_chat_active_session = self.design_chat_sessions.len().saturating_sub(1);
        self.sync_legacy_from_active_chat_session();
    }

    pub fn new(doc: DesignDoc) -> Self {
        let mut doc = doc;
        doc.normalize_groups();
        let layer_tree_metrics = compute_tree_metrics(&doc);
        let active_group_id = doc.first_group_id();
        let design_generation_parallel_pages = load_design_generation_parallel_pages();
        let design_chat_messages = Self::default_design_chat_messages();
        let mut state = Self {
            context_popover: None,
            canvas_context_menu_anchor: None,
            paste_anchor: None,
            doc: doc.clone(), // Clone for history
            file_path: None,
            layer_tree_metrics,
            canvas_cache: Cache::default(),
            selected_element_id: None,
            selected_element_ids: HashSet::new(),
            pan: Vector::new(0.0, 0.0),
            zoom: 1.0,
            active_group_id,
            expanded_nodes: HashSet::new(),
            active_tool: DesignTool::Move,
            brush_color_hex: DEFAULT_BRUSH_COLOR_HEX.to_string(),
            brush_width_px: DEFAULT_BRUSH_WIDTH_PX,
            editing_id: None,
            editing_content: String::new(),
            editing_editor: Content::new(),
            context_editor: Content::new(),
            context_element_id: None,
            context_expanded: false,
            content_editor: Content::new(),
            tailwind_html_editor: Content::new(),
            tailwind_node_class_editor: Content::new(),
            tailwind_node_text_editor: Content::new(),
            tailwind_inspector_collapsed: false,
            tailwind_tree_collapsed: HashSet::new(),
            tailwind_tree_scroll_id: Id::new("tailwind_tree"),
            selected_fill_index: None,
            selected_effect_index: None,
            show_zoom_menu: false,
            image_import_target: None,
            image_import_input: String::new(),
            image_import_error: None,
            image_import_loading: false,
            sticky_note_dialog_open: false,
            sticky_note_dialog_default_kind: StickyNoteKind::Note,
            active_layer_menu: None,
            active_page_menu: None,
            page_menu_anchor: None,
            renaming_page_id: None,
            renaming_page_name: String::new(),
            current_variable_collection: None,
            active_variable_collection_menu: None,
            renaming_variable_collection: None,
            variable_collection_rename_value: String::new(),
            confirm_delete_variable_collection: None,
            active_variable_theme_menu: None,
            renaming_variable_theme: None,
            variable_theme_rename_value: String::new(),
            confirm_delete_variable_theme: None,
            active_variable_menu: None,
            variable_move_target_picker: None,
            renaming_variable: None,
            variable_rename_value: String::new(),
            confirm_delete_variable: None,
            show_add_variable_menu: false,
            context_shape_group_hover: None,
            icon_filter_query: String::new(),
            toolbar_icon_family: "lucide".to_string(),
            toolbar_icon_name: "star".to_string(),
            toolbar_icon_family_tab: "lucide".to_string(),
            dragging_layer: None,
            drag_target_layer: None,
            hovered_layer_id: None,
            active_color_picker: None,
            show_element_html_preview: false,
            element_html_preview_editor: Content::new(),
            design_help_text: None,
            tailwind_filter_query: String::new(),
            font_filter_query: String::new(),
            new_group_name: String::new(),
            tailwind_class_input: String::new(),
            tailwind_node_class_input: String::new(),
            tailwind_node_class_dropdown_open: false,
            tailwind_inspector_hovered: false,
            design_planner_active_tab: DesignPlannerTab::Chat,
            design_chat_sessions: vec![DesignChatSession {
                id: 1,
                title: "New Chat".to_string(),
                messages: design_chat_messages.clone(),
                input: Content::new(),
            }],
            design_chat_active_session: 0,
            design_chat_session_seed: 1,
            design_chat_selected_message: None,
            design_chat_messages,
            design_chat_input: Content::new(),
            design_generation_executor: TaskExecutorBackend::Internal,
            design_generation_executor_popover: false,
            design_generation_theme_popover: false,
            design_generation_device_popover: false,
            design_generation_style_popover: false,
            design_generation_style: DesignStyle::Default,
            design_generation_model: TASK_MODEL_AUTO.to_string(),
            design_generation_model_popover: false,
            design_generation_theme: DesignGenerationTheme::Shadcn,
            design_generation_device: DesignGenerationDevice::Auto,
            design_generation_parallel_pages,
            design_generation_parallel_pages_input: design_generation_parallel_pages.to_string(),
            design_generation_loading: false,
            design_generation_brief: String::new(),
            design_generation_summary: None,
            design_generation_pages: Vec::new(),
            design_generation_logs: Vec::new(),
            design_generation_log_editor: Content::new(),
            design_generation_stream_rx: None,
            design_generation_stream_cursor: 0,
            design_generation_anim_frame: 0,
            design_generation_log_files: Vec::new(),
            design_generation_current_log_file: None,
            design_generation_show_all_logs: false,
            design_planner_quick_menu_open: false,
            history: vec![doc],
            history_index: 0,
            figma_progress: None,
            figma_progress_rx: None,
        };
        state.focus_first_element_in_active_group();
        state
    }
}
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
