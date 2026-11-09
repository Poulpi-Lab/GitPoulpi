//! Generic assertions run against `&dyn GitBackend`, one function per
//! behavior, so the same suite can be pointed at `SystemGitBackend` and
//! `Git2Backend` to catch behavioral divergence between them. Each
//! function builds its own fixture repo so cases stay independent.

use gitpoulpi_core::domain::{LogOptions, Oid, RebaseAction, RebaseStep};
use gitpoulpi_core::GitBackend;

mod cancel;
mod compare;
mod conflicts;
mod content;
mod count;
mod file_history;
mod history;
mod journal_plumbing;
mod lfs;
mod log_hidden;
mod log_refs;
mod mailmap;
mod outside;
mod predict;
mod pull_split;
mod reflog;
mod remotes_network;
mod rewrite;
mod rewrite_analyze;
mod rewrite_content;
mod rewrite_encoding;
mod rewrite_lfs;
mod rewrite_paths;
mod rewrite_purge;
mod rewrite_push;
mod rewrite_scopes;
mod rewrite_script;
mod rewrite_script_external;
mod rewrite_secrets;
mod rewrite_signatures;
mod rewrite_textconv;
mod rewrite_trailers;
mod search;
mod sparse;
mod split;
mod staging;
mod stash;
mod stuck_states;
mod tag_where;
mod workflow;

pub use cancel::*;
pub use compare::*;
pub use conflicts::*;
pub use content::*;
pub use count::*;
pub use file_history::*;
pub use history::*;
pub use journal_plumbing::*;
pub use lfs::*;
pub use log_hidden::*;
pub use log_refs::*;
pub use mailmap::*;
pub use outside::*;
pub use predict::*;
pub use pull_split::*;
pub use reflog::*;
pub use remotes_network::*;
pub use rewrite::*;
pub use rewrite_analyze::*;
pub use rewrite_content::*;
pub use rewrite_encoding::*;
pub use rewrite_lfs::*;
pub use rewrite_paths::*;
pub use rewrite_purge::*;
pub use rewrite_push::*;
pub use rewrite_scopes::*;
pub use rewrite_script::*;
pub use rewrite_script_external::*;
pub use rewrite_secrets::*;
pub use rewrite_signatures::*;
pub use rewrite_textconv::*;
pub use rewrite_trailers::*;
pub use search::*;
pub use sparse::*;
pub use split::*;
pub use staging::*;
pub use stash::*;
pub use stuck_states::*;
pub use tag_where::*;
pub use workflow::*;

fn head_oid(backend: &dyn GitBackend, repo: &gitpoulpi_core::domain::RepoRef) -> Oid {
    backend.log(repo, &LogOptions::default()).unwrap().nodes[0].id
}

