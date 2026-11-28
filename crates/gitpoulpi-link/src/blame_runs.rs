//! A blame result as runs: consecutive lines one revision last touched, with
//! an age heat per run. Computed once per result by a frontend, which adds its
//! own per-author colour (the runs name no colours).

use gitpoulpi_core::domain::{BlameLine, Oid};

/// A run of consecutive lines attributed to one revision.
#[derive(Debug, Clone, PartialEq)]
pub struct BlameRun {
    /// First line (0-based) and one past the last.
    pub start: usize,
    pub end: usize,
    pub commit: Oid,
    /// The lines are not saved in history yet (the working tree's own edits).
    pub uncommitted: bool,
    /// 0.0 for the file's oldest line, 1.0 for its newest (0.5 when the file
    /// has a single age or the run is uncommitted).
    pub heat: f32,
}

/// The runs of `lines` and, for every line, the index of its run.
pub fn build(lines: &[BlameLine]) -> (Vec<BlameRun>, Vec<u32>) {
    let mut runs: Vec<BlameRun> = Vec::new();
    let mut run_of = Vec::with_capacity(lines.len());
    let (mut oldest, mut newest) = (i64::MAX, i64::MIN);
    for (i, line) in lines.iter().enumerate() {
        let uncommitted = crate::wip_view::is_uncommitted(line.commit);
        if !uncommitted {
            oldest = oldest.min(line.timestamp);
            newest = newest.max(line.timestamp);
        }
        match runs.last_mut() {
            Some(last) if last.commit == line.commit => last.end = i + 1,
            _ => runs.push(BlameRun {
                start: i,
                end: i + 1,
                commit: line.commit,
                uncommitted,
                heat: 0.5,
            }),
        }
        run_of.push((runs.len() - 1) as u32);
    }
    if newest > oldest {
        let span = (newest - oldest) as f32;
        for r in &mut runs {
            if !r.uncommitted {
                r.heat = (lines[r.start].timestamp - oldest) as f32 / span;
            }
        }
    }
    (runs, run_of)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(rev: u8, timestamp: i64) -> BlameLine {
        BlameLine {
            commit: Oid([rev; 20]),
            author_name: "a".into(),
            author_email: "a@example.com".into(),
            timestamp,
            committer_time: 0,
            summary: String::new(),
            final_line: 0,
            orig_line: 0,
            orig_path: Default::default(),
            previous: None,
            content: String::new(),
        }
    }

    #[test]
    fn consecutive_lines_of_a_revision_form_a_run_with_heat() {
        let (runs, of) = build(&[line(1, 100), line(1, 100), line(2, 200)]);
        assert_eq!(of, [0, 0, 1]);
        assert_eq!((runs[0].start, runs[0].end), (0, 2));
        assert_eq!(runs[0].heat, 0.0);
        assert_eq!(runs[1].heat, 1.0);
    }
}
