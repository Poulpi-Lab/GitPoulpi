//! What of a conversation fits the request: the byte budget of
//! `ai.limits.hosting_reply_context_bytes` split between the item's
//! description, the thread being answered and the recent entries.
//!
//! Pure arithmetic over sizes, so it is unit tested without a conversation.
//! The rules, in priority order:
//!
//! 1. the thread being answered is always there, up to half the budget (its
//!    start is left out, the newest comments stay);
//! 2. the item's description gets up to a quarter;
//! 3. what remains goes to the other entries, **newest first**: an entry that
//!    does not fit is left out and the next older one is tried, so one huge
//!    log comment does not hide everything before it.
//!
//! Nothing here cuts silently: [`Plan`] says what was left out and the
//! caller turns it into [`crate::Cut`] entries.

/// How the budget is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    /// Bytes the description may use.
    pub description: usize,
    /// Bytes the thread being answered may use.
    pub thread: usize,
    /// Bytes left for the other entries.
    pub recent: usize,
}

/// Shares `budget` between a description of `description_len` bytes and a
/// thread of `thread_len` bytes; what they do not use flows to the recent
/// entries. `usize::MAX` (the caps are lifted) gives every part its whole size.
pub fn split_budget(budget: usize, description_len: usize, thread_len: usize) -> Split {
    if budget == usize::MAX {
        return Split {
            description: usize::MAX,
            thread: usize::MAX,
            recent: usize::MAX,
        };
    }
    let thread = thread_len.min(budget / 2);
    let description = description_len.min(budget / 4);
    Split {
        description,
        thread,
        recent: budget - thread - description,
    }
}

/// Which of the other entries are sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Indexes into the candidates, oldest first.
    pub kept: Vec<usize>,
    /// How many candidates were left out.
    pub omitted: usize,
    /// The newest entry alone is bigger than the room: it is sent cut to
    /// this many bytes (its end is left out) instead of not at all.
    pub newest_cap: Option<usize>,
}

/// Chooses among `sizes` (the rendered size of each candidate, oldest
/// first) what fits `room` bytes, newest first, skipping an entry that does
/// not fit. When the newest alone is bigger than `room`, it is the only one
/// kept, capped to `room`.
pub fn choose_recent(sizes: &[usize], room: usize) -> Plan {
    // The newest entry is what is being answered most often: it is never
    // dropped for being big, it is cut instead.
    if let Some(last) = sizes.len().checked_sub(1) {
        if sizes[last] > room && room > 0 {
            return Plan {
                kept: vec![last],
                omitted: last,
                newest_cap: Some(room),
            };
        }
    }
    let mut kept = Vec::new();
    let mut used = 0usize;
    for (index, size) in sizes.iter().enumerate().rev() {
        if used.saturating_add(*size) <= room {
            used += size;
            kept.push(index);
        }
    }
    kept.reverse();
    Plan {
        omitted: sizes.len() - kept.len(),
        kept,
        newest_cap: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_is_shared_thread_first_then_description_then_the_rest() {
        let s = split_budget(1000, 5000, 5000);
        assert_eq!(s.thread, 500, "the thread gets up to half");
        assert_eq!(s.description, 250, "the description up to a quarter");
        assert_eq!(s.recent, 250);
        assert_eq!(s.thread + s.description + s.recent, 1000);
    }

    #[test]
    fn what_a_part_does_not_use_flows_to_the_recent_entries() {
        let s = split_budget(1000, 40, 0);
        assert_eq!((s.description, s.thread, s.recent), (40, 0, 960));
    }

    #[test]
    fn lifted_caps_give_every_part_its_whole_size() {
        let s = split_budget(usize::MAX, 9, 9);
        assert_eq!(s.description, usize::MAX);
        assert_eq!(s.recent, usize::MAX);
    }

    #[test]
    fn the_newest_entries_are_kept_first() {
        let p = choose_recent(&[10, 10, 10, 10], 25);
        assert_eq!(p.kept, [2, 3], "the two newest fit");
        assert_eq!(p.omitted, 2);
        assert_eq!(p.newest_cap, None);
    }

    #[test]
    fn a_huge_entry_is_skipped_and_older_ones_still_fit() {
        let p = choose_recent(&[10, 10_000, 10], 25);
        assert_eq!(p.kept, [0, 2]);
        assert_eq!(p.omitted, 1);
    }

    #[test]
    fn everything_fits_when_there_is_room() {
        let p = choose_recent(&[10, 20, 30], 60);
        assert_eq!(p.kept, [0, 1, 2]);
        assert_eq!(p.omitted, 0);
    }

    #[test]
    fn a_newest_entry_bigger_than_the_room_is_kept_cut_not_dropped() {
        let p = choose_recent(&[5, 900], 100);
        assert_eq!(p.kept, [1], "the newest wins over an older small entry");
        assert_eq!(p.newest_cap, Some(100));
        assert_eq!(p.omitted, 1);
        let p = choose_recent(&[900], 100);
        assert_eq!(p.kept, [0]);
        assert_eq!(p.newest_cap, Some(100));
        assert_eq!(p.omitted, 0);
    }

    #[test]
    fn nothing_to_choose_is_nothing_kept() {
        let p = choose_recent(&[], 100);
        assert_eq!((p.kept.len(), p.omitted, p.newest_cap), (0, 0, None));
        let p = choose_recent(&[10], 0);
        assert_eq!((p.kept.len(), p.omitted), (0, 1));
    }
}
