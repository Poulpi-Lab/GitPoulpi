//! `LogLayoutCache`: an unchanged redraw rebuilds nothing, a changed input
//! rebuilds exactly once, and the search matches follow the log.

use std::collections::HashSet;

use gitpoulpi_core::domain::{CommitGraph, CommitNode, Oid, Ref, RefKind, Stash};

use super::cache::{build, LayoutInputs, LogLayoutCache};
use super::{GraphRender, GraphSettings, LaneBackground};
use crate::log_search::SearchIndex;

fn oid(i: usize) -> Oid {
    let mut b = [0u8; 20];
    b[..8].copy_from_slice(&(i as u64 + 1).to_be_bytes());
    Oid(b)
}

/// Newest first, a side branch merged back every 7th commit.
fn history(n: usize) -> CommitGraph {
    let nodes = (0..n)
        .map(|i| {
            let mut parents = Vec::new();
            if i + 1 < n {
                parents.push(oid(i + 1));
            }
            if i % 7 == 3 && i + 2 < n {
                parents.push(oid(i + 2));
            }
            CommitNode {
                id: oid(i),
                parents,
                summary: format!("commit {i}"),
                message: format!("commit {i}\n\nbody"),
                author_name: if i % 2 == 0 { "Alice" } else { "Bob" }.into(),
                author_email: "a@example.com".into(),
                timestamp: 1_700_000_000 - i as i64,
                refs: Vec::new(),
            }
        })
        .collect();
    CommitGraph { nodes }
}

struct World {
    log: gitpoulpi_core::Result<CommitGraph>,
    refs: gitpoulpi_core::Result<Vec<Ref>>,
    stashes: gitpoulpi_core::Result<Vec<Stash>>,
    solo: HashSet<String>,
    hidden: HashSet<String>,
    log_gen: u64,
}

impl World {
    fn new(n: usize) -> World {
        let main = Ref {
            kind: RefKind::LocalBranch {
                name: "main".into(),
            },
            target: oid(0),
            is_head: true,
            visibility: Default::default(),
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        };
        World {
            log: Ok(history(n)),
            refs: Ok(vec![main]),
            stashes: Ok(Vec::new()),
            solo: HashSet::new(),
            hidden: HashSet::new(),
            log_gen: 0,
        }
    }

    fn inputs(&self, settings: GraphSettings) -> LayoutInputs<'_> {
        LayoutInputs {
            log: &self.log,
            log_gen: self.log_gen,
            refs: &self.refs,
            refs_gen: 0,
            stashes: &self.stashes,
            stashes_gen: 0,
            solo: &self.solo,
            hidden: &self.hidden,
            has_wip: true,
            show_stashes: true,
            settings,
        }
    }
}

#[test]
fn redraws_with_unchanged_inputs_build_the_layout_once() {
    let w = World::new(200);
    let mut cache = LogLayoutCache::default();
    for _ in 0..50 {
        cache.update(&w.inputs(GraphSettings::default()), None);
    }
    assert_eq!(cache.layout_builds, 1);
    assert_eq!(cache.layout().rows.len(), 200);
}

#[test]
fn each_input_that_matters_rebuilds_once() {
    let mut w = World::new(50);
    let mut cache = LogLayoutCache::default();
    let s = GraphSettings::default();
    cache.update(&w.inputs(s), None);
    // A new log (a daemon refresh or a page).
    w.log = Ok(history(60));
    w.log_gen += 1;
    cache.update(&w.inputs(s), None);
    cache.update(&w.inputs(s), None);
    assert_eq!(cache.layout_builds, 2);
    assert_eq!(cache.layout().rows.len(), 60);
    // Hide, render mode and lane background.
    w.hidden.insert("main".into());
    cache.update(&w.inputs(s), None);
    assert_eq!(cache.layout_builds, 3);
    assert!(cache.layout().refs_for_pills.is_empty());
    let image = GraphSettings {
        render: GraphRender::Image,
        ..s
    };
    cache.update(&w.inputs(image), None);
    assert_eq!(cache.layout_builds, 4);
    let tinted = GraphSettings {
        lane_background: LaneBackground::RowTint,
        ..image
    };
    cache.update(&w.inputs(tinted), None);
    assert_eq!(cache.layout_builds, 5);
    cache.update(&w.inputs(tinted), None);
    assert_eq!(cache.layout_builds, 5);
    // The row tint strength is part of the key.
    let faded = GraphSettings {
        tint_strength: gitpoulpi_config::GraphRowTintStrength::Subtle,
        ..tinted
    };
    cache.update(&w.inputs(faded), None);
    assert_eq!(cache.layout_builds, 6);
    cache.update(&w.inputs(faded), None);
    assert_eq!(cache.layout_builds, 6);
    // Solo.
    w.solo.insert("main".into());
    cache.update(&w.inputs(faded), None);
    assert_eq!(cache.layout_builds, 7);
}

