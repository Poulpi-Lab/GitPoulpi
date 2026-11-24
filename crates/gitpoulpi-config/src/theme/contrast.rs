//! WCAG contrast checks over a [`ThemeSpec`].

use super::{ThemeColor, ThemeSpec};

/// One failed rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastIssue {
    pub rule: String,
    pub ratio: f64,
    pub min: f64,
}

fn channel(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// The WCAG contrast ratio (1.0 to 21.0) of two colors; `None` when either
/// is `default` (unknown).
pub fn contrast_ratio(a: ThemeColor, b: ThemeColor) -> Option<f64> {
    let (la, lb) = (luminance(a.approx_rgb()?), luminance(b.approx_rgb()?));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

/// How far the primary button's fill moves when hovered or focused.
pub const PRIMARY_HOVER_LIFT: i16 = 30;
/// How far a segmented control's active half moves under the pointer.
pub const SEGMENTED_HOVER_LIFT: i16 = 40;
/// How far a plain button's fill moves under the pointer in a confirm
/// popup and the remote form.
pub const BUTTON_HOVER_LIFT: i16 = 24;

/// The hover fill of a button: `base` moved by `amt` per channel *away from*
/// its label color, so a hovered button never loses contrast (a light label
/// on a mid fill darkens it, a dark label lightens it). Non-RGB colors are
/// returned unchanged.
pub fn hover_fill(base: ThemeColor, label: ThemeColor, amt: i16) -> ThemeColor {
    let (Some(b), Some(l)) = (base.approx_rgb(), label.approx_rgb()) else {
        return base;
    };
    if luminance(l) > luminance(b) {
        lighten(base, -amt)
    } else {
        lighten(base, amt)
    }
}

/// Lifts (or, negative, darkens) an RGB color per channel, clamped; other
/// color kinds are returned unchanged. The TUI's hover fills use the same.
pub fn lighten(color: ThemeColor, amt: i16) -> ThemeColor {
    match color {
        ThemeColor::Rgb(r, g, b) => {
            let bump = |c: u8| (i16::from(c) + amt).clamp(0, 255) as u8;
            ThemeColor::Rgb(bump(r), bump(g), bump(b))
        }
        other => other,
    }
}

impl ThemeSpec {
    /// The contrast rules of the plan (text 4.5, accents 3.0, lanes 2.0,
    /// distinct selection/hover/input); empty means the theme passes. Pairs
    /// involving `default` colors cannot be measured and are skipped.
    pub fn contrast_issues(&self) -> Vec<ContrastIssue> {
        let bg = self.effective_bg();
        let mut issues = Vec::new();
        let mut check = |rule: String, fg: ThemeColor, back: ThemeColor, min: f64| {
            if let Some(ratio) = contrast_ratio(fg, back) {
                if ratio < min {
                    issues.push(ContrastIssue { rule, ratio, min });
                }
            }
        };
        let u = &self.ui;
        let ch = &self.chrome;
        let rules: [(&str, ThemeColor, ThemeColor, f64); 45] = [
            ("text/bg", u.text, bg, 4.5),
            ("text/panel", u.text, u.panel, 4.5),
            ("text/selection", u.text, u.selection, 4.5),
            ("text/input_focus", u.text, u.input_focus, 4.5),
            ("on_accent/accent", u.on_accent, u.accent, 4.5),
            ("on_button/button", u.on_button, u.button, 4.5),
            // The diff bar, the editor bar and the dialogs' buttons all draw
            // `on_button` on `button` (rest) and `button_hover` (hover).
            ("on_button/button_hover", u.on_button, u.button_hover, 4.5),
            // Primary button hovered/focused: `on_accent` on the lightened accent.
            (
                "on_accent/accent_hover",
                u.on_accent,
                hover_fill(u.accent, u.on_accent, PRIMARY_HOVER_LIFT),
                4.5,
            ),
            (
                "on_accent/segmented_hover",
                u.on_accent,
                hover_fill(u.accent, u.on_accent, SEGMENTED_HOVER_LIFT),
                4.5,
            ),
            (
                "on_button/button_lift",
                u.on_button,
                hover_fill(u.button, u.on_button, BUTTON_HOVER_LIFT),
                4.5,
            ),
            (
                "on_button/confirm_danger",
                u.on_button,
                ch.confirm_danger,
                4.5,
            ),
            (
                "on_button/confirm_danger_lift",
                u.on_button,
                hover_fill(ch.confirm_danger, u.on_button, BUTTON_HOVER_LIFT),
                4.5,
            ),
            // Disabled controls: muted, still readable (3:1) on their fill.
            (
                "disabled/disabled_button_bg",
                u.disabled,
                ch.disabled_button_bg,
                3.0,
            ),
            (
                "disabled/disabled_button_bg_hover",
                u.disabled,
                lighten(ch.disabled_button_bg, 14),
                3.0,
            ),
            ("disabled/panel", u.disabled, u.panel, 3.0),
            ("editor.text/bg", self.editor.text, bg, 4.5),
            (
                "editor.text/current_line",
                self.editor.text,
                self.editor.current_line,
                4.5,
            ),
            (
                "editor.find_hit_fg/find_hit",
                self.editor.find_hit_fg,
                self.editor.find_hit,
                2.5,
            ),
            (
                "text_selection_fg/text_selection_bg",
                u.text_selection_fg,
                u.text_selection_bg,
                3.0,
            ),
            ("muted/bg", u.muted, bg, 3.0),
            ("accent/bg", u.accent, bg, 3.0),
            ("danger/bg", self.status.danger, bg, 3.0),
            (
                "diff.add_fg/add_bg",
                self.diff.add_fg,
                self.diff.add_bg,
                3.0,
            ),
            (
                "diff.del_fg/del_bg",
                self.diff.del_fg,
                self.diff.del_bg,
                3.0,
            ),
            (
                "diff.add_fg/add_emph_bg",
                self.diff.add_fg,
                self.diff.add_emph_bg,
                3.0,
            ),
            (
                "diff.del_fg/del_emph_bg",
                self.diff.del_fg,
                self.diff.del_emph_bg,
                3.0,
            ),
            ("on_button/pill_branch", u.on_button, ch.pill_branch, 3.0),
            ("on_button/pill_onto", u.on_button, ch.pill_onto, 3.0),
            ("on_button/toast_bg", u.on_button, ch.toast_bg, 3.0),
            ("on_button/proposal_bg", u.on_button, ch.proposal_bg, 3.0),
            (
                "on_button/banner_danger_bg",
                u.on_button,
                ch.banner_danger_bg,
                3.0,
            ),
            (
                "toolbar.pull/toolbar_bg",
                self.toolbar.pull,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.push/toolbar_bg",
                self.toolbar.push,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.stash/toolbar_bg",
                self.toolbar.stash,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.branch/toolbar_bg",
                self.toolbar.branch,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.lfs/toolbar_bg",
                self.toolbar.lfs,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.tag/toolbar_bg",
                self.toolbar.tag,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.search/toolbar_bg",
                self.toolbar.search,
                ch.toolbar_bg,
                3.0,
            ),
            (
                "toolbar.undo/toolbar_bg",
                self.toolbar.undo,
                ch.toolbar_bg,
                3.0,
            ),
            ("toolbar_fg/toolbar_bg", ch.toolbar_fg, ch.toolbar_bg, 4.5),
            (
                "toolbar_fg/toolbar_arrow_bg",
                ch.toolbar_fg,
                ch.toolbar_arrow_bg,
                3.0,
            ),
            (
                "toolbar_fg/toolbar_bg_hover",
                ch.toolbar_fg,
                ch.toolbar_bg_hover,
                3.0,
            ),
            (
                "toolbar_disabled_fg/toolbar_bg",
                ch.toolbar_disabled_fg,
                ch.toolbar_bg,
                3.0,
            ),
            ("on_bright/marker", ch.on_bright, ch.marker, 3.0),
            ("on_bright/dropdown_hit", ch.on_bright, ch.dropdown_hit, 3.0),
        ];
        // Text on the side panes' own background.
        let side = self.effective_side_bg();
        let side_rules: [(&str, ThemeColor, f64); 4] = [
            ("text/side_bg", u.text, 4.5),
            ("muted/side_bg", u.muted, 3.0),
            ("accent/side_bg", u.accent, 3.0),
            ("danger/side_bg", self.status.danger, 3.0),
        ];
        for (rule, fg, min) in side_rules {
            check(rule.to_string(), fg, side, min);
        }
        // The toolbar row (and the selected tab on it) and the status bar.
        let toolbar = self.effective_toolbar_bg();
        let status = self.effective_statusbar_bg();
        check("text/toolbar_bg".into(), u.text, toolbar, 4.5);
        check("muted/toolbar_bg".into(), u.muted, toolbar, 3.0);
        check(
            "statusbar_fg/statusbar_bg".into(),
            self.effective_statusbar_fg(),
            status,
            4.5,
        );
        for (rule, fg, back, min) in rules {
            check(rule.to_string(), fg, back, min);
        }
        for (i, lane) in self.graph_palette.iter().enumerate() {
            check(format!("graph_palette[{i}]/bg"), *lane, bg, 2.0);
        }
        let distinct = [
            ("selection", u.selection, "hover", u.hover),
            ("hover", u.hover, "input_rest", u.input_rest),
            ("selection", u.selection, "input_rest", u.input_rest),
        ];
        for (an, a, bn, b) in distinct {
            if a == b {
                issues.push(ContrastIssue {
                    rule: format!("{an} must differ from {bn}"),
                    ratio: 1.0,
                    min: f64::INFINITY,
                });
            }
        }
        // On a distinct side background the selection must still stand out.
        if self.ui.side_bg != ThemeColor::Default && self.ui.selection == self.ui.side_bg {
            issues.push(ContrastIssue {
                rule: "selection must differ from side_bg".into(),
                ratio: 1.0,
                min: f64::INFINITY,
            });
        }
        issues
    }
}
