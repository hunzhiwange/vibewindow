//! 画布覆盖层渲染模块
//!
//! 本模块负责在画布上渲染各种覆盖层元素，包括：
//! - 网格背景：用于视觉对齐和参考
//! - 选择框：显示选中元素的边界和控制手柄
//! - 悬停编辑覆盖层：显示可复用元素的内部布局结构
//! - 框选区域：用户拖拽选择时的可视化反馈
//!
//! 这些覆盖层在交互式设计工具中提供视觉反馈，
//! 帮助用户理解元素的位置、大小和布局关系。

use iced::{
    Color, Point, Rectangle, Size, Vector,
    widget::canvas::{Frame, LineDash, Path, Stroke},
};

use crate::app::views::design::canvas::tailwind;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject, MeshFill};
use crate::app::views::design::{
    canvas::{
        geometry::get_element_screen_bounds,
        layout::{
            calc::compute_layout,
            parse::{parse_layout, parse_padding},
        },
        types::LayoutDirection,
        utils::theme_mode_for_element,
    },
    models::DesignDoc,
};

/// 在画布帧上绘制网格背景
///
/// 网格用于提供视觉参考，帮助用户对齐元素。
/// 网格间距会根据缩放级别自动调整。
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `bounds` - 画布的可视边界
/// * `pan` - 平移偏移量，用于计算网格对齐
/// * `zoom` - 缩放级别，影响网格间距的视觉大小
/// * `color` - 网格线的基础颜色（会自动调整为半透明）
///
/// # 说明
///
/// - 当缩放后的间距小于5像素时，不绘制网格（避免视觉混乱）
/// - 网格线透明度被固定为0.1，确保不会干扰主要内容
pub fn draw_grid(frame: &mut Frame, bounds: Rectangle, pan: Vector, zoom: f32, color: Color) {
    // 计算缩放后的网格间距（基础间距20像素）
    let spacing = 20.0 * zoom;
    // 间距太小则不绘制，避免视觉混乱
    if spacing < 5.0 {
        return;
    }

    // 计算平移后的网格偏移，确保网格随画布移动
    let offset_x = pan.x % spacing;
    let offset_y = pan.y % spacing;

    // 计算需要绘制的网格线数量（额外+2确保边缘覆盖）
    let steps_x = (bounds.width / spacing).ceil() as i32 + 2;
    let steps_y = (bounds.height / spacing).ceil() as i32 + 2;

    // 网格线使用半透明效果（透明度0.1）
    let color_lines = Color { a: 0.1, ..color };

    // 绘制垂直网格线
    for i in -1..steps_x {
        let x = (i as f32 * spacing) + offset_x;
        if x <= 0.5 || x >= bounds.width - 0.5 {
            continue;
        }
        frame.stroke(
            &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
            Stroke::default().with_color(color_lines).with_width(1.0),
        );
    }
    // 绘制水平网格线
    for i in -1..steps_y {
        let y = (i as f32 * spacing) + offset_y;
        if y <= 0.5 || y >= bounds.height - 0.5 {
            continue;
        }
        frame.stroke(
            &Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
            Stroke::default().with_color(color_lines).with_width(1.0),
        );
    }
}

