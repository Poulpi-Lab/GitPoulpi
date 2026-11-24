//! Pins every tool's annotations (read-only / destructive / idempotent /
//! open-world) so a new tool must be classified deliberately: adding one
//! without a row here fails `every_tool_is_pinned`.

use super::is_open_world;
use crate::{policy, registry};

/// (tool, kind, touches the network). Kinds: `read`, `write`,
/// `destructive`, `view`, `propose`.
const TOOLS: &[(&str, &str, bool)] = &[
    ("alias_status", "read", false),
    ("ai_commit_message", "read", true),
    ("ai_explain", "read", true),
    ("ai_resolve_conflict", "read", true),
    ("ai_split_plan", "read", true),
    ("ai_rewrite_script", "read", true),
    ("ai_rewrite_rules", "read", true),
    ("ai_worktree_name", "read", true),
    ("ai_hosting_reply", "read", true),
    ("ai_prompt", "read", false),
    // Shows, or with a setting flag saves, the `[ai.review]` defaults.
    ("ai_review_context", "write", false),
    ("autostash_start_over", "destructive", false),
    ("blame", "read", false),
    ("branch_create", "write", false),
    ("branch_delete", "destructive", false),
    ("branch_delete_remote", "destructive", true),
    ("branch_fast_forward", "write", false),
    ("branch_list", "read", false),
    ("branch_rename", "write", false),
    ("branch_set_upstream", "write", false),
    ("branch_track", "write", false),
    ("branch_unset_upstream", "write", false),
    ("branch_upstream_add", "write", false),
    ("branch_upstream_list", "read", false),
    ("branch_upstream_primary", "write", false),
    ("branch_upstream_pull", "write", false),
    ("branch_upstream_remove", "write", false),
    ("checkout", "write", false),
    ("cherry_pick", "write", false),
    ("clone", "write", true),
    ("commit", "write", false),
    ("config_get", "read", false),
    ("config_path", "read", false),
    ("conflicts_abort", "destructive", false),
    ("conflicts_am_abort", "destructive", false),
    ("conflicts_am_continue", "write", false),
    ("conflicts_am_skip", "destructive", false),
    ("index_lock_remove", "destructive", false),
    ("index_lock_show", "read", false),
    ("conflicts_check", "read", false),
    ("conflicts_list", "read", false),
    ("conflicts_resolve", "destructive", false),
    ("conflicts_show", "read", false),
    ("conflicts_skip", "destructive", false),
    ("conflicts_state", "read", false),
    ("conflicts_unresolve", "write", false),
    ("diff", "read", false),
    ("discard", "destructive", false),
    ("fetch", "write", true),
    ("repo_auth_show", "read", false),
    ("repo_auth_test", "write", true),
    ("hosting_board_move", "write", true),
    ("hosting_board_show", "read", true),
    ("hosting_checkout_pr", "write", true),
    ("hosting_clone", "write", true),
    ("hosting_edit_url", "read", true),
    ("hosting_integrations_add", "write", false),
    ("hosting_integrations_list", "read", false),
    ("hosting_integrations_radar", "write", false),
    ("hosting_integrations_remove", "destructive", false),
    ("hosting_integrations_test", "read", true),
    ("hosting_issue_base", "write", false),
    ("hosting_issue_close", "write", false),
    ("hosting_issue_create", "write", true),
    ("hosting_issue_edit", "write", true),
    ("hosting_labels", "read", true),
    ("hosting_members", "read", true),
    ("hosting_milestones", "read", true),
    ("hosting_pr_edit", "write", true),
    ("hosting_issue_move", "write", false),
    ("hosting_issue_options", "read", true),
    ("hosting_issue_reopen", "write", false),
    ("hosting_issues", "read", true),
    ("hosting_link", "write", false),
    ("hosting_list", "read", false),
    ("hosting_logout", "destructive", false),
    ("hosting_open", "read", false),
    ("hosting_pr", "write", true),
    ("hosting_pr_for_branch", "read", true),
    ("hosting_projects", "read", true),
    ("hosting_radar", "read", true),
    ("hosting_prs", "read", true),
    ("hosting_refs", "read", false),
    ("hosting_file", "read", true),
    ("hosting_review_comment", "write", true),
    ("hosting_review_diff", "read", true),
    ("hosting_review_fetch", "write", true),
    ("hosting_review_push", "write", true),
    ("hosting_review_submit_change", "write", true),
    ("hosting_review_abandon", "write", true),
    ("hosting_review_reply", "write", true),
    ("hosting_review_resolve", "write", true),
    ("hosting_review_show", "read", true),
    ("hosting_review_submit", "write", true),
    ("hosting_review_suggest", "read", true),
    ("hosting_review_threads", "read", true),
    ("hosting_review_conversation", "read", true),
    ("hosting_review_comment_edit", "write", true),
    ("hosting_review_comment_delete", "destructive", true),
    ("hosting_issue_comments", "read", true),
    ("hosting_issue_comment", "write", true),
    ("hosting_issue_comment_edit", "write", true),
    ("hosting_issue_comment_delete", "destructive", true),
    ("hosting_markdown", "read", false),
    ("hosting_ssh_key_delete", "destructive", true),
    ("hosting_ssh_key_list", "read", true),
    ("hosting_use", "write", false),
    ("hosting_assignment", "write", false),
    ("ignore", "write", false),
    ("mailmap_add", "write", false),
    ("mailmap_check", "read", false),
    ("mailmap_list", "read", false),
    ("job_cancel", "write", false),
    ("job_list", "read", false),
    ("job_wait", "read", false),
    ("lfs_fetch", "write", true),
    ("lfs_files", "read", false),
    ("lfs_install", "write", false),
    ("lfs_lock", "write", true),
    ("lfs_checkout", "write", false),
    ("lfs_locks", "read", true),
    ("lfs_prune", "destructive", false),
    ("lfs_pull", "write", true),
    ("lfs_push", "destructive", true),
    ("lfs_status", "read", false),
    ("lfs_track", "write", false),
    ("lfs_unlock", "destructive", true),
    ("lfs_untrack", "write", false),
    ("log", "read", false),
    ("merge", "write", false),
    ("patch_apply", "write", false),
    ("patch_create", "write", false),
    ("path_status", "read", false),
    ("pull", "write", true),
    ("push", "destructive", true),
    ("rebase", "destructive", false),
    ("rebase_plan", "read", false),
    ("redo", "destructive", false),
    ("refs_hide", "write", false),
    ("refs_show", "write", false),
    ("refs_solo", "write", false),
    ("remote_add", "write", false),
    ("remote_clear_push_url", "write", false),
    ("remote_default", "write", false),
    ("remote_group_abort", "destructive", false),
    ("remote_group_continue", "destructive", true),
    ("remote_group_plan", "read", true),
    ("remote_group_retry", "write", true),
    ("remote_group_status", "read", false),
    ("remote_list", "read", false),
    ("remote_remove", "destructive", false),
    ("remote_rename", "write", false),
    ("remote_set_url", "write", false),
    ("repo_close", "write", false),
    ("repo_discover", "read", false),
    ("repo_init", "write", false),
    ("repo_config_set", "write", false),
    ("repo_config_show", "read", false),
    ("flow_config_init", "write", false),
    ("flow_config_show", "read", false),
    ("flow_detect", "read", false),
    ("flow_import", "write", false),
    ("flow_start", "write", true),
    ("flow_finish", "destructive", true),
    ("flow_sync", "destructive", false),
    ("flow_promote", "destructive", false),
    ("flow_set_parent", "write", false),
    ("flow_issue_new", "write", true),
    ("flow_issue_wait", "write", true),
    ("flow_pick_up", "write", true),
    ("flow_status", "read", false),
    ("flow_templates", "read", false),
    ("repo_config_unset", "write", false),
    ("repo_list", "read", false),
    ("repo_open", "write", false),
    ("repo_recent", "read", false),
    ("reset", "destructive", false),
    ("revert", "write", false),
    ("reword", "destructive", false),
    ("amend", "destructive", false),
    ("apply_commit", "write", false),
    ("split_commit", "destructive", false),
    ("split_source", "read", false),
    ("rewrite_apply", "destructive", false),
    ("rewrite_discard", "read", false),
    ("rewrite_preview", "read", false),
    ("rewrite_rows", "read", false),
    ("rewrite_diff", "read", false),
    ("rewrite_script_check", "read", false),
    ("rewrite_script_test", "read", false),
    ("rewrite_run", "destructive", false),
    ("rewrite_push", "destructive", true),
    ("rewrite_purge", "destructive", false),
    ("rewrite_note", "read", false),
    ("rewrite_maps", "write", false),
    ("rewrite_tools", "read", false),
    ("rewrite_check_signing", "read", false),
    ("rewrite_analyze", "read", false),
    ("rewrite_scan_secrets", "read", false),
    ("rm_file", "destructive", false),
    ("search", "read", false),
    ("sequence_list", "read", false),
    ("sequence_run", "destructive", true),
    ("sequence_show", "read", false),
    ("sequence_validate", "read", false),
    ("session_delete", "destructive", false),
    ("session_list", "read", false),
    ("session_new", "write", false),
    ("session_rename", "write", false),
    ("show", "read", false),
    ("sparse_checkout_add", "write", false),
    ("sparse_checkout_disable", "write", false),
    ("sparse_checkout_set", "write", false),
    ("sparse_checkout_status", "read", false),
    ("ssh_command_clear", "write", false),
    ("ssh_command_set", "write", false),
    ("ssh_command_show", "read", false),
    ("stage", "write", false),
    ("stash_apply", "write", false),
    ("stash_drop", "destructive", false),
    ("stash_from_commit", "write", false),
    ("stash_list", "read", false),
    ("stash_pop", "write", false),
    ("stash_push", "write", false),
    ("status", "read", false),
    ("submodule_init", "write", false),
    ("submodule_list", "read", false),
    ("submodule_update", "write", true),
    ("tag_annotate", "write", false),
    ("tag_move", "write", false),
    ("tag_merge_branch", "write", false),
    ("tag_create", "write", false),
    ("tag_delete", "destructive", true),
    ("tag_fetch", "write", true),
    ("tag_delete_remote", "destructive", true),
    ("tag_take_remote", "write", false),
    ("tag_force_push", "destructive", true),
    ("tag_list", "read", false),
    ("tag_push", "write", true),
    ("licenses", "read", false),
    ("theme_export", "read", false),
    ("theme_list", "read", false),
    ("convert", "write", false),
    ("file_history", "read", false),
    ("treat", "write", false),
    ("ui_back", "view", false),
    ("ui_highlight", "view", false),
    ("ui_notify", "view", false),
    ("ui_propose_branch", "propose", false),
    ("ui_propose_commit_message", "propose", false),
    ("ui_propose_reword", "propose", false),
    ("ui_propose_rewrite", "propose", false),
    ("ui_propose_resolution", "propose", false),
    ("ui_propose_split", "propose", false),
    ("ui_propose_tag", "propose", false),
    ("ui_show_commit", "view", false),
    ("ui_show_conflict", "view", false),
    ("ui_show_diff", "view", false),
    ("ui_show_file", "view", false),
    ("ui_show_home", "view", false),
    ("ui_show_issue", "view", false),
    ("ui_show_keys", "view", false),
    ("ui_show_palette", "view", false),
    ("ui_show_pr", "view", false),
    ("ui_show_ref", "view", false),
    ("ui_show_search", "view", false),
    ("ui_show_settings", "view", false),
    ("ui_show_tab", "view", false),
    ("ui_show_wip", "view", false),
    ("ui_state", "view", false),
    ("ui_viewers", "view", false),
    ("lost_commits", "read", false),
    ("reflog", "read", false),
    ("reflog_restore", "destructive", false),
    ("undo", "destructive", false),
    ("undo_restore", "write", false),
    ("undo_forget", "write", false),
    ("unstage", "write", false),
    ("workspace_add", "write", false),
    ("workspace_create", "write", false),
    ("workspace_delete", "destructive", false),
    ("workspace_list", "read", false),
    ("workspace_remove", "write", false),
    ("workspace_rename", "write", false),
    ("workspace_set_ai_instructions", "write", false),
    ("workspace_set_profile", "write", false),
    ("workspace_show", "read", false),
    ("worktree_add", "write", false),
    ("worktree_changes", "read", false),
    ("worktree_remove", "destructive", false),
    ("worktree_list", "read", false),
];