#[test]
fn search_rescans_only_when_the_query_or_the_layout_changes() {
    let mut w = World::new(100);
    let mut cache = LogLayoutCache::default();
    let s = GraphSettings::default();
    for _ in 0..20 {
        cache.update(&w.inputs(s), Some("bob"));
    }
    assert_eq!(cache.search_scans, 1);
    assert_eq!(cache.matches().len(), 50);
    assert!(cache.match_set().contains(&1));
    cache.update(&w.inputs(s), Some("commit 42"));
    assert_eq!(cache.search_scans, 2);
    assert_eq!(cache.matches(), &[42]);
    // A new log under the same query: the answer must follow it.
    w.log = Ok(history(40));
    w.log_gen += 1;
    cache.update(&w.inputs(s), Some("commit 42"));
    assert_eq!(cache.search_scans, 3);
    assert!(cache.matches().is_empty());
    // Closing the bar clears the matches (no stale dimming).
    cache.update(&w.inputs(s), None);
    assert!(cache.matches().is_empty());
    assert_eq!(cache.layout_builds, 2);
}

#[test]
fn the_index_agrees_with_find_matches() {
    let w = World::new(120);
    let nodes = &w.log.as_ref().unwrap().nodes;
    let refs = w.refs.as_ref().unwrap();
    let index = SearchIndex::new(nodes, refs);
    let hex = nodes[5].id.to_hex();
    for q in [
        "bob",
        "ALICE",
        "commit 1",
        "main",
        "  ",
        "",
        "body",
        &hex[..8],
        "zzz",
    ] {
        assert_eq!(
            index.find(q),
            crate::log_search::find_matches(nodes, refs, q),
            "{q}"
        );
    }
}

/// `cargo test --release -p gitpoulpi-tui perf_probe -- --ignored --nocapture`
#[test]
#[ignore]
fn perf_probe() {
    use std::time::Instant;
    let w = World::new(5000);
    let s = GraphSettings::default();
    let frames = 100u32;
    let t = Instant::now();
    for _ in 0..frames {
        let layout = build(&w.inputs(s));
        let nodes = layout.nodes.as_ref().unwrap();
        let _ = crate::log_search::find_matches(nodes, &layout.refs_for_pills, "bob");
    }
    let before = t.elapsed() / frames;
    let mut cache = LogLayoutCache::default();
    let t = Instant::now();
    for _ in 0..frames {
        cache.update(&w.inputs(s), Some("bob"));
    }
    let after = t.elapsed() / frames;
    println!(
        "5000 commits, search open: per frame before {before:?}, after {after:?} (builds {}, scans {})",
        cache.layout_builds, cache.search_scans
    );
}

fn named(slug: &str) -> super::LaneTheme {
    super::LaneTheme::from_slug(slug)
}

#[test]
fn each_graph_color_theme_yields_its_own_palette() {
    use ratatui::style::Color;
    assert_eq!(
        super::lane_palette(named("gitpoulpi")),
        &super::LANE_PALETTE[..]
    );
    assert_eq!(
        super::lane_palette(named("material"))[0],
        Color::Rgb(0x42, 0xa5, 0xf5)
    );
    assert_eq!(super::lane_palette(named("material")).len(), 16);
    assert_eq!(super::lane_palette(named("serie")).len(), 6);
    assert_eq!(
        super::lane_palette(named("serie"))[0],
        Color::Rgb(0xe0, 0x6c, 0x76)
    );
    let w = World::new(5);
    for slug in [
        "theme",
        "gitpoulpi",
        "light",
        "material",
        "serie",
        "nord",
        "light",
    ] {
        let theme = named(slug);
        let mut cache = LogLayoutCache::default();
        let settings = GraphSettings {
            theme,
            ..GraphSettings::default()
        };
        cache.update(&w.inputs(settings), None);
        let palette = super::lane_palette(theme);
        for (_, color) in &cache.layout().rows {
            assert!(
                palette.contains(color),
                "{slug}: {color:?} not in its palette"
            );
        }
        // The WIP row takes palette color 0.
        assert_eq!(
            cache.layout().wip_row.as_ref().unwrap().cells[0].color,
            palette[0]
        );
    }
}

