use crate::app::Message;
use crate::apps::mindmap::state::MindMapLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 思维导图布局格式预览器
///
/// 在画布上渲染思维导图布局的缩略预览，展示节点之间的连接关系。
#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutFormatPreview {
    /// 要预览的布局格式
    pub(crate) format: MindMapLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for LayoutFormatPreview {
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
        let node_w = (w * 0.34).clamp(18.0, 28.0);
        let radius = (node_h / 2.0).max(0.0);
        let stroke_w = (h * 0.06).clamp(1.0, 1.6);
        let root = Point::new(w * 0.42, h * 0.50);
        let y_gap = h * 0.22;
        let x_gap = w * 0.22;
        let child_ys = [root.y - y_gap, root.y + y_gap];

        let mut child_centers: Vec<Point> = Vec::new();
        match self.format {
            MindMapLayoutFormat::RightAligned => {
                for &y in &child_ys {
                    child_centers.push(Point::new(root.x + x_gap, y));
                }
            }
            MindMapLayoutFormat::LeftAligned => {
                for &y in &child_ys {
                    child_centers.push(Point::new(root.x - x_gap, y));
                }
            }
            MindMapLayoutFormat::Bidirectional => {
                child_centers.push(Point::new(root.x + x_gap, child_ys[0]));
                child_centers.push(Point::new(root.x - x_gap, child_ys[0]));
                child_centers.push(Point::new(root.x + x_gap, child_ys[1]));
                child_centers.push(Point::new(root.x - x_gap, child_ys[1]));
            }
        }

        let to_rect = |center: Point, ww: f32, hh: f32| {
            Rectangle::new(Point::new(center.x - ww / 2.0, center.y - hh / 2.0), Size::new(ww, hh))
        };

        let root_rect = to_rect(root, node_w, node_h);
        let root_path =
            Path::rounded_rectangle(root_rect.position(), root_rect.size(), radius.into());
        frame.fill(&root_path, self.color.scale_alpha(0.22));
        frame.stroke(
            &root_path,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.8)),
        );

        for c in child_centers {
            let child_rect = to_rect(c, node_w * 0.92, node_h * 0.90);
            let child_path =
                Path::rounded_rectangle(child_rect.position(), child_rect.size(), radius.into());
            let to_right = c.x >= root.x;
            let start = if to_right {
                Point::new(root_rect.x + root_rect.width, root.y)
            } else {
                Point::new(root_rect.x, root.y)
            };
            let end = if to_right {
                Point::new(child_rect.x, c.y)
            } else {
                Point::new(child_rect.x + child_rect.width, c.y)
            };
            let dx = (end.x - start.x).abs();
            let control = (dx * 0.5).clamp(10.0, 60.0);
            let dir = if end.x >= start.x { 1.0 } else { -1.0 };
            let c1 = Point::new(start.x + control * dir, start.y);
            let c2 = Point::new(end.x - control * dir, end.y);

            let edge = Path::new(|b| {
                b.move_to(start);
                b.bezier_curve_to(c1, c2, end);
            });

            frame.stroke(
                &edge,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
            );
            frame.fill(&child_path, self.color.scale_alpha(0.16));
            frame.stroke(
                &child_path,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.7)),
            );
        }

        vec![frame.into_geometry()]
    }
}
