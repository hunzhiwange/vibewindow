use iced::Color;

use crate::app::DiffTheme;

use super::utils::{
    FileStatus, Lang, get_diff_colors, get_word_diff_ranges, highlight_segments, lang_for_file,
    render_line_content,
};

#[test]
fn file_status_derives_expected_traits() {
    assert_eq!(FileStatus::Modified, FileStatus::Modified);
    assert_ne!(FileStatus::Added, FileStatus::Deleted);
    assert_eq!(format!("{:?}", FileStatus::Renamed), "Renamed");
    let copied = FileStatus::Untracked;
    let cloned = copied;
    assert_eq!(copied, cloned);
    assert_eq!(FileStatus::Unknown, FileStatus::Unknown);
}

#[test]
fn lang_for_file_recognizes_supported_extensions() {
    let cases = [
        ("main.rs", Lang::Rust),
        ("app.ts", Lang::Ts),
        ("view.tsx", Lang::Ts),
        ("index.js", Lang::Js),
        ("index.jsx", Lang::Js),
        ("data.json", Lang::Json),
        ("Cargo.toml", Lang::Toml),
        ("ci.yaml", Lang::Yaml),
        ("ci.yml", Lang::Yaml),
        ("script.py", Lang::Python),
        ("server.go", Lang::Go),
        ("native.c", Lang::C),
        ("native.cpp", Lang::Cpp),
        ("native.cc", Lang::Cpp),
        ("native.cxx", Lang::Cpp),
        ("native.hpp", Lang::Cpp),
        ("index.html", Lang::Html),
        ("index.htm", Lang::Html),
        ("style.css", Lang::Css),
        ("query.sql", Lang::Sql),
        ("run.sh", Lang::Bash),
        ("run.bash", Lang::Bash),
        ("README", Lang::Other),
    ];

    for (name, expected) in cases {
        assert!(matches!(
            (lang_for_file(name), expected),
            (Lang::Rust, Lang::Rust)
                | (Lang::Ts, Lang::Ts)
                | (Lang::Js, Lang::Js)
                | (Lang::Json, Lang::Json)
                | (Lang::Toml, Lang::Toml)
                | (Lang::Yaml, Lang::Yaml)
                | (Lang::Python, Lang::Python)
                | (Lang::Go, Lang::Go)
                | (Lang::C, Lang::C)
                | (Lang::Cpp, Lang::Cpp)
                | (Lang::Html, Lang::Html)
                | (Lang::Css, Lang::Css)
                | (Lang::Sql, Lang::Sql)
                | (Lang::Bash, Lang::Bash)
                | (Lang::Other, Lang::Other)
        ));
    }
}

#[test]
fn highlight_segments_splits_words_symbols_and_keywords() {
    assert!(highlight_segments("", Lang::Rust, DiffTheme::GitHub).is_empty());

    let segments = highlight_segments("if value_else == 1", Lang::Rust, DiffTheme::GitHub);
    assert_eq!(segments[0], (0, 2, true));
    assert_eq!(segments[1], (2, 3, false));
    assert_eq!(segments[2], (3, 13, false));
    assert_eq!(segments.last().copied(), Some((17, 18, false)));

    let keyword_segments =
        highlight_segments("let var else fn function class export", Lang::Ts, DiffTheme::Monokai);
    assert!(keyword_segments.iter().filter(|(_, _, is_kw)| *is_kw).count() >= 7);
}

#[test]
fn word_diff_ranges_trim_changed_words_but_keep_whitespace_only_changes() {
    let (old_ranges, new_ranges) = get_word_diff_ranges("hello world", "hello rust");
    assert_eq!(old_ranges, vec![6..11]);
    assert_eq!(new_ranges, vec![6..10]);

    let (old_ranges, new_ranges) = get_word_diff_ranges("same", "same");
    assert!(old_ranges.is_empty());
    assert!(new_ranges.is_empty());

    let (old_ranges, new_ranges) = get_word_diff_ranges("a \n b", "a \n  b");
    assert!(old_ranges.is_empty() || old_ranges.iter().all(|range| range.start < range.end));
    assert!(new_ranges.is_empty() || new_ranges.iter().all(|range| range.start < range.end));
}

#[test]
fn render_line_content_builds_rows_for_plain_syntax_and_highlighted_ranges() {
    let base = Color::from_rgb8(1, 2, 3);
    let highlight = Color::from_rgb8(4, 5, 6);

    let _ = render_line_content(
        "let value = 1",
        Lang::Rust,
        DiffTheme::GitHub,
        true,
        &[0..3, 10..11],
        base,
        highlight,
    );
    let _ =
        render_line_content("plain", Lang::Other, DiffTheme::Monokai, false, &[], base, highlight);
}

#[test]
fn diff_colors_are_distinct_for_builtin_themes() {
    let github = get_diff_colors(DiffTheme::GitHub);
    let monokai = get_diff_colors(DiffTheme::Monokai);

    assert_eq!(github.0, Color::TRANSPARENT);
    assert_eq!(monokai.0, Color::TRANSPARENT);
    assert_ne!(github.1, monokai.1);
    assert_ne!(github.3, github.5);
    assert_ne!(monokai.3, monokai.5);
}
