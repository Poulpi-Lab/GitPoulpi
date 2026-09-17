# `GitBackend`: native vs. fallback

`GitBackend` (`crates/gitpoulpi-core/src/backend.rs`) is the one port every
read and mutation goes through. `SystemGitBackend` (`gitpoulpi-cli-backend`)
drives the installed `git` and parses machine-readable output only
(`--porcelain=v2 -z`, NUL-delimited formats, `--json`), never human text.
`Git2Backend` decorates it: each method either calls libgit2 or forwards to
its inner `SystemGitBackend`. The choice is made per method (sometimes per
argument) in `crates/gitpoulpi-git2-backend/src/lib.rs`, never through trait
default methods, so it is explicit and in one file. Nothing reads this
document at runtime; `GitBackend::name()` (`"git2"` / `"system"`) replaced the
old `CapabilitySet`. Update the matrix when a dispatch changes.

## Capability matrix (`Git2Backend`)

| Operation | Dispatch | Why |
|---|---|---|
| status, diff, diff_files, diff_paths | Native; system git in an LFS repo for working-tree/index diffs | libgit2 has no LFS clean filter (see below) |
| log | Native; system git when `LogOptions::paths` is non-empty | git's history simplification for a pathspec has no libgit2 equivalent; a hand-rolled revwalk filter returns a different commit list on merges |
| count_commits | Delegated | Always system git: `rev-list --count` reads the commit-graph file, a libgit2 revwalk does not (50,000 commits: 12 ms vs 213 ms). |
| refs, remotes, stashes, resolve, read_blob, read_blob_bytes, list_tree, blame, commit_stats, worktrees, submodules, discover, init | Native | |
| journal plumbing: ref_snapshot, update_refs, index_tree/index_restore, snapshot_create/restore, path_hashes, config_section | Native | the undo journal calls them around every operation; they must cost no process |
| reflog, blob_bytes | Native (`Repository::reflog`, `find_blob`) | the system backend reads `logs/<ref>` (found with `git rev-parse --git-path`, so linked worktrees work) and `git cat-file blob`; both parse the same file, so `old`, `new`, committer, time and message are identical. A ref without a reflog (a deleted branch) is an empty page. A reftable repository has no such file (and libgit2 cannot open it): both backends read it with `git log -g` (`reflog_via_git`, `old` taken from the next older entry), pinned by `check_reflog_and_lost_commits_in_a_reftable_repository`. The outside-operations and read-cache fingerprints stat `reftable/` too |
| lost_commits | system git (one implementation) | `rev-list --reflog --not --exclude=refs/gitpoulpi/* --all` and the same over the undo pins give the reflog- and pin-kept commits; `deep` adds every commit of `cat-file --batch-all-objects` (fed to `git log --stdin`, so dangling ones show). Streamed and killed at `budget_ms`, the page then says `partial`; the daemon runs it like any read, off the render thread |
| stage/unstage File, Folder, All | Native; system git for Hunk/Line, in an LFS repo, or above `NATIVE_ADD_LIMIT` dirty paths | libgit2 applies whole hunks only; above the limit one `git add` beats libgit2's per-file filter/attribute checks (~10 ms a file on Windows with endpoint scanning) |
| commit (incl. amend) | Native; system git when MERGE_HEAD/CHERRY_PICK_HEAD/REVERT_HEAD is pending | git reads and clears the sequencer state |
| checkout Branch/Commit | Native (system git in an LFS repo) | |
| checkout File | system git (`Error::Unsupported` from `mutate.rs` triggers the forward) | single-file checkout into index + workdir needs careful index-entry handling |
| discard | Native on the index; system git in an LFS repo | |
| discard_part, delete_files, gitignore_add, patch_create/apply/files | system git | one patch builder (`patch.rs`) gets CRLF and missing final newlines right; `format-patch`/`apply` own the exact bytes |
| mailmap_resolve | Native on both, by design | the system backend runs `git check-mailmap --stdin` (one spawn for any number of identities); `Git2Backend` parses the same files (`.mailmap` in the worktree, `HEAD:.mailmap` in a bare repository, then `mailmap.file`) with `gitpoulpi_core`'s `Mailmap`, **not** libgit2's `Mailmap`: libgit2 compares the email case-sensitively where git does not (`ALEX@old.example.com` is not mapped by a line for `alex@old.example.com`; the contract test was red on it). The contract suite pins both against `git check-mailmap` and `git log --format=%aN`. |
| mailmap_identities, mailmap_add | system git (one implementation) | one `git log --format=%an%x01%ae%x01%cn%x01%ce` counts a scope's identities (the scope is resolved by the history engine's `refs::resolve`); the `.mailmap` edit is plain file I/O over the shared `mailmap::merged`, like `gitignore_add` |
| branch create/delete/rename/set-upstream, tag create/delete/annotate, remote set-url/set-push-url, ssh_command, repo_config, reword | Native | |
| split_source, split_commit | system git | byte-faithful partial patches (`patch.rs`) and `commit-tree` in a temporary index, like hunk staging; the last part keeps the original tree, so descendants are replayed as in reword (`rewrite.rs`) |
| rewrite_preview, rewrite_apply, rewrite_discard, rewrite_load, rewrite_blob_diff, rewrite_scan_secrets, rewrite_analyze, rewrite_check_signing, rewrite_script_check, rewrite_script_test | system git | one engine (`cli-backend/src/history/`): the byte-faithful commit serialiser, the SHA-1 hashing and the temporary pack writer live once, and a fixed number of `git` processes (rev-list, one `cat-file --batch`, `index-pack`, one `update-ref --stdin` with compare-and-swap) serve any history size; libgit2 would add a second serialiser to keep byte-identical for no gain |
| branch_create_tracking, fast_forward | Native (system git in an LFS repo) | they write the working tree |
| reset | Native; Mixed/Hard go to system git in an LFS repo | Soft only moves the branch |
| move_head_keep | system git | libgit2 has no `reset --keep` |
| merge, rebase (plain and interactive, continue/skip/abort/plan), cherry-pick, revert, am_*, operation_state/abort/skip | system git | strategies, merge drivers, todo-list editing and sequencer state are git's |
| rebase_state, conflicts | Native short-circuit when provably none, else system git | both run on every refresh; skipping the spawn matters |
| conflict_entries | Native | `Index::conflicts`, no process |
| merge_document | Native stage reads, system `git merge-file --zdiff3` for the merge | git2 0.19 has no `git_merge_file` binding, and libgit2's xdiff could place regions differently from git |
| resolve_conflict, unresolve_conflict, index_lock(_remove) | system git | one implementation is the identity guarantee |
| stash push/pop/apply/drop/store/paths | system git | |
| sparse_checkout_status/set/disable (and `CloneOptions::sparse`) | system git | libgit2 has no sparse-checkout API; cone mode only, see session-and-state.md |
| fetch, fetch_all, pull, push, clone, clone_with_progress, tag_push, tag_delete_remote, branch_delete_remote, fetch_ref_as_branch | system git | credentials stay with git's credential helper and ssh-agent (ADR 0004); one progress vocabulary |
| remote_add/remove/rename | system git | rename/remove must also move or drop the tag mirror, which has one writer |
| worktree_add | system git | libgit2 cannot create a worktree on a new branch at a commit in one step, does not create parent folders, and names the admin entry differently |
| submodule_init/update | system git | git's transports and credentials |
| lfs_files | Native | |
| every other `lfs_*` | system git | needs the `git-lfs` program (ADR 0009) |

