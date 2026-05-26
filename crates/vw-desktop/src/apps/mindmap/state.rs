//! 思维导图应用状态定义，保存画布、主题、节点和文件相关的用户工作状态。

use crate::app::components::mind_map::MindNode;
use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::canvas::theme::{MindMapCustomTheme, default_custom_themes};
use iced::Color;
use iced::Point;
use iced::Vector;
use iced::widget::canvas::Cache;
use iced::widget::text_editor;
use std::collections::HashMap;
use std::collections::HashSet;

/// EdgeStyle 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EdgeStyle {
    Solid,
    Dashed,
    Dotted,
}

/// MindMapCanvasTool 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MindMapCanvasTool {
    Pan,
    Select,
    Pen,
    Eraser,
}

/// MindMapDiagramType 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum MindMapDiagramType {
    #[default]
    MindMap,
    OrgChart,
    Fishbone,
    Timeline,
    Tree,
    Bracket,
}

/// MindMapLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum MindMapLayoutFormat {
    #[default]
    RightAligned,
    LeftAligned,
    Bidirectional,
}

/// OrgChartLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum OrgChartLayoutFormat {
    #[default]
    TopDown,
    LeftRight,
}

/// FishboneLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum FishboneLayoutFormat {
    #[default]
    HeadRight,
    HeadLeft,
}

/// BracketLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum BracketLayoutFormat {
    #[default]
    BraceRight,
    BraceLeft,
}

/// TimelineLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum TimelineLayoutFormat {
    #[default]
    UpDown,
    AllUp,
    AllDown,
}

/// TreeLayoutFormat 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum TreeLayoutFormat {
    SymmetricSplit,
    #[default]
    FanDown,
    LeftAligned,
    RightAligned,
}




impl FishboneLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            FishboneLayoutFormat::HeadRight => "鱼头在右",
            FishboneLayoutFormat::HeadLeft => "鱼头在左",
        }
    }
}

impl BracketLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            BracketLayoutFormat::BraceRight => "括号在右",
            BracketLayoutFormat::BraceLeft => "括号在左",
        }
    }
}

impl TimelineLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            TimelineLayoutFormat::UpDown => "上下",
            TimelineLayoutFormat::AllUp => "全上",
            TimelineLayoutFormat::AllDown => "全下",
        }
    }
}


impl OrgChartLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            OrgChartLayoutFormat::TopDown => "自上而下（曲线）",
            OrgChartLayoutFormat::LeftRight => "自上而下（折线）",
        }
    }
}



impl MindMapLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            MindMapLayoutFormat::RightAligned => "右侧展开",
            MindMapLayoutFormat::LeftAligned => "左侧展开",
            MindMapLayoutFormat::Bidirectional => "双侧展开",
        }
    }
}

impl TreeLayoutFormat {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            TreeLayoutFormat::SymmetricSplit => "左右对称分支",
            TreeLayoutFormat::FanDown => "顶部中心多分支",
            TreeLayoutFormat::LeftAligned => "左侧单边分支",
            TreeLayoutFormat::RightAligned => "右侧单边分支",
        }
    }
}


impl MindMapDiagramType {
    /// 构建或更新 label 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn label(&self) -> &'static str {
        match self {
            MindMapDiagramType::MindMap => "思维导图",
            MindMapDiagramType::OrgChart => "组织结构图",
            MindMapDiagramType::Fishbone => "鱼骨图",
            MindMapDiagramType::Timeline => "时间轴",
            MindMapDiagramType::Tree => "树形图",
            MindMapDiagramType::Bracket => "括号图",
        }
    }
}

/// MindMapDoodleStroke 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Clone)]
pub struct MindMapDoodleStroke {
    /// points world 字段，保存渲染或状态更新所需的输入数据。
    pub points_world: Vec<Point>,
    /// rgba 字段，保存渲染或状态更新所需的输入数据。
    pub rgba: u32,
    /// width px 字段，保存渲染或状态更新所需的输入数据。
    pub width_px: f32,
}

/// MindMapColorTarget 枚举，描述当前模块支持的有限状态或操作分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MindMapColorTarget {
    NodeFill,
    NodeText,
    NodeBorder,
    EdgeStroke,
    Background,
}

/// MindMapColorPicker 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Clone)]
pub struct MindMapColorPicker {
    /// color 字段，保存渲染或状态更新所需的输入数据。
    pub color: Color,
    /// format 字段，保存渲染或状态更新所需的输入数据。
    pub format: ColorFormat,
    /// target 字段，保存渲染或状态更新所需的输入数据。
    pub target: MindMapColorTarget,
    /// picking 字段，保存渲染或状态更新所需的输入数据。
    pub picking: bool,
}

