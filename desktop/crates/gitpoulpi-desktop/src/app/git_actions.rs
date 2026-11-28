//! Keyboard side of the toolbar's write buttons: Fetch, Pull, Push, Undo and Redo.
//!
//! Same keys as the terminal UI: `Ctrl+G` then `f`, `p` or `Shift+P` for the git operations, `Ctrl+Z`
//! and `Ctrl+Y` (`Ctrl+Shift+Z`) for Undo and Redo. A text field that has focus keeps its own `Ctrl+Z`.

use gpui_kit::{actions, App, KeyBinding};

use super::actions::with_workspace;
use crate::daemon::transfer::Transfer;

actions!(gp, [GitFetch, GitPull, GitPush, GitUndo, GitRedo]);

pub(super) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-g f", GitFetch, None),
        KeyBinding::new("ctrl-g p", GitPull, None),
        KeyBinding::new("ctrl-g shift-p", GitPush, None),
        KeyBinding::new("ctrl-z", GitUndo, None),
        KeyBinding::new("ctrl-y", GitRedo, None),
        KeyBinding::new("ctrl-shift-z", GitRedo, None),
    ]);
    cx.on_action(|_: &GitFetch, cx| run(cx, Transfer::Fetch));
    cx.on_action(|_: &GitPull, cx| run(cx, Transfer::Pull));
    cx.on_action(|_: &GitPush, cx| run(cx, Transfer::Push));
    cx.on_action(|_: &GitUndo, cx| run(cx, Transfer::Undo));
    cx.on_action(|_: &GitRedo, cx| run(cx, Transfer::Redo));
}

fn run(cx: &mut App, what: Transfer) {
    with_workspace(cx, move |ws, _, cx| ws.transfer(what, cx));
}
