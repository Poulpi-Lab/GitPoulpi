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

/// The hunks of `lines` and, for every line, the index of its hunk.
pub(crate) fn build(lines: &[BlameLine]) -> (Vec<Hunk>, Vec<u32>) {
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut hunk_of = Vec::with_capacity(lines.len());
    let mut colors: HashMap<&str, Color> = HashMap::new();
    let (mut oldest, mut newest) = (i64::MAX, i64::MIN);
    for (i, line) in lines.iter().enumerate() {
        let uncommitted = crate::wip_view::is_uncommitted(line.commit);
        if !uncommitted {
            oldest = oldest.min(line.timestamp);
            newest = newest.max(line.timestamp);
        }
        match hunks.last_mut() {
            Some(last) if last.commit == line.commit => last.end = i + 1,
            _ => {
                let color = *colors.entry(line.author_email.as_str()).or_insert_with(|| {
                    crate::avatar::badge(&line.author_name, &line.author_email).color
                });
                hunks.push(Hunk {
                    start: i,
                    end: i + 1,
                    commit: line.commit,
                    uncommitted,
                    color,
                    heat: 0.5,
                });
            }
        }
        hunk_of.push((hunks.len() - 1) as u32);
    }
    if newest > oldest {
        let span = (newest - oldest) as f32;
        for h in &mut hunks {
            if !h.uncommitted {
                h.heat = (lines[h.start].timestamp - oldest) as f32 / span;
            }
        }
    }
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
