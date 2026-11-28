# Parity with the terminal UI

The goal of the desktop frontend is full parity with the terminal UI: the same organisation, names, key
bindings and command palette, with desktop idioms (a menu bar) on top. This folder and
[`../docs/parity.md`](../docs/parity.md) track how far it is.

## What is generated

[`../docs/parity.md`](../docs/parity.md) has one row per key action, palette command and generated
command family of the shared registry (`gitpoulpi_app::registry::parity_rows`, every feature on):
id, context, English label, default keys and a status, with a summary count at the top. A palette
command that is the only twin of a key action shares that action's row.

## Statuses

- `done`: the desktop frontend runs the command. The command dispatcher (`src/commands/ops.rs`) is the
  only place that says so; `status.toml` lists the same ids and the tests make the two equal.
- `terminal only`: the registry says the entry only exists because of what a terminal is, and gives the
  reason (the terminal's own settings, for now). It never has to be done.
- `to do`: wanted on the desktop, not there yet. Running it in the desktop says "not available yet" in
  the status line, and its menu item is present but disabled.

## Marking a row done

1. Implement the command: add its `Op` and the arms of `op_for_action` / `op_for_palette` in
   `crates/gitpoulpi-desktop/src/commands/ops.rs`, and its execution in `commands/exec.rs`.
2. From `desktop/`, run `UPDATE_PARITY=1 cargo test -p gitpoulpi-desktop parity`. It rewrites
   `status.toml` from the dispatcher (the notes already written are kept; add one by hand if the row
   deserves it) and regenerates `docs/parity.md`.
3. Commit both files.

## Tests

- the table on disk is up to date (without `UPDATE_PARITY=1` it fails when `docs/parity.md` is stale);
- every id in `status.toml` is a row of the table and is not terminal only;
- `status.toml` equals the set of ids the dispatcher implements, so a row cannot be marked done by hand.

## The merge criterion

`parity_is_complete` fails while any row is `to do`. It is ignored in the normal run; the `desktop`
branch merges into `main` when it passes:

```sh
cd desktop
cargo test -p gitpoulpi-desktop parity_is_complete -- --ignored
```
