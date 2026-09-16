//! Display units for the settings Settings shows as "a number and a unit on
//! one line" (`general.auto_fetch_interval_unit` and its siblings).
//!
//! The stored number keeps its own key and its own base unit
//! (`auto_fetch_interval_minutes` is minutes, `script_timeout_ms` is
//! milliseconds, `max_file_mb` is MiB), so an older GitPoulpi that does not
//! know the unit key still reads the right value. The unit key only records
//! how the number is shown, like `log.retention_unit` does for the log
//! retention.

use serde::{Deserialize, Serialize};

config_enum! {
    /// The unit a stored duration or size is shown in. `auto` (also a config
    /// without the key) shows the largest unit that divides the number
    /// evenly. A unit that does not fit the setting (`"mib"` on a duration,
    /// or one that does not divide the stored number) is ignored and read as
    /// `auto`: the pair always means exactly the stored number.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DisplayUnit {
        #[default]
        Auto => "auto",
        Milliseconds => "milliseconds",
        Seconds => "seconds",
        Minutes => "minutes",
        Hours => "hours",
        Mib => "mib",
        Gib => "gib",
    }
}

impl DisplayUnit {
    /// The unit's size in the smallest unit of its family (milliseconds for
    /// durations, bytes for sizes), and the family (`true` for durations).
    /// `Auto` has none.
    fn scale(self) -> Option<(u64, bool)> {
        match self {
            DisplayUnit::Auto => None,
            DisplayUnit::Milliseconds => Some((1, true)),
            DisplayUnit::Seconds => Some((1_000, true)),
            DisplayUnit::Minutes => Some((60_000, true)),
            DisplayUnit::Hours => Some((3_600_000, true)),
            DisplayUnit::Mib => Some((1 << 20, false)),
            DisplayUnit::Gib => Some((1 << 30, false)),
        }
    }

    /// How many `base` this unit is (`Hours.weight_in(Minutes)` = 60), `None`
    /// for `Auto`, for another family, or when it is smaller than `base`.
    pub fn weight_in(self, base: DisplayUnit) -> Option<u64> {
        let (mine, family) = self.scale()?;
        let (theirs, base_family) = base.scale()?;
        (family == base_family && mine % theirs == 0).then_some(mine / theirs)
    }
}

/// A stored number as Settings reads it back: the count and a concrete unit
/// (never `Auto`). `base` is the unit the number is stored in, `choices` the
/// units the row offers (smallest first, `base` among them). The chosen unit
/// is used when it is offered and divides the number evenly, else the
/// largest offered unit that does (a `0` stays in the chosen or smallest
/// unit), so the pair always means exactly `total`.
pub fn read_quantity(
    total: u64,
    base: DisplayUnit,
    chosen: DisplayUnit,
    choices: &[DisplayUnit],
) -> (u64, DisplayUnit) {
    let weight = |u: DisplayUnit| u.weight_in(base).filter(|w| *w > 0);
    let fits = |u: DisplayUnit| choices.contains(&u) && weight(u).is_some_and(|w| total % w == 0);
    let unit = if fits(chosen) {
        chosen
    } else {
        choices
            .iter()
            .rev()
            .copied()
            .find(|u| total > 0 && fits(*u))
            .unwrap_or(base)
    };
    (total / weight(unit).unwrap_or(1), unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use DisplayUnit::*;

    const MINUTES: &[DisplayUnit] = &[Minutes, Hours];

    #[test]
    fn auto_shows_the_largest_unit_that_divides_evenly() {
        assert_eq!(read_quantity(60, Minutes, Auto, MINUTES), (1, Hours));
        assert_eq!(read_quantity(120, Minutes, Auto, MINUTES), (2, Hours));
        assert_eq!(read_quantity(90, Minutes, Auto, MINUTES), (90, Minutes));
        assert_eq!(read_quantity(0, Minutes, Auto, MINUTES), (0, Minutes));
    }

    #[test]
    fn a_fitting_chosen_unit_wins_and_a_wrong_one_is_ignored() {
        assert_eq!(read_quantity(60, Minutes, Minutes, MINUTES), (60, Minutes));
        assert_eq!(read_quantity(0, Minutes, Hours, MINUTES), (0, Hours));
        // Hand-edited: not a unit of this row, or not dividing the number.
        assert_eq!(read_quantity(90, Minutes, Hours, MINUTES), (90, Minutes));
        assert_eq!(read_quantity(60, Minutes, Mib, MINUTES), (1, Hours));
        assert_eq!(read_quantity(60, Minutes, Seconds, MINUTES), (1, Hours));
    }

    #[test]
    fn sizes_and_milliseconds_scale_from_their_base() {
        assert_eq!(Gib.weight_in(Mib), Some(1024));
        assert_eq!(Seconds.weight_in(Milliseconds), Some(1000));
        assert_eq!(Minutes.weight_in(Seconds), Some(60));
        assert_eq!(Seconds.weight_in(Minutes), None);
        assert_eq!(Mib.weight_in(Minutes), None);
        assert_eq!(
            read_quantity(5000, Milliseconds, Auto, &[Milliseconds, Seconds]),
            (5, Seconds)
        );
        assert_eq!(read_quantity(1024, Mib, Auto, &[Mib, Gib]), (1, Gib));
    }
}
