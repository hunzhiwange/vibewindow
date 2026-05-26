use crate::app::components::mind_map;
use crate::apps::mindmap::canvas::theme::MindMapCustomTheme;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    MindMapTab, OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};

/// JSON 文件格式标识符
///
/// 用于验证 JSON 文件是否为本应用生成的思维导图文件
pub(super) const MINDMAP_JSON_FORMAT: &str = "vibe-window-mindmap";

/// 返回默认的边框样式（实线）
pub(super) fn default_edge_style() -> EdgeStyle {
    EdgeStyle::Solid
}

/// 返回默认的"跟随主题背景"设置（启用）
pub(super) fn default_follow_theme_background() -> bool {
    true
}

/// 返回默认的主题组名称
pub(super) fn default_theme_group() -> String {
    "classic".to_string()
}

/// 返回默认的图表类型
pub(super) fn default_diagram_type() -> MindMapDiagramType {
    MindMapDiagramType::default()
}

/// 返回默认的思维导图布局格式
pub(super) fn default_layout_format() -> MindMapLayoutFormat {
    MindMapLayoutFormat::default()
}

/// 返回默认的组织架构图布局格式
pub(super) fn default_org_chart_layout_format() -> OrgChartLayoutFormat {
    OrgChartLayoutFormat::default()
}

/// 返回默认的鱼骨图布局格式
pub(super) fn default_fishbone_layout_format() -> FishboneLayoutFormat {
    FishboneLayoutFormat::default()
}

/// 返回默认的括号图布局格式
pub(super) fn default_bracket_layout_format() -> BracketLayoutFormat {
    BracketLayoutFormat::default()
}

/// 返回默认的时间轴布局格式
pub(super) fn default_timeline_layout_format() -> TimelineLayoutFormat {
    TimelineLayoutFormat::default()
}

/// 返回默认的树形图布局格式
pub(super) fn default_tree_layout_format() -> TreeLayoutFormat {
    TreeLayoutFormat::default()
}

/// JSON 文件的顶层结构
///
/// 包含格式标识、版本号和实际数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct MindMapJsonFile {
    /// 文件格式标识符，应为 `MINDMAP_JSON_FORMAT`
    pub(super) format: String,
    /// 文件格式版本号，当前为 1
    pub(super) version: u32,
    /// 实际的思维导图数据
    pub(super) data: MindMapJsonV1,
}

