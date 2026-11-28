//! Log column widths: what the user prefers, and what the layout makes of it in the room there is.
//!
//! The preference is never rewritten by layout. [`resolve`] derives the widths actually drawn from it
//! each time the pane's width is known, so a narrow window shows the same columns a wide one does and
//! the user's choice comes back as soon as there is room again. The rule is that nothing vanishes:
//! every column has a minimum, Ref and Graph give way first (proportionally), then Author, Date and
//! SHA, and Message always keeps a usable minimum.

use gitpoulpi_config::{DesktopColumn, DesktopColumnWidths};

/// The user's preferred width of each fixed column, in logical pixels. Message is not listed: it takes
/// whatever the others leave.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnWidths {
    pub ref_w: f32,
    pub graph_w: f32,
    pub author_w: f32,
    pub date_w: f32,
    pub sha_w: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        DesktopColumnWidths::default().into()
    }
}

/// The stored widths (whole logical pixels, `desktop.toml`) as the layout works with them.
impl From<DesktopColumnWidths> for ColumnWidths {
    fn from(w: DesktopColumnWidths) -> Self {
        Self {
            ref_w: w.ref_w as f32,
            graph_w: w.graph_w as f32,
            author_w: w.author_w as f32,
            date_w: w.date_w as f32,
            sha_w: w.sha_w as f32,
        }
    }
}

/// The widths to store: rounded to whole pixels and kept inside each column's limits.
impl From<ColumnWidths> for DesktopColumnWidths {
    fn from(w: ColumnWidths) -> Self {
        let mut out = DesktopColumnWidths::default();
        for col in Col::ALL {
            let px = w.get(col);
            let px = if px.is_finite() {
                px.round().max(0.0) as u32
            } else {
                0
            };
            out.set(col.shared(), px);
        }
        out
    }
}

/// A fixed column of the Log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Col {
    Ref,
    Graph,
    Author,
    Date,
    Sha,
}

impl Col {
    pub const ALL: [Col; 5] = [Col::Ref, Col::Graph, Col::Author, Col::Date, Col::Sha];

    fn ix(self) -> usize {
        self as usize
    }

    /// The shared schema's name for this column (its limits live there).
    pub const fn shared(self) -> DesktopColumn {
        match self {
            Col::Ref => DesktopColumn::Ref,
            Col::Graph => DesktopColumn::Graph,
            Col::Author => DesktopColumn::Author,
            Col::Date => DesktopColumn::Date,
            Col::Sha => DesktopColumn::Sha,
        }
    }

    /// Smallest width the column can be dragged to; also the floor of narrow layouts.
    pub const fn min(self) -> f32 {
        self.shared().min() as f32
    }

    /// Largest width a drag or an auto-fit may give the column.
    pub const fn max(self) -> f32 {
        self.shared().max() as f32
    }
}

/// The Message column keeps at least this much while any other column can still give room.
pub const MESSAGE_MIN: f32 = 160.0;
/// When even the minimums do not fit, everything scales down together but Message never gets less.
pub const MESSAGE_FLOOR: f32 = 96.0;

impl ColumnWidths {
    fn to_array(self) -> [f32; 5] {
        [
            self.ref_w,
            self.graph_w,
            self.author_w,
            self.date_w,
            self.sha_w,
        ]
    }

    pub fn get(&self, col: Col) -> f32 {
        self.to_array()[col.ix()]
    }

    /// Sets one preference, kept inside the column's own limits.
    pub fn set(&mut self, col: Col, width: f32) {
        let w = clamp_pref(col, width);
        match col {
            Col::Ref => self.ref_w = w,
            Col::Graph => self.graph_w = w,
            Col::Author => self.author_w = w,
            Col::Date => self.date_w = w,
            Col::Sha => self.sha_w = w,
        }
    }

    /// A copy with every preference inside its column's limits (what a hand-edited file may break).
    pub fn sanitized(self) -> Self {
        let mut out = self;
        for col in Col::ALL {
            out.set(col, self.get(col));
        }
        out
    }
}

