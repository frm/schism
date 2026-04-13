use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn render(text: &str, content_width: usize) -> Vec<Line<'static>> {
    if text.trim().is_empty() {
        return vec![Line::from(Span::styled(
            " (no description)".to_string(),
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(text, opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default().fg(Color::White)];
    let mut in_code_block = false;
    let mut in_blockquote = false;
    let mut in_html_comment = false;
    let mut link_url: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_line(&mut lines, &mut current_spans, in_blockquote);
                    if !lines.is_empty() {
                        lines.push(Line::from(""));
                    }
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                        }
                        _ => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    };
                    style_stack.push(style);
                }
                Tag::Paragraph => {}
                Tag::BlockQuote(_) => { in_blockquote = true; }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    flush_line(&mut lines, &mut current_spans, in_blockquote);
                    lines.push(Line::from(Span::styled(
                        " \u{250c}\u{2500}".to_string(),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                Tag::List(_) => {}
                Tag::Item => { flush_line(&mut lines, &mut current_spans, in_blockquote); }
                Tag::Emphasis => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.add_modifier(Modifier::ITALIC));
                }
                Tag::Strong => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.add_modifier(Modifier::BOLD));
                }
                Tag::Strikethrough => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.add_modifier(Modifier::CROSSED_OUT));
                }
                Tag::Link { dest_url, .. } => {
                    link_url = Some(dest_url.to_string());
                    current_spans.push(Span::styled(
                        "\u{2197} ".to_string(),
                        Style::default().fg(Color::Blue),
                    ));
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(Color::Blue).add_modifier(Modifier::UNDERLINED));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans, false);
                    let divider_width = content_width.min(40);
                    lines.push(Line::from(Span::styled(
                        format!(" {}", "\u{2500}".repeat(divider_width)),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                TagEnd::Paragraph => {
                    flush_line(&mut lines, &mut current_spans, in_blockquote);
                    lines.push(Line::from(""));
                }
                TagEnd::BlockQuote(_) => { in_blockquote = false; }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    lines.push(Line::from(Span::styled(
                        " \u{2514}\u{2500}".to_string(),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                TagEnd::List(_) => {}
                TagEnd::Item => { flush_line(&mut lines, &mut current_spans, in_blockquote); }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => { style_stack.pop(); }
                TagEnd::Link => {
                    style_stack.pop();
                    if let Some(url) = link_url.take() {
                        current_spans.push(Span::styled(
                            format!(" ({})", url),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }
                _ => {}
            },
            Event::Text(txt) => {
                if in_html_comment { continue; }
                if in_code_block {
                    for code_line in txt.lines() {
                        lines.push(Line::from(Span::styled(
                            format!(" \u{2502} {}", code_line),
                            Style::default().fg(Color::White).bg(Color::Rgb(25, 25, 35)),
                        )));
                    }
                } else {
                    current_spans.push(Span::styled(txt.to_string(), current_style(&style_stack)));
                }
            }
            Event::Code(code) => {
                current_spans.push(Span::styled(
                    format!("`{}`", code),
                    Style::default().fg(Color::Yellow),
                ));
            }
            Event::SoftBreak | Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans, in_blockquote);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans, in_blockquote);
                let divider_width = content_width.min(40);
                lines.push(Line::from(Span::styled(
                    format!(" {}", "\u{2500}".repeat(divider_width)),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            Event::TaskListMarker(checked) => {
                let (marker, color) = if checked {
                    ("\u{2611}", Color::Green)
                } else {
                    ("\u{2610}", Color::DarkGray)
                };
                current_spans.push(Span::styled(marker.to_string(), Style::default().fg(color)));
                current_spans.push(Span::raw("  "));
            }
            Event::Html(html) => {
                let trimmed = html.trim();
                if trimmed.starts_with("<!--") {
                    if !trimmed.ends_with("-->") { in_html_comment = true; }
                } else if trimmed.ends_with("-->") {
                    in_html_comment = false;
                }
            }
            _ => {}
        }
    }

    flush_line(&mut lines, &mut current_spans, false);

    while lines.last().map(|l| l.spans.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines
}

fn current_style(stack: &[Style]) -> Style {
    stack.last().copied().unwrap_or(Style::default().fg(Color::White))
}

fn flush_line(
    lines: &mut Vec<Line<'static>>,
    spans: &mut Vec<Span<'static>>,
    blockquote: bool,
) {
    if spans.is_empty() { return; }
    let mut final_spans = Vec::with_capacity(spans.len() + 1);
    if blockquote {
        final_spans.push(Span::styled(" \u{2502} ".to_string(), Style::default().fg(Color::DarkGray)));
    } else {
        final_spans.push(Span::raw(" "));
    }
    final_spans.append(spans);
    lines.push(Line::from(final_spans));
}