/// 思维导图 JSON 数据结构（V1 版本）
///
/// 完整保存思维导图的所有状态，包括：
/// - 节点内容和结构（通过 Markdown 格式）
/// - 布局配置（支持多种图表类型）
/// - 视图状态（平移、缩放）
/// - 节点样式（填充色、边框、文本颜色等）
/// - 边样式（线条样式、颜色）
/// - 涂鸦数据
/// - 主题配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct MindMapJsonV1 {
    /// 可选的标题（当无法从文件路径推断时使用）
    #[serde(default)]
    pub(super) title: Option<String>,
    /// 节点内容的 Markdown 表示
    pub(super) markdown: String,
    /// 图表类型（思维导图、组织架构图、鱼骨图等）
    #[serde(default = "default_diagram_type")]
    pub(super) diagram_type: MindMapDiagramType,
    /// 思维导图布局格式
    #[serde(default = "default_layout_format")]
    pub(super) layout_format: MindMapLayoutFormat,
    /// 组织架构图布局格式
    #[serde(default = "default_org_chart_layout_format")]
    pub(super) org_chart_layout_format: OrgChartLayoutFormat,
    /// 鱼骨图布局格式
    #[serde(default = "default_fishbone_layout_format")]
    pub(super) fishbone_layout_format: FishboneLayoutFormat,
    /// 时间轴布局格式
    #[serde(default = "default_timeline_layout_format")]
    pub(super) timeline_layout_format: TimelineLayoutFormat,
    /// 括号图布局格式
    #[serde(default = "default_bracket_layout_format")]
    pub(super) bracket_layout_format: BracketLayoutFormat,
    /// 树形图布局格式
    #[serde(default = "default_tree_layout_format")]
    pub(super) tree_layout_format: TreeLayoutFormat,
    /// 视图平移 X 偏移量
    pub(super) pan_x: f32,
    /// 视图平移 Y 偏移量
    pub(super) pan_y: f32,
    /// 视图缩放比例
    pub(super) zoom: f32,
    /// 当前选中节点的路径（索引序列）
    #[serde(default)]
    pub(super) selected_path: Option<Vec<usize>>,
    /// 持久化的节点位置覆盖
    #[serde(default)]
    pub(super) node_positions: Vec<PersistedNodePos>,
    /// 持久化的节点填充色（RGBA）
    #[serde(default)]
    pub(super) node_fills: Vec<PersistedNodeFill>,
    /// 持久化的节点文本颜色（RGBA）
    #[serde(default)]
    pub(super) node_text_colors: Vec<PersistedNodeTextColor>,
    /// 持久化的节点边框颜色（RGBA）
    #[serde(default)]
    pub(super) node_border_colors: Vec<PersistedNodeBorderColor>,
    /// 默认节点边框样式
    #[serde(default = "default_edge_style")]
    pub(super) node_border_style: EdgeStyle,
    /// 持久化的节点边框样式覆盖
    #[serde(default)]
    pub(super) node_border_styles: Vec<PersistedNodeBorderStyle>,
    /// 持久化的节点优先级（1-9）
    #[serde(default)]
    pub(super) node_priorities: Vec<PersistedNodePriority>,
    /// 持久化的节点 URL 链接
    #[serde(default)]
    pub(super) node_urls: Vec<PersistedNodeUrl>,
    /// 已折叠节点的路径列表
    #[serde(default)]
    pub(super) collapsed_paths: Vec<Vec<usize>>,
    /// 自定义背景颜色（RGBA）
    #[serde(default)]
    pub(super) background: Option<u32>,
    /// 是否跟随主题背景色
    #[serde(default = "default_follow_theme_background")]
    pub(super) follow_theme_background: bool,
    /// 默认边样式
    #[serde(default = "default_edge_style")]
    pub(super) edge_style: EdgeStyle,
    /// 持久化的边样式覆盖
    #[serde(default)]
    pub(super) edge_styles: Vec<PersistedEdgeStyle>,
    /// 持久化的边颜色（RGBA）
    #[serde(default)]
    pub(super) edge_colors: Vec<PersistedEdgeColor>,
    /// 涂鸦笔触颜色（RGBA）
    #[serde(default)]
    pub(super) doodle_rgba: u32,
    /// 涂鸦笔触宽度（像素）
    #[serde(default)]
    pub(super) doodle_width_px: f32,
    /// 涂鸦笔触列表
    #[serde(default)]
    pub(super) doodles: Vec<PersistedDoodleStroke>,
    /// 主题组名称
    #[serde(default = "default_theme_group")]
    pub(super) theme_group: String,
    /// 主题变体索引
    #[serde(default)]
    pub(super) theme_variant: usize,
    /// 自定义主题列表
    #[serde(default)]
    pub(super) custom_themes: Vec<MindMapCustomTheme>,
}

/// 持久化的节点位置
///
/// 记录单个节点的自定义位置覆盖
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodePos {
    /// 节点路径（从根到该节点的索引序列）
    pub(super) path: Vec<usize>,
    /// X 坐标
    pub(super) x: f32,
    /// Y 坐标
    pub(super) y: f32,
}

/// 持久化的节点填充色
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodeFill {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 填充颜色（RGBA 格式的 u32）
    pub(super) rgba: u32,
}

/// 持久化的节点文本颜色
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodeTextColor {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 文本颜色（RGBA 格式的 u32）
    pub(super) rgba: u32,
}

/// 持久化的节点边框颜色
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodeBorderColor {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 边框颜色（RGBA 格式的 u32）
    pub(super) rgba: u32,
}

/// 持久化的节点边框样式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodeBorderStyle {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 边框样式（实线、虚线等）
    pub(super) style: EdgeStyle,
}

/// 持久化的节点优先级
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodePriority {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 优先级值（1-9，1 最高）
    pub(super) priority: u8,
}

/// 持久化的节点 URL
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedNodeUrl {
    /// 节点路径
    pub(super) path: Vec<usize>,
    /// 关联的 URL
    pub(super) url: String,
}

/// 持久化的边样式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedEdgeStyle {
    /// 边的目标节点路径
    pub(super) path: Vec<usize>,
    /// 边样式（实线、虚线等）
    pub(super) style: EdgeStyle,
}

