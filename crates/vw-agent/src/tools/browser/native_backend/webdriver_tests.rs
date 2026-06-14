use super::*;

#[test]
fn selector_for_find_escapes_attribute_values() {
    assert_eq!(selector_for_find("role", "main\"nav"), r#"[role=\"main\"nav\"]"#);
    assert_eq!(selector_for_find("testid", "a\nb"), r#"[data-testid=\"a b\"]"#);
}

#[test]
fn parse_selector_uses_expected_selector_kinds() {
    match parse_selector(" text=Hello ") {
        SelectorKind::XPath(value) => assert!(value.contains("Hello")),
        SelectorKind::Css(_) => panic!("text selector should become XPath"),
    }
    match parse_selector("@node") {
        SelectorKind::Css(value) => assert!(value.contains("data-zc-ref")),
        SelectorKind::XPath(_) => panic!("ref selector should become CSS"),
    }
}

#[test]
fn xpath_literal_handles_quote_combinations() {
    assert_eq!(xpath_literal("plain"), "\"plain\"");
    assert_eq!(xpath_literal("say \"hi\""), "'say \"hi\"'");
    assert!(xpath_literal("a\"b'c").starts_with("concat("));
}

#[test]
fn webdriver_key_maps_known_aliases_and_preserves_unknown() {
    assert_eq!(webdriver_key("esc"), webdriver_key("escape"));
    assert_eq!(webdriver_key("left"), webdriver_key("arrowleft"));
    assert_eq!(webdriver_key("x"), "x");
}
