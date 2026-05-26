//! Canvas 模块单元测试
//!
//! 本模块提供思维导图画布功能的单元测试和可视化验证工具。
//! 测试涵盖布局算法、样式渲染、折叠行为以及 SVG/PNG 导出等功能。
//!
//! # 测试分类
//!
//! - **样式测试**：验证边框宽度、虚线段、点线段等视觉元素在不同缩放级别下的一致性
//! - **布局测试**：验证节点折叠时的布局行为，确保折叠后子树正确隐藏
//! - **可视化工具**：生成 SVG 和 PNG 格式的对比截图，用于人工验证样式效果
//!
//! # 平台支持
//!
//! SVG/PNG 渲染相关功能仅在非 WASM 目标平台可用（`#[cfg(not(target_arch = "wasm32"))]`），
//! 因为 `resvg` 和 `tiny_skia` 库不支持 WebAssembly 环境。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::layout::{compute_layout_for_diagram, layout_node_rect};
    use super::style::{dash_segments_px, node_border_width_px};
    use crate::app::components::mind_map::MindNode;
    use crate::apps::mindmap::state::EdgeStyle;
    use iced::Point;
    use std::collections::{HashMap, HashSet};

    /// 测试节点边框宽度在不同缩放级别下的取值范围
    ///
    /// 验证 `node_border_width_px` 函数在缩放级别从 0.1 到 10.0 范围内，
    /// 始终返回合理的边框宽度值（2.0 到 4.0 像素之间）。
    ///
    /// 这个范围确保了：
    /// - 缩放较小时边框不会太细而不可见
    /// - 缩放较大时边框不会太粗而影响美观
    #[test]
    fn node_border_width_matches_edges() {
        for zoom in [0.1, 0.5, 1.0, 2.0, 10.0] {
            let w = node_border_width_px(zoom);
            assert!(w >= 2.0 && w <= 4.0);
        }
    }

    /// 测试虚线段在不同缩放级别下保持视觉长度一致
    ///
    /// 验证 `dash_segments_px` 函数在不同缩放级别下返回相同的虚线段长度。
    /// 这确保了无论用户如何缩放视图，虚线的视觉密度保持一致。
    ///
    /// # 测试内容
    ///
    /// - 虚线样式（Dashed）在 1.0x 和 2.0x 缩放下返回相同的段长度
    /// - 点线样式（Dotted）在 1.0x 和 2.0x 缩放下返回相同的段长度
    #[test]
    fn dash_segments_keep_visual_length_across_zoom() {
        let s1 = dash_segments_px(EdgeStyle::Dashed, 1.0).unwrap();
        let s2 = dash_segments_px(EdgeStyle::Dashed, 2.0).unwrap();
        assert_eq!(s1, s2);

        let dot1 = dash_segments_px(EdgeStyle::Dotted, 1.0).unwrap();
        let dot2 = dash_segments_px(EdgeStyle::Dotted, 2.0).unwrap();
        assert_eq!(dot1, dot2);
    }

    /// 测试虚线和点线的长度具有可区分性
    ///
    /// 验证虚线样式（Dashed）的线段长度大于点线样式（Dotted）的线段长度，
    /// 确保两种样式在视觉上有明显区别。
    #[test]
    fn dash_and_dot_lengths_are_distinguishable() {
        let dash = dash_segments_px(EdgeStyle::Dashed, 1.0).unwrap();
        let dot = dash_segments_px(EdgeStyle::Dotted, 1.0).unwrap();
        assert!(dash[0] > dot[0]);
        assert!(dash[1] > dot[1]);
    }

    /// 测试实线样式不返回虚线段
    ///
    /// 验证实线样式（Solid）调用 `dash_segments_px` 时返回 `None`，
    /// 表示实线不需要虚线段定义。
    #[test]
    fn solid_style_has_no_dashes() {
        assert!(dash_segments_px(EdgeStyle::Solid, 1.0).is_none());
    }

    /// 测试折叠功能正确隐藏整个子树
    ///
    /// 创建一个多层级的思维导图结构，验证当某个节点被折叠后，
    /// 该节点的所有子孙节点都不应该出现在布局结果中。
    ///
    /// # 测试结构
    ///
    /// ```text
    /// root
    /// └── A (折叠此节点)
    ///     └── A-1
    ///         └── A-1-a
    /// ```
    ///
    /// # 预期结果
    ///
    /// - root 节点存在（path = []）
    /// - A 节点存在（path = [0]），但处于折叠状态
    /// - A-1 节点不存在（path = [0, 0]）
    /// - A-1-a 节点不存在（path = [0, 0, 0]）
    /// - 从 A 到 A-1 的边不存在
    #[test]
    fn collapse_hides_entire_subtree_in_layout() {
        // 构建测试用的思维导图树结构
        let doc = MindNode {
            text: "root".to_string(),
            children: vec![MindNode {
                text: "A".to_string(),
                children: vec![MindNode {
                    text: "A-1".to_string(),
                    children: vec![MindNode { text: "A-1-a".to_string(), children: vec![] }],
                }],
            }],
        };

        // 准备空的配置参数
        let empty_positions = HashMap::new();
        let empty_priorities = HashMap::new();
        let empty_urls = HashMap::new();

        // 标记路径 [0]（即 A 节点）为折叠状态
        let mut collapsed = std::collections::HashSet::new();
        collapsed.insert(vec![0usize]);

        // 计算布局
        let layout = compute_layout_for_diagram(
            &doc,
            &empty_positions,
            &empty_priorities,
            &empty_urls,
            &collapsed,
            crate::apps::mindmap::state::MindMapDiagramType::MindMap,
            crate::apps::mindmap::state::MindMapLayoutFormat::default(),
            crate::apps::mindmap::state::OrgChartLayoutFormat::default(),
            crate::apps::mindmap::state::FishboneLayoutFormat::default(),
            crate::apps::mindmap::state::TimelineLayoutFormat::default(),
            crate::apps::mindmap::state::BracketLayoutFormat::default(),
            crate::apps::mindmap::state::TreeLayoutFormat::default(),
        );

        // 验证：root 节点存在
        assert!(layout.nodes.iter().any(|n| n.path.is_empty()));
        // 验证：A 节点存在（折叠的节点本身仍在布局中）
        assert!(layout.nodes.iter().any(|n| n.path == vec![0usize]));
        // 验证：A-1 节点不存在（被折叠隐藏）
        assert!(!layout.nodes.iter().any(|n| n.path == vec![0usize, 0usize]));
        // 验证：A-1-a 节点不存在（被折叠隐藏）
        assert!(!layout.nodes.iter().any(|n| n.path == vec![0usize, 0usize, 0usize]));

        // 验证：从 A 到 A-1 的边不存在
        assert!(
            !layout.edges.iter().any(|e| e.from == vec![0usize] && e.to == vec![0usize, 0usize])
        );
    }

    /// XML 特殊字符转义函数
    ///
    /// 将字符串中的 XML/HTML 特殊字符转义为对应的实体引用，
    /// 确保文本内容可以安全地嵌入到 SVG 或 XML 文档中。
    ///
    /// # 参数
    ///
    /// - `s`: 需要转义的原始字符串
    ///
    /// # 返回值
    ///
    /// 转义后的安全字符串
    ///
    /// # 转义规则
    ///
    /// - `&` → `&amp;`
    /// - `<` → `&lt;`
    /// - `>` → `&gt;`
    /// - `"` → `&quot;`
    /// - `'` → `&apos;`
    #[cfg(not(target_arch = "wasm32"))]
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

    /// SVG 变体数据结构
    ///
    /// 存储单个思维导图 SVG 渲染结果的元数据和内容。
    /// 用于在生成对比截图时传递渲染结果。
    #[cfg(not(target_arch = "wasm32"))]
    struct SvgVariant {
        /// SVG 组元素内容（<g> 标签及其子元素）
        group: String,
        /// SVG 画布总宽度
        width: f32,
        /// SVG 画布总高度
        height: f32,
    }

    /// 构建单个思维导图的 SVG 变体
    ///
    /// 根据指定的边样式和缩放级别，将思维导图渲染为 SVG 格式。
    /// 返回包含 SVG 内容和尺寸信息的数据结构。
    ///
    /// # 参数
    ///
    /// - `doc`: 思维导图节点树结构
    /// - `style`: 边的样式（实线、虚线、点线）
    /// - `zoom`: 缩放级别（1.0 表示 100%）
    ///
    /// # 返回值
    ///
    /// 返回 `SvgVariant` 结构，包含：
    /// - `group`: SVG 组元素内容，可直接嵌入到更大的 SVG 中
    /// - `width`: 计算后的画布宽度
    /// - `height`: 计算后的画布高度
    ///
    /// # 实现细节
    ///
    /// 1. 计算布局位置
    /// 2. 计算包围盒（包括节点和边的控制点）
    /// 3. 生成边路径（使用贝塞尔曲线）
    /// 4. 生成节点矩形和文本标签
    #[cfg(not(target_arch = "wasm32"))]
    fn build_svg_variant(doc: &MindNode, style: EdgeStyle, zoom: f32) -> SvgVariant {
        // 准备空的配置参数（使用默认布局行为）
        let empty_positions = HashMap::new();
        let empty_priorities = HashMap::new();
        let empty_urls = HashMap::new();
        let empty_collapsed = HashSet::new();

        // 计算布局
        let layout = compute_layout_for_diagram(
            doc,
            &empty_positions,
            &empty_priorities,
            &empty_urls,
            &empty_collapsed,
            crate::apps::mindmap::state::MindMapDiagramType::MindMap,
            crate::apps::mindmap::state::MindMapLayoutFormat::default(),
            crate::apps::mindmap::state::OrgChartLayoutFormat::default(),
            crate::apps::mindmap::state::FishboneLayoutFormat::default(),
            crate::apps::mindmap::state::TimelineLayoutFormat::default(),
            crate::apps::mindmap::state::BracketLayoutFormat::default(),
            crate::apps::mindmap::state::TreeLayoutFormat::default(),
        );

        // 计算边框宽度（根据缩放级别调整）
        let stroke_width = node_border_width_px(zoom);

        // 初始化包围盒边界值
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        // 计算所有节点的包围盒
        for n in &layout.nodes {
            let r = layout_node_rect(n);
            // 应用缩放变换
            let x = r.x * zoom;
            let y = r.y * zoom;
            let w = r.width * zoom;
            let h = r.height * zoom;
            // 更新包围盒边界
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + w);
            max_y = max_y.max(y + h);
        }

        // 计算所有边（包括贝塞尔曲线控制点）的包围盒
        for e in &layout.edges {
            let from = layout.nodes.iter().find(|n| n.path == e.from);
            let to = layout.nodes.iter().find(|n| n.path == e.to);
            if let (Some(a), Some(b)) = (from, to) {
                let a_rect = layout_node_rect(a);
                let b_rect = layout_node_rect(b);

                // 判断边的方向（向右或向左）
                let to_right = b.pos.x >= a.pos.x;

                // 计算边的起点（从源节点的右侧或左侧出发）
                let start_world = if to_right {
                    Point::new(a_rect.x + a_rect.width, a.pos.y)
                } else {
                    Point::new(a_rect.x, a.pos.y)
                };

                // 计算边的终点（到达目标节点的左侧或右侧）
                let end_world = if to_right {
                    Point::new(b_rect.x, b.pos.y)
                } else {
                    Point::new(b_rect.x + b_rect.width, b.pos.y)
                };

                // 应用缩放变换
                let start = Point::new(start_world.x * zoom, start_world.y * zoom);
                let end = Point::new(end_world.x * zoom, end_world.y * zoom);

                // 计算贝塞尔曲线控制点距离
                let dx = (end.x - start.x).abs();
                let control = (dx * 0.5).clamp(40.0, 220.0);
                let dir = if end.x >= start.x { 1.0 } else { -1.0 };

                // 计算两个控制点
                let c1 = Point::new(start.x + control * dir, start.y);
                let c2 = Point::new(end.x - control * dir, end.y);

                // 将控制点纳入包围盒计算
                for p in [start, end, c1, c2] {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
        }

        // 如果没有有效的边界值（空图），使用默认尺寸
        if !min_x.is_finite() {
            min_x = 0.0;
            min_y = 0.0;
            max_x = 1.0;
            max_y = 1.0;
        }

        // 计算画布尺寸（添加边距）
        let margin = 28.0f32;
        let tx = -min_x + margin;
        let ty = -min_y + margin;
        let width = (max_x - min_x) + margin * 2.0;
        let height = (max_y - min_y) + margin * 2.0;

        // 定义颜色方案
        let stroke = "#D0D7DE"; // 边框颜色（灰色）
        let fill = "#FFFFFF"; // 填充颜色（白色）
        let text = "#111827"; // 文本颜色（深灰）

        // 构建虚线样式属性（如果样式需要）
        let dash_attr = dash_segments_px(style, zoom)
            .map(|s| format!(" stroke-dasharray=\"{:.2} {:.2}\"", s[0], s[1]))
            .unwrap_or_default();

        // 点线样式需要圆角端点
        let cap_attr = if style == EdgeStyle::Dotted { " stroke-linecap=\"round\"" } else { "" };

        // 开始构建 SVG 组元素
        let mut group = String::new();
        group.push_str(&format!("<g transform=\"translate({tx:.2},{ty:.2})\">"));

        // 渲染所有边
        for e in &layout.edges {
            let from = layout.nodes.iter().find(|n| n.path == e.from);
            let to = layout.nodes.iter().find(|n| n.path == e.to);
            if let (Some(a), Some(b)) = (from, to) {
                let a_rect = layout_node_rect(a);
                let b_rect = layout_node_rect(b);

                // 计算边的起点和终点（假设总是向右方向）
                let start_world = Point::new(a_rect.x + a_rect.width, a.pos.y);
                let end_world = Point::new(b_rect.x, b.pos.y);

                let start = Point::new(start_world.x * zoom, start_world.y * zoom);
                let end = Point::new(end_world.x * zoom, end_world.y * zoom);

                // 计算贝塞尔曲线控制点
                let dx = (end.x - start.x).abs();
                let control = (dx * 0.5).clamp(40.0, 220.0);
                let c1 = Point::new(start.x + control, start.y);
                let c2 = Point::new(end.x - control, end.y);

                // 生成贝塞尔曲线路径
                group.push_str(&format!(
                        "<path d=\"M {sx:.2} {sy:.2} C {c1x:.2} {c1y:.2} {c2x:.2} {c2y:.2} {ex:.2} {ey:.2}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                        sx = start.x,
                        sy = start.y,
                        c1x = c1.x,
                        c1y = c1.y,
                        c2x = c2.x,
                        c2y = c2.y,
                        ex = end.x,
                        ey = end.y,
                        sw = stroke_width,
                    ));
            }
        }

        // 渲染所有节点
        for n in &layout.nodes {
            let r = layout_node_rect(n);
            // 应用缩放变换
            let x = r.x * zoom;
            let y = r.y * zoom;
            let w = r.width * zoom;
            let h = r.height * zoom;

            // 圆角半径为高度的一半（胶囊形状）
            let radius = (h / 2.0).max(0.0);

            // 渲染节点矩形（胶囊形状）
            group.push_str(&format!(
                    "<rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{radius:.2}\" ry=\"{radius:.2}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw:.2}\"{dash_attr}{cap_attr} />",
                    sw = stroke_width
                ));

            // 渲染节点文本标签
            let font_size = (14.0 * zoom).clamp(10.0, 24.0);
            group.push_str(&format!(
                    "<text x=\"{cx:.2}\" y=\"{cy:.2}\" fill=\"{text}\" font-size=\"{fs:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"system-ui, -apple-system, Segoe UI, sans-serif\">{label}</text>",
                    cx = x + w / 2.0,
                    cy = y + h / 2.0,
                    fs = font_size,
                    label = escape_xml(&n.text)
                ));
        }

        group.push_str("</g>");

        SvgVariant { group, width, height }
    }

    /// 构建对比 SVG 图像
    ///
    /// 生成一个包含两个并排显示的思维导图变体的 SVG 图像，
    /// 用于可视化对比不同样式或缩放级别的效果。
    ///
    /// # 参数
    ///
    /// - `doc`: 思维导图节点树结构
    /// - `left_label`: 左侧变体的标签文字
    /// - `left_style`: 左侧变体的边样式
    /// - `left_zoom`: 左侧变体的缩放级别
    /// - `right_label`: 右侧变体的标签文字
    /// - `right_style`: 右侧变体的边样式
    /// - `right_zoom`: 右侧变体的缩放级别
    ///
    /// # 返回值
    ///
    /// 完整的 SVG 文档字符串
    ///
    /// # 布局结构
    ///
    /// ```text
    /// +------------------+------------------+
    /// | 左侧标签         | 右侧标签         |
    /// +------------------+------------------+
    /// |                  |                  |
    /// |   左侧思维导图   |   右侧思维导图   |
    /// |                  |                  |
    /// +------------------+------------------+
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    fn build_comparison_svg(
        doc: &MindNode,
        left_label: &str,
        left_style: EdgeStyle,
        left_zoom: f32,
        right_label: &str,
        right_style: EdgeStyle,
        right_zoom: f32,
    ) -> String {
        // 构建左右两个 SVG 变体
        let left = build_svg_variant(doc, left_style, left_zoom);
        let right = build_svg_variant(doc, right_style, right_zoom);

        // 计算整体布局尺寸
        let gap = 24.0f32; // 两个变体之间的间距
        let label_h = 28.0f32; // 标签区域高度
        let width = left.width + gap + right.width;
        let height = left.height.max(right.height) + label_h;

        // 开始构建 SVG 文档
        let mut svg = String::new();
        svg.push_str(&format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" viewBox=\"0 0 {w:.0} {h:.0}\">",
                w = width,
                h = height
            ));

        // 添加白色背景
        svg.push_str("<rect x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" fill=\"#FFFFFF\" />");

        // 添加左侧标签
        svg.push_str(&format!(
                "<text x=\"{x:.2}\" y=\"{y:.2}\" fill=\"#111827\" font-size=\"14\" font-family=\"system-ui, -apple-system, Segoe UI, sans-serif\">{t}</text>",
                x = 16.0,
                y = 18.0,
                t = escape_xml(left_label)
            ));

        // 添加右侧标签
        svg.push_str(&format!(
                "<text x=\"{x:.2}\" y=\"{y:.2}\" fill=\"#111827\" font-size=\"14\" font-family=\"system-ui, -apple-system, Segoe UI, sans-serif\">{t}</text>",
                x = left.width + gap + 16.0,
                y = 18.0,
                t = escape_xml(right_label)
            ));

        // 添加左侧思维导图组
        svg.push_str(&format!("<g transform=\"translate(0,{label_h:.2})\">{}</g>", left.group));

        // 添加右侧思维导图组
        svg.push_str(&format!(
            "<g transform=\"translate({x:.2},{label_h:.2})\">{}</g>",
            right.group,
            x = left.width + gap
        ));

        svg.push_str("</svg>");
        svg
    }

    /// 将 SVG 渲染为 PNG 图像
    ///
    /// 使用 `resvg` 库将 SVG 数据渲染为 PNG 格式的位图。
    /// 自动加载系统字体以支持文本渲染。
    ///
    /// # 参数
    ///
    /// - `svg_data`: SVG 文档字符串
    ///
    /// # 返回值
    ///
    /// 成功时返回 `Some(Vec<u8>)`，包含 PNG 编码的图像数据；
    /// 解析或渲染失败时返回 `None`。
    ///
    /// # 依赖
    ///
    /// - `resvg`: SVG 解析和渲染库
    /// - `tiny_skia`: 软件渲染后端
    /// - `usvg`: SVG 解析器
    #[cfg(not(target_arch = "wasm32"))]
    fn render_svg_to_png(svg_data: &str) -> Option<Vec<u8>> {
        use resvg::usvg::{self};
        use tiny_skia::{Pixmap, Transform};

        // 配置 SVG 解析选项
        let mut opt = usvg::Options::default();

        // 加载系统字体数据库（用于文本渲染）
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        opt.fontdb = std::sync::Arc::new(fontdb);

        // 解析 SVG 文档
        let tree = usvg::Tree::from_str(svg_data, &opt).ok()?;

        // 创建像素图缓冲区
        let size = tree.size().to_int_size();
        let mut pixmap = Pixmap::new(size.width(), size.height())?;

        // 渲染 SVG 到像素图
        let mut pm = pixmap.as_mut();
        resvg::render(&tree, Transform::default(), &mut pm);

        // 编码为 PNG 格式
        pixmap.encode_png().ok()
    }

    /// 生成思维导图对比截图
    ///
    /// 这是一个手动运行的测试（标记为 `#[ignore]`），用于生成思维导图
    /// 在不同样式和缩放级别下的对比截图，保存为 PNG 文件。
    ///
    /// # 用途
    ///
    /// - 验证样式效果的可视化呈现
    /// - 生成文档和演示材料
    /// - 对比不同配置下的渲染效果
    ///
    /// # 输出位置
    ///
    /// 所有截图保存到 `docs/mindmap-screenshots/` 目录。
    ///
    /// # 生成的截图
    ///
    /// 1. `01-solid-vs-dashed.png`: 实线样式 vs 虚线样式
    /// 2. `02-dashed-vs-dotted.png`: 虚线样式 vs 点线样式
    /// 3. `03-zoom-1-vs-2.png`: 1.0x 缩放 vs 2.0x 缩放
    ///
    /// # 运行方式
    ///
    /// ```bash
    /// cargo test generate_mindmap_comparison_screenshots -- --ignored
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore]
    fn generate_mindmap_comparison_screenshots() {
        use std::fs;
        use std::io::Write;

        // 构建示例思维导图（中文内容）
        let doc = MindNode {
            text: "中心主题".to_string(),
            children: vec![
                MindNode {
                    text: "分支 A".to_string(),
                    children: vec![
                        MindNode { text: "A-1".to_string(), children: vec![] },
                        MindNode { text: "A-2".to_string(), children: vec![] },
                    ],
                },
                MindNode {
                    text: "分支 B".to_string(),
                    children: vec![
                        MindNode { text: "B-1".to_string(), children: vec![] },
                        MindNode { text: "B-2".to_string(), children: vec![] },
                    ],
                },
                MindNode { text: "分支 C".to_string(), children: vec![] },
            ],
        };

        // 创建输出目录
        let out_dir = std::path::Path::new("docs/mindmap-screenshots");
        let _ = fs::create_dir_all(out_dir);

        // 生成实线 vs 虚线对比截图
        let svg1 = build_comparison_svg(
            &doc,
            "Solid (旧)",
            EdgeStyle::Solid,
            1.0,
            "Dashed (新)",
            EdgeStyle::Dashed,
            1.0,
        );
        let png1 = render_svg_to_png(&svg1).expect("render solid vs dashed");
        let mut f1 = fs::File::create(out_dir.join("01-solid-vs-dashed.png")).unwrap();
        f1.write_all(&png1).unwrap();

        // 生成虚线 vs 点线对比截图
        let svg2 = build_comparison_svg(
            &doc,
            "Dashed",
            EdgeStyle::Dashed,
            1.0,
            "Dotted",
            EdgeStyle::Dotted,
            1.0,
        );
        let png2 = render_svg_to_png(&svg2).expect("render dashed vs dotted");
        let mut f2 = fs::File::create(out_dir.join("02-dashed-vs-dotted.png")).unwrap();
        f2.write_all(&png2).unwrap();

        // 生成 1.0x vs 2.0x 缩放对比截图
        let svg3 = build_comparison_svg(
            &doc,
            "Zoom 1.0x",
            EdgeStyle::Dashed,
            1.0,
            "Zoom 2.0x",
            EdgeStyle::Dashed,
            2.0,
        );
        let png3 = render_svg_to_png(&svg3).expect("render zoom compare");
        let mut f3 = fs::File::create(out_dir.join("03-zoom-1-vs-2.png")).unwrap();
        f3.write_all(&png3).unwrap();
    }
}