/// 持久化的边颜色
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedEdgeColor {
    /// 边的目标节点路径
    pub(super) path: Vec<usize>,
    /// 边颜色（RGBA 格式的 u32）
    pub(super) rgba: u32,
}

/// 持久化的涂鸦笔触
///
/// 记录用户在画布上绘制的自由线条
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedDoodleStroke {
    /// 笔触颜色（RGBA）
    pub(super) rgba: u32,
    /// 笔触宽度（像素）
    pub(super) width_px: f32,
    /// 笔触的采样点序列
    pub(super) points: Vec<PersistedPoint>,
}

/// 持久化的点坐标
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedPoint {
    /// X 坐标
    pub(super) x: f32,
    /// Y 坐标
    pub(super) y: f32,
}

/// 将标签页转换为 JSON 文件结构
///
/// # 参数
///
/// - `tab`: 要序列化的思维导图标签页
///
/// # 返回
///
/// 可序列化为 JSON 的文件结构
pub(super) fn tab_to_json(tab: &MindMapTab) -> MindMapJsonFile {
    MindMapJsonFile {
        format: MINDMAP_JSON_FORMAT.to_string(),
        version: 1,
        data: MindMapJsonV1 {
            title: Some(tab.title.clone()),
            markdown: mind_map::to_markdown(&tab.doc),
            diagram_type: tab.diagram_type,
            layout_format: tab.layout_format,
            org_chart_layout_format: tab.org_chart_layout_format,
            fishbone_layout_format: tab.fishbone_layout_format,
            timeline_layout_format: tab.timeline_layout_format,
            bracket_layout_format: tab.bracket_layout_format,
            tree_layout_format: tab.tree_layout_format,
            pan_x: tab.pan.x,
            pan_y: tab.pan.y,
            zoom: tab.zoom,
            selected_path: tab.selected_path.clone(),
            node_positions: tab
                .node_positions
                .iter()
                .map(|(path, pt)| PersistedNodePos { path: path.clone(), x: pt.x, y: pt.y })
                .collect(),
            node_fills: tab
                .node_fills
                .iter()
                .map(|(path, rgba)| PersistedNodeFill { path: path.clone(), rgba: *rgba })
                .collect(),
            node_text_colors: tab
                .node_text_colors
                .iter()
                .map(|(path, rgba)| PersistedNodeTextColor { path: path.clone(), rgba: *rgba })
                .collect(),
            node_border_colors: tab
                .node_border_colors
                .iter()
                .map(|(path, rgba)| PersistedNodeBorderColor { path: path.clone(), rgba: *rgba })
                .collect(),
            node_border_style: tab.node_border_style,
            node_border_styles: tab
                .node_border_styles
                .iter()
                .map(|(path, style)| PersistedNodeBorderStyle { path: path.clone(), style: *style })
                .collect(),
            node_priorities: tab
                .node_priorities
                .iter()
                .map(|(path, priority)| PersistedNodePriority {
                    path: path.clone(),
                    priority: *priority,
                })
                .collect(),
            node_urls: tab
                .node_urls
                .iter()
                .map(|(path, url)| PersistedNodeUrl { path: path.clone(), url: url.clone() })
                .collect(),
            collapsed_paths: tab.collapsed_paths.iter().cloned().collect(),
            background: tab.background,
            follow_theme_background: tab.follow_theme_background,
            edge_style: tab.edge_style,
            edge_styles: tab
                .edge_styles
                .iter()
                .map(|(path, style)| PersistedEdgeStyle { path: path.clone(), style: *style })
                .collect(),
            edge_colors: tab
                .edge_colors
                .iter()
                .map(|(path, rgba)| PersistedEdgeColor { path: path.clone(), rgba: *rgba })
                .collect(),
            doodle_rgba: tab.doodle_rgba,
            doodle_width_px: tab.doodle_width_px,
            doodles: tab
                .doodles
                .iter()
                .map(|s| PersistedDoodleStroke {
                    rgba: s.rgba,
                    width_px: s.width_px,
                    points: s
                        .points_world
                        .iter()
                        .map(|p| PersistedPoint { x: p.x, y: p.y })
                        .collect(),
                })
                .collect(),
            theme_group: tab.theme_group.clone(),
            theme_variant: tab.theme_variant,
            custom_themes: tab.custom_themes.clone(),
        },
    }
}