/// MindMapTab 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug)]
pub struct MindMapTab {
    /// id 字段，保存渲染或状态更新所需的输入数据。
    pub id: String,
    /// title 字段，保存渲染或状态更新所需的输入数据。
    pub title: String,
    /// file path 字段，保存渲染或状态更新所需的输入数据。
    pub file_path: Option<String>,
    /// doc 字段，保存渲染或状态更新所需的输入数据。
    pub doc: MindNode,
    /// diagram type 字段，保存渲染或状态更新所需的输入数据。
    pub diagram_type: MindMapDiagramType,
    /// layout format 字段，保存渲染或状态更新所需的输入数据。
    pub layout_format: MindMapLayoutFormat,
    /// org chart layout format 字段，保存渲染或状态更新所需的输入数据。
    pub org_chart_layout_format: OrgChartLayoutFormat,
    /// fishbone layout format 字段，保存渲染或状态更新所需的输入数据。
    pub fishbone_layout_format: FishboneLayoutFormat,
    /// timeline layout format 字段，保存渲染或状态更新所需的输入数据。
    pub timeline_layout_format: TimelineLayoutFormat,
    /// bracket layout format 字段，保存渲染或状态更新所需的输入数据。
    pub bracket_layout_format: BracketLayoutFormat,
    /// tree layout format 字段，保存渲染或状态更新所需的输入数据。
    pub tree_layout_format: TreeLayoutFormat,
    /// pan 字段，保存渲染或状态更新所需的输入数据。
    pub pan: Vector,
    /// zoom 字段，保存渲染或状态更新所需的输入数据。
    pub zoom: f32,
    /// selected path 字段，保存渲染或状态更新所需的输入数据。
    pub selected_path: Option<Vec<usize>>,
    /// last click screen 字段，保存渲染或状态更新所需的输入数据。
    pub last_click_screen: Option<Point>,
    /// node positions 字段，保存渲染或状态更新所需的输入数据。
    pub node_positions: HashMap<Vec<usize>, Point>,
    /// node fills 字段，保存渲染或状态更新所需的输入数据。
    pub node_fills: HashMap<Vec<usize>, u32>,
    /// node text colors 字段，保存渲染或状态更新所需的输入数据。
    pub node_text_colors: HashMap<Vec<usize>, u32>,
    /// node border colors 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_colors: HashMap<Vec<usize>, u32>,
    /// node border style 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_style: EdgeStyle,
    /// node border styles 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_styles: HashMap<Vec<usize>, EdgeStyle>,
    /// node priorities 字段，保存渲染或状态更新所需的输入数据。
    pub node_priorities: HashMap<Vec<usize>, u8>,
    /// node urls 字段，保存渲染或状态更新所需的输入数据。
    pub node_urls: HashMap<Vec<usize>, String>,
    /// collapsed paths 字段，保存渲染或状态更新所需的输入数据。
    pub collapsed_paths: HashSet<Vec<usize>>,
    /// background 字段，保存渲染或状态更新所需的输入数据。
    pub background: Option<u32>,
    /// follow theme background 字段，保存渲染或状态更新所需的输入数据。
    pub follow_theme_background: bool,
    /// edge style 字段，保存渲染或状态更新所需的输入数据。
    pub edge_style: EdgeStyle,
    /// edge styles 字段，保存渲染或状态更新所需的输入数据。
    pub edge_styles: HashMap<Vec<usize>, EdgeStyle>,
    /// edge colors 字段，保存渲染或状态更新所需的输入数据。
    pub edge_colors: HashMap<Vec<usize>, u32>,
    /// active color picker 字段，保存渲染或状态更新所需的输入数据。
    pub active_color_picker: Option<MindMapColorPicker>,
    /// show diagram type picker 字段，保存渲染或状态更新所需的输入数据。
    pub show_diagram_type_picker: bool,
    /// show markdown import 字段，保存渲染或状态更新所需的输入数据。
    pub show_markdown_import: bool,
    /// show export menu 字段，保存渲染或状态更新所需的输入数据。
    pub show_export_menu: bool,
    /// show zoom menu 字段，保存渲染或状态更新所需的输入数据。
    pub show_zoom_menu: bool,
    /// show priority picker 字段，保存渲染或状态更新所需的输入数据。
    pub show_priority_picker: bool,
    /// show url editor 字段，保存渲染或状态更新所需的输入数据。
    pub show_url_editor: bool,
    /// show text editor 字段，保存渲染或状态更新所需的输入数据。
    pub show_text_editor: bool,
    /// show action menu 字段，保存渲染或状态更新所需的输入数据。
    pub show_action_menu: bool,
    /// url editor value 字段，保存渲染或状态更新所需的输入数据。
    pub url_editor_value: String,
    /// node text editor 字段，保存渲染或状态更新所需的输入数据。
    pub node_text_editor: text_editor::Content,
    /// markdown import editor 字段，保存渲染或状态更新所需的输入数据。
    pub markdown_import_editor: text_editor::Content,
    /// clipboard node 字段，保存渲染或状态更新所需的输入数据。
    pub clipboard_node: Option<MindNode>,
    /// show context menu 字段，保存渲染或状态更新所需的输入数据。
    pub show_context_menu: bool,
    /// context menu anchor 字段，保存渲染或状态更新所需的输入数据。
    pub context_menu_anchor: Option<Point>,
    /// canvas cache 字段，保存渲染或状态更新所需的输入数据。
    pub canvas_cache: Cache,
    /// undo stack 字段，保存渲染或状态更新所需的输入数据。
    pub undo_stack: Vec<MindNode>,
    /// redo stack 字段，保存渲染或状态更新所需的输入数据。
    pub redo_stack: Vec<MindNode>,
    /// canvas tool 字段，保存渲染或状态更新所需的输入数据。
    pub canvas_tool: MindMapCanvasTool,
    /// doodle rgba 字段，保存渲染或状态更新所需的输入数据。
    pub doodle_rgba: u32,
    /// doodle width px 字段，保存渲染或状态更新所需的输入数据。
    pub doodle_width_px: f32,
    /// doodles 字段，保存渲染或状态更新所需的输入数据。
    pub doodles: Vec<MindMapDoodleStroke>,
    /// show theme panel 字段，保存渲染或状态更新所需的输入数据。
    pub show_theme_panel: bool,
    /// theme group 字段，保存渲染或状态更新所需的输入数据。
    pub theme_group: String,
    /// theme variant 字段，保存渲染或状态更新所需的输入数据。
    pub theme_variant: usize,
    /// custom themes 字段，保存渲染或状态更新所需的输入数据。
    pub custom_themes: Vec<MindMapCustomTheme>,
}

