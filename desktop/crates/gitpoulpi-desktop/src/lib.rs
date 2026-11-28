//! GitPoulpi desktop frontend: a native window over the same daemon the terminal UI, the CLI and the
//! MCP server use.
//!
//! Layout of the crate:
//!
//! | Module    | Role |
//! |-----------|------|
//! | `app`     | startup, the main window, chrome (title bar, tabs, toolbar, status line), palette, actions |
//! | `daemon`  | the link to the session daemon (`gitpoulpi-link`): typed reads, the link host, Fetch/Pull/Push/Undo/Redo, the Log's page fetching (paging itself is `gitpoulpi_app::graph::LazyLog`) |
//! | `panels`  | Refs sidebar, Log, right pane, diff view |
//! | `dialogs` | question dialogs (a title, lines, numbered answers): the discard confirmation, the sensitive-files question |
//! | `form_dialog` | small forms in a dialog (text fields, check boxes, a reason line when the entry is refused): the branch, tag, rename, stash and upstream forms |
//! | `ref_menu` | the ref context menu: the terminal UI's rows and order for a branch, remote branch, tag or stash, and the view that shows them |
//! | `ref_target` | what a ref command acts on (a Refs row or a Log pill) and the selection facts the palette reads |
//! | `repo_menu` | the rows of the worktree, submodule, remote, LFS and file menus |
//! | `sensitive` | the sensitive-files question a refused stage raises |
//! | `commands`| the shared command registry in use: key bindings, command palette, the one dispatcher |
//! | `menu`    | the menu bar generated from the registry |
//! | `help`    | the keybindings view |
//! | `parity`  | the generated table of what the desktop covers compared with the terminal UI |
//! | `graph`   | commit graph layout and painter behind a small interface |
//! | `theme`   | GitPoulpi themes mapped onto the toolkit and the app's own colour set |
//! | `prefs`   | `desktop.toml` (desktop-only settings) and the column layout rule |
//! | `fonts`   | the bundled UI and monospace fonts |
//! | `cli`     | the command line |
//! | `strings` | the few texts that are not catalog strings |
//! | `util`    | dates and the diff patch bridge |

pub mod app;
pub mod cli;
pub mod commands;
pub mod daemon;
pub mod dialogs;
pub mod fonts;
pub mod form_dialog;
pub mod graph;
pub mod help;
pub mod menu;
pub mod panels;
pub mod parity;
pub mod prefs;
pub mod ref_menu;
pub mod ref_target;
pub mod repo_menu;
pub mod sensitive;
pub mod strings;
pub mod theme;
pub mod util;
