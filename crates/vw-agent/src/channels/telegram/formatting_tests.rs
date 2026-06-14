use super::TelegramChannel;

#[test]
fn escape_html_covers_quotes_and_angle_brackets() {
    assert_eq!(
        TelegramChannel::escape_html("\"x\" <tag> & 'y'"),
        "&quot;x&quot; &lt;tag&gt; &amp; &#39;y&#39;"
    );
}

#[test]
fn markdown_to_telegram_html_escapes_link_href() {
    assert_eq!(
        TelegramChannel::markdown_to_telegram_html("[x](https://e.test?a=\"b\")"),
        "<a href=\"https://e.test?a=&quot;b&quot;\">x</a>"
    );
}

#[test]
fn markdown_to_telegram_html_formats_inline_markdown() {
    let html = TelegramChannel::markdown_to_telegram_html(
        "# Title\n**bold** __also bold__ *italic* `code<&>` ~~gone~~",
    );

    assert_eq!(
        html,
        "<b>Title</b>\n<b>bold</b> <b>also bold</b> <i>italic</i> <code>code&lt;&amp;&gt;</code> <s>gone</s>"
    );
}

#[test]
fn markdown_to_telegram_html_only_links_http_schemes() {
    let html = TelegramChannel::markdown_to_telegram_html(
        "[safe](http://example.test) [also](https://example.test) [raw](mailto:x@y.test)",
    );

    assert_eq!(
        html,
        "<a href=\"http://example.test\">safe</a> <a href=\"https://example.test\">also</a> [raw](mailto:x@y.test)"
    );
}

#[test]
fn markdown_to_telegram_html_escapes_plain_text_and_unclosed_markup() {
    assert_eq!(
        TelegramChannel::markdown_to_telegram_html("<tag attr='x'>& **open"),
        "&lt;tag attr=&#39;x&#39;&gt;&amp; **open"
    );
}

#[test]
fn markdown_to_telegram_html_converts_fenced_code_blocks() {
    let html = TelegramChannel::markdown_to_telegram_html("before\n```\nif a < b && c\n```\nafter");

    assert_eq!(html, "before\n<pre><code>if a &lt; b &amp;&amp; c</code></pre>\nafter");
}

#[test]
fn markdown_to_telegram_html_closes_trailing_code_block() {
    let html = TelegramChannel::markdown_to_telegram_html("```\nno close <tag>");

    assert_eq!(html, "<pre><code>no close &lt;tag&gt;</code></pre>");
}

#[test]
fn markdown_to_telegram_html_treats_unclosed_italic_marker_as_plain_text() {
    assert_eq!(TelegramChannel::markdown_to_telegram_html("**x** *open"), "<b>x</b> *open");
}
