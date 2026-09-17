# Domain model

Defined in `crates/gitpoulpi-core/src/domain/`. No I/O-framework dependencies - pure data plus the trait definitions in `backend.rs`.

- **`RepoId` / `RepoRef`** - stable identity for a repo (canonicalized root path), used as the tab/session key.
- **`CommitGraph` / `CommitNode`** - the commit DAG, with graph layout computed backend-agnostically here so both the TUI and `gitpoulpi log --json` render the same structure.
- **`Ref` / `RefKind` / `RefVisibility`** - local/remote branches and tags, with `Shown`/`Hidden`/`Solo` visibility for the ref-filtering feature.
- **`Remote`** - name + fetch/push URLs.
- **`WorkingTree` / `FileStatus`** - working-tree file status.
- **`Hunk` / `DiffLine` / `LineSelector`** - diff structures for the hunk/line staging UI.
- **`StageTarget`** - unifies every staging granularity (`Line`, `Hunk`, `File`, `Folder`, `All`) behind one `GitBackend::stage`/`unstage` call, so frontends never need separate methods per granularity.
- **`Stash` / `StashId`**.
- **`ConflictSet` / `ConflictSide` / `ConflictResolution`** - conflict data for the resolution UI.
- **`lfs.rs`** - Git LFS, all pure (pointer parsing, `.gitattributes` patterns, `LfsBinary` detection, locks). `BlobContent::from_bytes` decides `LfsPointer` once so both backends agree. See ADR 0009.
- **`OperationKind`** - fieldless enum used as the operation-log/audit-trail tag and the IPC command key.
- **`Operation`** - an audit-log entry (kind, description, timestamp, whether it auto-stashed, whether it succeeded), built by the daemon around each mutation (`crates/gitpoulpi-daemon/src/server.rs`).
- **`ops.rs`** - per-operation option/outcome types used by `GitBackend` signatures.
- **`merge_doc/`** - the merge editor's pure document model (shared by TUI, CLI and MCP `conflicts`).
- **`journal.rs`** (crate root) - the undo journal's vocabulary (`domain/undo.rs`), see `../developer/the undo-redo design.

## `GitBackend`

See `git-backend-trait.md` for the full trait and the native-vs-fallback table - it's the seam every frontend and backend implementation is built around.
