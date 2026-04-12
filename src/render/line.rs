use crossterm::style::Color;

use crate::types::LineKind;

pub struct LineRenderer;

impl LineRenderer {
    pub fn format_lineno(lineno: Option<u32>, width: usize) -> String {
        match lineno {
            Some(n) => format!("{:>width$} ", n, width = width),
            None => format!("{:>width$} ", "", width = width),
        }
    }

    pub fn line_bg(kind: &LineKind) -> Option<Color> {
        match kind {
            LineKind::Added => Some(Color::Rgb { r: 0, g: 40, b: 0 }),
            LineKind::Removed => Some(Color::Rgb { r: 40, g: 0, b: 0 }),
            LineKind::Context => None,
        }
    }

    pub fn line_prefix(kind: &LineKind) -> &'static str {
        match kind {
            LineKind::Added => "+",
            LineKind::Removed => "-",
            LineKind::Context => " ",
        }
    }

    pub fn prefix_color(kind: &LineKind) -> Color {
        match kind {
            LineKind::Added => Color::Green,
            LineKind::Removed => Color::Red,
            LineKind::Context => Color::DarkGrey,
        }
    }
}
