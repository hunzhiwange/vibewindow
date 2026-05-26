use super::keybind::{Info, matches, parse, to_string};

#[test]
fn parses_formats_and_matches_keybindings() {
    let parsed = parse("<leader> ctrl+shift+del");

    assert_eq!(parsed.len(), 1);
    assert!(parsed[0].leader);
    assert_eq!(parsed[0].name, "del");
    assert!(matches(Some(&parsed[0]), &parsed[0]));
    assert_eq!(to_string(Some(&Info { name: "delete".into(), ctrl: true, meta: false, shift: false, super_key: false, leader: false })), "ctrl+del");
    assert!(parse("none").is_empty());
}
