//! The TUI's own commit-graph lane algorithm, copied from `crates/gitpoulpi-tui/src/graph/`
//! (`part.rs`, `layout.rs`, `stash.rs`) so that lanes, merges, forks, the WIP row and stash rows have
//! exactly the same topology and colour indices as the terminal UI on the same repository.
//!
//! What had to change to make it neutral (this is the list for extracting it into a shared crate):
//! 1. `ratatui::style::Color` is gone. A cell carries a [`ColorRef`]: `Lane(n)` is "palette slot
//!    `n`" (the TUI stores `palette[n % len]`; here the modulo is taken at paint time), `Wip` and
//!    `Stash` stand for `th().graph.wip` / `th().graph.stash`.
//! 2. The text rendering half (`TreeCell::span`, glyph tables, `row_tint_bg`, `Line`s, `NodeSet`,
//!    `GraphSettings`) is dropped, and so is `TreeCell::bold` (only the text renderer reads it).
//! 3. `build_graph_grid_themed(.., palette: &[Color])` takes `palette_len: usize` instead.
//! 4. `refs.iter().any(|r| r.target == commit.id)` (an O(commits x refs) scan per row) became a
//!    precomputed `&HashSet<Oid>` of ref targets; with 5 000 refs the scan was the dominant cost.
//! 5. `wip: Option<WipAnchor>` / `stashes: &[GraphStash]` are unchanged; `GraphStash::from_stashes`
//!    is replaced by building the struct from the daemon's `Stash` in `graph/mod.rs`.
//! 6. `LogRows` (display index <-> row mapping) is reduced to [`display_order`].
//!
//! The algorithm body is otherwise verbatim, comments trimmed.
//!
//! This module is private to `graph`: it is a stand-in for the shared `gitpoulpi-app` crate that will
//! own the algorithm. Only `graph/mod.rs` and `graph/paint.rs` may name anything in here, so replacing
//! it means rewriting those two files and nothing else.

mod layout;
mod part;
mod stash;

pub use layout::{build_graph_grid, GraphGrid, GraphRow, WipAnchor};
pub use part::{ColorRef, NodeKind, Sides, TreeCell, TreePart};
pub use stash::{display_order, GraphStash, LogEntry};
