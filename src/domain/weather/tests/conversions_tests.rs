use super::conversions::sanitize_text;

#[test]
fn test_sanitize_text_empty() {
    assert_eq!(sanitize_text(""), "");
}

#[test]
fn test_sanitize_text_no_html() {
    assert_eq!(sanitize_text("Hello World"), "Hello World");
    assert_eq!(sanitize_text("123!@#"), "123!@#");
}

#[test]
fn test_sanitize_text_with_html_tags() {
    assert_eq!(sanitize_text("Hello <b>World</b>"), "Hello World");
    assert_eq!(sanitize_text("<p>Hello</p>"), "Hello");
    assert_eq!(sanitize_text("<div><span>Nested</span></div>"), "Nested");
    assert_eq!(
        sanitize_text("<a href=\"https://example.com\">Link</a>"),
        "Link"
    );
}

#[test]
fn test_sanitize_text_with_html_entities() {
    assert_eq!(sanitize_text("Hello &amp; World"), "Hello & World");
    assert_eq!(sanitize_text("Tom &amp; Jerry"), "Tom & Jerry");
    assert_eq!(sanitize_text("1 &lt; 2 &gt; 0"), "1 < 2 > 0");
    assert_eq!(sanitize_text("&quot;Quote&quot;"), "\"Quote\"");
    assert_eq!(sanitize_text("&#39;Apostrophe&#39;"), "'Apostrophe'");
}

#[test]
fn test_sanitize_text_mixed_tags_and_entities() {
    assert_eq!(
        sanitize_text("<p>Tom &amp; <b>Jerry</b></p>"),
        "Tom & Jerry"
    );
    assert_eq!(sanitize_text("<a href=\"#\">1 &lt; 2</a>"), "1 < 2");
}

#[test]
fn test_sanitize_text_unicode() {
    assert_eq!(sanitize_text("Hello 🌍"), "Hello 🌍");
    assert_eq!(sanitize_text("<b>Hello</b> 🌍 &amp; 🌕"), "Hello 🌍 & 🌕");
}