Known gaps:

- `Git2Backend::commit` ignores `CommitOptions::sign` (no external signer
  hook); `SystemGitBackend` passes `-S` or `--no-gpg-sign`. Signed tags: same.
- Plain `rebase`'s `RebaseOutcome::Conflicted.remaining` is always empty;
  the interactive path parses `rebase-merge/git-rebase-todo` for it.
- Windows/MSVC: `crates/gitpoulpi-git2-backend/build.rs` links `advapi32`
  because `libgit2-sys` does not always emit it.

## Feature gating

Cargo features `lfs`, `worktree`, `submodule` (default on, forwarded from
`gitpoulpi-git2-backend` to `gitpoulpi-cli-backend`) swap in `*_stub.rs`
modules. A method of a feature not built returns
`Error::NotBuilt { feature }`, typed so each frontend words it and the CLI
reports a stable code. `gitpoulpi_core::BUILD_FEATURES` lists the names
that may cross the wire; an unknown one becomes `"unknown"`.

## Error model

`gitpoulpi_core::Error` has typed variants for every outcome a UI must act
on (`PushRejected`, `NotFastForward`, `*Refused`, `NeedsConfirmation`,
`LfsUnavailable`, `NeedsAutoStash`, `PendingAutoStash`, `NotBuilt`, ...)
and `Backend(String)` for the rest. Rule: no frontend ever matches on git's
English stderr (it breaks on the first localised git). Typed variants
cross IPC as `TypedError`; `Error::Unsupported` is internal to the
decorator (the signal to forward).

