# Parity with the terminal UI

`status.toml` records, action by action, what the desktop frontend does compared with the terminal UI.
Each `[[action]]` has the terminal UI's action id (the ids of `crates/gitpoulpi-tui/src/keymap/action.rs`),
a `status` and an optional `note`.

## Statuses

- `done`: the desktop frontend offers the action. The note says where (menu, palette entry, shortcut)
  when that is not obvious.
- `terminal only`: the action stays in the terminal UI. A `reason` is required, for example that it only
  makes sense in a terminal.
- `to do`: the action is wanted on the desktop and not there yet.

An id that is not listed is not tracked yet; it counts as neither done nor decided.

## What comes next

Today the file is written by hand and nothing reads it. Once the shared registry of actions exists in
`gitpoulpi-app`, a generated table of every action with its status will replace the hand-kept list, and a
test will fail when an action of the registry has no status here or when `status.toml` names an id the
registry does not know.
