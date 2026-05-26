use super::*;
use super::validation;
use crate::tools::shell::ast::parse_command;

#[test]
fn allows_print_and_substitute_subset() {
    assert!(matches!(
        validate_sed_command(&parse_command("sed -n '1,3p' file.txt")),
        SedValidationResult::Allowed {
            kind: SedCommandKind::Print { .. },
            in_place: false,
            extended_regex: false,
            ..
        }
    ));
    assert!(matches!(
        validate_sed_command(&parse_command("sed -E -i 's/a/b/g' file.txt")),
        SedValidationResult::Allowed {
            kind: SedCommandKind::Substitute { .. },
            in_place: true,
            extended_regex: true,
            ..
        }
    ));
}

#[test]
fn blocks_unsafe_sed_syntax() {
    for command in ["awk '{print}' file", "sed -f script.sed", "sed 's/a/b/w out' file"] {
        assert!(matches!(
            validate_sed_command(&parse_command(command)),
            SedValidationResult::Blocked { .. }
        ));
    }
}

#[test]
fn parses_substitute_escaped_delimiters() {
    let (pattern, replacement, flags) =
        validation::parse_substitute_parts(r"s/a\/b/c\/d/g").expect("valid substitute");

    assert_eq!(pattern, r"a\/b");
    assert_eq!(replacement, r"c\/d");
    assert_eq!(flags, "g");
}
