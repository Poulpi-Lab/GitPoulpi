# Architecture overview

GitPoulpi is a Cargo workspace built hexagonal (ports & adapters): `gitpoulpi-core` is the domain center with zero knowledge of any frontend or backend implementation detail; everything else is an adapter plugged into it.

## Crate dependency graph (conceptual)

```
                      gitpoulpi-core
                     (domain + GitBackend port)
                            ^
             ┌──────────────┼───────────────┐
             │               │                │
  gitpoulpi-cli-backend  gitpoulpi-git2-backend  gitpoulpi-ipc (protocol)
   (system git adapter)   (libgit2 + CLI fallback)      ^
             ^               ^                          │
             └───────┬───────┘                gitpoulpi-daemon
                      │                     (owns state, IPC endpoint)
              gitpoulpi-daemon                          ^
                                          ┌──────────────┼──────────────┐
                                    gitpoulpi-tui   gitpoulpi-cli   gitpoulpi-mcp
                                    (ratatui client) (clap client) (MCP client)
```

`gitpoulpi-config`, `gitpoulpi-secrets`, `gitpoulpi-session`, `gitpoulpi-hosting` and `gitpoulpi-ai` are secondary adapters consumed by the daemon (and, where relevant, directly by the CLI for local-only concerns like `gitpoulpi config path`) - never duplicated between frontends. `gitpoulpi-commands` holds the clap tree and its one `execute` mapping shared by the CLI and the MCP server; `gitpoulpi-i18n` holds the compiled-in UI catalogs (`../developer/i18n.md`).

Pure logic that more than one adapter needs lives in `gitpoulpi-core` as plain functions, not behind the trait: `textconv` (encodings, line breaks, indentation: the editor and the daemon's file conversion) and `domain::rules` (where an ignore or attribute rule is written, and its exact line: both backends, the TUI picker and the CLI). `gitpoulpi-config::os_apps` parses what the OS registers for a file type (the "Open with" list) from a file system or registry handed in, so it never reads the real desktop in a test.

## Why hexagonal

Two `GitBackend` implementations must behave identically, and three frontends (TUI/CLI/MCP) must never diverge in what they can do or how they describe state. Keeping the domain and the `GitBackend` trait dependency-free from any specific backend or frontend is what makes the contract-test suite in `gitpoulpi-testkit` meaningful: it's testing the actual seam, not incidental glue.

## Why a background daemon

See `decisions/0003-mcp-in-mvp.md`: MCP and CLI must be first-class, and an AI agent driving the CLI/MCP while the user has the TUI open must show up live in that TUI. A single per-session daemon owning all state makes that guarantee structural rather than something each frontend has to get right independently.

## Cross-platform strategy

Cross-platform (Windows primary, Linux incl. Wayland/Hyprland, macOS best-effort) is treated as a constraint on every component:

- **TUI**: `crossterm` talks to the terminal emulator, not the display server, so X11/Wayland is a non-issue by construction. Terminal-capability differences (mouse reporting, true color) are feature-detected and degrade to keyboard-only, which is always fully functional (`../user/keybindings.md`).
- **Daemon IPC**: **loopback TCP plus a discovery file**, not an OS-native transport. The daemon binds `127.0.0.1:0` (an OS-assigned ephemeral port), writes that port to `sessions/<name>.port` under the config dir (`gitpoulpi_config::ConfigPaths::ipc_port_file`), and clients read the file to find it (an embedded daemon also writes `<name>.host`, its process id, so a file left by a dead process is deleted instead of connected to; `session-and-state.md`); every message is a 4-byte big-endian length prefix followed by JSON. This is what makes the transport portable *without* `cfg`-gated code paths: there is exactly one implementation, and Windows, Linux and macOS all run it. It is strictly localhost-only, never bound to a real network interface. The original plan called for a named pipe on Windows and a Unix domain socket elsewhere (via something like `interprocess`); that would buy OS-level access control stricter than "bound to loopback", at the cost of two transports to keep in agreement. Revisit only if that access-control distinction starts to matter. `crates/gitpoulpi-ipc/src/transport.rs` carries the same note; there is no transport abstraction.
- **Paths/config dirs**: the `directories` crate, never hand-built path strings.
- **Git behavior**: line-ending (`core.autocrlf`/`core.eol`) and exec-bit handling come from the repository's git config on both backends; GitPoulpi only writes them when the user changes them in Settings (through `GitBackend`, undoable).
- **Credentials**: SSH via the platform's ssh-agent; HTTPS via the user's existing `git credential.helper` (Git Credential Manager / libsecret / Keychain), which is already OS-appropriate.
- **CI**: the GitHub workflow runs Ubuntu only on push (currently disabled, see `CLAUDE.md`); Windows is gated locally with `pwsh scripts/check.ps1`; Windows/macOS CI runs on demand.

## Cargo features

The three binary crates (`gitpoulpi-cli`, `-tui`, `-mcp`) have `default = ["full"]`, so a plain `cargo build` is everything on. Features switch off languages (`lang-*`, `all-langs`), the integrations (`github`, `gitlab`, `azure-devops`, `bitbucket`, `jira`, `trello`), and `ai`, `mcp`, `lfs`, `worktree`, `submodule`, `avatar-fetch`, `image-view`; theme presets are chosen at build time with `GITPOULPI_THEMES` (`gitpoulpi-config/build.rs`). `gitpoulpi-core`, `-config`, `-ipc`, `-session` and `-secrets` have none, so the domain, the config schema and the IPC protocol are identical in every build; whatever is not built answers with a typed "not built into this binary" result (`Error::NotBuilt`). The graph and its rules: `../developer/the custom-builds design.

- `cargo run -p xtask -- build-custom` (or `scripts/build-custom.sh` / `.ps1`) builds a chosen combination with the `custom` profile into `target/custom/`.
- `cargo run -p xtask -- features` is the matrix gate: clippy of the three binaries with each feature off once, plus the off-build tests. `--quick` runs three clippy builds (everything off; `lang-fr` + github; `lang-fr` + jira, a provider outside GitHub/GitLab). `scripts/check.ps1 -Features` runs the quick form.
- Sizes: `../developer/binary-size.md`.

## Where to look next

- `../roadmap.md` - the phase structure and what each phase contains; the authority on scope.
- `../developer/where-we-are.md` - what actually works today, and what to pick up next.
- `git-backend-trait.md` - the `GitBackend` trait and the table of what git2 does natively vs. via CLI fallback (documentation only; there is no runtime capability set).
- `domain-model.md` - the core entities.
- `session-and-state.md` - the per-feature decision record (sessions, daemon, every TUI feature's design choices and traps).
- `decisions/` - ADRs for the choices that shaped this architecture.
