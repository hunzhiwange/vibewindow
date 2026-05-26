use crate::app::Message;
use crate::apps::mindmap::state::BracketLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 括号布局格式预览器
///
/// 在画布上渲染括号布局的缩略预览，展示使用括号连接的节点结构。
#[derive(Debug, Clone, Copy)]
pub(crate) struct BracketLayoutFormatPreview {
    /// 要预览的括号布局格式
    pub(crate) format: BracketLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for BracketLayoutFormatPreview {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width.max(1.0);
        let h = bounds.height.max(1.0);
        let node_h = (h * 0.22).clamp(10.0, 14.0);
        let node_w = (w * 0.30).clamp(18.0, 30.0);
        let radius = (node_h / 2.0).max(0.0);
        let stroke_w = (h * 0.06).clamp(1.0, 1.6);
        let on_right = self.format == BracketLayoutFormat::BraceRight;
        let root_x = if on_right { w * 0.30 } else { w * 0.70 };
        let child_x = if on_right { w * 0.72 } else { w * 0.28 };
        let root = Point::new(root_x, h * 0.50);
        let child1 = Point::new(child_x, h * 0.35);
        let child2 = Point::new(child_x, h * 0.65);

        let to_rect = |center: Point| {
            Rectangle::new(
                Point::new(center.x - node_w / 2.0, center.y - node_h / 2.0),
                Size::new(node_w, node_h),
            )
        };

        let root_rect = to_rect(root);
        let root_path =
            Path::rounded_rectangle(root_rect.position(), root_rect.size(), radius.into());
        frame.fill(&root_path, self.color.scale_alpha(0.22));
        frame.stroke(
            &root_path,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.8)),
        );

        for child in [child1, child2] {
            let r = to_rect(child);
            let p = Path::rounded_rectangle(r.position(), r.size(), radius.into());
            frame.fill(&p, self.color.scale_alpha(0.16));
            frame.stroke(
                &p,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.7)),
            );
        }

        let gap = (w * 0.06).clamp(6.0, 10.0);
        let parent_edge_x = if on_right { root_rect.x + root_rect.width } else { root_rect.x };
        let x0 = if on_right { parent_edge_x + gap } else { parent_edge_x - gap };
        let concave_dir = if on_right { -1.0 } else { 1.0 };
        let brace_w = (w * 0.08).clamp(8.0, 14.0);
        let notch_x = x0 + concave_dir * brace_w;
        let bulge_x = x0 - concave_dir * brace_w;
        let y_top = child1.y - node_h / 2.0;
        let y_bottom = child2.y + node_h / 2.0;
        let y_mid = (y_top + y_bottom) / 2.0;
        let notch_dy = ((y_bottom - y_top) * 0.10).clamp(6.0, 10.0);

        let brace = Path::new(|b| {
            b.move_to(Point::new(x0, y_top));
            b.bezier_curve_to(
                Point::new(bulge_x, y_top),
                Point::new(bulge_x, y_mid - notch_dy),
                Point::new(x0, y_mid - notch_dy),
            );
            b.bezier_curve_to(
                Point::new(x0, y_mid - notch_dy * 0.35),
                Point::new(notch_x, y_mid - notch_dy * 0.35),
                Point::new(notch_x, y_mid),
            );
            b.bezier_curve_to(
                Point::new(notch_x, y_mid + notch_dy * 0.35),
                Point::new(x0, y_mid + notch_dy * 0.35),
                Point::new(x0, y_mid + notch_dy),
            );
            b.bezier_curve_to(
                Point::new(bulge_x, y_mid + notch_dy),
                Point::new(bulge_x, y_bottom),
                Point::new(x0, y_bottom),
            );
        });

        frame.stroke(
            &brace,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.62)),
        );

        let connector = Path::new(|b| {
            b.move_to(Point::new(parent_edge_x, root.y));
            b.line_to(Point::new(x0, root.y));
            b.line_to(Point::new(x0, y_mid));
        });

        frame.stroke(
            &connector,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
        );

        for (cy, child_edge_x) in [
            (child1.y, if on_right { child_x - node_w / 2.0 } else { child_x + node_w / 2.0 }),
            (child2.y, if on_right { child_x - node_w / 2.0 } else { child_x + node_w / 2.0 }),
        ] {
            frame.stroke(
                &Path::line(Point::new(notch_x, cy), Point::new(child_edge_x, cy)),
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
            );
        }

        vec![frame.into_geometry()]
    }
}
