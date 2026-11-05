//! Fitting Log columns to their content: the width overrides one fit writes
//! and the status line it ends with.
//!
//! Pure over the width map, so the double click, the `=` / `+` keys, the
//! palette and the chooser share one implementation (`commands/log_columns`
//! wraps it).

use super::*;
use gitpoulpi_config::LogColumnsLayout;
use gitpoulpi_i18n::msg::log_table as msg;

type Map = std::collections::HashMap<LogColumn, u16>;

/// What a fit needs to know about the Log: its pane, the columns shown and
/// the layout in force.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FitCtx<'a> {
    pub(crate) log_area: Rect,
    pub(crate) all_order: &'a [LogColumn],
    pub(crate) hidden: &'a std::collections::HashSet<LogColumn>,
    pub(crate) layout: LogColumnsLayout,
}

/// The two width maps a fit may write, one per layout.
pub(crate) struct FitMaps<'a> {
    pub(crate) fit: &'a mut Map,
    pub(crate) scroll: &'a mut Map,
}

impl FitCtx<'_> {
    fn geo(&self, maps: &FitMaps<'_>) -> LogColumnsGeo {
        LogColumnsGeo::compute(
            self.log_area,
            self.all_order,
            self.hidden,
            &LogWidthMaps {
                fit: maps.fit,
                scroll: maps.scroll,
            },
            (self.layout, 0),
        )
    }
}

/// Stores every visible column's current width in the `scroll` layout's map
/// (the ones it has already keep theirs): from then on a drag, a key or a fit
/// changes one column and nothing else moves. Until pinned, a column without
/// a width follows the `fit` layout at the pane's size.
pub(crate) fn pin_scroll_widths(
    log_area: Rect,
    hidden: &std::collections::HashSet<LogColumn>,
    scroll: &mut Map,
    fit: &Map,
) {
    let widths = LogColumnWidths::for_scroll(log_area.width, hidden, scroll, fit);
    for col in LogColumn::ALL {
        if !hidden.contains(&col) {
            scroll.entry(col).or_insert_with(|| widths.width_of(col));
        }
    }
}

/// Fits `col` to what `natural` measured. In the `fit` layout the column stops
/// at [`LogColumnWidths::fit_max`] (the pane's room) and `Message` takes up
/// what it leaves; the status line says so when the content did not fit.
/// `Message` is the flexible column there and is never stored: fitting it only
/// says where its width comes from. In the `scroll` layout every column,
/// `Message` too, simply takes its natural width (clamped to what a column can
/// be) and the table grows.
pub(crate) fn fit_column(
    ctx: &FitCtx<'_>,
    maps: &mut FitMaps<'_>,
    natural: &NaturalWidths,
    col: LogColumn,
) -> String {
    let need = natural.of(col);
    if ctx.layout == LogColumnsLayout::Scroll {
        pin_scroll_widths(ctx.log_area, ctx.hidden, maps.scroll, maps.fit);
        maps.scroll.insert(col, need);
        return msg::column_fitted(col.title(), u64::from(need));
    }
    if col == LogColumn::Message {
        return msg::COLUMN_FIT_MESSAGE.get().into();
    }
    let geo = ctx.geo(maps);
    let room = geo
        .widths
        .fit_max(ctx.log_area.width, &geo.order, maps.fit, col);
    maps.fit.insert(col, need.min(room));
    let drawn = ctx.geo(maps).widths.width_of(col);
    if drawn < need {
        msg::column_fit_cut(col.title(), u64::from(need), u64::from(drawn))
    } else {
        msg::column_fitted(col.title(), u64::from(drawn))
    }
}