/// The single list of every check in this module and its submodules, so
/// there is exactly one place to add a check. `contract_tests!` in `gitpoulpi-git2-backend/tests/contract.rs` expands
/// it into one `#[test]` per check per backend so cargo's harness runs
/// them in parallel (a single sequential test serialized every fixture
/// build, which is slow where spawning `git` is expensive, as on Windows).
///
/// `every_check_is_listed` (bottom of this file) fails loudly if a
/// `pub fn check_*` is added to a submodule and not to this list.
#[macro_export]
macro_rules! for_each_contract_check {
    ($callback:ident) => {
        $callback! {
            check_status_and_log,
            check_commit_message_body,
            check_log_all_refs_includes_every_branch_tip,
            check_log_skip_pages_history,
            check_log_hidden_drops_commits_only_hidden_refs_reach,
            check_log_hidden_keeps_head_even_if_its_branch_is_hidden,
            check_log_hidden_remote_prefix_covers_nested_and_later_branches,
            check_log_hidden_ignores_a_remote_head_symref,
            check_log_hidden_keep_overrides,
            check_log_hidden_pages_tile,
            check_log_hidden_peels_annotated_tags_and_skips_non_commit_tags,
            check_count_commits_honours_hidden,
            check_empty_filter_is_the_all_refs_walk,
            check_count_commits_matches_the_walk_the_graph_loads,
            check_head_ref_flag,
            check_tag_target_resolves_to_the_tagged_commit,
            check_refs_survive_a_ref_to_a_missing_object,
            check_log_hidden_survives_a_ref_to_a_missing_object,
            check_commit_messages_survive_a_ref_to_a_missing_object,
            check_rewrite_analyze_survives_a_ref_to_a_missing_object,
            check_diff_working_tree_vs_index,
            check_diff_of_a_commit_is_against_its_first_parent,
            check_diff_files_lists_exactly_what_diff_lists,
            check_diff_restricted_to_paths,
            check_diff_files_working_tree_lists_what_diff_lists,
            check_compare_two_commits_both_ways,
            check_compare_a_span_starts_at_the_oldests_parent,
            check_compare_a_commit_with_the_working_tree,
            check_compare_a_stash_with_a_commit,
            check_commit_stats_count_files_and_lines_against_the_first_parent,
            check_commit_messages_cover_every_ref_and_split_subject_from_body,
            check_commit_messages_with_known_tips_return_only_new_commits,
            check_commit_messages_after_a_rewrite_replace_everything,
            check_commit_messages_limit_and_unborn_repository,
            check_stage_and_commit,
            check_commit_honours_an_author_override,
            check_commit_runs_hooks_unless_no_verify,
            check_survives_output_reshaping_config,
            check_branch_create_checkout_delete,
            check_discard_several_paths_at_once,
            check_discard_with_no_paths_is_a_harmless_no_op,
            check_stage_all_matches_git_add_dash_a,
            check_stage_and_unstage_file_and_folder,
            check_unstage_all_on_an_unborn_branch,
            check_discard_a_mixed_batch,
            check_discard_hunk_reverts_only_that_hunk,
            check_discard_lines_spanning_adds_and_deletes,
            check_discard_lines_at_the_file_edges,
            check_discard_part_keeps_line_endings_exactly,
            check_discard_part_reverts_to_the_index_and_refuses_an_unknown_hunk,
            check_unstage_one_line_of_a_hunk_with_several_changes,
            check_stage_one_change_of_a_hunk_with_several_changes,
            check_stage_and_discard_parts_of_an_untracked_file,
            check_tag_create_delete,
            check_reset_hard,
            check_stash_push_pop,
            check_stash_push_with_nothing_to_stash_is_none,
            check_stash_apply_keeps_the_stash,
            check_stash_push_paths_stashes_only_those_paths,
            check_merge_fast_forward,
            check_resolve_revision_spellings,
            check_merge_base_is_the_fork_point,
            check_ahead_behind_counts_both_sides_of_a_fork,
            check_predict_merge_conflicts_names_the_files_a_merge_would_conflict_on,
            check_predict_merge_conflicts_is_empty_when_the_merge_goes_through,
            check_config_entries_lists_local_keys_under_prefixes,
            check_merge_no_ff_and_squash_shape_the_result,
            check_merge_squash_without_commit_stages_the_changes,
            check_a_recorded_parent_is_a_branch_config_key,
            check_revert_creates_an_inverse_commit,
            check_revert_of_a_merge_commit_takes_the_mainline_parent,
            check_cherry_pick_copies_a_commit_onto_head,
            check_commit_concludes_a_conflicted_merge,
            check_commit_concludes_a_conflicted_cherry_pick,
            check_commit_concludes_a_conflicted_revert,
            check_rebase_replays_onto_the_target,
            check_rebase_plan_lists_the_range_oldest_first,
            check_rebase_plan_refuses_a_merge_in_range,
            check_rebase_interactive_reorders_and_drops,
            check_rebase_interactive_squash_concatenates_and_fixup_keeps_first_message,
            check_rebase_interactive_reword_normalizes_crlf_and_keeps_hash_lines,
            check_rebase_interactive_reword_receiving_squash_keeps_exactly_its_message,
            check_cherry_pick_plan_reword_receiving_squash_keeps_exactly_its_message,
            check_rebase_interactive_edit_stops_then_continue_completes,
            check_rebase_interactive_conflict_resolve_then_continue,
            check_rebase_interactive_conflict_then_skip,
            check_rebase_abort_restores_branch_and_head,
            check_rebase_interactive_from_root,
            check_cherry_pick_plan_applies_reordered_and_squashed,
            check_cherry_pick_plan_conflict_then_continue,
            check_cherry_pick_plan_conflict_then_abort,
            check_cherry_pick_plan_refuses_duplicates_and_leading_squash,
            check_rebase_interactive_dropping_everything_lands_on_upstream,
            check_rebase_interactive_refusals_are_typed_and_change_nothing,
            check_rebase_continue_without_rebase_is_a_typed_refusal,
            check_rebase_interactive_ignores_user_editor_and_autosquash_config,
            check_rebase_interactive_in_a_path_with_spaces,
            check_remote_add_list_remove,
            check_conflicts_are_listed_and_resolvable,
            check_conflict_entries_report_kind_and_stage_oids,
            check_conflict_entries_are_sorted_and_empty_without_conflicts,
            check_merge_document_regions_match_the_worktree_git_wrote,
            check_merge_document_counts_every_conflict_of_a_file,
            check_merge_document_survives_marker_lookalikes,
            check_merge_document_composes_back_to_the_side_files,
            check_merge_document_add_add_has_no_base,
            check_merge_document_delete_modify_keeps_only_the_surviving_side,
            check_merge_document_binary_and_lfs_have_no_regions,
            check_read_blob_bytes_of_a_conflict_stage,
            check_merge_document_too_large,
            check_merge_document_symlink_mode_only_and_submodule_have_no_text_merge,
            check_merge_document_decodes_with_fallback_encoding,
            check_merge_document_labels_for_merge_cherry_pick_rebase,
            check_merge_document_reports_the_worktree_state,
            check_merge_document_of_a_path_that_is_not_conflicted_is_an_error,
            check_merge_document_aligns_the_panes_with_the_real_files,
            check_resolve_with_content_preserves_crlf_and_bom,
            check_resolve_without_stage_keeps_the_path_conflicted,
            check_resolve_refuses_stale_expected_oids_and_changes_nothing,
            check_resolve_with_encoded_content_writes_that_encodings_bytes,
            check_take_side_on_delete_modify,
            check_resolve_delete,
            check_take_side_on_submodule,
            check_take_side_keeps_symlink_and_exec_bit,
            check_unresolve_restores_stages_and_markers,
            check_unstage_one_path_during_a_merge_keeps_the_other_conflict_stages,
            check_snapshot_restore_replaces_a_conflicted_path_with_its_resolution,
            check_operation_state_and_abort,
            check_operation_abort_without_operation_is_a_typed_refusal,
            check_operation_abort_survives_a_stale_index_entry,
            check_operation_abort_names_files_with_local_changes,
            check_am_in_progress_is_its_own_operation_state,
            check_am_abort_restores_the_branch_tip,
            check_am_skip_drops_the_stopped_patch_and_goes_on,
            check_am_continue_after_resolving_applies_the_patch,
            check_am_without_an_am_is_a_typed_refusal,
            check_index_lock_is_detected_when_old_and_removable,
            check_index_lock_of_a_linked_worktree_is_the_worktrees_own,
            check_commit_concludes_a_merge_whose_result_equals_head,
            check_operation_skip_ends_an_empty_cherry_pick_or_revert,
            check_operation_skip_refusals,
            check_reflog_shows_moves_made_inside_and_outside,
            check_reflog_pages,
            check_reflog_and_lost_commits_in_a_reftable_repository,
            check_reflog_of_a_deleted_or_unknown_ref_is_empty,
            check_reflog_records_backend_resets_and_checkouts,
            check_outside_diff_of_an_unchanged_repo_is_quiet,
            check_outside_diff_names_commit_branch_delete_tag_delete_and_reset,
            check_outside_diff_of_a_rebase_is_one_change,
            check_outside_diff_folds_a_burst_of_commits_into_one_change,
            check_outside_diff_joins_a_checkout_with_the_branch_it_created,
            check_outside_diff_sees_a_detached_head_move,
            check_outside_diff_sees_stashes_pushed_and_dropped,
            check_outside_diff_counts_fetches_without_reporting_them,
            check_outside_diff_spans_many_kinds_of_moves_at_once,
            check_lost_commits_none_in_a_clean_repo,
            check_lost_commits_after_a_branch_delete,
            check_lost_commits_pinned_and_dangling,
            check_lost_commits_respects_a_tiny_budget,
            check_blob_bytes_reads_a_blob_by_id,
            check_ref_snapshot_reports_raw_tag_objects,
            check_ref_snapshot_reports_head_unborn_and_detached,
            check_update_refs_is_all_or_nothing,
            check_update_refs_refuses_pin_to_missing_object,
            check_update_refs_set_head,
            check_update_refs_set_symbolic,
            check_move_head_keep_cas,
            check_move_head_keep_refuses_overwrite,
            check_refs_hide_gitpoulpi_namespace,
            check_log_all_refs_skips_gitpoulpi_namespace,
            check_config_section_roundtrip,
            check_remote_config_sections_and_dependents,
            check_stash_store_and_paths,
            check_index_tree_and_restore,
            check_snapshot_round_trips_untracked_binary_exec_and_links,
            check_snapshot_touches_neither_index_nor_worktree,
            check_snapshot_restore_removes_paths_absent_in_the_snapshot,
            check_snapshot_index_and_worktree_sides_restore_independently,
            check_path_hashes_match_the_snapshot_and_notice_changes,
            check_delete_files_and_patch_files,
            check_convert_files,
            check_gitignore_add,
            check_gitignore_add_into_a_chosen_file,
            check_gitattributes_treat_binary_and_text,
            check_a_symlinked_root_behaves_like_the_real_one,
            check_read_blob_text_binary_and_line_endings,
            check_non_utf8_content_decodes_with_the_fallback_encoding,
            check_non_utf8_commit_messages_decode_for_display,
            check_read_blob_distinguishes_commit_index_and_worktree,
            check_read_blob_of_an_image_carries_its_header,
            check_read_blob_bytes_are_exact_for_commit_index_and_worktree,
            check_read_blob_bytes_respects_the_cap_and_missing_paths,
            check_read_blob_bytes_of_an_lfs_pointer_carries_the_local_object,
            check_log_filtered_by_path,
            check_blame_attributes_lines_to_their_last_commit,
            check_blame_at_an_older_revision,
            check_blame_reports_binary_files,
            check_file_history_lists_a_files_commits_with_stats,
            check_file_history_follows_a_rename_with_changes,
            check_file_history_across_delete_and_readd,
            check_file_history_of_merges_follows_git_log,
            check_file_history_pages,
            check_file_history_binary_files_have_no_line_counts,
            check_file_history_starts_at_the_given_commit,
            check_blame_follows_renames_and_reports_previous,
            check_read_blob_reports_an_lfs_pointer_at_commit_and_index,
            check_diff_of_a_pointer_change_is_recognized,
            check_list_tree_returns_every_path_at_the_revision,
            check_list_tree_is_answered_at_the_revision_asked_for,
            check_list_tree_uses_forward_slashes_for_nested_paths,
            check_list_tree_rejects_an_unknown_revision,
            check_discover_from_the_root_and_from_inside_it,
            check_discover_rejects_what_is_not_a_repository,
            check_discover_recognises_a_linked_worktree,
            check_discover_reports_a_detached_head_as_no_branch,
            check_init_creates_a_repository_discover_agrees_with,
            check_init_honours_the_initial_branch_name,
            check_init_refuses_a_path_that_is_already_a_repository,
            check_clone_produces_a_usable_repository,
            check_clone_reports_progress_that_ends_at_done,
            check_clone_honours_the_branch_option,
            check_a_rejected_push_carries_the_command_and_stderr,
            check_a_failed_network_operation_carries_detail_without_secrets,
            check_sparse_clone_checks_out_only_the_chosen_directories,
            check_sparse_set_adds_and_removes_directories,
            check_sparse_set_enables_it_on_a_full_checkout,
            check_sparse_disable_restores_everything,
            check_sparse_status_of_a_plain_repo_is_disabled,
            check_sparse_clone_without_directories_is_refused,
            check_refs_report_a_local_branchs_upstream,
            check_refs_split_a_remote_whose_name_contains_a_slash,
            check_branch_create_tracking_creates_sets_upstream_and_checks_out,
            check_branch_create_tracking_refuses_an_existing_local_name,
            check_a_new_tag_is_reported_as_unpushed,
            check_tag_push_sends_one_tag_and_clears_its_marker,
            check_tag_push_all_unpushed_sends_every_one,
            check_a_fetch_learns_the_tags_the_remote_already_had,
            check_tag_delete_remote_keeps_the_local_tag,
            check_a_tag_reports_where_it_lives,
            check_tag_delete_scopes_are_independent,
            check_a_fetch_brings_a_tag_back_unless_deleted_on_the_remote,
            check_tag_fetch_brings_one_remote_only_tag,
            check_a_fetch_notices_a_tag_deleted_on_the_remote,
            check_branch_delete_remote_removes_it_from_the_remote,
            check_tag_delete_is_local_only,
            check_tag_annotate_rewrites_a_tag_in_place,
            check_tag_annotate_refuses_an_unknown_tag,
            check_tag_push_refuses_an_unknown_tag,
            check_the_remote_tag_mirror_never_surfaces,
            check_tag_sync_survives_a_remote_whose_name_contains_a_slash,
            check_remote_set_url_and_push_url,
            check_ssh_command_is_set_and_cleared_per_repo,
            check_repo_config_sets_unsets_and_reads_back,
            check_repo_config_key_sections_roundtrip_for_undo,
            check_repo_config_writes_the_commit_identity,
            check_repo_config_reads_and_writes_the_commit_template,
            check_remote_rename_carries_branches_upstreams_and_tag_mirror,
            check_remote_remove_forgets_its_tag_mirror,
            check_remote_add_refuses_a_taken_name,
            check_option_lookalike_and_helper_urls_are_refused,
            check_branch_set_upstream_sets_and_unsets,
            check_fetch_all_reaches_every_remote_and_prunes,
            check_a_canceled_fetch_kills_git_and_removes_only_its_own_locks,
            check_a_canceled_push_kills_git,
            check_a_canceled_fetch_kills_the_helpers_git_started,
            check_a_canceled_clone_removes_the_destination_it_created,
            check_a_canceled_clone_keeps_a_destination_that_existed,
            check_fetch_ref_as_branch_tracks_a_pull_ref,
            check_fetch_refspec_lands_in_the_gitpoulpi_namespace,
            check_push_to_a_chosen_remote_sets_the_upstream,
            check_push_for_review_pushes_head_to_the_review_ref,
            check_a_diverged_push_is_a_typed_rejection,
            check_force_push_is_always_with_lease,
            check_pull_modes_on_a_diverged_branch,
            check_pull_fast_forwards_and_accepts_an_explicit_remote,
            check_a_split_pull_matches_the_one_step_pull_when_behind,
            check_a_split_pull_matches_the_one_step_pull_when_diverged,
            check_a_split_pull_matches_the_one_step_pull_on_conflicts_and_refusals,
            check_the_download_half_of_a_pull_leaves_the_checkout_alone,
            check_a_pull_canceled_during_its_download_changes_nothing,
            check_reword_head_changes_only_the_message,
            check_reword_older_commit_replays_descendants_and_merges,
            check_reword_refusals_are_typed_and_change_nothing,
            check_amend_head_changes_only_the_chosen_properties,
            check_amend_older_commit_replays_descendants_with_their_own_authors,
            check_amend_refusals_are_typed_and_change_nothing,
            check_log_timestamp_is_the_author_date,
            check_split_head_into_two_by_hunk,
            check_split_by_lines_keeps_crlf_and_no_newline_at_eof,
            check_split_older_commit_replays_descendants_and_merges,
            check_split_root_commit,
            check_split_by_whole_text_file,
            check_split_whole_only_files_binary_added_deleted_mode,
            check_split_refusals_are_typed_and_change_nothing,
            check_split_source_numbering_ignores_diff_renames_config,
            check_split_signs_when_asked,
            check_fast_forward_moves_a_branch_that_is_not_checked_out,
            check_fast_forward_of_the_checked_out_branch_updates_the_working_tree,
            check_fast_forward_refuses_diverged_and_elsewhere_checked_out_branches,
            check_fast_forward_of_the_checked_out_branch_keeps_local_changes,
            check_patch_create_from_a_commit_round_trips_through_apply,
            check_patch_create_from_a_range_applies_both_commits_to_the_same_file,
            check_patch_create_from_the_working_tree_covers_staged_and_unstaged_only,
            check_patch_create_from_commit_paths_carries_only_those_files,
            check_patch_keeps_binary_and_crlf_bytes_exactly,
            check_patch_create_refusals_write_nothing,
            check_patch_apply_that_does_not_apply_changes_nothing,
            check_rewrite_identity_on_a_linear_branch_matches_a_commit_tree_oracle,
            check_rewrite_keeps_the_unchanged_prefix_ids_and_signatures,
            check_rewrite_keeps_merge_parents_and_the_root_commit,
            check_rewrite_keeps_encoding_mergetag_and_non_utf8_messages,
            check_rewrite_message_rules_edit_utf8_messages,
            check_rewrite_anonymise_time_zone_and_committer_rules,
            check_rewrite_keeps_the_committer_unless_asked_for_me_now,
            check_rewrite_apply_refuses_a_moved_branch_and_runs_clean_up,
            check_rewrite_preview_cancel_and_refusals_leave_nothing,
            check_rewrite_ignores_replace_refs,
            check_rewrite_selection_range_and_since_scopes,
            check_rewrite_refs_scope_lists_branches_that_keep_the_old_history,
            check_rewrite_all_local_moves_every_branch_and_tag,
            check_rewrite_moves_lightweight_and_annotated_tags,
            check_rewrite_taggers_follow_identity_rules_only_when_asked,
            check_rewrite_notes_follow_the_rewritten_commits,
            check_rewrite_moves_a_branch_checked_out_in_a_linked_worktree,
            check_rewrite_classifies_what_is_already_pushed,
            check_rewrite_leaves_pushed_protected_branches_out_unless_listed,
            check_rewrite_never_moves_stashes_remote_tracking_or_gitpoulpi_refs,
            check_rewrite_scope_refusals_are_typed_and_change_nothing,
            check_rewrite_replaces_text_and_keeps_unrelated_blob_ids,
            check_rewrite_removes_a_folder_from_all_history,
            check_rewrite_drops_blobs_over_a_size,
            check_rewrite_prunes_commits_that_became_empty,
            check_rewrite_content_replace_skips_binary_blobs,
            check_rewrite_secret_literals_are_replaced_and_never_stored,
            check_rewrite_refuses_dirty_paths_the_rewrite_would_change,
            check_rewrite_content_run_refuses_a_branch_checked_out_elsewhere,
            check_rewrite_blob_diff_reads_the_preview,
            check_rewrite_scan_secrets_finds_a_planted_token_with_its_history,
            check_rewrite_removes_scanned_secrets_by_finding_id,
            check_rewrite_scan_rule_mixes_ids_and_literals_and_tolerates_no_findings,
            check_rewrite_analyze_ranks_blobs_folders_and_deleted_files,
            check_rewrite_analyze_reads_merges_odd_names_and_shared_content,
            check_rewrite_analysis_rows_turn_into_working_rules,
            check_rewrite_resign_signed_signs_only_the_signed_ones,
            check_rewrite_resign_all_signs_every_rewritten_commit,
            check_rewrite_strip_removes_signatures_and_signs_nothing,
            check_rewrite_signing_smoke_test_refuses_a_broken_key,
            check_rewrite_cancel_between_signatures_leaves_nothing,
            check_rewrite_remove_signatures_rule_strips_the_scope,
            check_rewrite_encoding_declared,
            check_rewrite_encoding_mojibake,
            check_rewrite_encoding_leaves_unprovable,
            check_rewrite_encoding_names_option,
            check_rewrite_push_sends_every_pushed_ref_with_a_lease,
            check_rewrite_push_rejects_a_remote_that_moved_and_atomic_pushes_nothing,
            check_rewrite_push_leaves_protected_refs_out_unless_asked,
            check_rewrite_push_dry_run_sends_nothing,
            check_rewrite_push_needs_an_applied_run,
            check_rewrite_maps_and_note_follow_the_applied_run,
            check_rewrite_push_leaves_a_record_for_the_undo_warning,
            check_rewrite_purge_deletes_the_old_objects,
            check_rewrite_purge_is_blocked_by_a_stash_on_the_old_history,
            check_rewrite_purge_is_blocked_by_refs_and_worktrees_outside_the_rewrite,
            check_rewrite_purge_is_blocked_by_the_pin_of_another_entry,
            check_rewrite_purge_keeps_stashes_and_unrelated_reflogs,
            check_rewrite_purge_of_nothing_is_a_clean_no_op,
            check_log_shows_mailmapped_authors_like_git,
            check_mailmap_resolve_matches_git_check_mailmap,
            check_mailmap_identities_count_pairs_and_show_what_is_mapped,
            check_mailmap_add_appends_lines_once_and_refuses_bad_input,
            check_rewrite_mailmap_rule_matches_git_use_mailmap,
            check_rewrite_mailmap_rule_reads_a_given_file_and_refuses_a_missing_one,
            check_rewrite_trailer_add_matches_interpret_trailers,
            check_rewrite_trailer_if_exists_remove_and_map_change_only_their_lines,
            check_rewrite_identity_rule_maps_trailers_only_when_asked,
            check_rewrite_trailer_rules_leave_non_utf8_messages_alone,
            check_rewrite_update_message_hashes_follows_the_new_ids,
            check_rewrite_trailer_rule_problems_are_refused_before_anything_runs,
            check_rewrite_renames_a_folder_across_history,
            check_rewrite_moves_the_whole_repository_into_a_subdirectory,
            check_rewrite_a_rename_never_overwrites_and_names_the_collision,
            check_rewrite_extracts_a_subdirectory_and_prunes_the_rest,
            check_rewrite_extract_simplifies_merges,
            check_rewrite_extract_into_a_clone_leaves_the_original_untouched,
            check_rewrite_extract_as_a_submodule_adds_one_commit_and_undoes,
            check_rewrite_eol_to_lf_converts_text_only_and_keeps_other_blob_ids,
            check_rewrite_eol_to_crlf_and_cr_use_the_chosen_ending,
            check_rewrite_eol_can_follow_gitattributes,
            check_rewrite_strip_exec_changes_the_mode_of_matching_paths_only,
            check_rewrite_reencode_detects_the_source_and_skips_the_undecodable,
            check_rewrite_reencode_writes_each_target_and_refuses_unmappable_characters,
            check_rewrite_eol_and_encoding_rules_compose_in_recipe_order,
        }
    };
}

