//! Date rules: shift, time zone, and "smart hours" (`window`). Pure
//! functions; the clock is never read (the seed and the inputs are the whole
//! state), so a preview and the run that follows agree.
//!
//! Git stores `<unix seconds> <+hhmm>`: an instant plus the author's own
//! offset. There are no IANA zones here, only fixed offsets.

use serde::{Deserialize, Serialize};

/// Which date(s) of a commit a rule moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhichDate {
    AuthorDate,
    CommitterDate,
    #[default]
    Both,
}

/// An instant with the offset it was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitDate {
    pub seconds: i64,
    /// Minutes east of UTC (`+0200` is 120).
    pub offset_minutes: i16,
}

impl GitDate {
    pub fn new(seconds: i64, offset_minutes: i16) -> Self {
        Self {
            seconds,
            offset_minutes,
        }
    }

    /// The wall-clock reading as seconds since the epoch, as if it were UTC.
    pub fn local_seconds(self) -> i64 {
        self.seconds + i64::from(self.offset_minutes) * 60
    }

    /// Git's `+hhmm` text for the offset.
    pub fn offset_text(self) -> String {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let abs = self.offset_minutes.unsigned_abs();
        format!("{sign}{:02}{:02}", abs / 60, abs % 60)
    }
}

/// What "set time zone" keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeepClock {
    /// Same moment, new zone: the instant stays, the wall clock changes.
    Instant,
    /// Same local time, new zone: the wall clock stays, the instant changes.
    WallClock,
}

/// Moves a date by `by_seconds` (negative: earlier), keeping its offset.
pub fn shift(date: GitDate, by_seconds: i64) -> GitDate {
    GitDate {
        seconds: date.seconds.saturating_add(by_seconds),
        offset_minutes: date.offset_minutes,
    }
}

/// Puts a date in another zone.
pub fn set_offset(date: GitDate, offset_minutes: i16, keep: KeepClock) -> GitDate {
    match keep {
        KeepClock::Instant => GitDate {
            seconds: date.seconds,
            offset_minutes,
        },
        KeepClock::WallClock => GitDate {
            seconds: date.local_seconds() - i64::from(offset_minutes) * 60,
            offset_minutes,
        },
    }
}

/// The largest offset accepted, in minutes (git stores `+hhmm`; real zones
/// stay within -12:00 .. +14:00, this leaves room for odd data).
pub const MAX_OFFSET_MINUTES: i16 = 1439;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    /// 0 for Monday.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Mon-Fri.
    pub const WEEKDAYS: [Weekday; 5] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ];

    /// Sat-Sun.
    pub const WEEKEND: [Weekday; 2] = [Weekday::Sat, Weekday::Sun];
}

/// Allowed hours on some weekdays: `start_minute..end_minute` of the day
/// (0..=1440, end exclusive).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekWindow {
    pub days: Vec<Weekday>,
    pub start_minute: u16,
    pub end_minute: u16,
}

impl WeekWindow {
    pub fn new(days: &[Weekday], start_minute: u16, end_minute: u16) -> Self {
        Self {
            days: days.to_vec(),
            start_minute,
            end_minute,
        }
    }
}

/// Where a commit outside every window goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Nearest,
    LaterOnly,
    EarlierOnly,
}

/// The zone the windows are read in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "zone", rename_all = "snake_case")]
pub enum WindowOffset {
    /// Each commit's own offset.
    #[default]
    Commit,
    Fixed {
        minutes: i16,
    },
}

/// "Smart hours": commits fall only within the given hours.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkWindowRule {
    pub windows: Vec<WeekWindow>,
    #[serde(default)]
    pub offset: WindowOffset,
    #[serde(default)]
    pub direction: Direction,
    /// A commit is never earlier than its parents (default on).
    #[serde(default = "yes")]
    pub keep_order: bool,
    /// Seconds a commit lands after a parent that was moved.
    #[serde(default = "default_gap")]
    pub min_gap_seconds: u32,
    /// Seeds the jitter: the same seed gives the same result.
    #[serde(default)]
    pub seed: u32,
    #[serde(default)]
    pub apply_to: WhichDate,
}

fn yes() -> bool {
    true
}

fn default_gap() -> u32 {
    60
}

impl WorkWindowRule {
    /// Mon-Fri evenings and weekend days, a typical "side project" rhythm.
    pub fn evenings_and_weekends() -> Self {
        Self {
            windows: vec![
                WeekWindow::new(&Weekday::WEEKDAYS, 18 * 60, 23 * 60 + 30),
                WeekWindow::new(&Weekday::WEEKEND, 10 * 60, 20 * 60),
            ],
            offset: WindowOffset::Commit,
            direction: Direction::Nearest,
            keep_order: true,
            min_gap_seconds: default_gap(),
            seed: 0,
            apply_to: WhichDate::Both,
        }
    }
}

