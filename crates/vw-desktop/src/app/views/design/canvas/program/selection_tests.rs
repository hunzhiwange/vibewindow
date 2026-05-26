#[test]
fn empty_element_list_has_no_intersections() {
    let doc = crate::app::views::design::models::DesignDoc::default();
    let ids = super::find_intersecting_ids(&[], &doc, iced::Vector::new(0.0, 0.0), 1.0, iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(10.0, 10.0)));
    assert!(ids.is_empty());
}