/// 绘制选择覆盖层，显示选中元素的边界框和控制手柄
///
/// 该函数处理多种选择状态的渲染：
/// - 常规选择：显示选中元素的边界框
/// - Mesh填充编辑：显示网格点和控制手柄
/// - Tailwind组件选择：显示嵌套元素的选择框
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `selected_ids` - 当前选中的元素ID集合
/// * `doc` - 设计文档引用，包含所有元素信息
/// * `pan` - 画布平移偏移量
/// * `zoom` - 画布缩放级别
/// * `selected_id` - 当前聚焦的主选中元素ID（用于编辑模式）
/// * `selected_fill_index` - 选中的填充项索引（用于Mesh编辑）
/// * `sel_color` - 选择框颜色
/// * `show_handles` - 是否显示调整大小的控制手柄
/// * `hovered_tailwind_selection` - 悬停的Tailwind元素选择路径
pub fn draw_selection_overlay(
    frame: &mut Frame,
    selected_ids: &std::collections::HashSet<String>,
    doc: &DesignDoc,
    pan: Vector,
    zoom: f32,
    selected_id: Option<&str>,
    selected_fill_index: Option<usize>,
    sel_color: Color,
    show_handles: bool,
    cursor_pos: Option<Point>,
    hovered_tailwind_selection: Option<&(String, Vec<usize>)>,
) {
    // 绘制所有选中元素的边界框
    for sel_id in selected_ids {
        if let Some(rect) = get_element_screen_bounds(doc, sel_id, pan, zoom) {
            // 绘制矩形边界框
            frame.stroke(
                &Path::rectangle(Point::new(rect.x, rect.y), Size::new(rect.width, rect.height)),
                Stroke::default().with_color(sel_color).with_width(2.2),
            );
            // 如果需要，绘制调整手柄
            if show_handles {
                draw_handles(
                    frame,
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    zoom,
                    sel_color,
                    cursor_pos,
                );
            }
        }
    }

    // 处理Mesh填充编辑模式：显示网格控制点
    if let Some(selected_id) = selected_id
        && let Some(rect) = get_element_screen_bounds(doc, selected_id, pan, zoom)
        && let Some(element) = doc.find_element(selected_id)
    {
        let fills = parse_fill_items(&element.fill);
        if let Some((_, mesh)) = extract_mesh_fill(&fills, selected_fill_index) {
            draw_mesh_points_and_handles(frame, rect, &mesh);
        }
    }

    // Tailwind组件选择的颜色配置
    let tailwind_green = Color::from_rgb(0.18, 0.78, 0.40);
    let tailwind_selected_bg = Color::from_rgba(0.18, 0.78, 0.40, 0.16);
    let dash_segments = [4.0, 3.0];

    // 绘制已选中的Tailwind组件内部元素
    if let Some(selected_id) = selected_id
        && let Some((tailwind_id, path)) = &doc.tailwind_selection
        && tailwind_id == selected_id
        && let Some(rect) = get_element_screen_bounds(doc, selected_id, pan, zoom)
        && let Some(element) = doc.find_element(selected_id)
        && element.kind.eq_ignore_ascii_case("tailwind")
        && let Some(content) = &element.content
    {
        // 解析HTML内容并定位选中的节点
        let nodes = tailwind::parse_html(content);
        if let Some(node_rect) = tailwind::renderer::bounds_for_path(
            &nodes,
            Rectangle { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
            zoom,
            path,
        ) {
            // 绘制选中节点的圆角矩形背景
            let path = Path::rounded_rectangle(
                Point::new(node_rect.x, node_rect.y),
                Size::new(node_rect.width, node_rect.height),
                4.0.into(),
            );
            frame.fill(&path, tailwind_selected_bg);
            // 绘制虚线边框
            let mut stroke =
                Stroke { style: tailwind_green.into(), width: 2.0, ..Stroke::default() };
            stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
            frame.stroke(&path, stroke);
        }
    }

    // 绘制悬停的Tailwind组件元素（非选中状态）
    if let Some((tailwind_id, path)) = hovered_tailwind_selection
        && let Some(rect) = get_element_screen_bounds(doc, tailwind_id, pan, zoom)
        && let Some(element) = doc.find_element(tailwind_id)
        && element.kind.eq_ignore_ascii_case("tailwind")
        && let Some(content) = &element.content
        // 确保不是已经选中的元素（避免重复绘制）
        && !doc.tailwind_selection.as_ref().is_some_and(|(sel_id, sel_path)| {
            sel_id == tailwind_id && sel_path.as_slice() == path.as_slice()
        })
    {
        let nodes = tailwind::parse_html(content);
        if let Some(node_rect) = tailwind::renderer::bounds_for_path(
            &nodes,
            Rectangle { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
            zoom,
            path,
        ) {
            // 悬停状态只绘制边框，不填充背景
            let path = Path::rounded_rectangle(
                Point::new(node_rect.x, node_rect.y),
                Size::new(node_rect.width, node_rect.height),
                4.0.into(),
            );
            let mut stroke =
                Stroke { style: tailwind_green.into(), width: 2.0, ..Stroke::default() };
            stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
            frame.stroke(&path, stroke);
        }
    }
}

/// 绘制悬停编辑覆盖层，显示可复用元素的内部布局结构
///
/// 当用户悬停在标记为可复用（reusable）的元素上时，
/// 此函数会显示该元素的内部布局细节，包括：
/// - 元素的外边界
/// - 子元素的虚线边框（特别是文本、图标和内容区域）
/// - 嵌套的内部布局结构
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `hovered_id` - 悬停的元素ID
/// * `doc` - 设计文档引用
/// * `pan` - 画布平移偏移量
/// * `zoom` - 画布缩放级别
/// * `border_color` - 外部边框颜色
/// * `internal_border_color` - 内部边框颜色（用于子元素）
pub fn draw_hover_edit_overlay(
    frame: &mut Frame,
    hovered_id: Option<&str>,
    doc: &DesignDoc,
    pan: Vector,
    zoom: f32,
    border_color: Color,
    internal_border_color: Color,
) {
    let Some(hovered_id) = hovered_id else {
        return;
    };
    let Some(rect) = get_element_screen_bounds(doc, hovered_id, pan, zoom) else {
        return;
    };

    let path = Path::rectangle(Point::new(rect.x, rect.y), Size::new(rect.width, rect.height));
    frame.stroke(&path, Stroke { style: border_color.into(), width: 2.2, ..Stroke::default() });

    let Some(el) = doc.find_element(hovered_id) else {
        return;
    };
    // 只为可复用元素显示内部布局
    if el.reusable != Some(true) {
        return;
    }

    // 解析元素的主题模式和内边距
    let theme_mode = theme_mode_for_element(doc, el, None);
    let padding = parse_padding(&el.padding, &doc.variables, theme_mode);
    // 计算容器和内容区域的实际尺寸
    let container_size = if zoom.abs() > f32::EPSILON {
        Size::new(rect.width / zoom, rect.height / zoom)
    } else {
        Size::new(0.0, 0.0)
    };
    let content_size = Size::new(
        (container_size.width - padding.left - padding.right).max(0.0),
        (container_size.height - padding.top - padding.bottom).max(0.0),
    );

    // 确定布局方向（水平或垂直）
    let layout = parse_layout(&el.layout).or_else(|| {
        if el.layout.as_deref() == Some("none") {
            None
        } else if el.justify_content.is_some() || el.align_items.is_some() || el.gap.is_some() {
            Some(LayoutDirection::Horizontal)
        } else {
            None
        }
    });

    let dash_segments = [3.0, 3.0];
    if let Some(direction) = layout {
        // 计算所有子元素的布局位置
        let layouts = compute_layout(direction, &el.children, content_size, el, doc, theme_mode);

        // 判断子元素是否应该显示虚线边框
        let should_show_dashed = |child: &crate::app::views::design::models::DesignElement| {
            let name = child.name.as_deref().unwrap_or("");
            let lower = name.to_ascii_lowercase();
            // 文本类型元素
            let is_text = child.kind.eq_ignore_ascii_case("typography")
                || child.kind.eq_ignore_ascii_case("text");
            // 图标元素（名称包含'icon'）
            let is_icon = lower.contains("icon");
            // 内容区域（名称包含'content'）
            let is_contents = lower.contains("content");

            // 检查是否有隐藏的边框（边框厚度存在但填充为空）
            let hidden_stroke = child.stroke.as_ref().is_some_and(|s| {
                s.thickness.is_some()
                    && s.fill.as_ref().map(|f| f.trim().is_empty()).unwrap_or(true)
            });

            hidden_stroke || is_text || is_icon || is_contents
        };

        // 遍历每个子元素
        for (child, layout) in el.children.iter().zip(layouts.into_iter()) {
            // 计算子元素在屏幕上的实际位置
            let child_rect = Rectangle::new(
                Point::new(
                    rect.x + (padding.left + layout.offset.x) * zoom,
                    rect.y + (padding.top + layout.offset.y) * zoom,
                ),
                Size::new(layout.size.width * zoom, layout.size.height * zoom),
            );

            // 为符合条件的子元素绘制虚线边框
            if should_show_dashed(child) {
                let child_path = Path::rounded_rectangle(
                    Point::new(child_rect.x, child_rect.y),
                    Size::new(child_rect.width, child_rect.height),
                    4.0.into(),
                );
                let mut stroke =
                    Stroke { style: internal_border_color.into(), width: 1.0, ..Stroke::default() };
                stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
                frame.stroke(&child_path, stroke);
            }

            // 处理嵌套的内容区域
            let name = child.name.as_deref().unwrap_or("");
            let lower = name.to_ascii_lowercase();
            let child_theme_mode = theme_mode_for_element(doc, child, theme_mode);
            let child_layout = parse_layout(&child.layout).or_else(|| {
                if child.layout.as_deref() == Some("none") {
                    None
                } else if child.justify_content.is_some()
                    || child.align_items.is_some()
                    || child.gap.is_some()
                {
                    Some(LayoutDirection::Horizontal)
                } else {
                    None
                }
            });

            // 如果是内容区域且有布局，递归显示其子元素
            if lower.contains("content")
                && let Some(child_direction) = child_layout {
                    let child_padding =
                        parse_padding(&child.padding, &doc.variables, child_theme_mode);
                    let child_container_size = if zoom.abs() > f32::EPSILON {
                        Size::new(child_rect.width / zoom, child_rect.height / zoom)
                    } else {
                        Size::new(0.0, 0.0)
                    };
                    let child_content_size = Size::new(
                        (child_container_size.width - child_padding.left - child_padding.right)
                            .max(0.0),
                        (child_container_size.height - child_padding.top - child_padding.bottom)
                            .max(0.0),
                    );

                    // 计算孙子元素的布局
                    let inner_layouts = compute_layout(
                        child_direction,
                        &child.children,
                        child_content_size,
                        child,
                        doc,
                        child_theme_mode,
                    );
                    // 为孙子元素绘制虚线边框
                    for (grandchild, inner_layout) in
                        child.children.iter().zip(inner_layouts.into_iter())
                    {
                        if !should_show_dashed(grandchild) {
                            continue;
                        }
                        let gc_rect = Rectangle::new(
                            Point::new(
                                child_rect.x + (child_padding.left + inner_layout.offset.x) * zoom,
                                child_rect.y + (child_padding.top + inner_layout.offset.y) * zoom,
                            ),
                            Size::new(
                                inner_layout.size.width * zoom,
                                inner_layout.size.height * zoom,
                            ),
                        );
                        let gc_path = Path::rounded_rectangle(
                            Point::new(gc_rect.x, gc_rect.y),
                            Size::new(gc_rect.width, gc_rect.height),
                            3.0.into(),
                        );
                        let mut stroke = Stroke {
                            style: internal_border_color.into(),
                            width: 1.0,
                            ..Stroke::default()
                        };
                        stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
                        frame.stroke(&gc_path, stroke);
                    }
                }
        }
    }
}

/// 绘制框选区域的可视化效果
///
/// 当用户拖拽框选多个元素时，显示一个带有视觉指示器的选择框：
/// - 半透明的蓝色填充区域
/// - 蓝色边框
/// - 起始点的蓝色圆形标记和十字准线
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `start` - 框选起始点坐标
/// * `end` - 框选结束点坐标
pub fn draw_selection_box(frame: &mut Frame, start: Point, end: Point) {
    // 计算选择框的位置和尺寸（处理任意方向的拖拽）
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);
    let width = (end.x - start.x).abs();
    let height = (end.y - start.y).abs();

    // 定义选择框的颜色
    let color = Color::from_rgba(0.0, 0.5, 1.0, 0.1);
    let stroke_color = Color::from_rgba(0.0, 0.5, 1.0, 0.8);
    let start_indicator_color = Color::from_rgba8(24, 160, 251, 1.0);

    // 绘制半透明的填充区域
    frame.fill_rectangle(Point::new(x, y), Size::new(width, height), color);
    // 绘制边框
    frame.stroke(
        &Path::rectangle(Point::new(x, y), Size::new(width, height)),
        Stroke::default().with_color(stroke_color).with_width(1.0),
    );

    // 绘制起始点指示器
    // 白色背景以增强对比度
    frame.fill(&Path::circle(start, 6.0), Color::WHITE);
    // 使用与选择框一致的蓝色起点标记，避免突兀的红色反馈
    frame.fill(&Path::circle(start, 4.0), start_indicator_color);
    // 蓝色边框
    frame.stroke(
        &Path::circle(start, 6.0),
        Stroke::default().with_color(start_indicator_color).with_width(2.0),
    );

    // 在起始点绘制十字准线
    let cross_size = 7.0;
    frame.stroke(
        &Path::line(
            Point::new(start.x - cross_size, start.y - cross_size),
            Point::new(start.x + cross_size, start.y + cross_size),
        ),
        Stroke::default().with_color(start_indicator_color).with_width(2.0),
    );
    frame.stroke(
        &Path::line(
            Point::new(start.x - cross_size, start.y + cross_size),
            Point::new(start.x + cross_size, start.y - cross_size),
        ),
        Stroke::default().with_color(start_indicator_color).with_width(2.0),
    );
}

/// 在元素边界框上绘制调整手柄
///
/// 绘制8个方形手柄（4个角 + 4个边中点），用于调整元素大小。
/// 每个手柄由白色填充和彩色边框组成。
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `x` - 元素左上角的X坐标
/// * `y` - 元素左上角的Y坐标
/// * `w` - 元素宽度
/// * `h` - 元素高度
/// * `_zoom` - 缩放级别（当前未使用）
/// * `sel_color` - 手柄边框颜色
fn draw_handles(
    frame: &mut Frame,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _zoom: f32,
    sel_color: Color,
    _cursor_pos: Option<Point>,
) {
    let handle_size = 10.0;
    let half = handle_size / 2.0;

    // 定义8个拉伸/缩放手柄的位置：4个角 + 4个边中点
    let handles = [
        (x, y),               // 左上角
        (x + w, y),           // 右上角
        (x, y + h),           // 左下角
        (x + w, y + h),       // 右下角
        (x + w / 2.0, y),     // 上边中点
        (x + w / 2.0, y + h), // 下边中点
        (x, y + h / 2.0),     // 左边中点
        (x + w, y + h / 2.0), // 右边中点
    ];

    // 绘制每个拉伸/缩放手柄（方形粗线）
    for (hx, hy) in handles {
        let top_left = Point::new(hx - half, hy - half);
        let rect = Path::rectangle(top_left, Size::new(handle_size, handle_size));
        frame.fill(&rect, Color::WHITE);
        frame.stroke(&rect, Stroke::default().with_color(sel_color).with_width(2.2));
    }
}

/// 解析填充项列表
///
/// 从JSON值中解析出填充项数组，支持数组和单个对象两种格式。
/// 对于Mesh类型的填充，会自动进行归一化处理。
///
/// # 参数
///
/// * `v` - 可选的JSON值，包含填充配置
///
/// # 返回
///
/// 返回解析后的填充项向量。如果解析失败或值为空，返回空向量。
fn parse_fill_items(v: &Option<serde_json::Value>) -> Vec<FillItem> {
    let Some(v) = v else {
        return vec![];
    };

    // 尝试解析为数组格式
    if let Ok(mut fills) = serde_json::from_value::<Vec<FillItem>>(v.clone()) {
        // 归一化所有Mesh类型的填充
        for item in &mut fills {
            if let FillItem::Object(FillObject::Mesh(m)) = item {
                m.normalize();
            }
        }
        return fills;
    }

    // 尝试解析为单个对象格式
    if let Ok(mut item) = serde_json::from_value::<FillItem>(v.clone()) {
        if let FillItem::Object(FillObject::Mesh(m)) = &mut item {
            m.normalize();
        }
        return vec![item];
    }

    // 解析失败，返回空数组
    vec![]
}

/// 从填充项列表中提取启用的Mesh填充
///
/// 根据指定的索引或查找第一个启用的Mesh填充。
///
/// # 参数
///
/// * `fills` - 填充项列表
/// * `selected_fill_index` - 可选的指定填充索引
///
/// # 返回
///
/// 如果找到启用的Mesh填充，返回包含索引和Mesh对象的元组；
/// 否则返回None。
fn extract_mesh_fill(
    fills: &[FillItem],
    selected_fill_index: Option<usize>,
) -> Option<(usize, MeshFill)> {
    // 如果指定了索引且该位置是启用的Mesh，直接返回
    if let Some(i) = selected_fill_index
        && let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get(i)
        && m.enabled
    {
        return Some((i, m.clone()));
    }

    // 否则查找第一个启用的Mesh填充
    for (i, item) in fills.iter().enumerate() {
        if let FillItem::Object(FillObject::Mesh(m)) = item
            && m.enabled
        {
            return Some((i, m.clone()));
        }
    }

    None
}

/// 绘制Mesh填充的控制点和手柄
///
/// 在Mesh填充编辑模式下，显示网格点和贝塞尔曲线控制手柄。
/// 这允许用户通过拖动来调整渐变效果。
///
/// # 参数
///
/// * `frame` - 要绘制到的画布帧
/// * `bounds` - 元素的边界矩形
/// * `mesh` - Mesh填充配置
fn draw_mesh_points_and_handles(frame: &mut Frame, bounds: Rectangle, mesh: &MeshFill) {
    // 确保网格至少有2x2的点
    let columns = mesh.columns.max(2);
    let rows = mesh.rows.max(2);
    let expected = columns.saturating_mul(rows);
    if expected == 0 {
        return;
    }

    // 初始化默认的点位置和手柄值
    let (mut points, mut handles) = MeshFill::default_points_and_handles(columns, rows);

    // 复制自定义的点位置（如果有）
    let copy = mesh.points.len().min(expected);
    for i in 0..copy {
        if let Some(p) = mesh.points.get(i) {
            // 坐标值限制在[0, 1]范围内（归一化坐标）
            let x = p.first().copied().unwrap_or(points[i][0]).clamp(0.0, 1.0);
            let y = p.get(1).copied().unwrap_or(points[i][1]).clamp(0.0, 1.0);
            points[i] = vec![x, y];
        }
    }

    // 复制自定义的手柄值（如果有）
    let copy_h = mesh.handles.len().min(expected);
    for i in 0..copy_h {
        if let Some(h) = mesh.handles.get(i)
            && h.len() >= 8
        {
            handles[i] = vec![h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
        }
    }

    // 定义可视化样式
    let point_radius = 5.0;
    let handle_radius = 4.0;
    let line_color = Color::from_rgba(0.0, 0.5, 1.0, 0.35);
    let point_border = Color::from_rgb(0.0, 0.5, 1.0);
    let selected_border = Color::from_rgb(0.95, 0.5, 0.0);

    // 将归一化坐标转换为屏幕坐标
    let mut pts = Vec::with_capacity(expected);
    for i in 0..expected {
        let p = points.get(i).cloned().unwrap_or_else(|| vec![0.0, 0.0]);
        let px = bounds.x + (p.first().copied().unwrap_or(0.0) as f32) * bounds.width;
        let py = bounds.y + (p.get(1).copied().unwrap_or(0.0) as f32) * bounds.height;
        pts.push(Point::new(px, py));
    }

    // 如果启用了轮廓显示，绘制网格线
    if mesh.outline {
        let grid_color = Color::from_rgba(0.0, 0.5, 1.0, 0.18);

        // 辅助函数：获取手柄点的屏幕坐标
        let handle_pt = |idx: usize, hi: usize| -> Point {
            let h = handles.get(idx);
            if let Some(h) = h
                && h.len() >= 8
                && hi < 4
            {
                let x = h[hi * 2] as f32;
                let y = h[hi * 2 + 1] as f32;
                Point::new(bounds.x + x * bounds.width, bounds.y + y * bounds.height)
            } else {
                *pts.get(idx).unwrap_or(&Point::ORIGIN)
            }
        };

        // 绘制网格的贝塞尔曲线
        for r in 0..rows {
            for c in 0..columns {
                let idx = r * columns + c;

                // 绘制水平连接线
                if c + 1 < columns {
                    let a = *pts.get(idx).unwrap_or(&Point::ORIGIN);
                    let b = *pts.get(idx + 1).unwrap_or(&Point::ORIGIN);
                    let cp1 = handle_pt(idx, 2); // 当前点的右侧手柄
                    let cp2 = handle_pt(idx + 1, 0); // 右侧点的左侧手柄
                    let path = Path::new(|builder| {
                        builder.move_to(a);
                        builder.bezier_curve_to(cp1, cp2, b);
                    });
                    frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));
                }

                // 绘制垂直连接线
                if r + 1 < rows {
                    let a = *pts.get(idx).unwrap_or(&Point::ORIGIN);
                    let b = *pts.get(idx + columns).unwrap_or(&Point::ORIGIN);
                    let cp1 = handle_pt(idx, 3); // 当前点的下侧手柄
                    let cp2 = handle_pt(idx + columns, 1); // 下侧点的上侧手柄
                    let path = Path::new(|builder| {
                        builder.move_to(a);
                        builder.bezier_curve_to(cp1, cp2, b);
                    });
                    frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));
                }
            }
        }
    }

    // 绘制所有的网格点
    for i in 0..expected {
        let point = *pts.get(i).unwrap_or(&Point::ORIGIN);

        // 检查当前点是否被选中
        let is_selected = mesh.selected_point_index.is_some_and(|idx| idx == i);
        let border = if is_selected { selected_border } else { point_border };

        // 绘制点（白色填充 + 彩色边框）
        frame.fill(&Path::circle(point, point_radius + 1.0), Color::WHITE);
        frame.stroke(
            &Path::circle(point, point_radius + 1.0),
            Stroke::default().with_color(border).with_width(2.0),
        );

        // 如果点被选中，绘制其控制手柄
        if is_selected {
            let h = mesh.effective_handles(i);
            // 确定点在网格中的位置，以决定显示哪些手柄
            let r = i / columns.max(1);
            let c = i % columns.max(1);
            let has_left = c > 0;
            let has_top = r > 0;
            let has_right = c + 1 < columns;
            let has_bottom = r + 1 < rows;

            // 四个方向的手柄：左、上、右、下
            let candidates = [(0, has_left), (1, has_top), (2, has_right), (3, has_bottom)];

            for (hi, enabled) in candidates {
                if !enabled {
                    continue;
                }
                // 计算手柄的屏幕位置
                let hx = h[hi * 2] as f32;
                let hy = h[hi * 2 + 1] as f32;
                let hp = Point::new(bounds.x + hx * bounds.width, bounds.y + hy * bounds.height);

                // 绘制从点到手柄的连线
                frame.stroke(
                    &Path::line(point, hp),
                    Stroke::default().with_color(line_color).with_width(1.0),
                );
                // 绘制手柄点
                frame.fill(&Path::circle(hp, handle_radius + 1.0), Color::WHITE);
                frame.stroke(
                    &Path::circle(hp, handle_radius + 1.0),
                    Stroke::default().with_color(border).with_width(1.0),
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod overlay_tests;
