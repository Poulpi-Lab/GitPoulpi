//! Local Markdown issues (slice L1): a folder of notes, one per issue,
//! usable with an Obsidian vault. No token and no network.
//!
//! `<id>-<slug>.md`, YAML frontmatter `id, title, status, assignees, labels,
//! milestone, created, updated`, Markdown body. Properties GitPoulpi does
//! not own and the body are preserved byte for byte, in their order; writes
//! are atomic. See `docs/user/local-issues.md`.

mod client;
pub mod frontmatter;
pub mod obsidian;
pub mod spec;
mod store;

pub(crate) use client::LocalMarkdownClient;
pub use client::{file_url, note_path, path_from_file_url};
pub use store::{
    id_from_file_name, iso_datetime, slug, write_atomic, Issue, NewNote, Patch, Store, UsedValues,
};