/// The history rewrite script checks: they need the `rewrite-scripts` feature
/// of the backend under test (the test file calls this list under
/// `#[cfg(feature = "rewrite-scripts")]`; without the feature it asserts the
/// typed `Unsupported` instead).
#[macro_export]
macro_rules! for_each_rewrite_script_check {
    ($callback:ident) => {
        $callback! {
            check_rewrite_script_check_reads_the_header_and_reports_problems,
            check_rewrite_script_rule_changes_people_dates_and_messages,
            check_rewrite_script_carries_state_between_commits,
            check_rewrite_script_window_is_rewritten_before_and_original_after,
            check_rewrite_script_failures_are_typed_with_commit_and_reason,
            check_rewrite_script_limits_stop_runaway_scripts,
            check_rewrite_script_has_no_file_clock_or_module_access,
            check_rewrite_script_output_is_deterministic,
            check_rewrite_script_test_reports_changes_without_writing,
            check_rewrite_script_file_source_and_unsupported_languages,
        }
    };
}

/// The external script checks (S11): shell, Python and external Lua scripts,
/// trust, the settings and what a hung or crashing script leaves behind. Same
/// feature gate as [`for_each_rewrite_script_check!`].
#[macro_export]
macro_rules! for_each_rewrite_external_script_check {
    ($callback:ident) => {
        $callback! {
            check_rewrite_external_script_runs_the_contract_and_changes_people,
            check_rewrite_external_script_carries_state_between_commits,
            check_rewrite_external_script_is_refused_until_trusted_and_again_after_a_change,
            check_rewrite_external_script_languages_switched_off_are_refused,
            check_rewrite_external_script_timeout_kills_the_process_and_leaves_nothing,
            check_rewrite_external_script_crash_mid_run_changes_nothing,
            check_rewrite_external_script_bad_answers_are_typed_failures,
            check_rewrite_external_script_environment_is_cleared,
            check_rewrite_external_script_check_reports_trust_and_does_a_handshake,
            check_rewrite_shell_script_runs_with_the_machines_shell,
        }
    };
}