#[test]
fn every_registry_lane_palette_resolves() {
    for p in gitpoulpi_config::theme::lane_palettes() {
        assert_eq!(
            super::lane_palette(named(&p.slug)).len(),
            p.colors.len(),
            "{}",
            p.slug
        );
    }
    assert!(named("theme") == super::LaneTheme::FOLLOW);
    assert!(named("") == super::LaneTheme::FOLLOW);
}

#[test]
fn changing_the_graph_color_theme_rebuilds_the_layout() {
    let w = World::new(5);
    let mut cache = LogLayoutCache::default();
    let base = GraphSettings::default();
    cache.update(&w.inputs(base), None);
    let before = cache.layout().rows[0].1;
    let inspired = GraphSettings {
        theme: named("material"),
        ..base
    };
    cache.update(&w.inputs(inspired), None);
    assert_eq!(cache.layout_builds, 2);
    assert_ne!(cache.layout().rows[0].1, before);
    cache.update(&w.inputs(inspired), None);
    assert_eq!(cache.layout_builds, 2);
}

#[test]
fn pills_by_oid_matches_a_scan_of_the_refs_for_every_commit() {
    use crate::log_pills::ref_pills_for_commit;
    use crate::test_support::{local_branch_at, remote_branch_at};
    let refs = vec![
        local_branch_at("main", oid(0)),
        remote_branch_at("origin", "main", oid(0)),
        remote_branch_at("origin", "dev", oid(0)),
        local_branch_at("topic", oid(5)),
    ];
    let by_oid = super::cache::pills_by_oid(&refs);
    for i in 0..8 {
        let scanned = ref_pills_for_commit(oid(i), &refs);
        let cached = by_oid.get(&oid(i)).map_or(&[][..], Vec::as_slice);
        assert_eq!(cached.len(), scanned.len(), "commit {i}");
        for (a, b) in cached.iter().zip(&scanned) {
            assert_eq!(
                (&a.name, a.checked_out, a.combined_remote),
                (&b.name, b.checked_out, b.combined_remote)
            );
        }
    }
}

#[test]
fn the_follow_theme_palette_is_the_active_themes_own() {
    use crate::theme::{override_theme, Theme};
    use ratatui::style::Color;
    let nord = Theme::from(&gitpoulpi_config::theme::preset("nord").unwrap());
    let _g = override_theme(&nord);
    assert_eq!(super::theme_lane_palette()[0], Color::Rgb(0x88, 0xc0, 0xd0));
    // The explicit picks do not follow it.
    assert_eq!(
        super::lane_palette(named("gitpoulpi"))[0],
        Color::Rgb(64, 156, 255)
    );
}

#[test]
fn the_layout_deduces_a_ghost_ref_for_commits_without_one_and_follows_hide() {
    let mut w = World::new(20);
    let mut cache = LogLayoutCache::default();
    let s = GraphSettings::default();
    cache.update(&w.inputs(s), None);
    let layout = cache.layout();
    // `main` points at commit 0 and walks first parents down the whole log.
    assert_eq!(
        layout.ghost_for(5).map(|g| g.pill.name.as_str()),
        Some("main")
    );
    assert_eq!(
        layout.ghost_for_commit(oid(9)).map(|g| g.tooltip()),
        Some("main (Local Branch)".to_string())
    );
    assert!(layout.ghost_for(10_000).is_none());
    // Hidden refs name nothing.
    w.hidden.insert("main".into());
    cache.update(&w.inputs(s), None);
    assert!(cache.layout().ghost_for(5).is_none());
}
