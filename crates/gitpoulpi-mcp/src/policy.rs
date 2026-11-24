//! Which tools are reads, which are destructive, and whether a mutation
//! is allowed to run at all -- `gitpoulpi.toml`'s `[mcp]` table is the
//! knob an operator turns; this module is what reads it.

use gitpoulpi_config::McpConfig;

use crate::registry::ToolSpec;

const READ_ONLY: &[&[&str]] = &[
    &["status"],
    // The uncommitted changes of a worktree: a question.
    &["worktree", "changes"],
    &["remote-group", "status"],
    // Asks the remotes (ls-remote, a selective fetch) and answers what a
    // grouped push or pull would do; changes no ref.
    &["remote-group", "plan"],
    // Pages of a reflog / of the commits no ref reaches.
    &["reflog"],
    &["lost-commits"],
    &["log"],
    &["diff"],
    &["show"],
    &["blame"],
    &["file-history"],
    &["conflicts", "check"],
    &["conflicts", "list"],
    &["conflicts", "show"],
    &["conflicts", "state"],
    &["job", "list"],
    &["job", "wait"],
    &["lfs", "status"],
    &["lfs", "files"],
    // History rewrite: the tool checks, the plan (it only keeps a temporary
    // run in the daemon), its rows, and dropping that run.
    &["rewrite", "tools"],
    &["rewrite", "check-signing"],
    &["rewrite", "scan-secrets"],
    &["rewrite", "analyze"],
    &["rewrite", "preview"],
    &["rewrite", "rows"],
    &["rewrite", "diff"],
    &["rewrite", "script", "check"],
    &["rewrite", "script", "test"],
    &["rewrite", "discard"],
    // The collaborator note of an applied run: text only.
    &["rewrite", "note"],
    &["lfs", "locks"],
    // `.mailmap`: the identities of a scope and how some resolve. `add`
    // edits the file, so it is a plain write.
    &["mailmap", "list"],
    &["mailmap", "check"],
    &["ssh-command", "show"],
    // Which identity each remote uses: no secret, no command for an agent.
    // (`repo-auth test` reaches the network, so it is gated like `fetch`.)
    &["repo-auth", "show"],
    &["repo-config", "show"],
    // Branching workflows: classification, templates, detection, the file.
    &["flow", "status"],
    &["flow", "templates"],
    &["flow", "detect"],
    &["flow", "config", "show"],
    &["sparse-checkout", "status"],
    &["repo", "discover"],
    &["repo", "recent"],
    &["hosting", "projects"],
    &["hosting", "integrations", "test"],
    &["hosting", "prs"],
    // The account-wide lists for the Radar: reads the host (and a local cache).
    &["hosting", "radar"],
    &["hosting", "pr-for-branch"],
    // Reads of a request under review: its detail and files, its threads.
    &["hosting", "review", "show"],
    &["hosting", "review", "diff"],
    &["hosting", "file"],
    &["hosting", "review", "threads"],
    // The whole conversation of an issue or a request: reads the host and
    // posts nothing. `markdown` only cleans text and finds its references.
    &["hosting", "issue", "comments"],
    &["hosting", "review", "conversation"],
    &["hosting", "markdown"],
    // Asks the model, prints the suggestions; posts nothing.
    &["hosting", "review", "suggest"],
    &["hosting", "issues"],
    // The board: the issues grouped into columns. Reads only.
    &["hosting", "board", "show"],
    // Slice M: what a project offers to assign, label and schedule with.
    &["hosting", "members"],
    &["hosting", "labels"],
    &["hosting", "milestones"],
    // What `issue create --type` / `--list` accept: a read of the project.
    &["hosting", "issue", "options"],
    // Scans a message or branch name through the repository's links: local.
    &["hosting", "refs"],
    &["ai", "commit-message"],
    &["ai", "explain"],
    &["ai", "resolve-conflict"],
    &["ai", "split-plan"],
    &["ai", "rewrite-script"],
    &["ai", "rewrite-rules"],
    &["ai", "sequence-draft"],
    &["ai", "worktree-name"],
    &["ai", "hosting-reply"],
    // Prints the layered system prompt; calls no model.
    &["ai", "prompt"],
    // They answer a question: `config get` reads one key, `config path`
    // prints where the file is, `rebase-plan` lists the commits a rebase
    // would replay, `hosting edit-url` prints a URL (its `--open` only
    // launches the user's browser).
    &["config", "get"],
    &["config", "path"],
    // `theme list` / `theme export` only print what the presets and the
    // `themes/` folder hold.
    &["alias", "status"],
    &["path", "status"],
    &["licenses"],
    // Sequences: what is saved and what a step would do; no daemon.
    &["sequence", "show"],
    &["sequence", "validate"],
    &["theme", "list"],
    &["theme", "export"],
    &["rebase-plan"],
    // Numbers a commit's changes for `split-commit`; reads only.
    &["split-source"],
    // The workspace search only reads (the daemon's index, the workspace
    // file); `workspace list` is a read by its name.
    &["search"],
    &["workspace", "show"],
    &["index-lock", "show"],
    &["hosting", "edit-url"],
    // Prints the web URL of a PR or issue (`--browser` only launches the
    // user's browser), like `edit-url`.
    &["hosting", "open"],
];

