#[test]
fn polyline_geometry_requires_at_least_two_points() {
    assert!(super::build_polyline_geometry(&[]).is_none());
    let geometry =
        super::build_polyline_geometry(&[iced::Point::new(1.0, 2.0), iced::Point::new(3.0, 4.0)]);
    assert_eq!(geometry.as_deref(), Some("M 1.00 2.00 L 3.00 4.00"));
}

fn value_f32(value: &Option<serde_json::Value>) -> f32 {
    value.as_ref().and_then(|v| v.as_f64()).unwrap() as f32
}

#[test]
fn basic_shape_factories_preserve_position_kind_and_defaults() {
    let position = iced::Point::new(12.0, 34.0);
    let cases = [
        (super::create_line_element(position), "line", 160.0, 2.0),
        (super::create_rectangle_element(position), "rectangle", 160.0, 160.0),
        (super::create_ellipse_element(position), "ellipse", 160.0, 160.0),
        (super::create_triangle_element(position), "triangle", 160.0, 160.0),
        (super::create_diamond_element(position), "diamond", 160.0, 160.0),
        (super::create_star_element(position), "star", 160.0, 160.0),
        (super::create_pentagon_element(position), "pentagon", 160.0, 160.0),
        (super::create_hexagon_element(position), "hexagon", 160.0, 160.0),
        (super::create_parallelogram_element(position), "parallelogram", 160.0, 96.0),
        (super::create_trapezoid_element(position), "trapezoid", 160.0, 96.0),
        (super::create_chevron_element(position), "chevron", 160.0, 128.0),
        (super::create_capsule_element(position), "capsule", 160.0, 80.0),
    ];

    for (element, kind, width, height) in cases {
        assert_eq!(element.kind, kind);
        assert!(element.id.starts_with(kind));
        assert_eq!(element.x, position.x);
        assert_eq!(element.y, position.y);
        assert_eq!(value_f32(&element.width), width);
        assert_eq!(value_f32(&element.height), height);
        assert_eq!(element.visible, Some(true));
    }
}

#[test]
fn text_icon_frame_and_sticky_note_factories_set_special_fields() {
    let position = iced::Point::new(5.0, 9.0);

    let text = super::create_text_element(position);
    assert_eq!(text.kind, "text");
    assert_eq!(text.content.as_deref(), Some("输入文本"));
    assert_eq!(text.text_growth.as_deref(), Some("auto"));

    let icon = super::create_icon_element(position);
    assert_eq!(icon.kind, "icon_font");
    assert_eq!(icon.icon_font_name.as_deref(), Some("star"));
    assert_eq!(icon.icon_font_family.as_deref(), Some("lucide"));

    let mut doc = crate::app::views::design::models::DesignDoc::default();
    doc.children.push(super::create_frame_element(
        position,
        &crate::app::views::design::models::DesignDoc::default(),
    ));
    let frame = super::create_frame_element(position, &doc);
    assert_eq!(frame.kind, "frame");
    assert_eq!(frame.name.as_deref(), Some("画板 2"));
    assert_eq!(frame.clip, Some(true));
    assert!(frame.stroke.is_some());

    for kind in crate::app::views::design::models::StickyNoteKind::ALL {
        let note = super::create_sticky_note_element(position, kind);
        assert_eq!(note.kind, "sticky_note");
        assert_eq!(note.note_type, Some(kind));
        assert_eq!(note.name, Some(kind.bilingual_label()));
        assert_eq!(note.color.as_deref(), Some(kind.text_color()));
        assert!(note.stroke.is_some());
    }
}

#[test]
fn image_factory_scales_large_images_and_uses_default_for_invalid_sizes() {
    let position = iced::Point::new(1.0, 2.0);

    let wide = super::create_image_element(position, "wide.png".to_string(), Some((640, 240)));
    assert_eq!(wide.kind, "image");
    assert_eq!(value_f32(&wide.width), 320.0);
    assert_eq!(value_f32(&wide.height), 120.0);

    let tiny = super::create_image_element(position, "tiny.png".to_string(), Some((1, 1)));
    assert_eq!(value_f32(&tiny.width), 48.0);
    assert_eq!(value_f32(&tiny.height), 48.0);

    let fallback = super::create_image_element(position, "bad.png".to_string(), Some((0, 10)));
    assert_eq!(value_f32(&fallback.width), 240.0);
    assert_eq!(value_f32(&fallback.height), 180.0);
}

#[test]
fn brush_path_requires_two_points_and_clamps_width() {
    assert!(
        super::create_brush_path_element(
            &[iced::Point::new(1.0, 1.0)],
            super::DEFAULT_BRUSH_COLOR_HEX,
            super::DEFAULT_BRUSH_WIDTH_PX,
        )
        .is_none()
    );

    let thin = super::create_brush_path_element(
        &[iced::Point::new(10.0, 20.0), iced::Point::new(30.0, 40.0)],
        "#123456",
        0.1,
    )
    .expect("brush path");
    assert_eq!(thin.kind, "path");
    assert_eq!(thin.class.as_deref(), Some(super::BRUSH_STROKE_CLASS));
    assert_eq!(thin.stroke.as_ref().unwrap().thickness.as_ref().unwrap().as_f64(), Some(1.0));
    assert_eq!(thin.stroke.as_ref().unwrap().fill.as_deref(), Some("#123456"));
    assert_eq!(thin.geometry.as_deref(), Some("M 2.50 2.50 L 22.50 22.50"));

    let thick = super::create_brush_path_element(
        &[iced::Point::new(-5.0, -5.0), iced::Point::new(5.0, 5.0)],
        "#000000",
        99.0,
    )
    .expect("brush path");
    assert_eq!(thick.stroke.as_ref().unwrap().thickness.as_ref().unwrap().as_f64(), Some(18.0));
}

#[test]
fn generated_ids_keep_prefix_and_are_unique() {
    let first = super::generate_id("unit");
    let second = super::generate_id("unit");
    assert!(first.starts_with("unit-"));
    assert!(second.starts_with("unit-"));
    assert_ne!(first, second);
}
