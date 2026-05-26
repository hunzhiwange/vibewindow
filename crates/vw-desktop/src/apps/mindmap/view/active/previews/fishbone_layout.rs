use crate::app::Message;
use crate::apps::mindmap::state::FishboneLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 鱼骨图布局格式预览器
///
/// 在画布上渲染鱼骨图布局的缩略预览，展示因果分析的图形结构。
#[derive(Debug, Clone, Copy)]
pub(crate) struct FishboneLayoutFormatPreview {
    /// 要预览的鱼骨图布局格式
    pub(crate) format: FishboneLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for FishboneLayoutFormatPreview {
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
        let spine_y = h * 0.52;
        let head_x = match self.format {
            FishboneLayoutFormat::HeadRight => w * 0.78,
            FishboneLayoutFormat::HeadLeft => w * 0.22,
        };
        let tail_x = match self.format {
            FishboneLayoutFormat::HeadRight => w * 0.08,
            FishboneLayoutFormat::HeadLeft => w * 0.92,
        };
        let head = Point::new(head_x, spine_y);

        let to_rect = |center: Point, ww: f32, hh: f32| {
            Rectangle::new(Point::new(center.x - ww / 2.0, center.y - hh / 2.0), Size::new(ww, hh))
        };

        let head_rect = to_rect(head, node_w, node_h);
        let head_path =
            Path::rounded_rectangle(head_rect.position(), head_rect.size(), radius.into());
        frame.fill(&head_path, self.color.scale_alpha(0.22));
        frame.stroke(
            &head_path,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.8)),
        );

        let dir = match self.format {
            FishboneLayoutFormat::HeadRight => 1.0,
            FishboneLayoutFormat::HeadLeft => -1.0,
        };
        let apex = Point::new(
            if dir > 0.0 { head_rect.x } else { head_rect.x + head_rect.width },
            spine_y,
        );
        let base = Point::new(apex.x + dir * 10.0, spine_y);
        let arrow_w = (h * 0.12).clamp(3.5, 6.5);

        let arrow = Path::new(|b| {
            b.move_to(apex);
            b.line_to(Point::new(base.x, base.y - arrow_w));
            b.line_to(Point::new(base.x, base.y + arrow_w));
            b.close();
        });

        frame.fill(&arrow, self.color.scale_alpha(0.55));
        frame.stroke(
            &Path::line(Point::new(tail_x, spine_y), base),
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
        );

        let rib1 = Point::new(w * 0.48, spine_y - h * 0.22);
        let rib2 = Point::new(w * 0.38, spine_y + h * 0.22);
        let rib3 = Point::new(w * 0.28, spine_y - h * 0.16);

        for (rib_x, rib_y) in [(rib1.x, rib1.y), (rib2.x, rib2.y), (rib3.x, rib3.y)] {
            let x = if self.format == FishboneLayoutFormat::HeadRight { rib_x } else { w - rib_x };
            let rib_end = Point::new(x, rib_y);
            let rib_start = Point::new(x + (head_x - x) * 0.25, spine_y);

            frame.stroke(
                &Path::line(rib_start, rib_end),
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
            );

            let rib_node = to_rect(rib_end, node_w * 0.90, node_h * 0.88);
            let rib_path =
                Path::rounded_rectangle(rib_node.position(), rib_node.size(), radius.into());
            frame.fill(&rib_path, self.color.scale_alpha(0.16));
            frame.stroke(
                &rib_path,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.7)),
            );
        }

        vec![frame.into_geometry()]
    }
}
