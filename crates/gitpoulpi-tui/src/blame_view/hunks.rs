//! A blame result as hunks: runs of consecutive lines one commit last
//! touched. Computed once per result, never per frame.

use std::collections::HashMap;

use gitpoulpi_core::domain::{BlameLine, Oid};
use ratatui::style::Color;

/// A run of consecutive lines attributed to one commit.
#[derive(Debug, Clone)]
pub(crate) struct Hunk {
    /// First line (0-based) and one past the last.
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) commit: Oid,
    /// The lines are not committed yet (the working tree's own edits).
    pub(crate) uncommitted: bool,
    /// The author's stable color (the initials badge's).
    pub(crate) color: Color,
    /// 0.0 for the file's oldest line, 1.0 for its newest.
    pub(crate) heat: f32,
}

/// The hunks of `lines` and, for every line, the index of its hunk: the shared
/// runs plus the author's colour.
pub(crate) fn build(lines: &[BlameLine]) -> (Vec<Hunk>, Vec<u32>) {
    let (runs, hunk_of) = gitpoulpi_link::blame_runs::build(lines);
    let mut colors: HashMap<&str, Color> = HashMap::new();
    let hunks = runs
        .into_iter()
        .map(|r| {
            let line = &lines[r.start];
            let color = *colors.entry(line.author_email.as_str()).or_insert_with(|| {
                crate::avatar::badge(&line.author_name, &line.author_email).color
            });
            Hunk {
                start: r.start,
                end: r.end,
                commit: r.commit,
                uncommitted: r.uncommitted,
                color,
                heat: r.heat,
            }
        })
        .collect();
    (hunks, hunk_of)
}

/// The age heatmap's color: cold blue (old) through yellow to a warm
/// red-orange (recent). Explicit RGB, readable on every theme's background.
pub(crate) fn heat_color(heat: f32) -> Color {
    const STOPS: [(f32, f32, f32); 3] = [
        (60.0, 110.0, 220.0),
        (230.0, 200.0, 70.0),
        (240.0, 95.0, 60.0),
    ];
    let h = heat.clamp(0.0, 1.0) * 2.0;
    let (a, b, t) = if h < 1.0 {
        (STOPS[0], STOPS[1], h)
    } else {
        (STOPS[1], STOPS[2], h - 1.0)
    };
    let mix = |x: f32, y: f32| (x + (y - x) * t).round() as u8;
    Color::Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}
