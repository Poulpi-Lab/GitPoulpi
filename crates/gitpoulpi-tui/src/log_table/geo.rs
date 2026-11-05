//! The Log table's column geometry, computed once.
//!
//! Every site that needs "where is column X on screen" (paint, header hits,
//! hover, boundary drags, the keyboard resize, the ref popup anchor) asks one
//! [`LogColumnsGeo`] instead of rebuilding widths, visible order and offsets
//! by hand. A new layout (a horizontal offset, say) then has exactly one
//! place to land.

use super::*;

/// The Log's columns for one pane rectangle: widths, visible display order
/// and on-screen `(column, x, width)` offsets.
#[derive(Debug, Default)]
pub(crate) struct LogColumnsGeo {
    pub(crate) widths: LogColumnWidths,
    /// Visible columns, in display order.
    pub(crate) order: Vec<LogColumn>,
    /// `(column, screen x, width)` per visible column, left to right.
    pub(crate) offsets: Vec<(LogColumn, u16, u16)>,
}

impl LogColumnsGeo {
    /// `all_order` is the full display order (hidden columns included), as
    /// stored; hidden ones are dropped here.
    pub(crate) fn compute(
        log_area: Rect,
        all_order: &[LogColumn],
        hidden: &std::collections::HashSet<LogColumn>,
        overrides: &std::collections::HashMap<LogColumn, u16>,
    ) -> Self {
        let widths = LogColumnWidths::for_area_width(log_area.width, hidden, overrides);
        let order = visible_in_order(all_order, hidden);
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        Self {
            widths,
            order,
            offsets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The geometry is the hand-rolled chain it replaced, at every width and
    /// with columns hidden.
    #[test]
    fn geo_equals_the_hand_built_chain() {
        let overrides = std::collections::HashMap::from([(LogColumn::Author, 15)]);
        for width in [20u16, 40, 80, 120, 200] {
            for hidden in [
                std::collections::HashSet::new(),
                [LogColumn::Changes].into(),
                [LogColumn::Ref, LogColumn::Date].into(),
            ] {
                let area = Rect::new(3, 2, width, 20);
                let order = LogColumn::ALL.to_vec();
                let geo = LogColumnsGeo::compute(area, &order, &hidden, &overrides);
                let widths = LogColumnWidths::for_area_width(width, &hidden, &overrides);
                let visible = visible_in_order(&order, &hidden);
                assert_eq!(geo.order, visible);
                assert_eq!(geo.offsets, column_x_offsets(area.x, &visible, &widths));
                let sum: u16 = geo.offsets.iter().map(|(_, _, w)| *w).sum();
                assert_eq!(sum, width.saturating_sub(2), "tiles the inner width");
            }
        }
    }
}