## Contract tests

`gitpoulpi_testkit::contract` holds `&dyn GitBackend` checks;
`crates/gitpoulpi-git2-backend/tests/contract.rs` runs every check in
`for_each_contract_check!` against both backends, one `#[test]` each so
fixture builds run in parallel. Any behavior that must be identical goes
there, not into backend-specific tests. Each check builds its own fixture
in its own temp dir and touches no shared cwd/env. `tests/spawn_budget.rs`
pins how many `git` processes each operation spawns (10-40 ms each on
Windows); raise a budget only with a reason.

No test touches the network: `FixtureRepoBuilder::with_origin()` creates a
bare repository beside the fixture as `origin` (a path is a first-class
remote), writing `[remote "origin"]` into `.git/config` directly to save a
spawn. `FixtureRepoBuilder::new()` is one spawn and must stay so.

## File content, blame and per-path history

- **Per-path log is `LogOptions::paths`, not a new operation**, so the
  daemon, CLI (`log --path`) and MCP reach it through existing plumbing.
  Empty means no filter. Renames are **not** followed here: `--follow`
  takes one pathspec only and libgit2 has none. Following is offered
  separately, as `file_history` (next bullet).
- **`file_history(path, start, skip, limit)` follows renames** like `git log
  --follow` and returns per-commit path, change kind (Added / Modified /
  Deleted / Renamed{from}) and line counts (`None` = binary). Both backends
  run the same single `git log --follow -M --diff-merges=first-parent --raw
  --numstat -z` process: libgit2 has no `--follow`, and re-implementing git's
  rename-chasing walk would make the backends disagree exactly on the cases
  the feature exists for (`Git2Backend` delegates to its system-git
  fallback, so the contract suite pins one behavior). Traps:
  - **`--skip` is not passed**: git skips before computing a commit's diff,
    so a rename inside the skipped range is never seen and the followed path
    goes stale. The backend asks for `skip + limit + 1` (the extra one sets
    `more`) and drops the first `skip` itself; a deep page costs its depth,
    the first page stays cheap because git walks lazily.
  - **Merges follow git's default history simplification**, not
    first-parent: a merge is an entry only when the file differs from every
    parent, both sides of a kept merge are walked, and its change and line
    counts are against its first parent (`--diff-merges=first-parent`).
  - `-M` is explicit (a user's `diff.renames=false` must not disable the
    feature), copies are not detected, `--literal-pathspecs` keeps a file
    named `*.rs` literal, `--no-show-signature` keeps `log.showSignature`
    out of the parsed stream.