fn clamp_pref(col: Col, width: f32) -> f32 {
    if width.is_finite() {
        width.clamp(col.min(), col.max())
    } else {
        ColumnWidths::default().get(col)
    }
}

/// The widths drawn for one pane width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnLayout {
    pub ref_w: f32,
    pub graph_w: f32,
    pub author_w: f32,
    pub date_w: f32,
    pub sha_w: f32,
    pub message_w: f32,
}

impl ColumnLayout {
    pub fn get(&self, col: Col) -> f32 {
        match col {
            Col::Ref => self.ref_w,
            Col::Graph => self.graph_w,
            Col::Author => self.author_w,
            Col::Date => self.date_w,
            Col::Sha => self.sha_w,
        }
    }

    #[cfg(test)]
    fn fixed_total(&self) -> f32 {
        self.ref_w + self.graph_w + self.author_w + self.date_w + self.sha_w
    }
}

/// Takes `deficit` pixels out of the columns in `group`, each in proportion to how far it is above its
/// minimum. Returns what could not be taken.
fn shrink(widths: &mut [f32; 5], group: &[Col], deficit: f32) -> f32 {
    let room: f32 = group
        .iter()
        .map(|c| (widths[c.ix()] - c.min()).max(0.0))
        .sum();
    if room <= 0.0 || deficit <= 0.0 {
        return deficit.max(0.0);
    }
    let take = deficit.min(room);
    for c in group {
        let spare = (widths[c.ix()] - c.min()).max(0.0);
        widths[c.ix()] -= take * spare / room;
    }
    deficit - take
}

