//! Commands: the one place where a key binding, a palette row and a menu item turn into behaviour.
//!
//! The registry in `gitpoulpi-app` says what exists (actions with their keys, palette commands with their
//! labels). This module says what the desktop does about each of them:
//!
//! | Module     | Role |
//! |------------|------|
//! | `catalog`  | [`Command`] and the stable row ids shared with the parity table |
//! | `ops`      | the dispatcher's `match`: which commands the desktop implements, and as which [`ops::Op`] |
//! | `exec`     | runs an op; answers "not available yet" for every other command |
//! | `keys`     | GPUI keystrokes to `KeyChord`s, resolved with the shared keymap, prefix chords |
//! | `context`  | the palette's availability context and the key context of the focused pane |
//! | `picker`   | the searchable list dialog (command palette, theme / language pickers) |
//! | `palette`  | the command palette: shared entries, shortcuts, fuzzy matching, reasons |
//! | `commit_ops` | the Log's commit operations: cherry-pick, revert, reset, checkout, copy, the menu key, comparing rows |
//! | `staging`  | running the staging and commit ops on the working-changes pane and the diff |
//! | `refs_ops` | the branch, tag and stash ops: which ids map to a `RefOp`, and running one |
//! | `status`   | the one-line message shown in the status line |

pub mod catalog;
pub mod commit_ops;
pub mod context;
pub mod exec;
pub mod keys;
pub mod ops;
pub mod palette;
pub mod picker;
pub mod refs_ops;
pub mod staging;
pub mod status;

pub use catalog::Command;
pub use exec::{run, run_id, Outcome};
pub use ops::{implemented_ids, op_for, DiffPartOp, Op, StatusOp};
