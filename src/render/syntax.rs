use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct StyledSpan {
    pub text: String,
    pub fg: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
}

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: two_face::syntax::extra_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_line(&self, line: &str, extension: &str) -> Vec<StyledSpan> {
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        let ext = Self::resolve_extension(extension);
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, theme);

        match h.highlight_line(line, &self.syntax_set) {
            Ok(regions) => regions
                .into_iter()
                .map(|(style, text)| style_to_span(style, text))
                .collect(),
            Err(_) => vec![StyledSpan {
                text: line.to_string(),
                fg: (255, 255, 255),
                bold: false,
                italic: false,
            }],
        }
    }

    pub fn extension_from_path(path: &str) -> &str {
        path.rsplit('.').next().unwrap_or("")
    }

    /// Map extensions not covered by two-face's extra set.
    fn resolve_extension(ext: &str) -> &str {
        match ext {
            "heex" | "leex" => "ex",
            _ => ext,
        }
    }
}

fn style_to_span(style: Style, text: &str) -> StyledSpan {
    StyledSpan {
        text: text.to_string(),
        fg: (style.foreground.r, style.foreground.g, style.foreground.b),
        bold: style.font_style.contains(FontStyle::BOLD),
        italic: style.font_style.contains(FontStyle::ITALIC),
    }
}