/// Why `rule` is unusable.
pub fn window_problem(rule: &WorkWindowRule) -> Option<String> {
    if rule.windows.is_empty() {
        return Some("no allowed hours are set".to_string());
    }
    for w in &rule.windows {
        if w.days.is_empty() {
            return Some("a window has no weekday".to_string());
        }
        if w.start_minute >= w.end_minute || w.end_minute > 1440 {
            return Some("a window must start before it ends, within one day".to_string());
        }
    }
    if let WindowOffset::Fixed { minutes } = rule.offset {
        if minutes.abs() > MAX_OFFSET_MINUTES {
            return Some("the fixed offset is out of range".to_string());
        }
    }
    None
}

/// One commit as `window` sees it. `parents` are indices of earlier
/// entries; a parent outside the walk is simply not listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInput {
    pub parents: Vec<usize>,
    pub author: GitDate,
    pub committer: GitDate,
}

/// The dates `window` chose for one commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewTimes {
    pub author: GitDate,
    pub committer: GitDate,
}

const HORIZON_DAYS: i64 = 8;
const DAY: i64 = 86_400;
/// The first and last hour of a window absorb the commits moved into it.
const SPREAD_SECONDS: i64 = 3600;
const JITTER_SECONDS: u64 = 7 * 60;

/// The allowed intervals as seconds of the wall clock, built per call
/// around a local time.
struct Frame<'a> {
    rule: &'a WorkWindowRule,
}

impl Frame<'_> {
    /// Allowed intervals `[start, end)` in local seconds, sorted by start,
    /// covering the days around `local`.
    fn intervals(&self, local: i64) -> Vec<(i64, i64)> {
        let day0 = local.div_euclid(DAY);
        let mut out = Vec::new();
        for day in day0 - HORIZON_DAYS..=day0 + HORIZON_DAYS {
            // 1970-01-01 was a Thursday.
            let weekday = (day + 3).rem_euclid(7) as usize;
            for w in &self.rule.windows {
                if w.days.iter().any(|d| d.index() == weekday) {
                    out.push((
                        day * DAY + i64::from(w.start_minute) * 60,
                        day * DAY + i64::from(w.end_minute) * 60,
                    ));
                }
            }
        }
        out.sort_unstable();
        out
    }

    fn inside(&self, local: i64) -> bool {
        self.intervals(local)
            .iter()
            .any(|(s, e)| *s <= local && local < *e)
    }

    /// The first interval that starts after `local`.
    fn next(&self, local: i64) -> Option<(i64, i64)> {
        self.intervals(local).into_iter().find(|(s, _)| *s > local)
    }

    /// The last interval that ended at or before `local`.
    fn previous(&self, local: i64) -> Option<(i64, i64)> {
        self.intervals(local)
            .into_iter()
            .rev()
            .find(|(_, e)| *e <= local)
    }
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn offset_for(rule: &WorkWindowRule, date: GitDate) -> i64 {
    match rule.offset {
        WindowOffset::Commit => i64::from(date.offset_minutes) * 60,
        WindowOffset::Fixed { minutes } => i64::from(minutes) * 60,
    }
}

/// The new instant for one date. `floor` is the earliest allowed instant
/// (`i64::MIN` for none).
fn place(rule: &WorkWindowRule, date: GitDate, floor: i64, index: usize) -> i64 {
    let frame = Frame { rule };
    let off = offset_for(rule, date);
    let local = date.seconds + off;
    let mut t = local;

    if !frame.inside(local) {
        let later = frame.next(local);
        let earlier = frame.previous(local).map(|(s, e)| (s, e - 1));
        // Which side to land on: (boundary instant, window start, window end, at_start)
        let choice = match rule.direction {
            Direction::LaterOnly => later.map(|(s, e)| (s, s, e, true)),
            Direction::EarlierOnly => earlier.map(|(s, e)| (e, s, e + 1, false)),
            Direction::Nearest => match (later, earlier) {
                (Some((s, e)), Some((ps, pe))) => {
                    if s - local <= local - pe {
                        Some((s, s, e, true))
                    } else {
                        Some((pe, ps, pe + 1, false))
                    }
                }
                (Some((s, e)), None) => Some((s, s, e, true)),
                (None, Some((ps, pe))) => Some((pe, ps, pe + 1, false)),
                (None, None) => None,
            },
        }
        // The other direction when the chosen one has nothing in range.
        .or_else(|| later.map(|(s, e)| (s, s, e, true)))
        .or_else(|| earlier.map(|(s, e)| (e, s, e + 1, false)));
        if let Some((boundary, start, end, at_start)) = choice {
            let len = end - start;
            let span = SPREAD_SECONDS.min(len).max(1);
            let spread = (boundary - local).abs() % span;
            let jitter =
                (splitmix(u64::from(rule.seed) ^ (date.seconds as u64) ^ ((index as u64) << 40))
                    % (JITTER_SECONDS + 1)) as i64;
            let inward = (spread + jitter).min(len - 1);
            t = if at_start {
                start + inward
            } else {
                (end - 1) - inward
            };
        }
    }

    if floor != i64::MIN && t - off < floor {
        t = floor + off;
        if !frame.inside(t) {
            if let Some((s, _)) = frame.next(t) {
                t = s;
            }
        }
    }
    t - off
}