const DESTRUCTIVE: &[&[&str]] = &[
    // Removes a worktree folder (what is not saved first is lost; not undoable).
    &["worktree", "remove"],
    // Every undo is redoable, but it moves refs and HEAD back (and undoing a
    // remote branch or tag deletion pushes it back to the server): gated like
    // `reset`.
    &["undo"],
    &["redo"],
    // Moves a branch (or HEAD) to a reflog position; journaled, so `undo`
    // takes it back, but gated like `reset`.
    &["reflog-restore"],
    // Discards the stash's files' changes and undoes the operation that
    // auto-stashed (a copy is kept under `refs/gitpoulpi-backup/`).
    &["autostash", "start-over"],
    &["reset"],
    &["discard"],
    &["rm-file"],
    &["branch", "delete"],
    // Deletes a branch on the remote: server-side (`undo` pushes it back).
    &["branch", "delete-remote"],
    &["tag", "delete"],
    &["tag", "delete-remote"],
    // Replaces a remote tag with ours (leased on the last fetch; `undo` pushes
    // the old value back): server-side, so gated like `push`.
    &["tag", "force-push"],
    &["stash", "drop"],
    &["remote", "remove"],
    &["push"],
    // Runs the steps of a sequence (commit, push, ...): as destructive as
    // its most destructive step, which is a push.
    &["sequence", "run"],
    &["lfs", "push"],
    &["lfs", "unlock"],
    // Deletes local LFS objects that are not referenced by recent commits.
    &["lfs", "prune"],
    // Keeps one side of a conflict and throws the other away (and
    // `--delete` removes the file).
    &["conflicts", "resolve"],
    // Drops the whole merge / rebase / cherry-pick / revert and every
    // resolution made since.
    &["conflicts", "abort"],
    // Drops the patch `git am` stopped on / the whole `am`.
    &["conflicts", "am-skip"],
    &["conflicts", "am-abort"],
    // Deletes the lock file (guarded: only once it is stale).
    &["index-lock", "remove"],
    &["conflicts", "skip"],
    &["rebase"],
    // Drops the saved grouped run and aborts the merge or rebase it started.
    &["remote-group", "abort"],
    // Resumes the saved run: pushes to every remote, and with `--resolve
    // REMOTE=force --yes` force-pushes (with leases), like `push`.
    &["remote-group", "continue"],
    // Branching workflow actions that merge, rebase and delete (undoable,
    // except what they push or open on the host).
    &["flow", "finish"],
    &["flow", "sync"],
    &["flow", "promote"],
    &["reword"],
    &["amend"],
    &["split-commit"],
    // Moves many refs to rewritten commits (one undo entry; pushed refs
    // then need a force push the caller does separately).
    &["rewrite", "apply"],
    &["rewrite", "run"],
    // Force-pushes the rewritten refs (with leases; protected branches only
    // when named). The remote keeps the result whatever `undo` does after.
    &["rewrite", "push"],
    // Deletes the old history of a rewrite for good (and the undo of it).
    // Not even listed unless `[mcp] allow_history_purge = true`.
    &["rewrite", "purge"],
    &["hosting", "integrations", "remove"],
    &["hosting", "logout"],
    &["hosting", "ssh-key", "delete"],
    // Deletes a comment on the host for good: nothing here brings it back.
    // (Posting and editing are plain mutations: outward, but the comment can
    // be edited or deleted again.)
    &["hosting", "issue", "comment-delete"],
    &["hosting", "review", "comment-delete"],
    // Removes a named session's saved tabs and layout.
    &["session", "delete"],
    // Removes a workspace (its repositories are untouched).
    &["workspace", "delete"],
];

