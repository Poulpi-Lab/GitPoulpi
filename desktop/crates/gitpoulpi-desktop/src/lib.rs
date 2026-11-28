//! GitPoulpi desktop frontend: a native window over the same daemon the terminal UI, the CLI and the
//! MCP server use.
//!
//! Layout of the crate:
//!
//! | Module    | Role |
//! |-----------|------|
//! | `app`     | startup, the main window, chrome (title bar, tabs, toolbar, status line), palette, actions |
//! | `daemon`  | the link to the session daemon (`gitpoulpi-link`): typed reads, the link host, Fetch/Pull/Push/Undo/Redo, the Log's paging plan |
//! | `panels`  | Refs sidebar, Log, right pane, diff view |
//! | `graph`   | commit graph layout and painter behind a small interface |
//! | `theme`   | GitPoulpi themes mapped onto the toolkit and the app's own colour set |
//! | `prefs`   | `desktop.toml` (desktop-only settings) and the column layout rule |
//! | `fonts`   | the bundled UI and monospace fonts |
//! | `cli`     | the command line |
//! | `strings` | the few texts that are not catalog strings |
//! | `util`    | dates and the diff patch bridge |

pub mod app;
pub mod cli;
pub mod daemon;
pub mod fonts;
pub mod graph;
pub mod panels;
pub mod prefs;
pub mod strings;
pub mod theme;
pub mod util;
