//! The bar's `File | All` selector: two segments of one fixed-width control,
//! the way the Inline / Split layout control is two neighbours. A click on a
//! segment picks that list and shows the column; a click on the segment that
//! is selected while the column is shown hides it (the list is remembered).
//!
//! Its width is the two words and the rule between them, whatever the column
//! is doing: nothing on the bar moves when the column opens, closes or
//! changes its list. The segment lit is the list shown; while the column is
//! closed, the list it will come back with is marked, not lit.

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use super::{painted, remembered, ColumnMode, ColumnState};
use crate::diff_view::DiffBarButton;
use crate::th;
use crate::ui_kit::display_width;
use gitpoulpi_i18n::msg::diff_view as msg;

/// The rule between the two segments.
const RULE: &str = "\u{2502}";

/// The two words, in the current language.
pub(crate) fn words() -> (&'static str, &'static str) {
    (msg::B_COLUMN_FILE.get(), msg::B_COLUMN_ALL.get())
}

/// The control's spans, and the hit of each segment as `(dx, width, button)`
/// from the control's own left edge.
pub(crate) fn selector(
    hovered: Option<DiffBarButton>,
) -> (Vec<Span<'static>>, [(u16, u16, DiffBarButton); 2]) {
    let (file, all) = words();
    let shown = painted();
    let mode = remembered();
    let seg = |text: &str, button: DiffBarButton, lit: bool, marked: bool| {
        let style = if lit {
            Style::default()
                .bg(th().ui.accent)
                .fg(th().ui.on_accent)
                .add_modifier(Modifier::BOLD)
        } else if hovered == Some(button) {
            Style::default()
                .bg(th().ui.button_hover)
                .fg(th().ui.on_button)
        } else if marked {
            // Not coloured: bold and underlined reads on every theme.
            Style::default()
                .bg(th().ui.button)
                .fg(th().ui.on_button)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().bg(th().ui.button).fg(th().ui.on_button)
        };
        Span::styled(format!(" {text} "), style)
    };
    let closed = shown == ColumnState::Off;
    let file_seg = seg(
        file,
        DiffBarButton::ColumnFile,
        shown == ColumnState::File,
        closed && mode == ColumnMode::File,
    );
    let all_seg = seg(
        all,
        DiffBarButton::ColumnAll,
        shown == ColumnState::Graph,
        closed && mode == ColumnMode::All,
    );
    let rule = Span::styled(
        RULE,
        Style::default().bg(th().ui.button).fg(th().ui.on_button),
    );
    let w_file = display_width(file) as u16 + 2;
    let w_all = display_width(all) as u16 + 2;
    let hits = [
        (0, w_file, DiffBarButton::ColumnFile),
        (w_file + 1, w_all, DiffBarButton::ColumnAll),
    ];
    (vec![file_seg, rule, all_seg], hits)
}