/// Smart hours. `commits` is in topological order, parents first (a parent
/// index must be smaller than the commit's own; others are ignored).
///
/// For each commit: inside a window the time is kept; outside it moves to
/// the nearest allowed boundary in the rule's direction, lands within the
/// first (or last) hour of that window with a seeded jitter of up to seven
/// minutes, and is then pushed after its parents (plus the minimum gap when
/// a parent moved) so order is kept. Committer dates keep their distance to
/// the author date when that still falls in a window, else equal the author
/// date. `apply_to` limits what moves: only author dates, or only committer
/// dates (then the committer date is the one placed and the author date is
/// left alone).
///
/// Properties (tested): every moved date is in a window, never earlier than
/// a parent (with `keep_order`), the same input and seed give the same
/// output, and a date already inside a window that no parent pushes is
/// unchanged.
pub fn window(commits: &[WindowInput], rule: &WorkWindowRule) -> Vec<NewTimes> {
    let frame = Frame { rule };
    let mut out: Vec<NewTimes> = Vec::with_capacity(commits.len());
    let committer_only = rule.apply_to == WhichDate::CommitterDate;
    for (i, c) in commits.iter().enumerate() {
        // The date that is placed, and the others follow from it.
        let driven = if committer_only {
            c.committer
        } else {
            c.author
        };
        let floor = if rule.keep_order {
            c.parents
                .iter()
                .filter(|p| **p < i)
                .map(|p| {
                    let parent = &commits[*p];
                    let (old, new) = if committer_only {
                        (parent.committer.seconds, out[*p].committer.seconds)
                    } else {
                        (parent.author.seconds, out[*p].author.seconds)
                    };
                    if new != old {
                        new.saturating_add(i64::from(rule.min_gap_seconds))
                    } else {
                        new
                    }
                })
                .max()
                .unwrap_or(i64::MIN)
        } else {
            i64::MIN
        };
        let placed = GitDate {
            seconds: place(rule, driven, floor, i),
            offset_minutes: driven.offset_minutes,
        };
        let times = match rule.apply_to {
            WhichDate::AuthorDate => NewTimes {
                author: placed,
                committer: c.committer,
            },
            WhichDate::CommitterDate => NewTimes {
                author: c.author,
                committer: placed,
            },
            WhichDate::Both => {
                let kept = c.committer.seconds + (placed.seconds - c.author.seconds);
                let local = kept + offset_for(rule, c.committer);
                let seconds = if frame.inside(local) {
                    kept
                } else {
                    placed.seconds
                };
                NewTimes {
                    author: placed,
                    committer: GitDate {
                        seconds,
                        offset_minutes: c.committer.offset_minutes,
                    },
                }
            }
        };
        out.push(times);
    }
    out
}

