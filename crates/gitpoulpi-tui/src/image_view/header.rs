//! The image view's header line: what the picture is on the left, the zoom
//! buttons on the right. Painted in this order, and the hit rects come from
//! that paint (maintainer rule 6), never from a second computation.
//!
//! ```text
//! PNG 512×512 · 34.2 KB                         [−] 100% [+] [Fit] [1:1]
//! ```

use crate::ui_kit::display_width;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use super::pane::ImageHit;
use crate::th;

/// The `Edit in` slot (plan D9/S7): the room it takes at the right end of the
/// header, where it paints, and the hit it registers (from the same rect).
/// Zero wide when the row is too narrow for it (the keys and the palette still
/// reach the list).
pub(crate) struct EditSlot {
    pub(crate) width: u16,
    pub(crate) rect: Option<Rect>,
    pub(crate) hits: Vec<(Rect, crate::chrome::HitTarget)>,
}

/// `row` is the header row. One call site per painter, so the Edit in slice
/// never edits the layout function.
pub(crate) fn edit_slot(row: Rect) -> EditSlot {
    let button = super::edit_in::button_width();
    // The info text and at least the zoom pair still have to fit.
    if row.width < button + 30 {
        return EditSlot {
            width: 0,
            rect: None,
            hits: Vec::new(),
        };
    }
    let rect = Rect::new(row.right() - button, row.y, button, 1);
    EditSlot {
        width: button + 1,
        rect: Some(rect),
        hits: vec![(rect, crate::chrome::HitTarget::Image(ImageHit::EditIn))],
    }
}

/// The glyph of the zoom-out button: a real minus sign, not a hyphen.
const MINUS: &str = "\u{2212}";
/// "1:1" is arithmetic, not a word: not translated.
const ACTUAL: &str = "1:1";

pub(crate) struct Header {
    pub(crate) line: Line<'static>,
    /// `(rect, button)` for each drawn button, from this very layout.
    pub(crate) buttons: Vec<(Rect, ImageHit)>,
}

fn button_style(hovered: bool) -> Style {
    if hovered {
        Style::default()
            .bg(th().ui.button_hover)
            .fg(th().ui.on_button)
    } else {
        Style::default().bg(th().ui.button).fg(th().ui.on_button)
    }
}

/// Lays the header out in `row`. `info` is clipped first when the row is
/// narrow, then the buttons drop from the right (Fit and 1:1 go before the
/// zoom pair).
pub(crate) fn layout(
    row: Rect,
    info: &str,
    percent: Option<u32>,
    fit_label: &str,
    hovered: Option<ImageHit>,
) -> Header {
    let slot = edit_slot(row);
    let width = row.width.saturating_sub(slot.width);
    let pct = format!(" {}% ", percent.unwrap_or(0));
    let all: Vec<(ImageHit, String)> = vec![
        (ImageHit::ZoomOut, format!(" {MINUS} ")),
        (ImageHit::ZoomIn, " + ".to_string()),
        (ImageHit::Fit, format!(" {fit_label} ")),
        (ImageHit::ActualSize, format!(" {ACTUAL} ")),
    ];
    let wide = |s: &str| display_width(s) as u16;
    // Buttons in drawing order: [-] 100% [+] [Fit] [1:1]. Keep as many as
    // fit, dropping Fit then 1:1 first.
    let mut keep = all.len();
    let needed = |n: usize| -> u16 {
        let mut w = 0u16;
        for (i, (_, label)) in all.iter().take(n).enumerate() {
            w += wide(label) + 1;
            if i == 0 {
                w += wide(&pct);
            }
        }
        w
    };
    while keep > 2 && needed(keep) + 2 > width {
        keep -= 1;
    }
    let buttons_w = if percent.is_some() && needed(keep) <= width {
        needed(keep)
    } else {
        0
    };
    let info_room = width.saturating_sub(buttons_w + 1) as usize;
    let mut info_text: String = info.to_string();
    if display_width(&info_text) > info_room {
        let mut out = String::new();
        let mut w = 0;
        for c in info_text.chars() {
            let cw = display_width(c.encode_utf8(&mut [0u8; 4]));
            if w + cw > info_room {
                break;
            }
            out.push(c);
            w += cw;
        }
        info_text = out;
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buttons = Vec::new();
    let info_w = wide(&info_text);
    spans.push(Span::styled(info_text, Style::default().fg(th().ui.muted)));
    let mut x = info_w;
    if buttons_w > 0 {
        let start = width - buttons_w;
        if start > x {
            spans.push(Span::raw(" ".repeat((start - x) as usize)));
            x = start;
        }
        for (i, (hit, label)) in all.iter().take(keep).enumerate() {
            let w = wide(label);
            spans.push(Span::styled(
                label.clone(),
                button_style(hovered == Some(*hit)),
            ));
            buttons.push((Rect::new(row.x + x, row.y, w, 1), *hit));
            x += w;
            if i == 0 {
                spans.push(Span::styled(
                    pct.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                x += wide(&pct);
            }
            spans.push(Span::raw(" "));
            x += 1;
        }
    }
    Header {
        line: Line::from(spans),
        buttons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buttons_sit_at_the_right_end_and_their_rects_match_the_paint() {
        let row = Rect::new(2, 5, 60, 1);
        let h = layout(row, "PNG 4×4 · 71 B", Some(100), "Fit", None);
        assert_eq!(h.buttons.len(), 4);
        let text: String = h.line.spans.iter().map(|s| s.content.as_ref()).collect();
        for (rect, hit) in &h.buttons {
            let label: String = text
                .chars()
                .skip((rect.x - row.x) as usize)
                .take(rect.width as usize)
                .collect();
            let want = match hit {
                ImageHit::ZoomOut => " \u{2212} ",
                ImageHit::ZoomIn => " + ",
                ImageHit::Fit => " Fit ",
                ImageHit::ActualSize => " 1:1 ",
                ImageHit::Flip(_) | ImageHit::Divider | ImageHit::EditIn => {
                    unreachable!("not a layout button")
                }
            };
            assert_eq!(label, want, "{hit:?} is where it was painted");
        }
    }

    #[test]
    fn the_hovered_button_is_lit_and_only_it() {
        let lit = |hovered: Option<ImageHit>| -> Vec<String> {
            layout(Rect::new(0, 0, 60, 1), "PNG", Some(100), "Fit", hovered)
                .line
                .spans
                .iter()
                .filter(|s| s.style.bg == Some(crate::th().ui.button_hover))
                .map(|s| s.content.to_string())
                .collect()
        };
        assert!(lit(None).is_empty());
        assert_eq!(lit(Some(ImageHit::ZoomIn)), vec![" + ".to_string()]);
        assert_eq!(lit(Some(ImageHit::Fit)), vec![" Fit ".to_string()]);
    }

    #[test]
    fn a_narrow_row_drops_fit_and_one_to_one_before_the_zoom_pair() {
        let h = layout(Rect::new(0, 0, 20, 1), "PNG 4×4", Some(100), "Fit", None);
        let hits: Vec<_> = h.buttons.iter().map(|(_, b)| *b).collect();
        assert_eq!(hits, vec![ImageHit::ZoomOut, ImageHit::ZoomIn]);
    }
}