fn kind(spec: &registry::ToolSpec) -> &'static str {
    if policy::is_propose(&spec.cli_path) {
        "propose"
    } else if policy::is_view(&spec.cli_path) {
        "view"
    } else if spec.read_only {
        "read"
    } else if spec.destructive {
        "destructive"
    } else {
        "write"
    }
}

#[cfg(feature = "full")]
#[test]
fn every_tool_is_pinned() {
    let specs = registry::tools();
    let mut names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
    names.sort_unstable();
    let mut pinned: Vec<&str> = TOOLS.iter().map(|t| t.0).collect();
    pinned.sort_unstable();
    assert_eq!(names, pinned, "classify new tools in annotations.rs");
}

#[test]
fn annotations_match_the_pinned_table() {
    for spec in registry::tools() {
        let (_, want_kind, want_net) = TOOLS.iter().find(|t| t.0 == spec.name).unwrap();
        assert_eq!(kind(&spec), *want_kind, "{} kind", spec.name);
        assert_eq!(
            is_open_world(&spec.name),
            *want_net,
            "{} open-world",
            spec.name
        );
        // Reads and views are repeatable; so is every destructive tool as
        // annotated today; plain writes and proposals are not claimed to be.
        let idempotent = matches!(*want_kind, "read" | "view" | "destructive");
        assert_eq!(spec.idempotent, idempotent, "{} idempotent", spec.name);
        assert_eq!(
            spec.read_only,
            matches!(*want_kind, "read" | "view"),
            "{} read-only",
            spec.name
        );
        assert_eq!(
            spec.destructive,
            *want_kind == "destructive",
            "{}",
            spec.name
        );
    }
}

/// The reads that were once mis-classed as mutations (so
/// `allow_mutations = false` blocked them) and the remote/AI tools that
/// were missing their open-world hint.
#[cfg(feature = "full")]
#[test]
fn the_audited_fixes_hold() {
    let cfg = gitpoulpi_config::McpConfig {
        allow_mutations: false,
        ..Default::default()
    };
    for name in [
        "config_get",
        "config_path",
        "rebase_plan",
        "hosting_edit_url",
    ] {
        let spec = registry::tools()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap();
        assert!(spec.read_only, "{name}");
        assert!(policy::allowed(&spec, &cfg).is_ok(), "{name}");
    }
    for name in [
        "tag_push",
        "tag_delete_remote",
        "tag_force_push",
        "tag_fetch",
        "branch_delete_remote",
        "lfs_fetch",
        "lfs_pull",
        "lfs_push",
        "ai_commit_message",
        "ai_explain",
        "ai_resolve_conflict",
        "ai_split_plan",
        "ai_rewrite_script",
        "ai_rewrite_rules",
        "ai_worktree_name",
        "ai_hosting_reply",
    ] {
        assert!(is_open_world(name), "{name}");
    }
}
