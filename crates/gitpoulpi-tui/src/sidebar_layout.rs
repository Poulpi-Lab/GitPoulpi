//! Heights of the Refs sidebar's sub-sections, and how the user resizes
//! them (draggable separators between LOCAL / REMOTE / ...).
//!
//! Pure arithmetic, no terminal: the event loop feeds it the section
//! contents and the room available and draws whatever comes back.
//!
//! **Heights are total heights, borders included.** A collapsed section is
//! [`COLLAPSED_HEIGHT`] (just its title bar); an expanded one is at least
//! [`MIN_SECTION_HEIGHT`] (title, one content row, bottom border).
//!
//! * A section the user never resized has an *automatic* height: exactly
//!   its content (`2 + rows`). When the sections together do not fit, the
//!   tallest automatic sections give rows back first, then the resized
//!   ones -- every section keeps its own scrollbar and scroll offset, so
//!   nothing becomes unreachable. When there is room to spare, automatic
//!   sections that were squeezed grow back up to their content.
//! * A section the user dragged has that height (a `0` means "not
//!   resized"). It is persisted in the session's layout
//!   (`PaneLayout::sidebar_heights`).

/// Title bar only.
pub(crate) const COLLAPSED_HEIGHT: u16 = 2;
/// Title, one content row, bottom border.
pub(crate) const MIN_SECTION_HEIGHT: u16 = 3;

/// What the layout needs to know about one visible section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SectionSpec {
    pub(crate) expanded: bool,
    /// Content rows the section would show if it had all the room it
    /// wanted (an empty section still shows its "(none)" line: 1).
    pub(crate) rows: u16,
    /// The user's dragged height, `0` when never resized.
    pub(crate) user: u16,
}

impl SectionSpec {
    fn min_height(&self) -> u16 {
        if self.expanded {
            MIN_SECTION_HEIGHT
        } else {
            COLLAPSED_HEIGHT
        }
    }

    fn content_height(&self) -> u16 {
        (2 + self.rows.max(1)).max(MIN_SECTION_HEIGHT)
    }
}

/// The total height of every section, in order, for `avail` rows.
pub(crate) fn section_heights(avail: u16, specs: &[SectionSpec]) -> Vec<u16> {
    let mut heights: Vec<u16> = specs
        .iter()
        .map(|s| {
            if !s.expanded {
                COLLAPSED_HEIGHT
            } else if s.user > 0 {
                s.user.max(MIN_SECTION_HEIGHT)
            } else {
                s.content_height()
            }
        })
        .collect();

    // Too tall: take a row at a time from the tallest section that can
    // still give one, automatic ones before resized ones, later ones
    // before earlier ones on a tie.
    let total = |h: &[u16]| h.iter().map(|&x| x as u32).sum::<u32>();
    for user_pass in [false, true] {
        while total(&heights) > avail as u32 {
            let victim = specs
                .iter()
                .enumerate()
                .filter(|(i, s)| {
                    s.expanded && (s.user > 0) == user_pass && heights[*i] > s.min_height()
                })
                .max_by_key(|(i, _)| (heights[*i], *i))
                .map(|(i, _)| i);
            match victim {
                Some(i) => heights[i] -= 1,
                None => break,
            }
        }
    }

    // Room to spare: let squeezed automatic sections grow back to their
    // content, top to bottom.
    let mut spare = (avail as u32).saturating_sub(total(&heights));
    for (i, s) in specs.iter().enumerate() {
        if spare == 0 {
            break;
        }
        if s.expanded && s.user == 0 && heights[i] < s.content_height() {
            let give = (s.content_height() - heights[i]).min(spare.min(u16::MAX as u32) as u16);
            heights[i] += give;
            spare -= give as u32;
        }
    }
    heights
}

/// The most height section `index` can be dragged to: everything the
/// other sections do not need at their minimum.
pub(crate) fn max_section_height(avail: u16, specs: &[SectionSpec], index: usize) -> u16 {
    let others: u16 = specs
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, s)| s.min_height())
        .sum();
    avail.saturating_sub(others).max(MIN_SECTION_HEIGHT)
}

/// Clamps a requested total height for section `index`.
pub(crate) fn clamp_section_height(
    requested: u16,
    avail: u16,
    specs: &[SectionSpec],
    index: usize,
) -> u16 {
    requested.clamp(MIN_SECTION_HEIGHT, max_section_height(avail, specs, index))
}

/// The persisted override for the section at `index` (`0` = none).
pub(crate) fn stored_height(stored: &[u16], index: usize) -> u16 {
    stored.get(index).copied().unwrap_or(0)
}

/// Sets one override, growing the vector as needed.
pub(crate) fn store_height(stored: &mut Vec<u16>, index: usize, height: u16) {
    if stored.len() <= index {
        stored.resize(index + 1, 0);
    }
    stored[index] = height;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auto(rows: u16) -> SectionSpec {
        SectionSpec {
            expanded: true,
            rows,
            user: 0,
        }
    }

    #[test]
    fn automatic_sections_take_exactly_their_content_when_there_is_room() {
        assert_eq!(section_heights(40, &[auto(3), auto(5)]), vec![5, 7]);
    }

    #[test]
    fn an_empty_section_still_gets_room_for_its_none_line() {
        assert_eq!(section_heights(40, &[auto(0)]), vec![3]);
    }

    #[test]
    fn a_collapsed_section_is_its_title_bar() {
        let collapsed = SectionSpec {
            expanded: false,
            rows: 9,
            user: 0,
        };
        assert_eq!(section_heights(40, &[collapsed, auto(2)]), vec![2, 4]);
    }

    #[test]
    fn when_it_does_not_fit_the_tallest_automatic_section_gives_rows_back_first() {
        let h = section_heights(16, &[auto(10), auto(4), auto(2)]);
        assert_eq!(h.iter().sum::<u16>(), 16);
        assert_eq!(h, vec![6, 6, 4]);
    }

    #[test]
    fn a_dragged_section_keeps_its_height_while_automatic_ones_shrink() {
        let dragged = SectionSpec {
            expanded: true,
            rows: 30,
            user: 10,
        };
        assert_eq!(section_heights(16, &[dragged, auto(10)]), vec![10, 6]);
    }

    #[test]
    fn a_dragged_section_shrinks_only_after_every_automatic_one_is_at_its_floor() {
        let dragged = SectionSpec {
            expanded: true,
            rows: 30,
            user: 20,
        };
        assert_eq!(section_heights(10, &[dragged, auto(10)]), vec![7, 3]);
    }

    #[test]
    fn a_shrunk_section_leaves_its_neighbor_whole_when_there_is_room() {
        let small = SectionSpec {
            expanded: true,
            rows: 30,
            user: 4,
        };
        assert_eq!(section_heights(20, &[small, auto(10)]), vec![4, 12]);
    }

    #[test]
    fn a_drag_is_clamped_between_the_floor_and_what_the_others_leave() {
        let specs = [auto(9), auto(9), auto(9)];
        assert_eq!(clamp_section_height(1, 30, &specs, 0), MIN_SECTION_HEIGHT);
        assert_eq!(clamp_section_height(99, 30, &specs, 0), 30 - 3 - 3);
    }

    #[test]
    fn stored_heights_grow_on_demand_and_default_to_zero() {
        let mut v = Vec::new();
        assert_eq!(stored_height(&v, 4), 0);
        store_height(&mut v, 2, 7);
        assert_eq!(v, vec![0, 0, 7]);
        assert_eq!(stored_height(&v, 2), 7);
    }
}
