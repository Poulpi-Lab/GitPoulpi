//! Syntax highlighting for the editor, the merge editor, the blame view and
//! Markdown code blocks. The highlighter itself (grammar choice, the
//! checkpointed incremental parse, the time budget) is
//! `gitpoulpi_app::highlight` and yields neutral runs; this file hands it the
//! terminal's current syntect theme and turns a run's [`SyntaxStyle`] into a
//! ratatui [`Style`].

use std::path::Path;

use ratatui::style::{Color, Modifier, Style};

use crate::theme::{th, theme_generation};

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::diff::SyntaxStyle;
#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::highlight::{Highlight, Run, ThemeRef, STRIDE};

/// The syntect theme the active theme names, and its generation.
pub(crate) fn theme_source() -> ThemeRef {
    ThemeRef {
        name: th().syntax,
        generation: theme_generation(),
    }
}

/// [`Highlight::for_file`] under the terminal's theme.
pub(crate) fn for_file(path: &Path, first_line: &str) -> Highlight {
    Highlight::for_file(path, first_line, theme_source)
}

/// [`Highlight::for_token`] under the terminal's theme.
pub(crate) fn for_token(token: &str) -> Highlight {
    Highlight::for_token(token, theme_source)
}

/// A run's look as a ratatui style: the foreground and the font style, or
/// the default style for plain text.
pub(crate) fn style_of(style: Option<SyntaxStyle>) -> Style {
    let Some(s) = style else {
        return Style::default();
    };
    let gitpoulpi_app::diff::Rgb(r, g, b) = s.fg;
    let mut out = Style::default().fg(Color::Rgb(r, g, b));
    if s.bold {
        out = out.add_modifier(Modifier::BOLD);
    }
    if s.italic {
        out = out.add_modifier(Modifier::ITALIC);
    }
    if s.underline {
        out = out.add_modifier(Modifier::UNDERLINED);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The theme change reaches a highlighter through the terminal's theme
    /// generation, not only through the app crate's own test source.
    #[test]
    fn a_theme_change_re_highlights_through_the_terminal_theme() {
        use crate::theme::{override_theme, Theme};
        let mut hl = for_file(Path::new("a.rs"), "");
        assert!(hl.advance(|_| "fn main() {}", 1, 1, std::time::Duration::from_secs(60)));
        let dark_runs = hl.line(0).unwrap().to_vec();
        let mut light = Theme::dark();
        light.syntax = "InspiredGitHub";
        let _g = override_theme(&light);
        assert!(hl.line(0).is_none(), "stale colors are never served");
        assert!(hl.advance(|_| "fn main() {}", 1, 1, std::time::Duration::from_secs(60)));
        assert_ne!(hl.line(0).unwrap(), dark_runs.as_slice());
    }
}