/// Tools an agent does not see at all unless the operator turned them on:
/// `rewrite purge` (`[mcp] allow_history_purge`).
pub fn hidden(spec: &ToolSpec, cfg: &McpConfig) -> bool {
    spec.cli_path == ["rewrite", "purge"] && !cfg.allow_history_purge
}

/// A tool is read-only when it is explicitly listed, or when its last
/// path segment is `list` (`branch list`, `tag list`, `worktree list`,
/// `stash list`, `remote list`, `session list`, `repo list`, `hosting
/// list`, `hosting ssh-key list`, ...) -- every `*_list` leaf answers a
/// question, none of them changes anything.
pub fn is_read_only(cli_path: &[String]) -> bool {
    // The `ui` tools show something to the user. They change neither the
    // repository nor GitPoulpi's saved state, so they are annotated as
    // reads and `allow_mutations = false` does not block them; their own
    // gate is `allow_ui` / `ui_control` (see `allowed`).
    if is_view(cli_path) {
        // A proposal is not a read: it puts a change in front of the user
        // (who still has to confirm it), so it is annotated honestly.
        return !is_propose(cli_path);
    }
    if cli_path.last().map(String::as_str) == Some("list") {
        return true;
    }
    READ_ONLY.iter().any(|p| path_eq(p, cli_path))
}

/// The view tools (`gitpoulpi ui ...`): an agent moving the user's window.
pub fn is_view(cli_path: &[String]) -> bool {
    cli_path.first().map(String::as_str) == Some("ui")
}

/// The propose tools (`gitpoulpi ui propose ...`, `ui sequence run`): a
/// dialog or banner opened for the user to confirm. Their own gate is
/// `allow_ui_propose`, on top of `allow_ui` / `ui_control`.
pub fn is_propose(cli_path: &[String]) -> bool {
    is_view(cli_path)
        && matches!(
            cli_path.get(1).map(String::as_str),
            Some("propose") | Some("sequence")
        )
}

/// `ui state` and `ui viewers` only *read* what the window shows, so
/// `allow_ui = false` leaves them available.
pub fn is_view_query(cli_path: &[String]) -> bool {
    is_view(cli_path)
        && matches!(
            cli_path.get(1).map(String::as_str),
            Some("state") | Some("viewers")
        )
}

pub fn is_destructive(cli_path: &[String]) -> bool {
    DESTRUCTIVE.iter().any(|p| path_eq(p, cli_path))
}