- **Blame follows whole-file renames on both backends** (git's default,
  libgit2's too; pinned by the contract suite). `BlameLine::orig_path` is the
  path in the blamed commit and `previous` is `(first parent, path there)`
  for "blame previous revision": system git reads it from the porcelain
  `previous` header, libgit2 computes it per `(commit, path)` (cached) from
  the first parent's tree, falling back to 50% rename detection when the
  parent lacks the path. They agree except where git blames through a merge's
  second parent, which the contract does not cover.
- **`read_blob` takes an explicit `BlobSource`** (`Commit`, `Index`,
  `WorkingTree`): a path has three contents at once. `Index` is stage 0
  only; an unmerged path is an error, not an arbitrary side.
- **`blame` takes a commit, never "the working tree"**: blaming the
  worktree yields the all-zero "Not Committed Yet" pseudo-commit nothing
  can click through to.
- **Binary and oversized are successful results**, not errors:
  `BlobContent::Binary` (a NUL in the first 8000 bytes, git's heuristic,
  or invalid UTF-8) and `TooLarge` (> `MAX_TEXT_BLOB_BYTES`, 8 MiB). The
  size is read first (`cat-file -s`, `odb.read_header`, `fs::metadata`) so
  a huge blob is never loaded to be rejected. A missing path is an error.
- **One line splitter**: `BlobContent::from_bytes` in core is the only
  place bytes become lines. With `core.autocrlf` the blob is LF and the
  checkout CRLF; stripping `\r` centrally makes `Commit` and `WorkingTree`
  reads yield identical lines (`crlf` records it for write-back). The
  trailing empty element after a final newline is dropped
  (`final_newline` records it), else every file gets a phantom last row.
- **Blame lines come from `read_blob`** in both backends; only the
  attribution comes from git/libgit2 (the CLI ignores `--line-porcelain`'s
  content records). A gutter that disagrees with its text is impossible.

## LFS repositories: which git2-native operations are routed to system git and why

libgit2 has no LFS filter and `git2` 0.19 cannot register one. In an LFS
repository a native operation would (a) write the 3-line pointer into the
working tree instead of the content (checkout, reset Mixed/Hard,
fast_forward of the checked-out branch, branch_create_tracking, discard),
and (b) hash files raw against the pointer blob, so a stat-dirty LFS file
shows as Modified and a diff is an opaque binary change (status,
WorkingTreeVsIndex/IndexVsHead diffs, staging). So when `lfs::is_active`
is true those calls go to system git; commit-to-commit diffs and
`reset --soft` stay native. `is_active` is true when the root
`.gitattributes` or `info/attributes` mention `filter=lfs`, or
`<common dir>/lfs` exists; it is not cached. A repo whose patterns live
only in a nested `.gitattributes` is missed until LFS has been used once
(documented limitation). The `check_*_lfs_*` contract checks (they need
`git-lfs`) failed on `Git2Backend` before the routing existed.

A repo configured for LFS with `git-lfs` missing makes git itself fail in
a filter or hook. `process.rs::classify_lfs_failure` recognises those
messages (incl. the Windows `cannot run git-lfs`) in every `run*` helper
and returns `Error::LfsUnavailable`, so the UI shows the install link.
LFS mutations refuse up front when the probe says `git-lfs` is missing or
older than 3.0; `SystemGitBackend::without_lfs()` /
`Git2Backend::without_lfs()` force that answer in tests.

## Frontend surface matrix

An operation is finished only when the daemon carries it *and* at least
one frontend offers it; a `GitBackend` method no frontend calls is dead
code with tests. The CLI and MCP cannot drift: `gitpoulpi-commands` maps
each `Cli` subcommand to the backend once, and `gitpoulpi-mcp/src/registry.rs`
generates one MCP tool per CLI leaf (exceptions listed with reasons in
`EXCLUDED`; read-only/destructive flags in `policy.rs`). So the per-op
table this section used to hold reduces to: check `gitpoulpi-commands`'
`cli.rs` for the CLI/MCP side and the TUI's actions for the UI side.
CLI `--json` payloads are the serialized domain types.

Deliberate non-cells:

- The TUI never calls `resolve`: it holds `Oid`s from the graph. CLI/MCP
  take typed revision strings, and no frontend may run `git rev-parse`
  itself (that would read the repo outside the seam). Frontends pass
  whole `Ref`s to `merge`/`rebase`, never strings.
- `clone` and `discover` have no repo: they are dispatched before the
  daemon resolves a `RepoId`, and the CLI skips repo discovery for them.
  A TUI clone runs on its own IPC connection, reports through
  `Event::CloneProgress`, and opens as a new tab (ADR 0006).
- `clone_with_progress` has a default trait implementation (plain clone,
  one final `Done`), so the contract suite can demand the same end report
  from both backends. `SystemGitBackend` overrides it (`--progress`,
  stderr split on `\r`); the CLI uses plain `clone` (it blocks on one).

## Tags: local, remote, and how "unpushed" is known

git keeps no remote-tracking refs for tags: `git fetch` writes a remote's
tags into `refs/tags/*`, indistinguishable from local ones. The only
authoritative answer is `git ls-remote --tags`, a network round trip per
refresh, which the "input loop never waits on a git subprocess" rule and a
2 s refresh both forbid.

Chosen: a mirror namespace `refs/gitpoulpi/remote-tags/<remote>/<tag>`
(`REMOTE_TAG_MIRROR_PREFIX`) holding what the remote last said.

- Read for free: `refs()` walks it in the same ref iteration and fills
  `Ref::tag_sync`.
- Refreshed by `fetch` with its own `git fetch --prune --no-tags <remote>
  +refs/tags/*:<mirror>/*`, run **beside** the user's fetch (joined, so a
  slow remote costs one round trip of wall time, not two). It is never an
  extra refspec on the user's fetch (`-c remote.<n>.fetch=` made the two
  share one prune setting) and it always prunes the mirror, because
  command-line refspecs are the only ones `--prune` considers: the user's
  branches keep their own opt-in. A mirror failure does not fail the fetch.