/// Resolves `prefs` for a pane `available` pixels wide.
pub fn resolve(prefs: &ColumnWidths, available: f32) -> ColumnLayout {
    let mut w = prefs.sanitized().to_array();
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let total: f32 = w.iter().sum();
    let mut deficit = total + MESSAGE_MIN - available;
    if deficit > 0.0 {
        deficit = shrink(&mut w, &[Col::Ref, Col::Graph], deficit);
        deficit = shrink(&mut w, &[Col::Author, Col::Date, Col::Sha], deficit);
    }
    let mut fixed: f32 = w.iter().sum();
    if deficit > 0.0 {
        // Even the minimums plus the Message minimum do not fit: Message keeps what is left down to its
        // floor, and the remaining columns scale together so the row still fits the pane.
        let room_for_fixed = (available - MESSAGE_FLOOR).max(0.0);
        if fixed > room_for_fixed && fixed > 0.0 {
            let scale = room_for_fixed / fixed;
            for v in &mut w {
                *v *= scale;
            }
            fixed = room_for_fixed;
        }
    }
    let message_w = (available - fixed).max(0.0);
    ColumnLayout {
        ref_w: w[0],
        graph_w: w[1],
        author_w: w[2],
        date_w: w[3],
        sha_w: w[4],
        message_w,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum(l: &ColumnLayout) -> f32 {
        l.fixed_total() + l.message_w
    }

    #[test]
    fn wide_pane_keeps_the_preferences_and_gives_message_the_rest() {
        let prefs = ColumnWidths::default();
        let l = resolve(&prefs, 1400.0);
        assert_eq!(l.ref_w, prefs.ref_w);
        assert_eq!(l.graph_w, prefs.graph_w);
        assert_eq!(l.sha_w, prefs.sha_w);
        assert!((l.message_w - (1400.0 - 704.0)).abs() < 0.01);
    }

    #[test]
    fn narrow_pane_shrinks_ref_and_graph_first_and_message_keeps_its_minimum() {
        let prefs = ColumnWidths::default();
        // 704 fixed + 160 message = 864 needed; 700 available leaves a 164 px deficit.
        let l = resolve(&prefs, 700.0);
        assert!(l.ref_w < prefs.ref_w && l.graph_w < prefs.graph_w);
        assert!(l.ref_w >= Col::Ref.min() && l.graph_w >= Col::Graph.min());
        assert_eq!(l.author_w, prefs.author_w);
        assert_eq!(l.date_w, prefs.date_w);
        assert_eq!(l.sha_w, prefs.sha_w);
        assert!(l.message_w >= MESSAGE_MIN - 0.01, "message {}", l.message_w);
        assert!((sum(&l) - 700.0).abs() < 0.01);
    }

    #[test]
    fn ref_and_graph_shrink_in_proportion_to_their_room() {
        let prefs = ColumnWidths {
            ref_w: 264.0,
            graph_w: 148.0,
            ..ColumnWidths::default()
        };
        // Rooms above the minimums: Ref 200, Graph 100.
        let total = prefs.ref_w + prefs.graph_w + prefs.author_w + prefs.date_w + prefs.sha_w;
        let l = resolve(&prefs, total + MESSAGE_MIN - 90.0);
        assert!((prefs.ref_w - l.ref_w - 60.0).abs() < 0.01);
        assert!((prefs.graph_w - l.graph_w - 30.0).abs() < 0.01);
    }

    #[test]
    fn at_1280_px_nothing_vanishes() {
        // Window 1280 with the default side panels (260 + 380) leaves about 630 for the Log.
        let l = resolve(&ColumnWidths::default(), 630.0);
        for col in Col::ALL {
            assert!(l.get(col) >= col.min() - 0.01, "{col:?} {}", l.get(col));
        }
        assert!(l.message_w >= MESSAGE_MIN - 0.01, "message {}", l.message_w);
        assert!((sum(&l) - 630.0).abs() < 0.01);
    }

    #[test]
    fn author_date_and_sha_give_way_only_after_ref_and_graph_are_at_their_minimum() {
        let l = resolve(&ColumnWidths::default(), 500.0);
        assert_eq!(l.ref_w, Col::Ref.min());
        assert_eq!(l.graph_w, Col::Graph.min());
        assert!(l.author_w < ColumnWidths::default().author_w);
        // The minimums plus the Message minimum (540) do not fit in 500: Message is left with its floor or more.
        assert!(l.message_w >= MESSAGE_FLOOR - 0.01);
        assert!((sum(&l) - 500.0).abs() < 0.01);
    }

    #[test]
    fn tiny_pane_scales_everything_down_and_message_keeps_its_floor() {
        let l = resolve(&ColumnWidths::default(), 300.0);
        assert!((sum(&l) - 300.0).abs() < 0.01);
        assert!(l.message_w >= MESSAGE_FLOOR - 0.01);
        for col in Col::ALL {
            assert!(l.get(col) > 0.0);
        }
    }

    #[test]
    fn layout_never_changes_the_preferences() {
        let prefs = ColumnWidths::default();
        let before = prefs;
        let _ = resolve(&prefs, 400.0);
        assert_eq!(prefs, before);
    }

    #[test]
    fn preferences_come_back_when_there_is_room_again() {
        let prefs = ColumnWidths {
            ref_w: 250.0,
            ..ColumnWidths::default()
        };
        let narrow = resolve(&prefs, 600.0);
        let wide = resolve(&prefs, 1600.0);
        assert!(narrow.ref_w < 250.0);
        assert_eq!(wide.ref_w, 250.0);
    }

    #[test]
    fn set_clamps_to_the_column_limits() {
        let mut prefs = ColumnWidths::default();
        prefs.set(Col::Ref, 5.0);
        assert_eq!(prefs.ref_w, Col::Ref.min());
        prefs.set(Col::Graph, 99_999.0);
        assert_eq!(prefs.graph_w, Col::Graph.max());
        prefs.set(Col::Sha, f32::NAN);
        assert_eq!(prefs.sha_w, ColumnWidths::default().sha_w);
    }

    #[test]
    fn hand_edited_values_are_pulled_into_range() {
        let wild = ColumnWidths {
            ref_w: -3.0,
            graph_w: 1e9,
            author_w: 0.0,
            date_w: 5.0,
            sha_w: 5000.0,
        };
        let l = resolve(&wild, 2000.0);
        for col in Col::ALL {
            assert!(l.get(col) >= col.min() && l.get(col) <= col.max());
        }
    }
}
