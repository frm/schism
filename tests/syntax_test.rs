use schism::render::syntax::Highlighter;

#[test]
fn test_highlight_rust_line() {
    let hl = Highlighter::new();
    let spans = hl.highlight_line("fn main() {", "rs");

    assert!(!spans.is_empty());

    let text: String = spans.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(text, "fn main() {");
}

#[test]
fn test_highlight_unknown_extension() {
    let hl = Highlighter::new();
    let spans = hl.highlight_line("just plain text", "xyz_unknown");

    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "just plain text");
}

#[test]
fn test_highlight_empty_line() {
    let hl = Highlighter::new();
    let spans = hl.highlight_line("", "rs");

    let text: String = spans.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(text, "");
}