/// `true` when `date` falls inside one of the rule's windows.
pub fn in_window(date: GitDate, rule: &WorkWindowRule) -> bool {
    Frame { rule }.inside(date.seconds + offset_for(rule, date))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2024-03-04 is a Monday.
    const MON_MIDNIGHT: i64 = 1_709_510_400;
    const H: i64 = 3600;

    fn utc(seconds: i64) -> GitDate {
        GitDate::new(seconds, 0)
    }

    #[test]
    fn shift_keeps_the_offset_and_goes_both_ways() {
        let d = GitDate::new(1_000_000, 120);
        assert_eq!(shift(d, 3600), GitDate::new(1_003_600, 120));
        assert_eq!(shift(d, -1_000_000), GitDate::new(0, 120));
        assert_eq!(shift(GitDate::new(i64::MAX, 0), 5).seconds, i64::MAX);
    }

    #[test]
    fn same_moment_new_zone_keeps_the_instant() {
        let d = GitDate::new(MON_MIDNIGHT + 10 * H, 0);
        let z = set_offset(d, 120, KeepClock::Instant);
        assert_eq!(z.seconds, d.seconds);
        assert_eq!(z.local_seconds() - d.local_seconds(), 2 * H);
    }

    #[test]
    fn same_local_time_new_zone_keeps_the_wall_clock() {
        let d = GitDate::new(MON_MIDNIGHT + 10 * H, 60);
        let z = set_offset(d, -300, KeepClock::WallClock);
        assert_eq!(z.local_seconds(), d.local_seconds());
        assert_eq!(z.offset_minutes, -300);
        assert_eq!(z.seconds - d.seconds, 6 * H);
    }

    #[test]
    fn offset_text_is_git_style() {
        assert_eq!(GitDate::new(0, 120).offset_text(), "+0200");
        assert_eq!(GitDate::new(0, -330).offset_text(), "-0530");
        assert_eq!(GitDate::new(0, 0).offset_text(), "+0000");
    }

    fn rule() -> WorkWindowRule {
        WorkWindowRule::evenings_and_weekends()
    }

    #[test]
    fn a_date_inside_a_window_is_kept() {
        let inside = utc(MON_MIDNIGHT + 20 * H);
        let out = window(
            &[WindowInput {
                parents: vec![],
                author: inside,
                committer: inside,
            }],
            &rule(),
        );
        assert_eq!(out[0].author, inside);
        assert_eq!(out[0].committer, inside);
    }

    #[test]
    fn a_morning_commit_moves_to_that_evening() {
        // Monday 09:00 -> nearest allowed: 18:00 the same day (9 h away),
        // not Sunday 20:00 (13 h away).
        let morning = utc(MON_MIDNIGHT + 9 * H);
        let out = window(
            &[WindowInput {
                parents: vec![],
                author: morning,
                committer: morning,
            }],
            &rule(),
        );
        let moved = out[0].author.seconds;
        assert!((MON_MIDNIGHT + 18 * H..MON_MIDNIGHT + 19 * H + 600).contains(&moved));
        assert!(in_window(out[0].author, &rule()));
    }

    #[test]
    fn directions_pick_the_side() {
        let morning = utc(MON_MIDNIGHT + 9 * H);
        let input = [WindowInput {
            parents: vec![],
            author: morning,
            committer: morning,
        }];
        let mut r = rule();
        r.direction = Direction::EarlierOnly;
        let earlier = window(&input, &r)[0].author.seconds;
        // Sunday evening, in the last hour of the 10:00-20:00 window.
        assert!(earlier < MON_MIDNIGHT && earlier > MON_MIDNIGHT - 5 * H);
        r.direction = Direction::LaterOnly;
        let later = window(&input, &r)[0].author.seconds;
        assert!(later >= MON_MIDNIGHT + 18 * H);
    }

    #[test]
    fn windows_are_read_in_the_commits_own_zone_or_a_fixed_one() {
        // 18:30 at +0200 is 16:30 UTC. Inside with the commit's offset...
        let d = GitDate::new(MON_MIDNIGHT + 16 * H + 1800, 120);
        assert!(in_window(d, &rule()));
        // ...outside when the windows are read in UTC.
        let mut utc_rule = rule();
        utc_rule.offset = WindowOffset::Fixed { minutes: 0 };
        assert!(!in_window(d, &utc_rule));
    }

    /// A tiny deterministic generator for the property tests.
    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state >> 33
    }

    fn history(n: usize, seed: u64, offset: i16) -> Vec<WindowInput> {
        let mut s = seed;
        let mut t = MON_MIDNIGHT;
        (0..n)
            .map(|i| {
                t += (lcg(&mut s) % (30 * H as u64)) as i64;
                let a = GitDate::new(t, offset);
                WindowInput {
                    parents: if i == 0 { vec![] } else { vec![i - 1] },
                    author: a,
                    committer: GitDate::new(t + (lcg(&mut s) % 600) as i64, offset),
                }
            })
            .collect()
    }

    #[test]
    fn every_output_is_in_a_window_and_never_before_its_parent() {
        for (seed, offset) in [(1u64, 0i16), (7, 120), (42, -300)] {
            let input = history(300, seed, offset);
            let r = rule();
            let out = window(&input, &r);
            for (i, o) in out.iter().enumerate() {
                assert!(in_window(o.author, &r), "author {i} seed {seed}");
                if i > 0 {
                    assert!(
                        o.author.seconds >= out[i - 1].author.seconds,
                        "order at {i} seed {seed}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_same_input_and_seed_give_the_same_output_and_another_seed_differs() {
        let input = history(120, 9, 60);
        let mut r = rule();
        r.seed = 5;
        assert_eq!(window(&input, &r), window(&input, &r));
        let mut other = r.clone();
        other.seed = 6;
        assert_ne!(window(&input, &r), window(&input, &other));
    }

    #[test]
    fn dates_already_in_a_window_and_not_pushed_are_unchanged() {
        // Evening commits an hour apart, all inside Monday 18:00-23:30.
        let input: Vec<WindowInput> = (0..4)
            .map(|i| {
                let d = utc(MON_MIDNIGHT + 18 * H + i * H);
                WindowInput {
                    parents: if i == 0 { vec![] } else { vec![i as usize - 1] },
                    author: d,
                    committer: d,
                }
            })
            .collect();
        let out = window(&input, &rule());
        for (i, o) in out.iter().enumerate() {
            assert_eq!(o.author, input[i].author);
            assert_eq!(o.committer, input[i].committer);
        }
    }

    #[test]
    fn a_child_lands_after_a_parent_that_was_pushed_later() {
        // Parent at 09:00 moves to ~18:00; the child at 09:05 must follow it
        // by at least the gap, even though its own nearest slot is earlier.
        let input = [
            WindowInput {
                parents: vec![],
                author: utc(MON_MIDNIGHT + 9 * H),
                committer: utc(MON_MIDNIGHT + 9 * H),
            },
            WindowInput {
                parents: vec![0],
                author: utc(MON_MIDNIGHT + 9 * H + 300),
                committer: utc(MON_MIDNIGHT + 9 * H + 300),
            },
        ];
        let out = window(&input, &rule());
        assert!(out[1].author.seconds >= out[0].author.seconds + 60);
        assert!(in_window(out[1].author, &rule()));
    }

    #[test]
    fn a_merge_follows_its_latest_parent() {
        let input = [
            WindowInput {
                parents: vec![],
                author: utc(MON_MIDNIGHT + 19 * H),
                committer: utc(MON_MIDNIGHT + 19 * H),
            },
            WindowInput {
                parents: vec![],
                author: utc(MON_MIDNIGHT + 22 * H),
                committer: utc(MON_MIDNIGHT + 22 * H),
            },
            WindowInput {
                parents: vec![0, 1],
                author: utc(MON_MIDNIGHT + 20 * H),
                committer: utc(MON_MIDNIGHT + 20 * H),
            },
        ];
        let out = window(&input, &rule());
        assert!(out[2].author.seconds >= out[1].author.seconds);
    }

    #[test]
    fn apply_to_limits_what_moves() {
        let morning = utc(MON_MIDNIGHT + 9 * H);
        let input = [WindowInput {
            parents: vec![],
            author: morning,
            committer: morning,
        }];
        let mut r = rule();
        r.apply_to = WhichDate::AuthorDate;
        let o = window(&input, &r)[0];
        assert_ne!(o.author, morning);
        assert_eq!(o.committer, morning);
        r.apply_to = WhichDate::CommitterDate;
        let o = window(&input, &r)[0];
        assert_eq!(o.author, morning);
        assert!(in_window(o.committer, &r));
    }

    #[test]
    fn committer_keeps_its_distance_when_it_still_fits() {
        let a = utc(MON_MIDNIGHT + 9 * H);
        let input = [WindowInput {
            parents: vec![],
            author: a,
            committer: utc(a.seconds + 120),
        }];
        let o = window(&input, &rule())[0];
        // The 2 minute distance still fits inside the window (the author
        // lands in the first hour, not at its last second).
        assert_eq!(o.committer.seconds - o.author.seconds, 120);
    }

    #[test]
    fn unusable_rules_are_described() {
        let mut r = rule();
        assert_eq!(window_problem(&r), None);
        r.windows.clear();
        assert!(window_problem(&r).is_some());
        let mut r = rule();
        r.windows[0].end_minute = r.windows[0].start_minute;
        assert!(window_problem(&r).is_some());
        let mut r = rule();
        r.windows[0].end_minute = 1441;
        assert!(window_problem(&r).is_some());
        let mut r = rule();
        r.windows[0].days.clear();
        assert!(window_problem(&r).is_some());
        let mut r = rule();
        r.offset = WindowOffset::Fixed { minutes: 3000 };
        assert!(window_problem(&r).is_some());
    }
}