fn path_eq(a: &[&str], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

/// Whether `spec` may run given `cfg` (`gitpoulpi.toml`'s `[mcp]` table):
/// a read is always allowed; a mutation needs `allow_mutations = true`,
/// and - when `mutation_allowlist` is non-empty - needs its own name on
/// that list.
pub fn allowed(spec: &ToolSpec, cfg: &McpConfig) -> Result<(), String> {
    if hidden(spec, cfg) {
        return Err(format!(
            "`{}` is not available: it deletes the old history of a rewrite, the undo of it \
             and its reflogs for good, so it only exists when mcp.allow_history_purge = true \
             in config.toml",
            spec.name
        ));
    }
    if is_view(&spec.cli_path) && !is_view_query(&spec.cli_path) {
        if !cfg.allow_ui {
            return Err(format!(
                "ui tools are disabled (mcp.allow_ui = false in config.toml) - `{}` would \
                 move the user's view",
                spec.name
            ));
        }
        if cfg.ui_control == gitpoulpi_core::domain::UiControl::Off {
            return Err(format!(
                "ui control is off (mcp.ui_control = off, or --ui-control off) - `{}` would \
                 move the user's view",
                spec.name
            ));
        }
    }
    if is_propose(&spec.cli_path) {
        if !cfg.allow_ui_propose {
            return Err(format!(
                "proposals are disabled (mcp.allow_ui_propose = false in config.toml) - `{}` \
                 would open a pre-filled dialog for the user to confirm",
                spec.name
            ));
        }
        // Nothing changes until the user confirms in the dialog, so
        // `allow_mutations` does not apply: the change is the user's.
        return Ok(());
    }
    if spec.read_only {
        return Ok(());
    }
    if !cfg.allow_mutations {
        return Err(format!(
            "mutations are disabled (mcp.allow_mutations = false in config.toml) - \
             `{}` is a mutation",
            spec.name
        ));
    }
    if !cfg.mutation_allowlist.is_empty() && !cfg.mutation_allowlist.iter().any(|n| n == &spec.name)
    {
        return Err(format!(
            "`{}` is not on mcp.mutation_allowlist in config.toml",
            spec.name
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    fn spec(name: &str) -> ToolSpec {
        registry::tools()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("no such tool: {name}"))
    }

    #[cfg_attr(not(feature = "full"), allow(dead_code))]
    fn path(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    /// Every CLI leaf is exactly one of read-only, destructive or a plain
    /// mutation, and the hand-written lists name only leaves that exist
    /// (so a renamed subcommand cannot silently drop out of the gating).
    #[cfg(feature = "full")]
    #[test]
    fn every_cli_leaf_is_classified_exactly_once() {
        let tools = registry::tools();
        for t in &tools {
            assert!(
                !(t.read_only && t.destructive),
                "{} is both read-only and destructive",
                t.name
            );
        }

        let leaves: Vec<Vec<String>> = tools.iter().map(|t| t.cli_path.clone()).collect();
        for (list, name) in [(READ_ONLY, "READ_ONLY"), (DESTRUCTIVE, "DESTRUCTIVE")] {
            for (i, entry) in list.iter().enumerate() {
                let entry_path = path(entry);
                assert!(
                    leaves.contains(&entry_path),
                    "{name} lists {entry:?}, which is not an exposed CLI leaf"
                );
                assert!(
                    list.iter()
                        .skip(i + 1)
                        .all(|other| !path_eq(other, &entry_path)),
                    "{name} lists {entry:?} twice"
                );
            }
        }
        for entry in READ_ONLY {
            assert!(
                !is_destructive(&path(entry)),
                "{entry:?} is in both READ_ONLY and DESTRUCTIVE"
            );
        }

        let mutating: Vec<String> = tools
            .iter()
            .filter(|t| !t.read_only && !t.destructive)
            .map(|t| t.name.clone())
            .collect();
        for name in ["commit", "stage", "branch_create", "hosting_use"] {
            assert!(
                mutating.iter().any(|m| m == name),
                "{name} is a plain mutation"
            );
        }
        for t in &tools {
            let classes = [t.read_only, t.destructive, mutating.contains(&t.name)];
            assert_eq!(
                classes.iter().filter(|c| **c).count(),
                1,
                "{} must be in exactly one class",
                t.name
            );
        }
    }

    /// The conversation tools: reading is allowed with mutations off, posting
    /// and editing are plain mutations, deleting a comment is destructive.
    #[test]
    fn the_conversation_tools_are_classified_by_what_they_do() {
        for name in [
            "hosting_issue_comments",
            "hosting_review_conversation",
            "hosting_markdown",
        ] {
            assert!(spec(name).read_only, "{name} only reads");
            assert!(!spec(name).destructive, "{name} deletes nothing");
        }
        for name in [
            "hosting_issue_comment",
            "hosting_issue_comment_edit",
            "hosting_review_comment_edit",
        ] {
            assert!(!spec(name).read_only, "{name} writes to the host");
            assert!(!spec(name).destructive, "{name} can be edited again");
        }
        for name in [
            "hosting_issue_comment_delete",
            "hosting_review_comment_delete",
        ] {
            assert!(spec(name).destructive, "{name} deletes for good");
        }
        let denied = McpConfig {
            allow_mutations: false,
            ..McpConfig::default()
        };
        assert!(allowed(&spec("hosting_issue_comments"), &denied).is_ok());
        assert!(allowed(&spec("hosting_issue_comment"), &denied).is_err());
        assert!(allowed(&spec("hosting_review_comment_delete"), &denied).is_err());
    }

    /// Undo and redo move refs and HEAD: mutations, gated like `reset`.
    #[test]
    fn undo_and_redo_are_destructive_mutations() {
        for name in ["undo", "redo"] {
            assert!(!spec(name).read_only, "{name} is a mutation");
            assert!(spec(name).destructive, "{name} is gated like reset");
        }
        let denied = McpConfig {
            allow_mutations: false,
            ..McpConfig::default()
        };
        assert!(allowed(&spec("undo"), &denied).is_err());
    }

    /// The safety net's tools: the reflog and the lost-commit scan are reads
    /// (allowed with `allow_mutations = false`), "restore only this" is a
    /// plain mutation (it only adds), moving a ref to a reflog position is
    /// destructive like `reset`.
    #[test]
    fn the_undo_safety_net_tools_are_classified() {
        let denied = McpConfig {
            allow_mutations: false,
            ..McpConfig::default()
        };
        for name in ["reflog", "lost_commits"] {
            assert!(spec(name).read_only, "{name} is a read");
            assert!(allowed(&spec(name), &denied).is_ok(), "{name}");
        }
        let restore = spec("undo_restore");
        assert!(!restore.read_only && !restore.destructive);
        assert!(allowed(&restore, &denied).is_err());
        let granted = McpConfig {
            allow_mutations: true,
            ..McpConfig::default()
        };
        assert!(allowed(&restore, &granted).is_ok());
        let moved = spec("reflog_restore");
        assert!(!moved.read_only && moved.destructive);
        assert!(allowed(&moved, &denied).is_err());
    }

    /// Pin the exact destructive set, so a new tool or a dropped entry is a
    /// visible diff, and name the server-side deletions explicitly.
    #[cfg(feature = "full")]
    #[test]
    fn the_destructive_set_is_pinned() {
        for name in [
            "branch_delete",
            "branch_delete_remote",
            "tag_delete",
            "tag_delete_remote",
            "tag_force_push",
            "lfs_prune",
            "lfs_push",
            "lfs_unlock",
            "conflicts_resolve",
            "conflicts_abort",
            "conflicts_skip",
            "discard",
            "reset",
            "rm_file",
            "push",
            "remote_group_abort",
            "remote_group_continue",
            "undo",
            "redo",
            "reflog_restore",
            "autostash_start_over",
            "stash_drop",
            "remote_remove",
            "rebase",
            "reword",
            "amend",
            "split_commit",
            "rewrite_apply",
            "rewrite_run",
            "rewrite_push",
            "rewrite_purge",
            "sequence_run",
            "session_delete",
            "workspace_delete",
            "worktree_remove",
        ] {
            let s = spec(name);
            assert!(s.destructive, "{name} must be destructive");
            assert!(!s.read_only, "{name} must not be read-only");
        }
        let mut got: Vec<String> = registry::tools()
            .into_iter()
            .filter(|s| s.destructive)
            .map(|s| s.name)
            .collect();
        got.sort();
        assert_eq!(got.len(), 46, "destructive set changed: {got:?}");
    }

    #[cfg(feature = "full")]
    #[test]
    fn hosting_reads_and_mutations_are_classified() {
        for name in [
            "hosting_integrations_list",
            "hosting_integrations_test",
            "hosting_prs",
            "hosting_radar",
            "hosting_pr_for_branch",
            "hosting_review_show",
            "hosting_review_diff",
            "hosting_file",
            "hosting_review_threads",
            "hosting_issues",
            "hosting_issue_options",
            "hosting_members",
            "hosting_labels",
            "hosting_milestones",
            "hosting_refs",
            "hosting_open",
        ] {
            assert!(spec(name).read_only, "{name} should be read-only");
        }
        for name in [
            "hosting_integrations_add",
            "hosting_integrations_remove",
            "hosting_integrations_radar",
            "hosting_logout",
            "hosting_use",
            "hosting_issue_create",
            "hosting_issue_edit",
            "hosting_pr_edit",
            "hosting_ssh_key_delete",
        ] {
            assert!(!spec(name).read_only, "{name} should be a mutation");
        }
        for name in [
            "hosting_integrations_remove",
            "hosting_logout",
            "hosting_ssh_key_delete",
        ] {
            assert!(spec(name).destructive, "{name} should be destructive");
        }
    }

    /// A secret must not pass through an agent's tool-call transcript.
    #[test]
    fn login_is_not_a_tool() {
        assert!(registry::tools().iter().all(|s| s.name != "hosting_login"));
        assert!(registry::excluded_reason(&["hosting".to_string(), "login".to_string()]).is_some());
    }

    #[test]
    fn reads_always_allowed() {
        let cfg = McpConfig {
            allow_mutations: false,
            ..McpConfig::default()
        };
        assert!(allowed(&spec("status"), &cfg).is_ok());
    }

    #[test]
    fn mutations_refused_when_disallowed() {
        let cfg = McpConfig {
            allow_mutations: false,
            ..McpConfig::default()
        };
        assert!(allowed(&spec("commit"), &cfg).is_err());
    }

    #[test]
    fn allowlist_limits_mutations() {
        let cfg = McpConfig {
            allow_mutations: true,
            mutation_allowlist: vec!["commit".to_string()],
            ..McpConfig::default()
        };
        assert!(allowed(&spec("commit"), &cfg).is_ok());
        assert!(allowed(&spec("push"), &cfg).is_err());
    }

    #[test]
    fn an_empty_allowlist_means_every_mutation() {
        let cfg = McpConfig {
            allow_mutations: true,
            mutation_allowlist: Vec::new(),
            ..McpConfig::default()
        };
        assert!(allowed(&spec("commit"), &cfg).is_ok());
        assert!(allowed(&spec("push"), &cfg).is_ok());
    }

    /// Every `ui` leaf is in the view class: annotated read-only (never a
    /// mutation, never destructive), and ungated by `allow_mutations`.
    #[test]
    fn every_ui_tool_is_view_only() {
        let ui: Vec<ToolSpec> = registry::tools()
            .into_iter()
            .filter(|t| t.cli_path.first().map(String::as_str) == Some("ui"))
            .collect();
        let names: Vec<&str> = ui.iter().map(|t| t.name.as_str()).collect();
        for expected in [
            "ui_viewers",
            "ui_state",
            "ui_show_commit",
            "ui_show_diff",
            "ui_show_file",
            "ui_show_settings",
            "ui_show_tab",
            "ui_notify",
            "ui_back",
            "ui_highlight",
        ] {
            assert!(
                names.contains(&expected),
                "{expected} missing from {names:?}"
            );
        }
        let no_mutations = McpConfig {
            allow_mutations: false,
            mutation_allowlist: vec!["commit".into()],
            ..McpConfig::default()
        };
        for t in ui.iter().filter(|t| !is_propose(&t.cli_path)) {
            assert!(is_view(&t.cli_path), "{}", t.name);
            assert!(t.read_only && !t.destructive && t.idempotent, "{}", t.name);
            assert!(allowed(t, &no_mutations).is_ok(), "{}", t.name);
        }
    }

    #[test]
    fn allow_ui_false_refuses_show_tools_but_not_the_queries() {
        let cfg = McpConfig {
            allow_ui: false,
            ..McpConfig::default()
        };
        for name in [
            "ui_show_commit",
            "ui_show_diff",
            "ui_notify",
            "ui_back",
            "ui_highlight",
        ] {
            let err = allowed(&spec(name), &cfg).unwrap_err();
            assert!(err.contains("allow_ui"), "{name}: {err}");
        }
        for name in ["ui_state", "ui_viewers"] {
            assert!(allowed(&spec(name), &cfg).is_ok(), "{name}");
        }
        // Non-ui reads are unaffected.
        assert!(allowed(&spec("status"), &cfg).is_ok());
    }

    #[test]
    fn ui_control_off_refuses_show_tools() {
        let cfg = McpConfig {
            ui_control: gitpoulpi_core::domain::UiControl::Off,
            ..McpConfig::default()
        };
        assert!(allowed(&spec("ui_show_file"), &cfg).is_err());
        assert!(allowed(&spec("ui_state"), &cfg).is_ok());
        assert!(allowed(&spec("ui_show_file"), &McpConfig::default()).is_ok());
    }

    #[test]
    fn propose_tools_are_their_own_gated_class() {
        let names = [
            "ui_propose_reword",
            "ui_propose_rewrite",
            "ui_propose_commit_message",
            "ui_propose_branch",
            "ui_propose_tag",
        ];
        for name in names {
            let s = spec(name);
            assert!(is_propose(&s.cli_path), "{name}");
            assert!(!s.read_only && !s.destructive && !s.idempotent, "{name}");
            // Off by default.
            let err = allowed(&s, &McpConfig::default()).unwrap_err();
            assert!(err.contains("allow_ui_propose"), "{name}: {err}");
        }
        let on = McpConfig {
            allow_ui_propose: true,
            allow_mutations: false,
            ..McpConfig::default()
        };
        for name in names {
            assert!(
                allowed(&spec(name), &on).is_ok(),
                "{name}: the user confirms"
            );
        }
        // `allow_ui` and `ui_control` still apply on top.
        for cfg in [
            McpConfig {
                allow_ui: false,
                ..on.clone()
            },
            McpConfig {
                ui_control: gitpoulpi_core::domain::UiControl::Off,
                ..on.clone()
            },
        ] {
            assert!(allowed(&spec("ui_propose_branch"), &cfg).is_err());
        }
    }
}