- Updated by our own writes: `tag_push` / `tag_delete_remote` record the
  change in one `update-ref --stdin` transaction, so the badge clears the
  moment you push (a lingering badge reads as a failed push).
- **One writer**: `Git2Backend` reads the mirror natively but forwards
  every write (`tag_push`, `tag_delete_remote`, `remote_rename`,
  `remote_remove`); two writers is how the backends would disagree.
- `LogOptions::all_refs` means `--branches --tags --remotes` (three
  `push_glob`s in libgit2), never `--all`/`push_glob("*")`, which would
  draw the mirror in the graph. Pinned by
  `check_the_remote_tag_mirror_never_surfaces`.
- A tag counts as pushed to a remote only if the mirror ref peels to the
  same commit (else only `--force` would land it). The rule lives once,
  in `gitpoulpi_cli_backend::tag_sync_for`, which `Git2Backend` calls.
- `tag_sync: None` means "not computed"; `Ref::is_unpushed_tag` is false
  for it, so a backend that never fills it marks nothing.

- **Where a tag lives** (maintainer, 2026-10-22). `Ref::tag_sync` carries
  `pushed_to` (remotes at the same commit), `differs_on` (another commit) and
  `local` (does `refs/tags/<name>` exist). A mirror tag with no local ref is
  listed as an extra `RefKind::Tag` row with `local == false` ("remote only",
  target = the remote's commit), built by `apply_tag_sync`, which both backends
  call. Consumers that act on a *local* tag must skip `is_remote_only_tag()`
  (`tag_push` does); `TagWhere` is the one answer every surface renders.
- `FetchOptions::prune_tags` (`general.auto_prune_tags`) deletes the local tag
  only when the pruned mirror ref was its remote's, no remote still has the name
  and it still points at the commit the remote's tag did (needs `fetch
  --porcelain`, git 2.41+). Not git's `--prune-tags`, which also deletes
  never-pushed local tags.
- `tag_fetch` is `git fetch --no-tags <remote> refs/tags/x:refs/tags/x` with no
  `+`: a local tag at another commit is refused, not overwritten.
- Combined delete = `tag_delete_remote` then `tag_delete`, two requests, remote
  first (a refused push loses nothing local). Only the local half is journaled;
  Undo restores the local tag and a `tag_push` puts it back.

`tag_push` takes `TagSelection::{Named, AllUnpushed}`; `AllUnpushed` is
resolved in the backend at push time: one operation, one `git push` (one
pre-push hook run, one credential prompt, one `RepoChanged`), and tags
created since the last refresh are included. An empty selection is a
success with no refs ("everything is already pushed").