/// Fits every visible column. In the `fit` layout `Message` is left out and
/// each of the others asks for its natural width, `for_area_width` sharing
/// what the pane has (the columns served first are those nearest the start of
/// its growth order); the status line names the ones that were cut. In the
/// `scroll` layout every column, `Message` included, gets its natural width.
pub(crate) fn fit_all(ctx: &FitCtx<'_>, maps: &mut FitMaps<'_>, natural: &NaturalWidths) -> String {
    let order = visible_in_order(ctx.all_order, ctx.hidden);
    if ctx.layout == LogColumnsLayout::Scroll {
        pin_scroll_widths(ctx.log_area, ctx.hidden, maps.scroll, maps.fit);
        for &col in &order {
            maps.scroll.insert(col, natural.of(col));
        }
        return msg::COLUMNS_FITTED_ALL.get().into();
    }
    for &col in order.iter().filter(|c| **c != LogColumn::Message) {
        maps.fit.insert(col, natural.of(col));
    }
    let geo = ctx.geo(maps);
    let cut: Vec<&str> = order
        .iter()
        .filter(|c| **c != LogColumn::Message)
        .filter(|c| geo.widths.width_of(**c) < natural.of(**c))
        .map(|c| c.title())
        .collect();
    if cut.is_empty() {
        msg::COLUMNS_FITTED_ALL.get().into()
    } else {
        msg::columns_fit_all_cut(cut.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn natural(author: usize) -> NaturalWidths {
        let mut n = NaturalWidths::new();
        n.grow(LogColumn::Author, author);
        n
    }

    fn hidden() -> std::collections::HashSet<LogColumn> {
        [LogColumn::Changes].into()
    }

    const ORDER: [LogColumn; 7] = LogColumn::ALL;

    fn ctx<'a>(
        width: u16,
        layout: LogColumnsLayout,
        hidden: &'a std::collections::HashSet<LogColumn>,
    ) -> FitCtx<'a> {
        FitCtx {
            log_area: Rect::new(0, 0, width, 20),
            all_order: &ORDER,
            hidden,
            layout,
        }
    }

    /// Runs `f` against fresh maps and returns the fit map, the scroll map and
    /// what `f` returned.
    fn run<T>(f: impl FnOnce(&mut FitMaps<'_>) -> T) -> (Map, Map, T) {
        let (mut fit, mut scroll) = (Map::new(), Map::new());
        let out = f(&mut FitMaps {
            fit: &mut fit,
            scroll: &mut scroll,
        });
        (fit, scroll, out)
    }

    fn tiled(width: u16, fit: &Map) -> (LogColumnsGeo, u16) {
        let geo = LogColumnsGeo::compute(
            Rect::new(0, 0, width, 20),
            &ORDER,
            &hidden(),
            &LogWidthMaps {
                fit,
                scroll: &Map::new(),
            },
            (LogColumnsLayout::Fit, 0),
        );
        let sum = geo.offsets.iter().map(|(_, _, w)| *w).sum();
        (geo, sum)
    }

    /// A column that fits is drawn at its natural width, the widths still
    /// tile the pane and `Message` took up the difference.
    #[test]
    fn a_column_that_fits_gets_its_natural_width_and_the_pane_stays_tiled() {
        let h = hidden();
        let (fit, _, status) = run(|m| {
            fit_column(
                &ctx(130, LogColumnsLayout::Fit, &h),
                m,
                &natural(27),
                LogColumn::Author,
            )
        });
        let (geo, sum) = tiled(130, &fit);
        assert_eq!(geo.widths.width_of(LogColumn::Author), 28, "{status}");
        assert_eq!(sum, 128);
        assert!(status.contains("28"), "{status}");
    }

    /// In a narrow pane the column stops at `fit_max` and says what it
    /// needed. The implicit maximum: the pane's room less `Message`'s
    /// reservation and every other column's floor.
    #[test]
    fn a_column_stops_at_fit_max_and_the_status_says_so() {
        let h = hidden();
        let (fit, _, status) = run(|m| {
            fit_column(
                &ctx(60, LogColumnsLayout::Fit, &h),
                m,
                &natural(80),
                LogColumn::Author,
            )
        });
        let (geo, sum) = tiled(60, &fit);
        let w = geo.widths.width_of(LogColumn::Author);
        let max = geo.widths.fit_max(60, &geo.order, &fit, LogColumn::Author);
        assert_eq!(w, max, "{status}");
        assert!(w < 81);
        assert!(status.contains("needs 81 cells"), "{status}");
        assert_eq!(sum, 58, "still tiles the inner width");
    }

    #[test]
    fn fitting_message_stores_nothing_in_the_fit_layout() {
        let h = hidden();
        let (fit, scroll, status) = run(|m| {
            fit_column(
                &ctx(100, LogColumnsLayout::Fit, &h),
                m,
                &natural(5),
                LogColumn::Message,
            )
        });
        assert!(fit.is_empty() && scroll.is_empty());
        assert!(status.contains("Message"), "{status}");
    }

    #[test]
    fn fit_all_names_the_columns_that_were_cut() {
        let h = hidden();
        let mut n = natural(40);
        n.grow(LogColumn::Ref, 40);
        let (fit, _, status) = run(|m| fit_all(&ctx(50, LogColumnsLayout::Fit, &h), m, &n));
        assert!(!fit.contains_key(&LogColumn::Message));
        assert!(status.contains("not enough room"), "{status}");
        assert_eq!(tiled(50, &fit).1, 48);
    }

    /// A pane wide enough for everything: nothing cut.
    #[test]
    fn fit_all_in_a_wide_pane_cuts_nothing() {
        let h = hidden();
        let (_, _, status) =
            run(|m| fit_all(&ctx(240, LogColumnsLayout::Fit, &h), m, &natural(20)));
        assert_eq!(status, "All columns fitted to their content.");
    }

    /// The scroll layout: the column takes its natural width however narrow
    /// the pane, the others are pinned where they were (nothing else moves),
    /// and `Message` can be fitted too.
    #[test]
    fn in_the_scroll_layout_a_fit_pins_the_others_and_takes_the_natural_width() {
        let h = hidden();
        let (fit, scroll, status) = run(|m| {
            fit_column(
                &ctx(60, LogColumnsLayout::Scroll, &h),
                m,
                &natural(80),
                LogColumn::Author,
            )
        });
        assert!(fit.is_empty(), "the fit layout's widths are untouched");
        assert_eq!(scroll.get(&LogColumn::Author), Some(&81), "{status}");
        for col in LogColumn::ALL.iter().filter(|c| !h.contains(c)) {
            assert!(scroll.contains_key(col), "{col:?} pinned");
        }
        let mut n = natural(1);
        n.grow(LogColumn::Message, 120);
        let (_, scroll, _) = run(|m| {
            fit_column(
                &ctx(60, LogColumnsLayout::Scroll, &h),
                m,
                &n,
                LogColumn::Message,
            )
        });
        assert_eq!(scroll.get(&LogColumn::Message), Some(&121));
    }

    /// Pinning keeps what a column already has and fills in the rest from
    /// what the pane shows.
    #[test]
    fn pinning_keeps_stored_widths() {
        let h = hidden();
        let mut scroll = Map::from([(LogColumn::Date, 33)]);
        pin_scroll_widths(Rect::new(0, 0, 100, 20), &h, &mut scroll, &Map::new());
        assert_eq!(scroll.get(&LogColumn::Date), Some(&33));
        assert!(!scroll.contains_key(&LogColumn::Changes));
        assert!(scroll.get(&LogColumn::Message).is_some_and(|w| *w >= 8));
    }
}
