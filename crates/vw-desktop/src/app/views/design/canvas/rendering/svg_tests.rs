#[test]
fn parser_reads_commands_numbers_and_flags() {
    let mut parser = super::SvgPathParser::new(" M -1.5,2 1");
    assert_eq!(parser.next_command(), Some('M'));
    assert_eq!(parser.next_number(), Some(-1.5));
    assert_eq!(parser.next_number(), Some(2.0));
    assert_eq!(parser.next_flag(), Some(1.0));
}

#[test]
fn empty_or_degenerate_path_does_not_build() {
    assert!(super::build_svg_path("", iced::Point::ORIGIN, 1.0).is_none());
    assert!(
        super::build_svg_path_fit("M 0 0 L 0 0", iced::Point::ORIGIN, iced::Size::new(10.0, 10.0))
            .is_none()
    );
}
