# 0009 - Git LFS goes through the system `git-lfs` binary

## Status
Accepted (2026-09-30).

## Context
GitPoulpi has to work in repositories that store large files with Git LFS: show them sensibly, clone them, fetch / pull / push their objects, track patterns and (where the server supports it) lock files. LFS is not part of git itself. It is a set of *filters* (clean and smudge, run when a file is added or checked out), a *pre-push hook*, and a separate HTTP protocol (batch API for transfers, locking API for locks) with its own credential handling.

Survey of the options (2026-09):

- **`git-lfs` (Rust port, v0.7).** Says of itself that the upstream Go program is still "the answer" for production. No custom transfer adapters, no pure-SSH transfer, no NTLM.
- **`git-lfs-pointer`.** A small pointer parser, but edition 2024 (Rust 1.85+); the workspace pins 1.80.
- **`git-lfs-api`.** A reqwest client for the batch and locking APIs. Using it means doing the credential lookups (credential helper, SSH-based `git-lfs-authenticate`) ourselves, which is a project of its own and a second place for authentication bugs.
- **`maw-lfs`.** Tied to another tool. **gitoxide** has no LFS. **libgit2** has no LFS filter, and `git2` 0.19 does not expose `git_filter_register` (confirmed: no filter API anywhere in its source).

## Decision
1. **Every LFS mutation or network step runs the system `git-lfs`**, through `SystemGitBackend`. `Git2Backend` delegates those operations to it (`CliFallback`).
2. **The pointer format is ours.** About 80 lines of pure code in `gitpoulpi-core::domain::lfs` (`LfsPointer::parse`, `lfs_change`), no dependency.
3. **LFS *reads* need no binary.** "Is this an LFS repository", "which files are pointers", "is the object stored locally" are answered from `.gitattributes`, git config, blobs under 1024 bytes and a stat of the object store. So the UI can say "LFS object, 12 MB, not downloaded" even when `git-lfs` is missing.
4. **A missing or too-old binary is a typed state, never a crash.** `LfsBinary::{Installed, TooOld, Missing}` (minimum 3.0) is probed by `git lfs version` and cached per backend for 60 s. Operations that need it fail with `Error::LfsUnavailable`, whose message carries `LFS_INSTALL_URL` (https://git-lfs.com). No account is ever needed: LFS uses the remote's existing git credentials.
5. **git2 routing rule.** libgit2 has no smudge filter, so in an LFS repository `Git2Backend` must not run `status`, `diff` (working tree vs index, index vs HEAD), `checkout`, `reset` (mixed / hard) or `fast_forward` itself: checkout, reset and fast-forward would write *pointer text* into the working tree, and status / diff would compare the smudged file to the pointer blob and report every stat-dirty LFS file as modified. Those go to the system-git fallback whenever the repository uses LFS. The contract suite pins this.
6. **Tests stay offline.** git-lfs has a built-in `lfs-standalone-file` adapter for `file://` URLs; the testkit fixtures use a `.lfsconfig` pointing at the local bare origin. Locking needs HTTP, so the testkit carries a tiny loopback lock server.
7. **Non-goals:** a bundled or pure-Rust LFS implementation, `git lfs migrate` (history rewriting), custom transfer agents, SSH-only `git-lfs-transfer`.

## Consequences
- One extra optional runtime dependency, detected and explained rather than required.
- Progress for fetch / pull / push comes from `GIT_LFS_PROGRESS=<file>`, which keeps the "machine-readable output only" rule.
- A repository with LFS patterns only in a *nested* `.gitattributes` is not detected until it has an LFS object store (`.git/lfs`); accepted and documented.
