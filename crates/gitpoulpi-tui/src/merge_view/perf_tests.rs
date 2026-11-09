//! Timing probes for the merge editor on big files (slice SP), and the
//! equivalences the caches must keep. The probes are `#[ignore]`d:
//! `cargo test --release -p gitpoulpi-tui --lib merge_view::perf_tests -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{
    BlobContent, ConflictEntry, ConflictSide, Eol, MergeDocument, MergeRegion, Oid, SideBlob,
    StageEntry, WorktreeFileState,
};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use crate::clipboard::Clipboard;

use super::model::MergeModel;
use super::MergeView;

fn stage(n: u8) -> Option<StageEntry> {
    Some(StageEntry {
        mode: 0o100644,
        oid: Oid([n; 20]),
    })
}

fn blob(n: u8, content: Vec<String>) -> Option<SideBlob> {
    Some(SideBlob {
        oid: Oid([n; 20]),
        mode: 0o100644,
        size: 0,
        content: BlobContent::Text {
            lines: content,
            crlf: false,
            final_newline: true,
        },
    })
}

/// A Rust-looking file of `lines` lines with `conflicts` three-line
/// conflicts spread evenly.
pub(super) fn big_document(lines: usize, conflicts: usize) -> MergeDocument {
    let line = |i: usize| {
        if i.is_multiple_of(25) {
            format!("pub fn step_{i}(input: &[u32], offset: u32) -> Option<u32> {{")
        } else if i % 25 == 24 {
            "}".to_string()
        } else {
            format!("    let value_{i} = compute(input[{i} % 7], {i}) + offset; // step {i}")
        }
    };
    let mut regions = Vec::new();
    let mut at = 0;
    for c in 0..conflicts {
        let p = (c * 2 + 1) * lines / (conflicts * 2);
        regions.push(MergeRegion::Merged {
            lines: (at..p).map(line).collect(),
        });
        let mk = |tag: &str| -> Vec<String> {
            (0..3)
                .map(|k| format!("    let {tag}_{c}_{k} = compute(input[{k}], {c}); // {tag}"))
                .collect()
        };
        regions.push(MergeRegion::Conflict {
            base: (p..p + 3).map(line).collect(),
            ours: mk("ours"),
            theirs: mk("theirs"),
        });
        at = p + 3;
    }
    regions.push(MergeRegion::Merged {
        lines: (at..lines).map(line).collect(),
    });
    let projection = |side: ConflictSide| -> Vec<String> {
        regions
            .iter()
            .flat_map(|r| r.lines_of(side).iter().cloned())
            .collect()
    };
    let (base, ours, theirs) = (
        projection(ConflictSide::Base),
        projection(ConflictSide::Ours),
        projection(ConflictSide::Theirs),
    );
    MergeDocument {
        entry: ConflictEntry::from_stages("src/big.rs".into(), stage(1), stage(2), stage(3))
            .unwrap(),
        base: blob(1, base),
        ours: blob(2, ours),
        theirs: blob(3, theirs),
        regions,
        labels: Default::default(),
        worktree: WorktreeFileState::Markers,
        eol: Eol::Lf,
        bom: false,
        final_newline: true,
        encoding: None,
        marker_size: 7,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

struct Rig {
    view: MergeView,
    term: Terminal<TestBackend>,
    clip: Clipboard,
}

impl Rig {
    fn new(lines: usize, conflicts: usize) -> Rig {
        let t = Instant::now();
        let doc = big_document(lines, conflicts);
        println!("  build doc        {:>8.1} ms", ms(t.elapsed()));
        let t = Instant::now();
        let model = MergeModel::new(doc);
        println!("  MergeModel::new  {:>8.1} ms", ms(t.elapsed()));
        let t = Instant::now();
        let view = MergeView::new(model);
        println!("  MergeView::new   {:>8.1} ms", ms(t.elapsed()));
        Rig {
            view,
            term: Terminal::new(TestBackend::new(200, 60)).unwrap(),
            clip: Clipboard::new(),
        }
    }

    fn frame(&mut self) -> Duration {
        let t = Instant::now();
        let view = &mut self.view;
        self.term
            .draw(|f| {
                view.render(f, Rect::new(0, 0, 200, 60), None);
            })
            .unwrap();
        t.elapsed()
    }

    fn press(&mut self, code: KeyCode) -> Duration {
        let t = Instant::now();
        self.view.on_key(&key(code), &mut self.clip);
        t.elapsed()
    }

    /// Settles the highlighters (frames until nothing is pending).
    fn settle(&mut self) -> Duration {
        let t = Instant::now();
        for _ in 0..100_000 {
            self.frame();
            if !self.view.highlight_pending() {
                break;
            }
        }
        t.elapsed()
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn stats(label: &str, mut v: Vec<Duration>) {
    v.sort();
    let sum: Duration = v.iter().sum();
    println!(
        "  {label:<28} n={:<4} mean {:>7.2} ms  p50 {:>7.2}  p95 {:>7.2}  max {:>7.2}",
        v.len(),
        ms(sum) / v.len() as f64,
        ms(v[v.len() / 2]),
        ms(v[v.len() * 95 / 100]),
        ms(*v.last().unwrap())
    );
}

fn run(lines: usize, conflicts: usize) {
    println!("--- {lines} lines / {conflicts} conflicts");
    if std::env::var_os("PROBE_NO_HL").is_some() {
        let cfg = gitpoulpi_config::EditorConfig {
            syntax_highlighting: false,
            ..Default::default()
        };
        crate::editor_prefs::install(&cfg);
        println!("  (syntax highlighting off)");
    }
    let mut rig = Rig::new(lines, conflicts);
    println!("  first frame      {:>8.1} ms", ms(rig.frame()));
    println!("  settle highlight {:>8.1} ms", ms(rig.settle()));
    // Idle repaint of the same state.
    stats("idle frame", (0..20).map(|_| rig.frame()).collect());
    // Held Down in A (the cursor walks the conflict lines).
    let mut v = Vec::new();
    for _ in 0..150 {
        let t = Instant::now();
        rig.press(KeyCode::Down);
        rig.frame();
        v.push(t.elapsed());
    }
    stats("A Down key+frame", v);
    let mut v = Vec::new();
    for _ in 0..120 {
        let t = Instant::now();
        rig.press(KeyCode::PageDown);
        rig.frame();
        v.push(t.elapsed());
    }
    stats("A PgDn key+frame", v);
    rig.press(KeyCode::Home);
    rig.frame();
    // Full file toggle.
    let mut v = Vec::new();
    for _ in 0..4 {
        let t = Instant::now();
        rig.press(KeyCode::Char('f'));
        rig.frame();
        v.push(t.elapsed());
    }
    stats("Full file toggle+frame", v);
    // Pick, unpick.
    let mut v = Vec::new();
    for k in ['a', 'x', 'b', 'x'] {
        let t = Instant::now();
        rig.press(KeyCode::Char(k));
        rig.frame();
        v.push(t.elapsed());
    }
    stats("pick/unpick key+frame", v);
    // Output.
    rig.press(KeyCode::F(6));
    rig.press(KeyCode::F(6));
    let mut v = Vec::new();
    for i in 0..150 {
        let t = Instant::now();
        rig.press(KeyCode::Down);
        rig.frame();
        v.push(t.elapsed());
        if std::env::var_os("PROBE_TRACE").is_some() && i % 15 == 0 {
            println!(
                "    down {i}: {:.2} ms, scroll {}, caret line {}, parsed {}, window {:?}, pending {}",
                ms(*v.last().unwrap()),
                rig.view.out_scroll,
                rig.view.model.output().cursor_line_col().0,
                rig.view.hl_out.highlighted(),
                rig.view.hl_out.window_span(),
                rig.view.highlight_pending()
            );
        }
    }
    stats("Output Down key+frame", v);
    let mut v = Vec::new();
    for _ in 0..120 {
        let t = Instant::now();
        rig.press(KeyCode::PageDown);
        rig.frame();
        v.push(t.elapsed());
    }
    stats("Output PgDn key+frame", v);
    let (mut v, mut keys, mut frames) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..120 {
        let t = Instant::now();
        keys.push(rig.press(if i % 20 == 19 {
            KeyCode::Enter
        } else {
            KeyCode::Char('x')
        }));
        frames.push(rig.frame());
        v.push(t.elapsed());
    }
    stats("Output typing key+frame", v);
    stats("  of which the key", keys);
    stats("  of which the frame", frames);
    // Syntax highlighting speed: a fresh highlighter over 3000 lines.
    let lines: Vec<String> = rig.view.model.merge().side_lines()[1]
        .unwrap()
        .iter()
        .take(3000)
        .cloned()
        .collect();
    let mut hl = crate::editor_highlight::Highlight::for_file(std::path::Path::new("a.rs"), "");
    let t = Instant::now();
    hl.advance(
        |i| lines[i].as_str(),
        lines.len(),
        100,
        Duration::from_secs(5),
    );
    let warm = t.elapsed();
    let t = Instant::now();
    hl.advance(
        |i| lines[i].as_str(),
        lines.len(),
        3000,
        Duration::from_secs(5),
    );
    println!(
        "  highlight: first 100 lines {:.1} ms, next 2900 lines {:.1} ms ({:.1} us/line)",
        ms(warm),
        ms(t.elapsed()),
        t.elapsed().as_secs_f64() * 1e6 / 2900.0
    );
}

#[test]
#[ignore = "timing probe"]
fn probe_10k_50() {
    run(10_000, 50);
}

#[test]
#[ignore = "timing probe"]
fn probe_100k_3() {
    run(100_000, 3);
}

/// The view is built on a worker thread for big files (`merge_flow.rs`).
#[test]
fn the_view_can_move_between_threads() {
    fn send<T: Send>() {}
    send::<MergeView>();
}

fn big_view(lines: usize, conflicts: usize) -> MergeView {
    MergeView::new(MergeModel::new(big_document(lines, conflicts)))
}

/// The cached gap and origin answers follow every kind of change: typing in
/// a gap, a pick, an undo.
#[test]
fn the_origin_cache_follows_typing_picks_and_undo() {
    use super::model::Origin;
    let mut v = big_view(600, 3);
    let m = &mut v.model;
    let (first, _) = (0, 0);
    assert_eq!(m.origin_of_line(first), Origin::Common);
    // Typed in the first gap: its lines are Edited now, the others are not.
    m.edit(|d| {
        d.set_cursor(0, false);
        d.insert_text("x");
    });
    assert_eq!(m.origin_of_line(0), Origin::Edited);
    assert_eq!(m.origin_of_line(1), Origin::Edited, "per gap, not per line");
    let (c0, _) = m.region_lines(0).unwrap();
    assert_eq!(
        m.origin_of_line(c0),
        Origin::Common,
        "an empty region has no line"
    );
    let later = m.region_lines(2).unwrap().0 + 1;
    assert_eq!(m.origin_of_line(later), Origin::Common, "another gap");
    // A pick: the region's lines carry the side's letter.
    m.take_a(0);
    let (c0, _) = m.region_lines(0).unwrap();
    assert_eq!(
        m.origin_of_line(c0),
        Origin::Side(ConflictSide::Ours),
        "after the pick"
    );
    m.take_b(0);
    assert_eq!(m.origin_of_line(c0), Origin::Side(ConflictSide::Theirs));
    assert!(m.undo());
    assert_eq!(m.origin_of_line(c0), Origin::Side(ConflictSide::Ours));
    assert!(m.undo());
    assert_eq!(m.status(0), Some(super::model::ConflictStatus::Unpicked));
}

/// A held key skips the highlighting a single key gets.
#[test]
fn a_held_key_defers_highlighting_and_a_single_key_does_not() {
    let mut rig = Rig {
        view: big_view(2000, 3),
        term: Terminal::new(TestBackend::new(200, 60)).unwrap(),
        clip: Clipboard::new(),
    };
    rig.settle();
    assert!(!rig.view.highlight_pending(), "settled");
    rig.press(KeyCode::PageDown);
    rig.settle();
    let single = rig.view.hl[1].highlighted();
    assert!(single > 60, "the visible lines were colored: {single}");
    for _ in 0..30 {
        // A held key is one that follows the last within `BUSY` (150 ms). A
        // press and its frame take longer than that on a loaded machine in
        // a debug build, which turned the "stream" into single keys and
        // parsed the lines the test says it must not: the previous key is
        // stamped just now, so the stream is a stream whatever the CPU does.
        rig.view.last_input = Some(Instant::now());
        rig.press(KeyCode::PageDown);
        rig.view.last_input = Some(Instant::now());
        rig.frame();
    }
    assert!(
        rig.view.highlight_pending(),
        "frames of a held key leave the coloring for later"
    );
    assert!(
        rig.view.hl[1].highlighted() <= single + 60,
        "nothing was parsed while the key was held"
    );
    std::thread::sleep(Duration::from_millis(200));
    rig.settle();
    assert!(!rig.view.highlight_pending());
}

/// The scrollbar marks cost one pass over the conflicts: with hundreds of
/// them every track still shows at most one mark per cell and the hit list
/// stays bounded by the track heights.
#[test]
fn the_conflict_marks_stay_bounded_on_a_file_with_many_conflicts() {
    use super::MergeHit;
    let mut v = big_view(20_000, 400);
    let mut term = Terminal::new(TestBackend::new(160, 50)).unwrap();
    let mut hits = Vec::new();
    let t = Instant::now();
    term.draw(|f| hits = v.render(f, Rect::new(0, 0, 160, 50), None))
        .unwrap();
    let frame = t.elapsed();
    let marks = hits
        .iter()
        .filter(|(_, h)| matches!(h, MergeHit::ConflictMark(_)))
        .count();
    assert!(marks > 20 && marks <= 100, "{marks} marks");
    assert!(frame < Duration::from_secs(3), "a frame took {frame:?}");
}