/// The checks that need the `lfs` feature of the backend under test (the
/// test file calls this list under `#[cfg(feature = "lfs")]`; without the
/// feature it asserts `NotBuilt` instead).
#[macro_export]
macro_rules! for_each_lfs_check {
    ($callback:ident) => {
        $callback! {
            check_lfs_status_of_a_plain_repo_says_unused,
            check_lfs_status_reports_patterns_hooks_and_binary,
            check_lfs_files_lists_pointers_with_local_flag,
            check_lfs_files_at_a_commit,
            check_checkout_in_an_lfs_repo_writes_content_not_pointers,
            check_reset_hard_in_an_lfs_repo_writes_content,
            check_fast_forward_of_the_checked_out_branch_in_an_lfs_repo,
            check_lfs_file_is_clean_after_its_mtime_changes,
            check_diff_of_a_modified_lfs_file_is_a_pointer_change,
            check_lfs_install_configures_filters_and_hooks,
            check_lfs_track_writes_gitattributes_and_next_commit_stores_a_pointer,
            check_lfs_untrack_removes_the_pattern,
            check_lfs_push_uploads_objects_to_the_remote,
            check_lfs_fetch_then_files_are_local,
            check_lfs_pull_materialises_content,
            check_lfs_checkout_replaces_pointers_with_content,
            check_lfs_progress_ends_done_and_never_decreases,
            check_clone_of_an_lfs_repo_has_real_content,
            check_clone_with_lfs_skip_keeps_pointers,
            check_lfs_lock_list_unlock_round_trip,
            check_lfs_unlock_of_someone_elses_lock_needs_force,
            check_lfs_locking_on_a_server_without_locking_is_typed,
            check_rewrite_lfs_import_makes_pointers_objects_and_attributes,
            check_rewrite_lfs_import_min_size_and_pointers_already_there,
            check_rewrite_lfs_export_round_trips_to_the_original_blob_ids,
            check_rewrite_lfs_export_refuses_missing_objects_with_the_list,
        }
    };
}

