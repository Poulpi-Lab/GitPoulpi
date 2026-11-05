//! Fitting Log columns to their content: the width overrides one fit writes
//! and the status line it ends with.
//!
//! Pure over the width map, so the double click, the `=` / `+` keys, the
//! palette and the chooser share one implementation (`commands/log_columns`
//! wraps it).

use super::*;
use gitpoulpi_i18n::msg::log_table as msg;

type Overrides = std::collections::HashMap<LogColumn, u16>;

/// Fits `col` to what `natural` measured. In the fit layout the column stops
/// at [`LogColumnWidths::fit_max`] (the pane's room) and `Message` takes up
/// what it leaves; the status line says so when the content did not fit.
/// `Message` is the flexible column and is never stored: fitting it only
/// says where its width comes from.
pub(crate) fn fit_column(
    log_area: Rect,
    all_order: &[LogColumn],
    hidden: &std::collections::HashSet<LogColumn>,
    overrides: &mut Overrides,
    natural: &NaturalWidths,
    col: LogColumn,
) -> String {
    if col == LogColumn::Message {
        return msg::COLUMN_FIT_MESSAGE.get().into();
    }
    let geo = LogColumnsGeo::compute(log_area, all_order, hidden, overrides);
    let need = natural.of(col);
    let room = geo
        .widths
        .fit_max(log_area.width, &geo.order, overrides, col);
    overrides.insert(col, need.min(room));
    let drawn = LogColumnsGeo::compute(log_area, all_order, hidden, overrides)
        .widths
        .width_of(col);
    if drawn < need {
        msg::column_fit_cut(col.title(), u64::from(need), u64::from(drawn))
    } else {
        msg::column_fitted(col.title(), u64::from(drawn))
    }
}

/// Fits every visible column but `Message`: each asks for its natural width
/// and `for_area_width` shares what the pane has (the columns that are
/// served first are the ones nearest the start of its growth order).
pub(crate) fn fit_all(
    log_area: Rect,
    all_order: &[LogColumn],
    hidden: &std::collections::HashSet<LogColumn>,
    overrides: &mut Overrides,
    natural: &NaturalWidths,
) -> String {
    let order = visible_in_order(all_order, hidden);
    for &col in order.iter().filter(|c| **c != LogColumn::Message) {
        overrides.insert(col, natural.of(col));
    }
    let geo = LogColumnsGeo::compute(log_area, all_order, hidden, overrides);
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

    fn order() -> Vec<LogColumn> {
        LogColumn::ALL.to_vec()
    }

    fn hidden() -> std::collections::HashSet<LogColumn> {
        [LogColumn::Changes].into()
    }

    /// A column that fits is drawn at its natural width, the widths still
    /// tile the pane and `Message` took up the difference.
    #[test]
    fn a_column_that_fits_gets_its_natural_width_and_the_pane_stays_tiled() {
        let area = Rect::new(0, 0, 130, 20);
        let mut overrides = Overrides::new();
        let status = fit_column(
            area,
            &order(),
            &hidden(),
            &mut overrides,
            &natural(27),
            LogColumn::Author,
        );
        let geo = LogColumnsGeo::compute(area, &order(), &hidden(), &overrides);
        assert_eq!(geo.widths.width_of(LogColumn::Author), 28, "{status}");
        let sum: u16 = geo.offsets.iter().map(|(_, _, w)| *w).sum();
        assert_eq!(sum, 128);
        assert!(status.contains("28"), "{status}");
    }

    /// In a narrow pane the column stops at `fit_max` and says what it
    /// needed. The implicit maximum: the pane's room less `Message`'s
    /// reservation and every other column's floor.
    #[test]
    fn a_column_stops_at_fit_max_and_the_status_says_so() {
        let area = Rect::new(0, 0, 60, 20);
        let mut overrides = Overrides::new();
        let status = fit_column(
            area,
            &order(),
            &hidden(),
            &mut overrides,
            &natural(80),
            LogColumn::Author,
        );
        let geo = LogColumnsGeo::compute(area, &order(), &hidden(), &overrides);
        let w = geo.widths.width_of(LogColumn::Author);
        let max = geo
            .widths
            .fit_max(60, &geo.order, &overrides, LogColumn::Author);
        assert_eq!(w, max, "{status}");
        assert!(w < 81);
        assert!(status.contains("needs 81 cells"), "{status}");
        let sum: u16 = geo.offsets.iter().map(|(_, _, w)| *w).sum();
        assert_eq!(sum, 58, "still tiles the inner width");
    }

    #[test]
    fn fitting_message_stores_nothing() {
        let mut overrides = Overrides::new();
        let status = fit_column(
            Rect::new(0, 0, 100, 20),
            &order(),
            &hidden(),
            &mut overrides,
            &natural(5),
            LogColumn::Message,
        );
        assert!(overrides.is_empty());
        assert!(status.contains("Message"), "{status}");
    }

    #[test]
    fn fit_all_names_the_columns_that_were_cut() {
        let area = Rect::new(0, 0, 50, 20);
        let mut overrides = Overrides::new();
        let mut n = natural(40);
        n.grow(LogColumn::Ref, 40);
        let status = fit_all(area, &order(), &hidden(), &mut overrides, &n);
        assert!(!overrides.contains_key(&LogColumn::Message));
        assert!(status.contains("not enough room"), "{status}");
        let sum: u16 = LogColumnsGeo::compute(area, &order(), &hidden(), &overrides)
            .offsets
            .iter()
            .map(|(_, _, w)| *w)
            .sum();
        assert_eq!(sum, 48);
    }

    /// A pane wide enough for everything: nothing cut.
    #[test]
    fn fit_all_in_a_wide_pane_cuts_nothing() {
        let mut overrides = Overrides::new();
        let status = fit_all(
            Rect::new(0, 0, 240, 20),
            &order(),
            &hidden(),
            &mut overrides,
            &natural(20),
        );
        assert_eq!(status, "All columns fitted to their content.");
    }
}