impl MindMapTab {
    /// 构建或更新 new 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub fn new(id: String, title: String, file_path: Option<String>, doc: MindNode) -> Self {
        Self {
            id,
            title,
            file_path,
            doc,
            diagram_type: MindMapDiagramType::default(),
            layout_format: MindMapLayoutFormat::default(),
            org_chart_layout_format: OrgChartLayoutFormat::default(),
            fishbone_layout_format: FishboneLayoutFormat::default(),
            timeline_layout_format: TimelineLayoutFormat::default(),
            bracket_layout_format: BracketLayoutFormat::default(),
            tree_layout_format: TreeLayoutFormat::default(),
            pan: Vector::new(300.0, 200.0),
            zoom: 1.0,
            selected_path: None,
            last_click_screen: None,
            node_positions: HashMap::new(),
            node_fills: HashMap::new(),
            node_text_colors: HashMap::new(),
            node_border_colors: HashMap::new(),
            node_border_style: EdgeStyle::Solid,
            node_border_styles: HashMap::new(),
            node_priorities: HashMap::new(),
            node_urls: HashMap::new(),
            collapsed_paths: HashSet::new(),
            background: None,
            follow_theme_background: true,
            edge_style: EdgeStyle::Solid,
            edge_styles: HashMap::new(),
            edge_colors: HashMap::new(),
            active_color_picker: None,
            show_diagram_type_picker: false,
            show_markdown_import: false,
            show_export_menu: false,
            show_zoom_menu: false,
            show_priority_picker: false,
            show_url_editor: false,
            show_text_editor: false,
            show_action_menu: false,
            url_editor_value: String::new(),
            node_text_editor: text_editor::Content::new(),
            markdown_import_editor: text_editor::Content::new(),
            clipboard_node: None,
            show_context_menu: false,
            context_menu_anchor: None,
            canvas_cache: Cache::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            canvas_tool: MindMapCanvasTool::Select,
            doodle_rgba: 0x111827FF,
            doodle_width_px: 3.0,
            doodles: Vec::new(),
            show_theme_panel: false,
            theme_group: "classic".to_string(),
            theme_variant: 0,
            custom_themes: default_custom_themes(),
        }
    }
}