/// The checks that need the `worktree` feature of the backend under test
/// (called under `#[cfg(feature = "worktree")]`; without it the test file
/// asserts `NotBuilt`).
#[macro_export]
macro_rules! for_each_worktree_check {
    ($callback:ident) => {
        $callback! {
            check_worktrees_lists_main_and_linked,
            check_worktrees_includes_main_when_there_are_no_linked_ones,
            check_worktree_add_on_a_new_branch_an_existing_branch_and_a_commit,
            check_worktree_add_refusals_leave_no_worktree,
            check_outside_diff_sees_worktrees_added_and_removed,
        }
    };
}

/// The checks that need the `submodule` feature of the backend under test
/// (see `for_each_worktree_check!`).
#[macro_export]
macro_rules! for_each_submodule_check {
    ($callback:ident) => {
        $callback! {
            check_submodules_list_init_and_update,
        }
    };
}

// -- listing a whole revision --------------------------------------------
//
// `diff` and `log` answer "what did this commit change"; this answers
// "what did the project contain" (the commit-detail pane's `View all
// files` toggle).

/// Drives one future to completion on the calling thread.
///
/// The contract suite is a set of synchronous `fn(&dyn GitBackend)`
/// checks, which is what lets it run against both backends from an
/// ordinary `#[test]`, while `clone` is `async`. Rather than make every
/// check async, the clone checks block here, on a current-thread tokio
/// runtime (both backends' clone paths spawn through `tokio::process`,
/// which needs a reactor).
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build the test runtime")
        .block_on(future)
}

