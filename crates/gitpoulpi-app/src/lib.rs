//! Frontend-neutral application logic, shared by the terminal UI and the
//! desktop UI.
//!
//! Everything here is pure: no `ratatui`, no `crossterm`, no UI toolkit, no
//! I/O framework. A frontend is an IPC client of the daemon; what it shares
//! with the other frontend is the logic that decides *what* is shown (which
//! lane a commit sits in, where a stash row goes, when the next page of
//! history is fetched), never *how* it is drawn. Colours are therefore
//! references ([`graph::ColorRef`]) that each frontend resolves against its
//! own theme.
//!
//! `tests/dependency_guard.rs` fails if a UI crate ever becomes a dependency.

pub mod commit_form;
pub mod compare;
pub mod dates;
pub mod diff;
pub mod editor_prefs;
pub mod features;
pub mod file_selection;
pub mod flow;
pub mod graph;
pub mod highlight;
pub mod keymap;
pub mod log_multiselect;
pub mod merge_view;
pub mod palette;
pub mod provider_caps;
pub mod refs_sections;
pub mod refs_tree;
pub mod registry;
pub mod repo_facts;
pub mod sequences;
pub mod settings;
pub mod status_rows;
pub mod surface;
pub mod tag_where;
pub mod text;
