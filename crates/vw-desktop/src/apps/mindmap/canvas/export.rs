/// 将思维导图标签页导出为 SVG 格式字符串
///
/// # 参数
///
/// * `tab` - 思维导图标签页的状态引用，包含文档、布局、主题等信息
///
/// # 返回值
///
/// 返回完整的 SVG XML 字符串，可直接保存为 .svg 文件或嵌入 HTML
///
/// # 功能说明
///
/// 此函数执行以下主要步骤：
/// 1. 计算完整的布局数据（节点位置、边连接）
/// 2. 计算视图边界（包含所有节点、边和涂鸦的最小包围盒）
/// 3. 根据图表类型绘制背景元素（如鱼骨图的主脊线）
/// 4. 绘制所有边（支持直线、曲线、括号等多种样式）
/// 5. 绘制所有节点（包括文本、优先级标记、URL 图标）
/// 6. 绘制用户涂鸦
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn export_svg(tab: &crate::apps::mindmap::state::MindMapTab) -> String {
    use crate::apps::mindmap::canvas::{layout, style};
    use crate::apps::mindmap::canvas::layout::compute_layout_for_diagram;
    use crate::apps::mindmap::canvas::theme::resolve_theme;
    use crate::apps::mindmap::state::EdgeStyle;
    use iced::{Color, Point};
    use std::collections::HashMap;

    /// 对 XML 特殊字符进行转义，防止 XSS 和格式错误
    ///
    /// 转义规则：
    /// - `&` -> `&amp;`
    /// - `<` -> `&lt;`
    /// - `>` -> `&gt;`
    /// - `"` -> `&quot;`
    /// - `'` -> `&apos;`
    fn escape_xml(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&apos;"),
                _ => out.push(ch),
            }
        }
        out
    }

    /// 将 32 位 RGBA 颜色值转换为 CSS rgba() 格式字符串
    ///
    /// # 格式
    ///
    /// 输入格式：`0xRRGGBBAA`（R 在最高位，A 在最低位）
    /// 输出格式：`rgba(R,G,B,A)` 其中 A 范围为 0.000-1.000
    fn rgba_u32_to_css(rgba: u32) -> String {
        let r = ((rgba >> 24) & 0xFF) as u8;
        let g = ((rgba >> 16) & 0xFF) as u8;
        let b = ((rgba >> 8) & 0xFF) as u8;
        let a = (rgba & 0xFF) as u8;
        let af = (a as f32 / 255.0).clamp(0.0, 1.0);
        format!("rgba({r},{g},{b},{af:.3})")
    }

    /// 将 iced Color 结构体转换为 32 位 RGBA 整数
    ///
    /// # 格式
    ///
    /// 输入：Color { r, g, b, a } 其中各分量范围为 0.0-1.0
    /// 输出：`0xRRGGBBAA`（R 在最高位，A 在最低位）
    fn rgba_u32_from_color(color: Color) -> u32 {
        let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u32;
        let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u32;
        let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u32;
        let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }

    /// 获取优先级对应的 RGBA 颜色值
    ///
    /// # 参数
    ///
    /// * `p` - 优先级值（1-10，其中 10 表示已完成/打勾状态）
    fn priority_rgba(p: u8) -> u32 {
        rgba_u32_from_color(style::priority_color(p))
    }

    /// 根据背景色计算最佳的文本颜色（用于保证可读性）
    ///
    /// # 参数
    ///
    /// * `bg` - 背景色的 32 位 RGBA 值
    ///
    /// # 返回值
    ///
    /// 返回适合在指定背景上显示的文本颜色
    fn ideal_text_rgba(bg: u32) -> u32 {
        rgba_u32_from_color(style::ideal_text_color(style::rgba_u32_to_color(bg)))
    }

    // 计算完整的布局数据，包含所有节点和边的位置信息
    let layout = compute_layout_for_diagram(
        &tab.doc,
        &tab.node_positions,
        &tab.node_priorities,
        &tab.node_urls,
        &tab.collapsed_paths,
        tab.diagram_type,
        tab.layout_format,
        tab.org_chart_layout_format,
        tab.fishbone_layout_format,
        tab.timeline_layout_format,
        tab.bracket_layout_format,
        tab.tree_layout_format,
    );

    // 缩放级别和样式参数
    let zoom = tab.zoom.clamp(0.1, 10.0);
    let stroke_width = style::node_border_width_px(zoom);

    // 解析当前主题配置
    let current_theme = resolve_theme(&tab.theme_group, tab.theme_variant, &tab.custom_themes);

    // 确定背景颜色：优先使用自定义背景，其次跟随主题，最后使用白色
    let bg_rgba = if let Some(rgba) = tab.background {
        rgba
    } else if tab.follow_theme_background {
        current_theme.background_color
    } else {
        0xFFFFFFFF
    };

    // 初始化视图边界计算变量
    // 这些变量将用于计算包含所有内容的最小包围盒
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // 遍历所有节点，计算节点区域对视图边界的贡献
    for n in &layout.nodes {
        let r = layout::layout_node_rect(n);
        // 将世界坐标转换为缩放后的屏幕坐标
        let x = r.x * zoom;
        let y = r.y * zoom;
        let w = r.width * zoom;
        let h = r.height * zoom;
        // 更新边界值
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }

    // 如果是鱼骨图，需要额外考虑主脊线和箭头的几何范围
    if tab.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Fishbone
        && let Some(root) = layout.nodes.iter().find(|n| n.path.is_empty()) {
            let root_rect = layout::layout_node_rect(root);
            // 根据布局方向确定主脊线方向
            let spine_dir = match tab.fishbone_layout_format {
                crate::apps::mindmap::state::FishboneLayoutFormat::HeadRight => -1.0,
                crate::apps::mindmap::state::FishboneLayoutFormat::HeadLeft => 1.0,
            };
            // 计算主脊线的极端 X 坐标（考虑所有一级分支）
            let mut extreme_x = root.pos.x + spine_dir * 480.0;
            for n in layout.nodes.iter().filter(|n| n.path.len() == 1) {
                if spine_dir < 0.0 {
                    extreme_x = extreme_x.min(n.pos.x);
                } else {
                    extreme_x = extreme_x.max(n.pos.x);
                }
            }
            // 计算鱼骨图关键点：尾部、箭头基部、箭头顶点
            let tail_x = extreme_x + spine_dir * 320.0;
            let spine_y = root.pos.y;
            let apex_x = if spine_dir < 0.0 { root_rect.x } else { root_rect.x + root_rect.width };
            let apex = Point::new(apex_x * zoom, spine_y * zoom);
            let base = Point::new((apex_x + spine_dir * 18.0) * zoom, spine_y * zoom);
            let tail = Point::new(tail_x * zoom, spine_y * zoom);
            // 将这些点纳入边界计算
            for p in [apex, base, tail] {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
        }

    // 提取鱼骨图的元数据（根节点位置和主脊线方向），用于后续边的绘制
    let fishbone_meta =
        if tab.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Fishbone {
            layout.nodes.iter().find(|n| n.path.is_empty()).map(|root| {
                let spine_dir = match tab.fishbone_layout_format {
                    crate::apps::mindmap::state::FishboneLayoutFormat::HeadRight => -1.0,
                    crate::apps::mindmap::state::FishboneLayoutFormat::HeadLeft => 1.0,
                };
                (root.pos, root.size, spine_dir)
            })
        } else {
            None
        };

    // 如果是括号图，需要计算括号曲线的几何范围
    if tab.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Bracket {
        // 构建父子关系映射
        let mut children_by_parent: HashMap<Vec<usize>, Vec<Vec<usize>>> = HashMap::new();
        for e in &layout.edges {
            children_by_parent.entry(e.from.clone()).or_default().push(e.to.clone());
        }

        let gap = (14.0 * zoom).clamp(10.0, 26.0);
        let prefer_on_right = tab.bracket_layout_format
            == crate::apps::mindmap::state::BracketLayoutFormat::BraceRight;

        // 遍历每个父节点及其子节点，计算括号曲线的范围
        for (parent_path, child_paths) in children_by_parent {
            let Some(parent) = layout.nodes.iter().find(|n| n.path == parent_path) else {
                continue;
            };

            let pr = layout::layout_node_rect(parent);
            let px = pr.x * zoom;
            let py = pr.y * zoom;
            let pw = pr.width * zoom;
            let ph = pr.height * zoom;
            let parent_center_y = py + ph / 2.0;

            // 计算子节点的 Y 范围
            let mut y_top = f32::INFINITY;
            let mut y_bottom = f32::NEG_INFINITY;
            let mut child_count = 0usize;

            for child_path in child_paths {
                let Some(child) = layout.nodes.iter().find(|n| n.path == child_path) else {
                    continue;
                };
                let cr = layout::layout_node_rect(child);
                let cy = cr.y * zoom;
                let ch = cr.height * zoom;
                y_top = y_top.min(cy);
                y_bottom = y_bottom.max(cy + ch);
                child_count += 1;
            }

            // 单个子节点不需要括号曲线，跳过
            if child_count <= 1 || !y_top.is_finite() {
                continue;
            }

            // 计算括号曲线的关键 X 坐标
            let parent_edge_x = if prefer_on_right { px + pw } else { px };
            let x0 = if prefer_on_right { parent_edge_x + gap } else { parent_edge_x - gap };
            let concave_dir = if prefer_on_right { -1.0 } else { 1.0 };
            let _h = (y_bottom - y_top).max(1.0);
            let w = (10.0 * zoom).clamp(8.0, 22.0);
            let notch_x = x0 + concave_dir * w;
            let bulge_x = x0 - concave_dir * w;

            // 将括号曲线的关键点纳入边界计算
            for x in [parent_edge_x, x0, notch_x, bulge_x] {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
            for y in [y_top, y_bottom, parent_center_y] {
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    } else {
        // 对于非括号图，计算所有边对视图边界的贡献
        for e in &layout.edges {
            let from = layout.nodes.iter().find(|n| n.path == e.from);
            let to = layout.nodes.iter().find(|n| n.path == e.to);
            if let (Some(a), Some(b)) = (from, to) {
                let a_rect = layout::layout_node_rect(a);
                let b_rect = layout::layout_node_rect(b);
                // 判断是否为组织结构图的垂直布局
                let org_chart_vertical = tab.diagram_type
                    == crate::apps::mindmap::state::MindMapDiagramType::OrgChart
                    && matches!(
                        tab.org_chart_layout_format,
                        crate::apps::mindmap::state::OrgChartLayoutFormat::TopDown
                            | crate::apps::mindmap::state::OrgChartLayoutFormat::LeftRight
                    );
                // 判断是否为组织结构图的直角折线布局
                let org_chart_elbow = tab.diagram_type
                    == crate::apps::mindmap::state::MindMapDiagramType::OrgChart
                    && tab.org_chart_layout_format
                        == crate::apps::mindmap::state::OrgChartLayoutFormat::LeftRight;

                // 计算边的起点和终点坐标
                let (start_world, end_world) = if let Some((root_pos, root_size, spine_dir)) =
                    fishbone_meta
                {
                    // 鱼骨图的边连接逻辑
                    let spine_y = root_pos.y;
                    let base_branch_dx = 160.0f32;
                    let to_len = e.to.len();
                    let from_len = e.from.len();

                    if to_len == 1 {
                        // 从主脊线到一级分支
                        let branch_dx =
                            base_branch_dx.max(root_size.width / 2.0 + b.size.width / 2.0 + 140.0);
                        let spine_x = b.pos.x - spine_dir * branch_dx;
                        let start_world = Point::new(spine_x, spine_y);
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, b.pos.y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, b.pos.y)
                        };
                        (start_world, end_world)
                    } else if from_len == 1 && to_len == 2 {
                        // 从一级分支到二级分支
                        let branch_dx =
                            base_branch_dx.max(root_size.width / 2.0 + a.size.width / 2.0 + 140.0);
                        let spine_x = a.pos.x - spine_dir * branch_dx;
                        let parent_attach_x =
                            if spine_dir > 0.0 { a_rect.x } else { a_rect.x + a_rect.width };
                        let y = b.pos.y;
                        // 计算肋骨线上的连接点
                        let denom = a.pos.y - spine_y;
                        let t = if denom.abs() < 1.0 {
                            1.0
                        } else {
                            ((y - spine_y) / denom).clamp(0.0, 1.0)
                        };
                        let rib_x = spine_x + t * (parent_attach_x - spine_x);
                        let start_world = Point::new(rib_x, y);
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, y)
                        };
                        (start_world, end_world)
                    } else {
                        // 深层分支之间的连接
                        let start_world = if spine_dir > 0.0 {
                            Point::new(a_rect.x + a_rect.width, a.pos.y)
                        } else {
                            Point::new(a_rect.x, a.pos.y)
                        };
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, b.pos.y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, b.pos.y)
                        };
                        (start_world, end_world)
                    }
                } else if org_chart_vertical {
                    // 组织结构图垂直布局：从父节点底部到子节点顶部
                    (Point::new(a.pos.x, a_rect.y + a_rect.height), Point::new(b.pos.x, b_rect.y))
                } else {
                    // 默认水平布局：根据节点相对位置确定连接点
                    let to_right = b.pos.x >= a.pos.x;
                    let start_world = if to_right {
                        Point::new(a_rect.x + a_rect.width, a.pos.y)
                    } else {
                        Point::new(a_rect.x, a.pos.y)
                    };
                    let end_world = if to_right {
                        Point::new(b_rect.x, b.pos.y)
                    } else {
                        Point::new(b_rect.x + b_rect.width, b.pos.y)
                    };
                    (start_world, end_world)
                };

                // 转换为缩放后的屏幕坐标
                let start = Point::new(start_world.x * zoom, start_world.y * zoom);
                let end = Point::new(end_world.x * zoom, end_world.y * zoom);

                // 计算贝塞尔曲线的控制点
                let extra_points = if fishbone_meta.is_some() {
                    // 鱼骨图使用直线连接
                    [start, end]
                } else if org_chart_elbow {
                    // 直角折线：中点位置
                    let mid_y = (start.y + end.y) / 2.0;
                    [Point::new(start.x, mid_y), Point::new(end.x, mid_y)]
                } else if org_chart_vertical {
                    // 垂直布局的曲线控制点
                    let dy = (end.y - start.y).abs();
                    let control = (dy * 0.5).clamp(40.0, 220.0);
                    let dir = if end.y >= start.y { 1.0 } else { -1.0 };
                    [
                        Point::new(start.x, start.y + control * dir),
                        Point::new(end.x, end.y - control * dir),
                    ]
                } else {
                    // 水平布局的曲线控制点
                    let dx = (end.x - start.x).abs();
                    let control = (dx * 0.5).clamp(40.0, 220.0);
                    let dir = if end.x >= start.x { 1.0 } else { -1.0 };
                    [
                        Point::new(start.x + control * dir, start.y),
                        Point::new(end.x - control * dir, end.y),
                    ]
                };

                // 将边的所有点纳入边界计算
                for p in [start, end, extra_points[0], extra_points[1]] {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
        }
    }

    // 计算所有涂鸦点对视图边界的贡献
    for s in &tab.doodles {
        for p in &s.points_world {
            let x = p.x * zoom;
            let y = p.y * zoom;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    // 如果没有有效内容，设置默认边界
    if !min_x.is_finite() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 1.0;
        max_y = 1.0;
    }

    // 计算最终的 SVG 画布尺寸和变换参数
    let margin = 28.0f32;
    let tx = -min_x + margin;
    let ty = -min_y + margin;
    let width = (max_x - min_x) + margin * 2.0;
    let height = (max_y - min_y) + margin * 2.0;

    // 开始构建 SVG 字符串
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" viewBox=\"0 0 {w:.0} {h:.0}\">",
        w = width.max(1.0),
        h = height.max(1.0)
    ));
    // 添加背景矩形
    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" fill=\"{}\" />",
        rgba_u32_to_css(bg_rgba)
    ));
    // 添加变换组，将内容平移到可见区域
    svg.push_str(&format!("<g transform=\"translate({tx:.2},{ty:.2})\">"));

    let default_stroke_rgba = style::DEFAULT_STROKE_RGBA;

    // 绘制鱼骨图的主脊线和箭头
    if tab.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Fishbone
        && let Some(root) = layout.nodes.iter().find(|n| n.path.is_empty()) {
            let root_rect = layout::layout_node_rect(root);
            // 确定主脊线方向
            let spine_dir = match tab.fishbone_layout_format {
                crate::apps::mindmap::state::FishboneLayoutFormat::HeadRight => -1.0,
                crate::apps::mindmap::state::FishboneLayoutFormat::HeadLeft => 1.0,
            };
            // 计算主脊线的极端 X 坐标
            let mut extreme_x = root.pos.x + spine_dir * 480.0;
            for n in layout.nodes.iter().filter(|n| n.path.len() == 1) {
                if spine_dir < 0.0 {
                    extreme_x = extreme_x.min(n.pos.x);
                } else {
                    extreme_x = extreme_x.max(n.pos.x);
                }
            }
            // 计算关键点坐标
            let tail_x = extreme_x + spine_dir * 320.0;
            let spine_y = root.pos.y;
            let apex_x = if spine_dir < 0.0 { root_rect.x } else { root_rect.x + root_rect.width };
            let apex = Point::new(apex_x * zoom, spine_y * zoom);
            let base = Point::new((apex_x + spine_dir * 18.0) * zoom, spine_y * zoom);
            let tail = Point::new(tail_x * zoom, spine_y * zoom);

            // 绘制主脊线
            let spine_rgba = current_theme.line_color.unwrap_or(default_stroke_rgba);
            svg.push_str(&format!(
                "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\" />",
                x1 = tail.x,
                y1 = tail.y,
                x2 = base.x,
                y2 = base.y,
                stroke = rgba_u32_to_css(spine_rgba),
                sw = stroke_width
            ));

            // 绘制箭头
            let aw = (7.0 * zoom).clamp(4.0, 10.0);
            let b1 = Point::new(base.x, base.y - aw);
            let b2 = Point::new(base.x, base.y + aw);
            svg.push_str(&format!(
                "<polygon points=\"{ax:.2},{ay:.2} {b1x:.2},{b1y:.2} {b2x:.2},{b2y:.2}\" fill=\"{fill}\" />",
                ax = apex.x,
                ay = apex.y,
                b1x = b1.x,
                b1y = b1.y,
                b2x = b2.x,
                b2y = b2.y,
                fill = rgba_u32_to_css(spine_rgba)
            ));
        }

    // 绘制括号图的括号曲线
    if tab.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Bracket {
        /// 子节点信息结构体，用于括号图绘制
        #[derive(Clone)]
        struct Child {
            path: Vec<usize>,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            cy: f32,
        }

        // 构建父子关系映射
        let mut children_by_parent: HashMap<Vec<usize>, Vec<Vec<usize>>> = HashMap::new();
        for e in &layout.edges {
            children_by_parent.entry(e.from.clone()).or_default().push(e.to.clone());
        }

        let gap = (14.0 * zoom).clamp(10.0, 26.0);
        let prefer_on_right = tab.bracket_layout_format
            == crate::apps::mindmap::state::BracketLayoutFormat::BraceRight;

        // 绘制单个父节点下的括号组
        //
        // 参数说明：
        // - parent_edge_x: 父节点边缘的 X 坐标
        // - parent_center_y: 父节点中心的 Y 坐标
        // - children: 子节点列表（已按 Y 坐标排序）
        // - on_right: 括号是否在右侧
        let mut draw_group = |parent_edge_x: f32,
                              parent_center_y: f32,
                              children: &[Child],
                              on_right: bool| {
            if children.is_empty() {
                return;
            }

            // 获取第一条边的样式和颜色
            let first_path = &children[0].path;
            let style = tab.edge_styles.get(first_path).copied().unwrap_or(tab.edge_style);
            let color_rgba = tab.edge_colors.get(first_path).copied().unwrap_or_else(|| {
                if let Some(c) = current_theme.line_color {
                    c
                } else {
                    let branch_idx = first_path.first().copied().unwrap_or(0);
                    current_theme.palette(branch_idx)
                }
            });

            // 计算虚线属性
            let dash_attr = style::dash_segments_px(style, zoom)
                .map(|s| format!(" stroke-dasharray=\"{:.2} {:.2}\"", s[0], s[1]))
                .unwrap_or_default();
            let cap_attr =
                if style == EdgeStyle::Dotted { " stroke-linecap=\"round\"" } else { "" };

            // 单个子节点：绘制直线连接
            if children.len() == 1 {
                let c = &children[0];
                let child_edge_x = if on_right { c.x } else { c.x + c.w };
                svg.push_str(&format!(
                    "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                    x1 = parent_edge_x,
                    y1 = parent_center_y,
                    x2 = child_edge_x,
                    y2 = c.cy,
                    stroke = rgba_u32_to_css(color_rgba),
                    sw = stroke_width
                ));
                return;
            }

            // 多个子节点：绘制括号曲线
            // 计算子节点的 Y 范围
            let mut y_top = f32::INFINITY;
            let mut y_bottom = f32::NEG_INFINITY;
            for c in children {
                y_top = y_top.min(c.y);
                y_bottom = y_bottom.max(c.y + c.h);
            }
            if !y_top.is_finite() {
                return;
            }

            // 计算括号曲线的关键坐标
            let y_mid = (y_top + y_bottom) / 2.0;
            let x0 = if on_right { parent_edge_x + gap } else { parent_edge_x - gap };
            let concave_dir = if on_right { -1.0 } else { 1.0 };
            let h = (y_bottom - y_top).max(1.0);
            let w = (10.0 * zoom).clamp(8.0, 22.0);
            let notch_dy = (h * 0.07).clamp(8.0, 18.0);

            let notch_x = x0 + concave_dir * w;
            let bulge_x = x0 - concave_dir * w;

            // 绘制括号主体（使用贝塞尔曲线）
            let brace_d = format!(
                "M {x0:.2} {y0:.2} C {bx:.2} {y0:.2} {bx:.2} {y1:.2} {x0:.2} {y1:.2} \
C {x0:.2} {y2:.2} {nx:.2} {y2:.2} {nx:.2} {ym:.2} \
C {nx:.2} {y3:.2} {x0:.2} {y3:.2} {x0:.2} {y4:.2} \
C {bx:.2} {y4:.2} {bx:.2} {y5:.2} {x0:.2} {y5:.2}",
                x0 = x0,
                y0 = y_top,
                bx = bulge_x,
                y1 = y_mid - notch_dy,
                y2 = y_mid - notch_dy * 0.5,
                nx = notch_x,
                ym = y_mid,
                y3 = y_mid + notch_dy * 0.5,
                y4 = y_mid + notch_dy,
                y5 = y_bottom
            );
            svg.push_str(&format!(
                "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                d = brace_d,
                stroke = rgba_u32_to_css(color_rgba),
                sw = stroke_width
            ));

            // 绘制父节点到括号中点的连接线
            let connector_d = format!(
                "M {px:.2} {py:.2} L {x0:.2} {py:.2} L {x0:.2} {ym:.2}",
                px = parent_edge_x,
                py = parent_center_y,
                x0 = x0,
                ym = y_mid
            );
            svg.push_str(&format!(
                "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                d = connector_d,
                stroke = rgba_u32_to_css(color_rgba),
                sw = stroke_width
            ));

            // 绘制括号到各子节点的连接线
            for c in children {
                let child_edge_x = if on_right { c.x } else { c.x + c.w };
                svg.push_str(&format!(
                    "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                    x1 = notch_x,
                    y1 = c.cy,
                    x2 = child_edge_x,
                    y2 = c.cy,
                    stroke = rgba_u32_to_css(color_rgba),
                    sw = stroke_width
                ));
            }
        };

        // 遍历每个父节点，绘制括号组
        for (parent_path, child_paths) in children_by_parent {
            let Some(parent) = layout.nodes.iter().find(|n| n.path == parent_path) else {
                continue;
            };
            let pr = layout::layout_node_rect(parent);
            let px = pr.x * zoom;
            let py = pr.y * zoom;
            let pw = pr.width * zoom;
            let ph = pr.height * zoom;
            let parent_center_y = py + ph / 2.0;

            // 收集并处理子节点信息
            let mut children: Vec<Child> = Vec::new();

            for child_path in child_paths {
                let Some(child) = layout.nodes.iter().find(|n| n.path == child_path) else {
                    continue;
                };
                let cr = layout::layout_node_rect(child);
                let x = cr.x * zoom;
                let y = cr.y * zoom;
                let w = cr.width * zoom;
                let h = cr.height * zoom;
                let cy = y + h / 2.0;
                children.push(Child { path: child.path.clone(), x, y, w, h, cy });
            }

            // 按 Y 坐标排序子节点
            children.sort_by(|a, b| a.cy.total_cmp(&b.cy));

            let parent_right_x = px + pw;
            let parent_left_x = px;

            let parent_edge_x = if prefer_on_right { parent_right_x } else { parent_left_x };
            draw_group(parent_edge_x, parent_center_y, &children, prefer_on_right);
        }
    } else {
        // 绘制非括号图的边
        for e in &layout.edges {
            let from = layout.nodes.iter().find(|n| n.path == e.from);
            let to = layout.nodes.iter().find(|n| n.path == e.to);
            if let (Some(a), Some(b)) = (from, to) {
                let a_rect = layout::layout_node_rect(a);
                let b_rect = layout::layout_node_rect(b);
                // 判断是否为组织结构图的垂直布局
                let org_chart_vertical = tab.diagram_type
                    == crate::apps::mindmap::state::MindMapDiagramType::OrgChart
                    && matches!(
                        tab.org_chart_layout_format,
                        crate::apps::mindmap::state::OrgChartLayoutFormat::TopDown
                            | crate::apps::mindmap::state::OrgChartLayoutFormat::LeftRight
                    );
                // 判断是否为组织结构图的直角折线布局
                let org_chart_elbow = tab.diagram_type
                    == crate::apps::mindmap::state::MindMapDiagramType::OrgChart
                    && tab.org_chart_layout_format
                        == crate::apps::mindmap::state::OrgChartLayoutFormat::LeftRight;

                // 计算边的起点和终点坐标
                let (start_world, end_world) = if let Some((root_pos, root_size, spine_dir)) =
                    fishbone_meta
                {
                    // 鱼骨图的边连接逻辑
                    let spine_y = root_pos.y;
                    let base_branch_dx = 160.0f32;
                    let to_len = e.to.len();
                    let from_len = e.from.len();

                    if to_len == 1 {
                        // 从主脊线到一级分支
                        let branch_dx =
                            base_branch_dx.max(root_size.width / 2.0 + b.size.width / 2.0 + 140.0);
                        let spine_x = b.pos.x - spine_dir * branch_dx;
                        let start_world = Point::new(spine_x, spine_y);
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, b.pos.y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, b.pos.y)
                        };
                        (start_world, end_world)
                    } else if from_len == 1 && to_len == 2 {
                        // 从一级分支到二级分支
                        let branch_dx =
                            base_branch_dx.max(root_size.width / 2.0 + a.size.width / 2.0 + 140.0);
                        let spine_x = a.pos.x - spine_dir * branch_dx;
                        let parent_attach_x =
                            if spine_dir > 0.0 { a_rect.x } else { a_rect.x + a_rect.width };
                        let y = b.pos.y;
                        // 计算肋骨线上的连接点
                        let denom = a.pos.y - spine_y;
                        let t = if denom.abs() < 1.0 {
                            1.0
                        } else {
                            ((y - spine_y) / denom).clamp(0.0, 1.0)
                        };
                        let rib_x = spine_x + t * (parent_attach_x - spine_x);
                        let start_world = Point::new(rib_x, y);
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, y)
                        };
                        (start_world, end_world)
                    } else {
                        // 深层分支之间的连接
                        let start_world = if spine_dir > 0.0 {
                            Point::new(a_rect.x + a_rect.width, a.pos.y)
                        } else {
                            Point::new(a_rect.x, a.pos.y)
                        };
                        let end_world = if spine_dir > 0.0 {
                            Point::new(b_rect.x, b.pos.y)
                        } else {
                            Point::new(b_rect.x + b_rect.width, b.pos.y)
                        };
                        (start_world, end_world)
                    }
                } else if org_chart_vertical {
                    // 组织结构图垂直布局：从父节点底部到子节点顶部
                    (Point::new(a.pos.x, a_rect.y + a_rect.height), Point::new(b.pos.x, b_rect.y))
                } else {
                    // 默认水平布局：根据节点相对位置确定连接点
                    let to_right = b.pos.x >= a.pos.x;
                    let start_world = if to_right {
                        Point::new(a_rect.x + a_rect.width, a.pos.y)
                    } else {
                        Point::new(a_rect.x, a.pos.y)
                    };
                    let end_world = if to_right {
                        Point::new(b_rect.x, b.pos.y)
                    } else {
                        Point::new(b_rect.x + b_rect.width, b.pos.y)
                    };
                    (start_world, end_world)
                };

                // 转换为缩放后的屏幕坐标
                let start = Point::new(start_world.x * zoom, start_world.y * zoom);
                let end = Point::new(end_world.x * zoom, end_world.y * zoom);

                // 根据图表类型生成边的路径数据
                let path_d = if fishbone_meta.is_some() {
                    // 鱼骨图：直线连接
                    format!(
                        "M {sx:.2} {sy:.2} L {ex:.2} {ey:.2}",
                        sx = start.x,
                        sy = start.y,
                        ex = end.x,
                        ey = end.y
                    )
                } else if org_chart_elbow {
                    // 直角折线：通过中点连接
                    let mid_y = (start.y + end.y) / 2.0;
                    format!(
                        "M {sx:.2} {sy:.2} L {sx:.2} {my:.2} L {ex:.2} {my:.2} L {ex:.2} {ey:.2}",
                        sx = start.x,
                        sy = start.y,
                        ex = end.x,
                        ey = end.y,
                        my = mid_y
                    )
                } else if org_chart_vertical {
                    // 垂直布局：贝塞尔曲线
                    let dy = (end.y - start.y).abs();
                    let control = (dy * 0.5).clamp(40.0, 220.0);
                    format!(
                        "M {sx:.2} {sy:.2} C {c1x:.2} {c1y:.2} {c2x:.2} {c2y:.2} {ex:.2} {ey:.2}",
                        sx = start.x,
                        sy = start.y,
                        c1x = start.x,
                        c1y = start.y + control,
                        c2x = end.x,
                        c2y = end.y - control,
                        ex = end.x,
                        ey = end.y
                    )
                } else {
                    // 水平布局：贝塞尔曲线
                    let dx = (end.x - start.x).abs();
                    let control = (dx * 0.5).clamp(40.0, 220.0);
                    format!(
                        "M {sx:.2} {sy:.2} C {c1x:.2} {c1y:.2} {c2x:.2} {c2y:.2} {ex:.2} {ey:.2}",
                        sx = start.x,
                        sy = start.y,
                        c1x = start.x + control,
                        c1y = start.y,
                        c2x = end.x - control,
                        c2y = end.y,
                        ex = end.x,
                        ey = end.y
                    )
                };

                // 获取边的样式和颜色
                let style = tab.edge_styles.get(&e.to).copied().unwrap_or(tab.edge_style);
                let color_rgba = tab.edge_colors.get(&e.to).copied().unwrap_or_else(|| {
                    if let Some(c) = current_theme.line_color {
                        c
                    } else {
                        let branch_idx = if e.to.is_empty() { 0 } else { e.to[0] };
                        current_theme.palette(branch_idx)
                    }
                });

                // 计算虚线属性
                let dash_attr = style::dash_segments_px(style, zoom)
                    .map(|s| format!(" stroke-dasharray=\"{:.2} {:.2}\"", s[0], s[1]))
                    .unwrap_or_default();
                let cap_attr =
                    if style == EdgeStyle::Dotted { " stroke-linecap=\"round\"" } else { "" };

                // 绘制边
                svg.push_str(&format!(
                    "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                    d = path_d,
                    stroke = rgba_u32_to_css(color_rgba),
                    sw = stroke_width
                ));
            }
        }
    }

    // 绘制所有节点
    for n in &layout.nodes {
        let world_rect = layout::layout_node_rect(n);
        let x = world_rect.x * zoom;
        let y = world_rect.y * zoom;
        let w = world_rect.width * zoom;
        let h = world_rect.height * zoom;

        // 获取节点的优先级和 URL 标记
        let priority = tab.node_priorities.get(&n.path).copied().filter(|p| (1..=10).contains(p));
        let has_url = tab.node_urls.get(&n.path).is_some_and(|u| !u.trim().is_empty());

        // 确定节点在树中的层级
        let is_root = n.path.is_empty();

        // 根据层级获取主题颜色
        let theme_fill = if is_root {
            current_theme.root_fill
        } else if n.path.len() == 1 {
            current_theme.palette(n.path[0])
        } else {
            current_theme.leaf_fill
        };

        let theme_text = if is_root {
            current_theme.root_text
        } else if n.path.len() == 1 {
            current_theme.branch_text
        } else {
            current_theme.leaf_text
        };

        // 应用自定义颜色（如果存在）
        let fill_rgba = tab.node_fills.get(&n.path).copied().unwrap_or(theme_fill);
        let text_rgba = tab.node_text_colors.get(&n.path).copied().unwrap_or(theme_text);
        let border_rgba =
            tab.node_border_colors.get(&n.path).copied().unwrap_or(default_stroke_rgba);
        let node_style =
            tab.node_border_styles.get(&n.path).copied().unwrap_or(tab.node_border_style);

        // 计算圆角半径（根节点使用较小的圆角）
        let radius = if is_root { (8.0 * zoom).clamp(4.0, 12.0) } else { (h / 2.0).max(0.0) };

        // 计算边框虚线属性
        let dash_attr = style::dash_segments_px(node_style, zoom)
            .map(|s| format!(" stroke-dasharray=\"{:.2} {:.2}\"", s[0], s[1]))
            .unwrap_or_default();
        let cap_attr =
            if node_style == EdgeStyle::Dotted { " stroke-linecap=\"round\"" } else { "" };

        // 绘制节点矩形
        svg.push_str(&format!(
            "<rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{radius:.2}\" ry=\"{radius:.2}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
            fill = rgba_u32_to_css(fill_rgba),
            stroke = rgba_u32_to_css(border_rgba),
            sw = stroke_width
        ));

        // 绘制优先级标记（如果有）
        if let Some(p) = priority {
            let pad = (8.0 * zoom).clamp(4.0, 10.0);
            let r = (9.0 * zoom).clamp(4.0, 12.0);
            let cx = x + pad + r;
            let cy = y + h / 2.0;
            let bg = priority_rgba(p);
            let stroke_w = (1.0 * zoom).clamp(0.8, 1.6);
            // 绘制优先级圆圈背景
            svg.push_str(&format!(
                "<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{r:.2}\" fill=\"{fill}\" stroke=\"rgba(0,0,0,0.12)\" stroke-width=\"{sw:.2}\" />",
                fill = rgba_u32_to_css(bg),
                sw = stroke_w
            ));

            // 绘制优先级数字或完成标记
            if (1..=9).contains(&p) {
                // 数字优先级：显示数字
                let fs = (12.0 * zoom).clamp(9.0, 18.0);
                let fg = ideal_text_rgba(bg);
                svg.push_str(&format!(
                    "<text x=\"{cx:.2}\" y=\"{cy:.2}\" fill=\"{fill}\" font-size=\"{fs:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"system-ui, -apple-system, Segoe UI, sans-serif\">{t}</text>",
                    fill = rgba_u32_to_css(fg),
                    fs = fs,
                    t = p
                ));
            } else {
                // 优先级 10：显示完成标记（对勾图标）
                let check_geometry = "M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425z";
                let icon_size = (r * 1.15).clamp(6.0, 14.0);
                let ox = cx - icon_size / 2.0;
                let oy = cy - icon_size / 2.0;
                let s = icon_size / 16.0;
                svg.push_str(&format!(
                    "<path d=\"{d}\" fill=\"rgba(255,255,255,1)\" transform=\"translate({ox:.2},{oy:.2}) scale({s:.4})\" />",
                    d = check_geometry,
                    ox = ox,
                    oy = oy,
                    s = s
                ));
            }
        }

        // 绘制 URL 图标（如果有）
        if has_url {
            let pad = (8.0 * zoom).clamp(4.0, 10.0);
            let r = (8.0 * zoom).clamp(4.0, 12.0);
            let cx = x + w - pad - r;
            let cy = y + h / 2.0;
            let bg = 0x6B7280FF;
            let stroke_w = (1.0 * zoom).clamp(0.8, 1.6);
            // 绘制链接图标背景
            svg.push_str(&format!(
                "<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{r:.2}\" fill=\"{fill}\" stroke=\"rgba(0,0,0,0.12)\" stroke-width=\"{sw:.2}\" />",
                fill = rgba_u32_to_css(bg),
                sw = stroke_w
            ));

            // 绘制链接图标
            let link_geometry = "M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9q-.13 0-.25.031A2 2 0 0 1 7 10.5H4a2 2 0 1 1 0-4h1.535c.218-.376.495-.714.82-1z M9 5.5a3 3 0 0 0-2.83 4h1.098A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0 4h-1.535a4 4 0 0 1-.82 1H12a3 3 0 1 0 0-6z";
            let icon_size = (r * 1.25).clamp(7.0, 16.0);
            let ox = cx - icon_size / 2.0;
            let oy = cy - icon_size / 2.0;
            let s = icon_size / 16.0;
            svg.push_str(&format!(
                "<path d=\"{d}\" fill=\"rgba(255,255,255,1)\" transform=\"translate({ox:.2},{oy:.2}) scale({s:.4})\" />",
                d = link_geometry,
                ox = ox,
                oy = oy,
                s = s
            ));
        }

        // 计算文本样式参数
        let font_size =
            if is_root { (18.0 * zoom).clamp(14.0, 32.0) } else { (14.0 * zoom).clamp(10.0, 24.0) };
        let font_weight = if is_root { " font-weight=\"700\"" } else { "" };
        let has_deco = priority.is_some() || has_url;

        // 计算文本位置和锚点
        let (cx, anchor) = if has_deco {
            // 有装饰元素时，文本左对齐
            let pad = (8.0 * zoom).clamp(4.0, 10.0);
            let mut tx = x + pad;
            if priority.is_some() {
                let r = (9.0 * zoom).clamp(4.0, 12.0);
                let after = (6.0 * zoom).clamp(3.0, 10.0);
                tx += r * 2.0 + after;
            }
            (tx, "start")
        } else {
            // 无装饰元素时，文本居中
            (x + w / 2.0, "middle")
        };

        // 计算多行文本的布局
        let mut line_count = 0usize;
        for _ in n.text.split('\n') {
            line_count += 1;
        }
        let line_count = line_count.max(1);
        let line_step = (if is_root { 22.0 } else { 18.0 }) * zoom;
        let first_y = y + h / 2.0 - (line_count.saturating_sub(1) as f32) * line_step / 2.0;

        // 绘制文本
        svg.push_str(&format!(
            "<text fill=\"{text}\" font-size=\"{fs:.2}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\" font-family=\"system-ui, -apple-system, Segoe UI, sans-serif\"{font_weight}>",
            text = rgba_u32_to_css(text_rgba),
            fs = font_size,
            anchor = anchor,
            font_weight = font_weight
        ));
        // 处理多行文本
        for (i, line) in n.text.split('\n').enumerate() {
            if i == 0 {
                svg.push_str(&format!(
                    "<tspan x=\"{cx:.2}\" y=\"{y:.2}\">{t}</tspan>",
                    cx = cx,
                    y = first_y,
                    t = escape_xml(line)
                ));
            } else {
                svg.push_str(&format!(
                    "<tspan x=\"{cx:.2}\" dy=\"{dy:.2}\">{t}</tspan>",
                    cx = cx,
                    dy = line_step,
                    t = escape_xml(line)
                ));
            }
        }
        svg.push_str("</text>");
    }

    // 绘制所有涂鸦
    for s in &tab.doodles {
        if s.points_world.len() < 2 {
            continue;
        }
        // 构建涂鸦路径
        let mut d = String::new();
        let first = s.points_world[0];
        d.push_str(&format!("M {:.2} {:.2}", first.x * zoom, first.y * zoom));
        for p in &s.points_world[1..] {
            d.push_str(&format!(" L {:.2} {:.2}", p.x * zoom, p.y * zoom));
        }
        svg.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" />",
            d = d,
            stroke = rgba_u32_to_css(s.rgba),
            sw = s.width_px.clamp(1.0, 18.0)
        ));
    }

    // 关闭 SVG 标签
    svg.push_str("</g></svg>");
    svg
}

/// WASM 平台的 SVG 导出桩函数
///
/// 由于 WASM 平台不支持完整的文件系统操作和某些依赖库，
/// 此函数返回空字符串。实际导出功能需要在 WASM 环境中使用其他方式实现。
#[cfg(target_arch = "wasm32")]
pub(crate) fn export_svg(_tab: &crate::apps::mindmap::state::MindMapTab) -> String {
    String::new()
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;