macro_rules! define_run_all {
    ($($name:ident),* $(,)?) => {
        /// The names in `for_each_contract_check!`, for the coverage guard.
        pub const CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}

macro_rules! define_lfs_names {
    ($($name:ident),* $(,)?) => {
        /// The names in `for_each_lfs_check!`, for the coverage guard.
        pub const LFS_CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}

crate::for_each_contract_check!(define_run_all);
crate::for_each_lfs_check!(define_lfs_names);

macro_rules! define_rewrite_script_names {
    ($($name:ident),* $(,)?) => {
        pub const REWRITE_SCRIPT_CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}
crate::for_each_rewrite_script_check!(define_rewrite_script_names);
macro_rules! define_rewrite_external_script_names {
    ($($name:ident),* $(,)?) => {
        pub const REWRITE_EXTERNAL_SCRIPT_CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}
crate::for_each_rewrite_external_script_check!(define_rewrite_external_script_names);

macro_rules! define_worktree_names {
    ($($name:ident),* $(,)?) => {
        pub const WORKTREE_CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}
macro_rules! define_submodule_names {
    ($($name:ident),* $(,)?) => {
        pub const SUBMODULE_CHECK_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}
crate::for_each_worktree_check!(define_worktree_names);
crate::for_each_submodule_check!(define_submodule_names);

fn pick(commit: Oid) -> RebaseStep {
    RebaseStep {
        commit,
        action: RebaseAction::Pick,
        message: None,
    }
}

fn rev(repo: &crate::FixtureRepo, spec: &str) -> Oid {
    Oid::from_hex(&repo.git(&["rev-parse", spec])).unwrap()
}

#[cfg(test)]
mod meta_tests {
    /// A check nobody calls never runs, silently. Reading the modules'
    /// own source is crude but catches "added `check_foo`, forgot to list
    /// it", which a bare count would not (add one, delete another).
    #[test]
    fn every_check_is_listed() {
        let source = [
            include_str!("cancel.rs"),
            include_str!("log_refs.rs"),
            include_str!("log_hidden.rs"),
            include_str!("staging.rs"),
            include_str!("history.rs"),
            include_str!("stash.rs"),
            include_str!("remotes_network.rs"),
            include_str!("tag_where.rs"),
            include_str!("sparse.rs"),
            include_str!("lfs.rs"),
            include_str!("journal_plumbing.rs"),
            include_str!("reflog.rs"),
            include_str!("outside.rs"),
            include_str!("pull_split.rs"),
            include_str!("conflicts.rs"),
            include_str!("predict.rs"),
            include_str!("stuck_states.rs"),
            include_str!("content.rs"),
            include_str!("file_history.rs"),
            include_str!("compare.rs"),
            include_str!("count.rs"),
            include_str!("search.rs"),
            include_str!("split.rs"),
            include_str!("mailmap.rs"),
            include_str!("rewrite.rs"),
            include_str!("rewrite_scopes.rs"),
            include_str!("rewrite_content.rs"),
            include_str!("rewrite_paths.rs"),
            include_str!("rewrite_purge.rs"),
            include_str!("rewrite_push.rs"),
            include_str!("rewrite_script.rs"),
            include_str!("rewrite_script_external.rs"),
            include_str!("rewrite_trailers.rs"),
            include_str!("rewrite_lfs.rs"),
            include_str!("rewrite_secrets.rs"),
            include_str!("rewrite_analyze.rs"),
            include_str!("rewrite_signatures.rs"),
            include_str!("rewrite_textconv.rs"),
            include_str!("rewrite_encoding.rs"),
            include_str!("workflow.rs"),
            include_str!("mod.rs"),
        ]
        .concat();
        let declared: Vec<&str> = source
            .lines()
            .filter_map(|line| line.strip_prefix("pub fn check_"))
            .filter_map(|rest| rest.split('(').next())
            .collect();
        assert!(
            declared.len() >= 20,
            "source scan found only {} checks - the scan itself is broken",
            declared.len()
        );
        for name in declared {
            let full = format!("check_{name}");
            assert!(
                super::CHECK_NAMES.contains(&full.as_str())
                    || super::LFS_CHECK_NAMES.contains(&full.as_str())
                    || super::REWRITE_SCRIPT_CHECK_NAMES.contains(&full.as_str())
                    || super::REWRITE_EXTERNAL_SCRIPT_CHECK_NAMES.contains(&full.as_str())
                    || super::WORKTREE_CHECK_NAMES.contains(&full.as_str())
                    || super::SUBMODULE_CHECK_NAMES.contains(&full.as_str()),
                "`{full}` is not in `for_each_contract_check!`, so no test runs it"
            );
        }
        for listed in super::CHECK_NAMES
            .iter()
            .chain(super::LFS_CHECK_NAMES)
            .chain(super::REWRITE_SCRIPT_CHECK_NAMES)
            .chain(super::WORKTREE_CHECK_NAMES)
            .chain(super::SUBMODULE_CHECK_NAMES)
        {
            assert!(
                source.contains(&format!("pub fn {listed}(")),
                "`{listed}` is listed in `for_each_contract_check!` but not defined here"
            );
        }
    }
}
