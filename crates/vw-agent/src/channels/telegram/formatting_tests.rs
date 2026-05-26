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
