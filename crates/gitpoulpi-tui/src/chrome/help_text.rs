//! The help overlay's content (a table of entries, wrapped at draw time)
//! and its keymap-aware, bilingual filtering.
//!
//! Every message lives in the `help_section`, `help_text` and `help_keys`
//! catalog sections. A description is one sentence with no hand wrapping;
//! `{ctx.action}` / `{*ctx.action}` inside a message (written `{{..}}` in
//! the catalog) and in a [`HelpKeys::Lit`] key column is resolved from the
//! user's keymap at draw time, so a rebinding shows up here. The layout
//! (key column, wrap to the overlay width) is computed per draw because a
//! translation is longer than English and the overlay width follows the
//! terminal. What keeps the table honest is a test, not construction: the
//! cross-checks `help::every_rebindable_action_is_in_help` and
//! `palette::every_ctrl_chord_shortcut_has_a_help_entry` walk it.
//! Sections roughly follow reading order: global keys and mouse basics,
//! each pane/dialog left-to-right, the two chord prefixes, a mouse recap.

use super::*;
use crate::ui_kit::display_width;
use gitpoulpi_i18n::{msg, Lang, M};

/// The key column of a help row.
#[derive(Clone, Copy)]
pub(crate) enum HelpKeys {
    /// Key names and `{ctx.action}` placeholders only: never translated.
    Lit(&'static str),
    /// A column with words in it ("double-click", "(WIP row)"), translated.
    Msg(M),
}

/// One row of the help table.
#[derive(Clone, Copy)]
pub(crate) enum HelpEntry {
    /// A section title; starts a paragraph (the filter keeps or drops it whole).
    Sec(M),
    /// A lead-in line; the rows after it are indented one step further.
    Sub(M),
    /// A key column and its description.
    Item(HelpKeys, M),
    /// An [`Item`] shown only in a build that has the feature.
    Gated(crate::build_features::Feature, HelpKeys, M),
}

use HelpEntry::{Gated, Item, Sec, Sub};
use HelpKeys::{Lit, Msg};

/// A key column made of key names and `{ctx.action}` placeholders. The
/// macro name marks the literal as "not prose" for the i18n source lint.
macro_rules! keys {
    ($s:literal) => {
        HelpKeys::Lit($s)
    };
}

pub(crate) const HELP_TEXT: &[HelpEntry] = &[
    Sec(msg::help_section::GLOBAL),
    Item(keys!("{global.palette}"), msg::help_text::GLOBAL_1),
    Item(keys!("{global.nav_prefix}"), msg::help_text::GLOBAL_2),
    Item(keys!("{global.git_prefix}"), msg::help_text::GLOBAL_3),
    Item(keys!("{global.help}"), msg::help_text::GLOBAL_4),
    Item(keys!("{global.refresh}"), msg::help_text::GLOBAL_5),
    Item(keys!("{global.refresh_all}"), msg::help_text::GLOBAL_6),
    Item(keys!("{global.undo} / {global.redo}"), msg::help_text::GLOBAL_7),
    Item(
        keys!("{global.undo_history}"),
        msg::undo_safety::HELP_HISTORY,
    ),
    Item(keys!("{global.undo_offer}"), msg::undo_safety::HELP_OFFER),
    Item(keys!("{global.commit_column}"), msg::commit_column::HELP),
    Item(keys!("{global.quit}"), msg::help_text::GLOBAL_9),
    Item(Msg(msg::help_keys::GLOBAL_10_KEYS), msg::help_text::GLOBAL_10),
    Item(keys!("Tab / Shift+Tab"), msg::help_text::GLOBAL_11),
    Item(keys!("j / k, Up/Down"), msg::help_text::GLOBAL_12),
    Item(keys!("4j / 10k / ..."), msg::help_text::GLOBAL_13),
    Item(keys!("h / l, Left/Right"), msg::help_text::GLOBAL_14),
    Item(keys!(""), msg::help_text::GLOBAL_15),
    Item(keys!(""), msg::help_text::GLOBAL_16),
    Sec(msg::help_section::MOUSE_BASICS),
    Item(Msg(msg::help_keys::MOUSE_BASICS_1_KEYS), msg::help_text::MOUSE_BASICS_1),
    Item(Msg(msg::help_keys::MOUSE_BASICS_2_KEYS), msg::help_text::MOUSE_BASICS_2),
    Item(Msg(msg::help_keys::MOUSE_BASICS_3_KEYS), msg::help_text::MOUSE_BASICS_3),
    Item(Msg(msg::help_keys::MOUSE_BASICS_4_KEYS), msg::help_text::MOUSE_BASICS_4),
    Item(Msg(msg::help_keys::MOUSE_BASICS_5_KEYS), msg::help_text::MOUSE_BASICS_5),
    Sec(msg::help_section::NAVIGATION_PREFIX_THEN),
    Item(keys!("{nav_prefix.collapse_refs}"), msg::help_text::NAVIGATION_PREFIX_THEN_1),
    Item(keys!("{nav_prefix.collapse_status}"), msg::help_text::NAVIGATION_PREFIX_THEN_2),
    Item(keys!("{nav_prefix.refs_narrower} / {nav_prefix.refs_wider}"), msg::help_text::NAVIGATION_PREFIX_THEN_17),
    Item(keys!("{nav_prefix.status_wider} / {nav_prefix.status_narrower}"), msg::help_text::NAVIGATION_PREFIX_THEN_18),
    Item(keys!("{nav_prefix.grow_section}"), msg::help_text::NAVIGATION_PREFIX_THEN_3),
    Item(keys!("{nav_prefix.grow_message}"), msg::help_text::NAVIGATION_PREFIX_THEN_4),
    Item(Msg(msg::help_keys::NAVIGATION_PREFIX_THEN_5_KEYS), msg::help_text::NAVIGATION_PREFIX_THEN_5),
    Item(keys!("{nav_prefix.settings}"), msg::help_text::NAVIGATION_PREFIX_THEN_6),
    Item(keys!("{nav_prefix.about}"), msg::help_text::NAVIGATION_PREFIX_THEN_16),
    Item(keys!("{nav_prefix.activity_log}"), msg::activity_log::HELP_NAV),
    Item(
        keys!("{nav_prefix.global_log}"),
        msg::activity_log::HELP_NAV_GLOBAL,
    ),
    Item(
        keys!("{activity_log.scroll_left} / {activity_log.scroll_right}"),
        msg::activity_log::HELP_SCROLL,
    ),
    Item(keys!("{nav_prefix.jobs}"), msg::jobs::HELP_NAV),
    Item(keys!("{nav_prefix.updates}"), msg::updates::HELP_NAV),
    Item(keys!("{nav_prefix.help}"), msg::help_text::NAVIGATION_PREFIX_THEN_7),
    Item(keys!("{global.select_tab} / {nav_prefix.select_tab}"),msg::help_text::NAVIGATION_PREFIX_THEN_8),
    Item(keys!("{global.workspace_search}"), msg::help_text::WORKSPACE_SEARCH),
    Item(keys!("{nav_prefix.change_theme}"), msg::help_text::NAVIGATION_PREFIX_THEN_12),
    Item(keys!("{nav_prefix.change_language}"), msg::help_text::NAVIGATION_PREFIX_THEN_13),
    Item(keys!("{nav_prefix.change_graph_color_theme}"), msg::help_text::NAVIGATION_PREFIX_THEN_14),
    Item(keys!("{nav_prefix.toggle_graph_row_tints}"), msg::help_text::NAVIGATION_PREFIX_THEN_15),
    Item(keys!("{nav_prefix.arrange_dialog}"), msg::help_text::NAVIGATION_PREFIX_THEN_19),
    Item(keys!("{nav_prefix.agent_show}"), msg::help_text::NAVIGATION_PREFIX_THEN_9),
    Item(keys!("{nav_prefix.agent_back}"), msg::help_text::NAVIGATION_PREFIX_THEN_10),
    Item(Msg(msg::help_keys::NAVIGATION_PREFIX_THEN_11_KEYS), msg::help_text::NAVIGATION_PREFIX_THEN_11),
    Sec(msg::help_section::GIT_ACTIONS_THEN),
    Item(keys!("{git_prefix.stage_file}"), msg::help_text::GIT_ACTIONS_THEN_1),
    Item(keys!("{git_prefix.unstage_file}"), msg::help_text::GIT_ACTIONS_THEN_2),
    Item(keys!("{git_prefix.stage_all}"), msg::help_text::GIT_ACTIONS_THEN_3),
    Item(keys!("{git_prefix.discard_all}"), msg::help_text::GIT_ACTIONS_THEN_4),
    Item(keys!("{git_prefix.compose_commit}"), msg::help_text::GIT_ACTIONS_THEN_5),
    Item(keys!("{git_prefix.commit_now}"), msg::help_text::GIT_ACTIONS_THEN_COMMIT_NOW),
    Item(keys!("{git_prefix.stash_message}"), msg::help_text::GIT_ACTIONS_THEN_6),
    Item(keys!("{git_prefix.ai_commit_message}"), msg::help_text::GIT_ACTIONS_THEN_7),
    Item(keys!("{git_prefix.ai_explain}"), msg::help_text::GIT_ACTIONS_THEN_8),
    Item(keys!("{git_prefix.ai_compose}"), msg::help_text::GIT_ACTIONS_THEN_9),
    Item(keys!("{git_prefix.pull}"), msg::help_text::GIT_ACTIONS_THEN_10),
    Item(keys!("{git_prefix.pull_all_branches}"), msg::help_text::GIT_ACTIONS_THEN_PULL_ALL_BRANCHES),
    Item(keys!("{git_prefix.push}"), msg::help_text::GIT_ACTIONS_THEN_11),
    Item(keys!("{git_prefix.fetch_all}"), msg::help_text::GIT_ACTIONS_THEN_12),
    Item(keys!("{git_prefix.new_branch}"), msg::help_text::GIT_ACTIONS_THEN_13),
    Item(keys!("{git_prefix.new_tag}"), msg::help_text::GIT_ACTIONS_THEN_14),
    Item(keys!("{git_prefix.stash_push}"), msg::help_text::GIT_ACTIONS_THEN_15),
    Item(keys!("{git_prefix.stash_pop}"), msg::help_text::GIT_ACTIONS_THEN_16),
    Item(keys!("{git_prefix.push_tags}"), msg::help_text::GIT_ACTIONS_THEN_17),
    Item(keys!("{git_prefix.open_linked_pr}"), msg::help_text::GIT_ACTIONS_THEN_18),
    Item(keys!("{git_prefix.delete_ref}"), msg::help_text::GIT_ACTIONS_THEN_19),
    Item(keys!("{git_prefix.rebase_onto}"), msg::help_text::GIT_ACTIONS_THEN_20),
    Item(keys!("{git_prefix.edit_message}"), msg::help_text::GIT_ACTIONS_THEN_21),
    Item(keys!("{git_prefix.amend}"), msg::help_text::GIT_ACTIONS_THEN_AMEND),
    Item(keys!("{git_prefix.edit_author}"), msg::help_text::GIT_ACTIONS_THEN_EDIT_AUTHOR),
    Item(keys!("{git_prefix.cherry_pick}"), msg::help_text::GIT_ACTIONS_THEN_22),
    Item(keys!("{git_prefix.revert}"), msg::help_text::GIT_ACTIONS_THEN_23),
    Item(keys!("{git_prefix.reset_mixed}"), msg::help_text::GIT_ACTIONS_THEN_24),
    Item(keys!("{git_prefix.lfs_menu}"), msg::help_text::GIT_ACTIONS_THEN_25),
    Item(keys!("{git_prefix.lfs_fetch_all}"), msg::help_text::GIT_ACTIONS_THEN_26),
    Item(keys!("{git_prefix.lfs_checkout}"), msg::help_text::GIT_ACTIONS_THEN_27),
    Item(keys!("{git_prefix.lfs_pull}"), msg::help_text::GIT_ACTIONS_THEN_28),
    Item(keys!("{git_prefix.lfs_push_all}"), msg::help_text::GIT_ACTIONS_THEN_29),
    Item(keys!("{git_prefix.lfs_prune}"), msg::help_text::GIT_ACTIONS_THEN_30),
    Item(keys!("{git_prefix.flow_menu}"), msg::flow::HELP_MENU),
    Sec(msg::help_section::PULL_REQUESTS_AND),
    Item(Msg(msg::help_keys::PULL_REQUESTS_AND_1_KEYS), msg::help_text::PULL_REQUESTS_AND_1),
    Item(keys!("{refs.hosting_edit}"), msg::help_text::PULL_REQUESTS_AND_2),
    Item(keys!("{refs.new_pull_request}"), msg::help_text::PULL_REQUESTS_AND_3),
    Item(Msg(msg::help_keys::PULL_REQUESTS_AND_4_KEYS), msg::help_text::PULL_REQUESTS_AND_4),
    Item(Msg(msg::help_keys::PULL_REQUESTS_AND_5_KEYS), msg::help_text::PULL_REQUESTS_AND_5),
    Item(keys!("{refs.solo}"), msg::help_text::PULL_REQUESTS_AND_6),
    Sec(msg::help_section::REFS_PANE),
    Item(Msg(msg::help_keys::REFS_PANE_1_KEYS), msg::help_text::REFS_PANE_1),
    Item(keys!(""), msg::help_text::REFS_PANE_2),
    Item(keys!(""), msg::help_text::REFS_PANE_3),
    Item(keys!(""), msg::help_text::REFS_PANE_4),
    Item(keys!(""), msg::help_text::REFS_PANE_5),
    Item(keys!(""), msg::help_text::REFS_PANE_6),
    Item(Msg(msg::help_keys::REFS_PANE_7_KEYS), msg::help_text::REFS_PANE_7),
    Item(Msg(msg::help_keys::REFS_PANE_8_KEYS), msg::help_text::REFS_PANE_8),
    Item(keys!("{refs.solo}"), msg::help_text::REFS_PANE_9),
    Item(keys!("{refs.clear_solo}"), msg::help_text::REFS_PANE_10),
    Item(keys!("{refs.show_all}"), msg::help_text::REFS_PANE_11),
    Item(keys!("{refs.add_remote}"), msg::help_text::REFS_PANE_12),
    Item(keys!("Enter / Space"), msg::help_text::REFS_PANE_13),
    Item(keys!("h / l, Left/Right"), msg::help_text::REFS_PANE_14),
    Item(keys!("{refs.filter}"), msg::help_text::REFS_PANE_15),
    Item(keys!("{refs.toggle_hierarchy}"), msg::help_text::REFS_PANE_16),
    Item(keys!(""), msg::help_text::REFS_PANE_17),
    Item(keys!("{refs.reveal}"), msg::help_text::REFS_PANE_18),
    Item(keys!("{refs.mark_drop} / {refs.drop_marked}"), msg::help_text::REFS_PANE_19),
    Item(keys!("{refs.remotes}"), msg::help_text::REFS_PANE_20),
    Item(
        keys!("{refs.expand_all} / {refs.collapse_all} / {refs.expand_below} / {refs.collapse_below}"),
        msg::tree_fold::HELP_FOLDS,
    ),
    Item(keys!("{refs.scroll_left} / {refs.scroll_right}"), msg::pane_hscroll::HELP),
    Sec(msg::help_section::STATUS_PANE_WORKING),
    Item(keys!("j / k, Up/Down"), msg::help_text::STATUS_PANE_WORKING_1),
    Item(keys!("h / l, Left/Right"), msg::help_text::STATUS_PANE_WORKING_2),
    Item(keys!("Enter"), msg::help_text::STATUS_PANE_WORKING_3),
    Item(keys!("{status.stage_toggle}"), msg::help_text::STATUS_PANE_WORKING_4),
    Item(keys!("{status.toggle_file_mode}"), msg::help_text::STATUS_PANE_WORKING_5),
    Item(keys!("{status.view_all_files}"), msg::help_text::STATUS_PANE_WORKING_6),
    Item(keys!("{status.collapse_all}"), msg::help_text::STATUS_PANE_WORKING_7),
    Item(
        keys!("{status.expand_all} / {status.collapse_all_folders} / {status.expand_below} / {status.collapse_below}"),
        msg::tree_fold::HELP_FOLDS,
    ),
    Item(keys!("{status.scroll_left} / {status.scroll_right}"), msg::pane_hscroll::HELP),
    Item(Msg(msg::help_keys::STATUS_PANE_WORKING_8_KEYS), msg::help_text::STATUS_PANE_WORKING_8),
    Item(keys!("{log_files.filter}"), msg::help_text::STATUS_PANE_WORKING_9),
    Item(keys!(""), msg::help_text::STATUS_PANE_WORKING_10),
    Sec(msg::help_section::COMMIT_COMPOSER_BOTTOM),
    Item(Msg(msg::help_keys::COMMIT_COMPOSER_BOTTOM_1_KEYS), msg::help_text::COMMIT_COMPOSER_BOTTOM_1),
    Item(Msg(msg::help_keys::COMMIT_COMPOSER_BOTTOM_2_KEYS), msg::help_text::COMMIT_COMPOSER_BOTTOM_2),
    Item(keys!("{status.amend}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_3),
    Item(keys!("{status.skip_hooks}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_14),
    Item(keys!("{status.open_diff}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_4),
    Item(keys!("{log.toggle_file_mode}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_5),
    Item(keys!("{log_files.collapse_all}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_6),
    Item(
        keys!("{log_files.expand_all} / {log_files.collapse_all_folders} / {log_files.expand_below} / {log_files.collapse_below}"),
        msg::tree_fold::HELP_FOLDS,
    ),
    Item(keys!("{log_files.scroll_left} / {log_files.scroll_right}"), msg::pane_hscroll::HELP),
    Item(Msg(msg::help_keys::COMMIT_COMPOSER_BOTTOM_7_KEYS), msg::help_text::COMMIT_COMPOSER_BOTTOM_7),
    Item(Msg(msg::help_keys::COMMIT_COMPOSER_BOTTOM_8_KEYS), msg::help_text::COMMIT_COMPOSER_BOTTOM_8),
    Item(Msg(msg::help_keys::COMMIT_COMPOSER_BOTTOM_9_KEYS), msg::help_text::COMMIT_COMPOSER_BOTTOM_9),
    Item(keys!("{*composer.commit}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_10),
    Item(keys!("{composer.insert_issue_ref}"), msg::help_text::COMMIT_COMPOSER_BOTTOM_13),
    Item(keys!("Esc"), msg::help_text::COMMIT_COMPOSER_BOTTOM_11),
    Item(keys!(""), msg::help_text::COMMIT_COMPOSER_BOTTOM_12),
    Sec(msg::help_section::LOG_PANE_COMMIT),
    Item(keys!("PageUp/PageDown"), msg::help_text::LOG_PANE_COMMIT_1),
    Item(keys!("Home / End"), msg::help_text::LOG_PANE_COMMIT_2),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_3_KEYS), msg::help_text::LOG_PANE_COMMIT_3),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_4_KEYS), msg::help_text::LOG_PANE_COMMIT_4),
    Item(keys!("{log.compare_selection}"), msg::compare::HELP_COMPARE),
    Item(keys!("{log.compare_swap}"), msg::compare::HELP_SWAP),
    Item(keys!("{log.compare_next}"), msg::compare::HELP_NEXT_CARD),
    Item(keys!("{log.copy_mermaid}"), msg::mermaid::HELP_COPY),
    Item(
        keys!("{log.pick_column} / {log_columns.next} / {log_columns.prev} / {log_columns.move_left} / {log_columns.move_right} / {log_columns.wider} / {log_columns.narrower} / {log_columns.reset_width} / {log_columns.fit} / {log_columns.fit_all}"),
        msg::help_text::LOG_COLUMNS_1,
    ),
    Item(
        keys!("{log.scroll_left} / {log.scroll_right} / {log_columns.layout}"),
        msg::help_text::LOG_COLUMNS_2,
    ),
    Item(keys!("{log.details_scroll_left} / {log.details_scroll_right}"), msg::pane_hscroll::HELP_DETAILS),
    Item(keys!(""), msg::help_text::LOG_PANE_COMMIT_5),
    Item(keys!("{rebase.pick}"), msg::help_text::LOG_PANE_COMMIT_6),
    Item(keys!("{rebase.reword}"), msg::help_text::LOG_PANE_COMMIT_7),
    Item(keys!("{rebase.squash}"), msg::help_text::LOG_PANE_COMMIT_8),
    Item(keys!("{rebase.fixup}"), msg::help_text::LOG_PANE_COMMIT_9),
    Item(keys!("{rebase.drop}"), msg::help_text::LOG_PANE_COMMIT_10),
    Item(keys!("{rebase.edit}"), msg::help_text::LOG_PANE_COMMIT_11),
    Item(keys!("{rebase.autosquash}"), msg::help_text::LOG_PANE_COMMIT_12),
    Item(keys!("{rebase.move_up}"), msg::help_text::LOG_PANE_COMMIT_13),
    Item(keys!("{rebase.move_down}"), msg::help_text::LOG_PANE_COMMIT_14),
    Item(keys!("{rebase.split}"), msg::help_text::LOG_PANE_COMMIT_SPLIT),
    Item(keys!(""), msg::help_text::LOG_PANE_COMMIT_15),
    Item(keys!("{rebase.reset}"), msg::help_text::LOG_PANE_COMMIT_16),
    Item(keys!("{rebase.start}"), msg::help_text::LOG_PANE_COMMIT_17),
    Item(keys!("Esc"), msg::help_text::LOG_PANE_COMMIT_18),
    Item(keys!(""), msg::help_text::LOG_PANE_COMMIT_19),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_20_KEYS), msg::help_text::LOG_PANE_COMMIT_20),
    Item(keys!("c"), msg::help_text::LOG_PANE_COMMIT_21),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_22_KEYS), msg::help_text::LOG_PANE_COMMIT_22),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_23_KEYS), msg::help_text::LOG_PANE_COMMIT_23),
    Item(Msg(msg::help_keys::LOG_PANE_COMMIT_24_KEYS), msg::help_text::LOG_PANE_COMMIT_24),
    Item(keys!(""), msg::help_text::LOG_PANE_COMMIT_25),
    Sec(msg::help_section::SPLIT),
    Item(keys!("1-9, 0, n"), msg::help_text::SPLIT_1),
    Item(keys!("{split.line_mode}"), msg::help_text::SPLIT_2),
    Item(keys!("{split.move_up} / {split.move_down}"), msg::help_text::SPLIT_3),
    Item(keys!("{split.reset}"), msg::help_text::SPLIT_4),
    Item(keys!("{compose.apply}"), msg::help_text::SPLIT_5),
    Item(keys!("s"), msg::help_text::SPLIT_6),
    Item(keys!("1-9"), msg::help_text::SPLIT_7),
    Sec(msg::help_section::REWRITE),
    Item(keys!("{git_prefix.rewrite_workbench}"), msg::help_text::REWRITE_1),
    Item(keys!("{rewrite.preview} / {rewrite.apply}"), msg::help_text::REWRITE_2),
    Item(keys!("{rewrite.cancel}"), msg::help_text::REWRITE_3),
    Item(keys!("{rewrite.add_rule} / {rewrite.delete_rule} / {rewrite.restore_rule} / {rewrite.duplicate_rule}"), msg::help_text::REWRITE_4),
    Item(keys!("{rewrite.toggle_rule} / {rewrite.move_up} / {rewrite.move_down}"), msg::help_text::REWRITE_5),
    Item(keys!("{rewrite.next_tab} / {rewrite.prev_tab}"), msg::help_text::REWRITE_6),
    Item(keys!("{rewrite.save_shared} / {rewrite.save_mine} / {rewrite.load} / {rewrite.name}"), msg::help_text::REWRITE_7),
    Item(keys!("{rewrite.protected_all} / {rewrite.protected_none} / {rewrite.check_tools}"), msg::help_text::REWRITE_8),
    Item(keys!("{rewrite.analyze}"), msg::help_text::REWRITE_9),
    Item(keys!("{rewrite.host_howto}"), msg::help_text::REWRITE_HOST_HOWTO),
    Item(keys!("{rewrite.ai_rules}"), msg::help_text::REWRITE_AI_RULES),
    Sec(msg::help_section::EDITING_A_COMMIT),
    Item(Msg(msg::help_keys::EDITING_A_COMMIT_1_KEYS), msg::help_text::EDITING_A_COMMIT_1),
    Item(Msg(msg::help_keys::EDITING_A_COMMIT_2_KEYS), msg::help_text::EDITING_A_COMMIT_2),
    Item(Msg(msg::help_keys::EDITING_A_COMMIT_3_KEYS), msg::help_text::EDITING_A_COMMIT_3),
    Item(keys!("Tab / Shift+Tab"), msg::help_text::EDITING_A_COMMIT_4),
    Item(Msg(msg::help_keys::EDITING_A_COMMIT_5_KEYS), msg::help_text::EDITING_A_COMMIT_5),
    Item(keys!(""), msg::help_text::EDITING_A_COMMIT_6),
    Gated(crate::build_features::Feature::HistoryRewrite, Msg(msg::help_keys::EDITING_A_COMMIT_7_KEYS), msg::help_text::EDITING_A_COMMIT_7),
    Sec(msg::help_section::SEARCH_THE_COMMIT),
    Item(keys!("{global.find_in_log}"), msg::help_text::SEARCH_THE_COMMIT_1),
    Item(Msg(msg::help_keys::SEARCH_THE_COMMIT_2_KEYS), msg::help_text::SEARCH_THE_COMMIT_2),
    Item(Msg(msg::help_keys::SEARCH_THE_COMMIT_3_KEYS), msg::help_text::SEARCH_THE_COMMIT_3),
    Item(Msg(msg::help_keys::SEARCH_THE_COMMIT_4_KEYS), msg::help_text::SEARCH_THE_COMMIT_4),
    Item(Msg(msg::help_keys::SEARCH_THE_COMMIT_5_KEYS), msg::help_text::SEARCH_THE_COMMIT_5),
    Item(keys!(""), msg::help_text::SEARCH_THE_COMMIT_6),
    Sec(msg::help_section::CONTEXT_MENUS_COMMIT),
    Item(keys!("j / k, Up/Down"), msg::help_text::CONTEXT_MENUS_COMMIT_1),
    Item(keys!("Home/End"), msg::help_text::CONTEXT_MENUS_COMMIT_2),
    Item(keys!("Enter / Space"), msg::help_text::CONTEXT_MENUS_COMMIT_3),
    Item(keys!("Right / l"), msg::help_text::CONTEXT_MENUS_COMMIT_4),
    Item(keys!("Left / h / Esc"), msg::help_text::CONTEXT_MENUS_COMMIT_5),
    Item(Msg(msg::help_keys::CONTEXT_MENUS_COMMIT_6_KEYS), msg::help_text::CONTEXT_MENUS_COMMIT_6),
    Item(keys!("Enter / l"), msg::help_text::CONTEXT_MENUS_COMMIT_7),
    Item(keys!("Esc / h"), msg::help_text::CONTEXT_MENUS_COMMIT_8),
    Item(keys!("Right"), msg::help_text::CONTEXT_MENUS_COMMIT_9),
    Sec(msg::help_section::DIFF_VIEW_REPLACES),
    Item(Msg(msg::help_keys::DIFF_VIEW_REPLACES_1_KEYS), msg::help_text::DIFF_VIEW_REPLACES_1),
    Item(Msg(msg::help_keys::DIFF_VIEW_REPLACES_2_KEYS), msg::help_text::DIFF_VIEW_REPLACES_2),
    Item(keys!("j / k, Up/Down"), msg::help_text::DIFF_VIEW_REPLACES_3),
    Item(keys!("PageUp/PageDown, Home/End"), msg::help_text::DIFF_VIEW_REPLACES_4),
    Item(keys!("h / l, Left/Right"), msg::help_text::DIFF_VIEW_REPLACES_5),
    Item(keys!("{diff.next_hunk}"), msg::help_text::DIFF_VIEW_REPLACES_6),
    Item(keys!("{diff.prev_hunk}"), msg::help_text::DIFF_VIEW_REPLACES_7),
    Item(keys!("{diff.toggle_layout}"), msg::help_text::DIFF_VIEW_REPLACES_8),
    Item(keys!("{diff.toggle_hunks_only}"), msg::help_text::DIFF_VIEW_REPLACES_9),
    Item(keys!("{diff.toggle_wrap}"), msg::help_text::DIFF_VIEW_REPLACES_10),
    Item(keys!("{diff.toggle_minimap}"), msg::help_text::DIFF_VIEW_REPLACES_11),
    Item(keys!("{diff.choose_encoding}"), msg::help_text::DIFF_VIEW_ENCODING),
    Item(keys!("{diff.toggle_whitespace}"), msg::help_text::DIFF_VIEW_REPLACES_12),
    Item(keys!("{diff.stage_hunk}"), msg::help_text::DIFF_VIEW_REPLACES_13),
    Item(keys!("{diff.select_lines}"), msg::help_text::DIFF_VIEW_REPLACES_14),
    Item(keys!("{diff.stage_lines}"), msg::help_text::DIFF_VIEW_REPLACES_15),
    Item(keys!("{diff.discard_hunk}"), msg::help_text::DIFF_VIEW_REPLACES_16),
    Item(keys!("{diff.discard_lines}"), msg::help_text::DIFF_VIEW_REPLACES_17),
    Item(Msg(msg::help_keys::DIFF_VIEW_REPLACES_18_KEYS), msg::help_text::DIFF_VIEW_REPLACES_18),
    Item(keys!("{diff.copy}"), msg::help_text::DIFF_VIEW_REPLACES_19),
    Item(keys!("Esc"), msg::help_text::DIFF_VIEW_REPLACES_20),
    Item(keys!(""), msg::help_text::DIFF_VIEW_REPLACES_21),
    Item(Msg(msg::help_keys::DIFF_VIEW_REPLACES_22_KEYS), msg::help_text::DIFF_VIEW_REPLACES_22),
    Item(keys!("{diff.lfs_raw}"), msg::help_text::DIFF_VIEW_REPLACES_23),
    Item(keys!(""), msg::help_text::DIFF_VIEW_REPLACES_24),
    Item(keys!(""), msg::help_text::DIFF_VIEW_REPLACES_25),
    Sec(msg::help_section::FILE_VIEW_BLAME),
    Item(keys!("Esc / {file_insight.close}"), msg::help_text::FILE_VIEW_BLAME_1),
    Item(keys!("j / k, Up/Down"), msg::help_text::FILE_VIEW_BLAME_2),
    Item(keys!("PageUp/PageDown"), msg::help_text::FILE_VIEW_BLAME_3),
    Item(keys!("Home"), msg::help_text::FILE_VIEW_BLAME_4),
    Item(keys!("End / {file_insight.last_row}"), msg::help_text::FILE_VIEW_BLAME_5),
    Item(keys!("h / l, Left/Right"), msg::help_text::FILE_VIEW_BLAME_6),
    Item(keys!("{file_insight.open_commit}"), msg::help_text::FILE_VIEW_BLAME_7),
    Item(keys!("{file_insight.copy}"), msg::help_text::FILE_VIEW_BLAME_8),
    Item(keys!("Esc"), msg::help_text::FILE_VIEW_BLAME_9),
    Item(keys!("{file_insight.blame_next} / {file_insight.blame_prev}"), msg::help_text::FILE_VIEW_BLAME_18),
    Item(keys!("{file_insight.blame_next_same} / {file_insight.blame_prev_same}"), msg::help_text::FILE_VIEW_BLAME_19),
    Item(keys!("{file_insight.blame_goto}"), msg::help_text::FILE_VIEW_BLAME_20),
    Item(keys!("{file_insight.blame_card}"), msg::help_text::FILE_VIEW_BLAME_21),
    Item(
        keys!("{file_insight.blame_diff} / {file_insight.blame_parent} / {file_insight.blame_history} / {file_insight.blame_copy_sha}"),
        msg::help_text::FILE_VIEW_BLAME_22,
    ),
    Item(keys!("{file_insight.blame_wider} / {file_insight.blame_narrower}"), msg::help_text::FILE_VIEW_BLAME_23),
    Item(keys!(""), msg::help_text::FILE_VIEW_BLAME_10),
    Item(Msg(msg::help_keys::FILE_VIEW_BLAME_11_KEYS), msg::help_text::FILE_VIEW_BLAME_11),
    Item(Msg(msg::help_keys::FILE_VIEW_BLAME_12_KEYS), msg::help_text::FILE_VIEW_BLAME_12),
    Item(Msg(msg::help_keys::FILE_VIEW_BLAME_13_KEYS), msg::help_text::FILE_VIEW_BLAME_13),
    Item(keys!(""), msg::help_text::FILE_VIEW_BLAME_14),
    Item(
        keys!("{image.zoom_in} / {image.zoom_out} / {image.fit} / {image.actual_size}"),
        msg::help_text::FILE_VIEW_BLAME_15,
    ),
    Item(
        keys!("{image.edit_in} / {image.edit_default}"),
        msg::help_text::FILE_VIEW_BLAME_16,
    ),
    Item(
        keys!("{image.flip_side} / {image.widen_before} / {image.widen_after} / {image.reset_split}"),
        msg::help_text::FILE_VIEW_BLAME_17,
    ),
    Sec(msg::help_section::FILE_HISTORY),
    Item(keys!("{history.show_in_log}"), msg::help_text::FILE_HISTORY_1),
    Item(keys!("j / k, Up/Down, PageUp/PageDown, Home/End"), msg::help_text::FILE_HISTORY_2),
    Item(keys!("{history.range_up} / {history.range_down}"), msg::help_text::FILE_HISTORY_3),
    Item(keys!("{history.switch_focus}"), msg::help_text::FILE_HISTORY_4),
    Item(keys!("{history.copy_sha} / {history.copy_summary}"), msg::help_text::FILE_HISTORY_5),
    Item(keys!("{history.show_diff} / {history.open_file} / {history.blame}"), msg::help_text::FILE_HISTORY_6),
    Item(keys!("{history.compare_working_tree}"), msg::help_text::FILE_HISTORY_7),
    Item(keys!("{history.restore}"), msg::help_text::FILE_HISTORY_8),
    Item(
        keys!("{history.widen_list} / {history.narrow_list} / {history.reset_split}"),
        msg::help_text::FILE_HISTORY_9,
    ),
    Item(keys!(""), msg::help_text::FILE_HISTORY_10),
    Sec(msg::help_section::COMMIT_COLUMN),
    Item(
        keys!("{commit_column.scroll_left} / {commit_column.scroll_right}"),
        msg::help_text::COMMIT_COLUMN_1,
    ),
    Item(
        keys!("{commit_column.next_part} / {commit_column.wider_part} / {commit_column.narrower_part} / {commit_column.reset_widths} / {commit_column.fit} / {commit_column.fit_all}"),
        msg::help_text::COMMIT_COLUMN_2,
    ),
    Item(keys!(""), msg::help_text::COMMIT_COLUMN_3),
    Item(
        keys!("{commit_column.move_left} / {commit_column.move_right}"),
        msg::help_text::COMMIT_COLUMN_4,
    ),
    Sec(msg::help_section::EDITOR_EDIT_IN),
    Item(Msg(msg::help_keys::EDITOR_EDIT_IN_1_KEYS), msg::help_text::EDITOR_EDIT_IN_1),
    Item(keys!("Tab / Shift+Tab"), msg::help_text::EDITOR_EDIT_IN_2),
    Item(Msg(msg::help_keys::EDITOR_EDIT_IN_3_KEYS), msg::help_text::EDITOR_EDIT_IN_3),
    Item(keys!("Ctrl+A / Ctrl+C / Ctrl+X"), msg::help_text::EDITOR_EDIT_IN_4),
    Item(keys!("{editor.paste}"), msg::help_text::EDITOR_EDIT_IN_5),
    Item(keys!("Ctrl+U / Ctrl+K"), msg::help_text::EDITOR_EDIT_IN_6),
    Item(keys!("{editor.undo}"), msg::help_text::EDITOR_EDIT_IN_7),
    Item(keys!("{editor.redo}"), msg::help_text::EDITOR_EDIT_IN_8),
    Item(keys!("{editor.save}"), msg::help_text::EDITOR_EDIT_IN_9),
    Item(keys!("{editor.find}"), msg::help_text::EDITOR_EDIT_IN_10),
    Item(keys!("{editor.find_next}"), msg::help_text::EDITOR_EDIT_IN_11),
    Item(keys!("{editor.toggle_minimap}"), msg::help_text::DIFF_VIEW_REPLACES_11),
    Item(keys!("Esc"), msg::help_text::EDITOR_EDIT_IN_12),
    Item(keys!(""), msg::help_text::EDITOR_EDIT_IN_13),
    Item(keys!(""), msg::help_text::EDITOR_EDIT_IN_14),
    Item(keys!(""), msg::help_text::EDITOR_EDIT_IN_15),
    Sec(msg::help_section::MERGE_CONFLICT_EDITOR),
    Item(keys!(""), msg::help_text::MERGE_CONFLICT_EDITOR_1),
    Item(keys!("{merge_sides.toggle_line}"), msg::help_text::MERGE_CONFLICT_EDITOR_2),
    Item(keys!("{merge_sides.toggle_chunk}"), msg::help_text::MERGE_CONFLICT_EDITOR_3),
    Item(keys!("{merge_sides.take_a} / {merge_sides.take_b} / {merge_sides.take_both} / {merge_sides.take_both_reversed}"), msg::help_text::MERGE_CONFLICT_EDITOR_4),
    Item(keys!("{merge_sides.clear}"), msg::help_text::MERGE_CONFLICT_EDITOR_5),
    Item(keys!("{merge_sides.swap_order}"), msg::help_text::MERGE_CONFLICT_EDITOR_6),
    Item(keys!("{merge_sides.take_all_a} / {merge_sides.take_all_b}"), msg::help_text::MERGE_CONFLICT_EDITOR_7),
    Item(keys!("{merge_sides.next_conflict} / {merge_sides.prev_conflict}"), msg::help_text::MERGE_CONFLICT_EDITOR_8),
    Item(keys!("{merge_sides.toggle_base}"), msg::help_text::MERGE_CONFLICT_EDITOR_9),
    Item(keys!("{merge_sides.toggle_full_file}"), msg::help_text::MERGE_CONFLICT_EDITOR_10),
    Item(keys!("{merge_sides.cycle_height}"), msg::help_text::MERGE_CONFLICT_EDITOR_11),
    Item(keys!("{merge.next_conflict} / {merge.prev_conflict}"), msg::help_text::MERGE_CONFLICT_EDITOR_12),
    Item(keys!("{merge.next_file} / {merge.prev_file}"), msg::help_text::MERGE_CONFLICT_EDITOR_13),
    Item(keys!("{merge.next_pane} / {merge.prev_pane}"), msg::help_text::MERGE_CONFLICT_EDITOR_14),
    Item(keys!("{merge.toggle_base}"), msg::help_text::MERGE_CONFLICT_EDITOR_15),
    Item(keys!("{merge_sides.scroll_left} / {merge_sides.scroll_right}"), msg::help_text::MERGE_CONFLICT_EDITOR_16),
    Item(keys!("{merge.toggle_panes}"), msg::help_text::MERGE_CONFLICT_EDITOR_17),
    Item(keys!("{merge.toggle_full_file}"), msg::help_text::MERGE_CONFLICT_EDITOR_18),
    Item(keys!("{merge.undo} / {merge.redo}"), msg::help_text::MERGE_CONFLICT_EDITOR_19),
    Item(keys!("{merge.ai_suggest}"), msg::help_text::MERGE_CONFLICT_EDITOR_20),
    Item(keys!("{merge.save}"), msg::help_text::MERGE_CONFLICT_EDITOR_21),
    Item(keys!("{merge.save_resolve}"), msg::help_text::MERGE_CONFLICT_EDITOR_22),
    Item(keys!("Esc"), msg::help_text::MERGE_CONFLICT_EDITOR_23),
    Item(keys!(""), msg::help_text::MERGE_CONFLICT_EDITOR_24),
    Item(keys!(""), msg::help_text::MERGE_CONFLICT_EDITOR_25),
    Item(keys!(""), msg::help_text::MERGE_CONFLICT_EDITOR_26),
    Item(keys!("{merge_card.take_a} / {merge_card.take_b} / {merge_card.take_base}"), msg::help_text::MERGE_CONFLICT_EDITOR_27),
    Item(keys!("{merge_card.keep} / {merge_card.delete}"), msg::help_text::MERGE_CONFLICT_EDITOR_28),
    Item(keys!("{merge_card.external_tool}"), msg::help_text::MERGE_CONFLICT_EDITOR_29),
    Item(keys!("{merge.widen_incoming} / {merge.widen_current} / {merge.reset_panes}"), msg::help_text::MERGE_CONFLICT_EDITOR_30),
    Item(keys!("{merge_sides.unpick} / {merge_sides.reset_all} / {merge.unpick} / {merge.reset_all}"), msg::help_text::MERGE_CONFLICT_EDITOR_31),
    Item(keys!("h / l / ← / →"), msg::help_text::MERGE_CONFLICT_EDITOR_32),
    Item(keys!("{merge.focus_a} / {merge.focus_b} / {merge.focus_output} / {merge.focus_base}"), msg::help_text::MERGE_CONFLICT_EDITOR_33),
    Item(keys!("3j / 2k / 3n"), msg::help_text::MERGE_CONFLICT_EDITOR_34),
    Item(keys!(""), msg::help_text::MERGE_CONFLICT_EDITOR_35),
    Sec(msg::help_section::REPOSITORIES_AND_TABS),
    Item(keys!("⌂ / {global.home}"), msg::help_text::REPOSITORIES_AND_TABS_1),
    Item(keys!("{global.clone}"), msg::help_text::REPOSITORIES_AND_TABS_2),
    Item(keys!("{global.next_tab}"), msg::help_text::REPOSITORIES_AND_TABS_3),
    Item(keys!("{global.prev_tab}"), msg::help_text::REPOSITORIES_AND_TABS_4),
    Item(keys!("{global.new_tab}"), msg::menu_repo::NEW_TAB),
    Item(keys!("{global.radar}"), msg::radar::HELP_OPEN),
    Item(keys!("{global.close_tab}"), msg::help_text::REPOSITORIES_AND_TABS_5),
    Item(Msg(msg::help_keys::REPOSITORIES_AND_TABS_6_KEYS), msg::help_text::REPOSITORIES_AND_TABS_6),
    Item(keys!("{global.tab_open_in_popup} / {global.tab_open_in_default}"), msg::help_text::REPOSITORIES_AND_TABS_OPEN_IN),
    Sub(msg::help_section::REPOSITORIES_AND_TABS_SUB),
    Item(Msg(msg::help_keys::REPOSITORIES_AND_TABS_7_KEYS), msg::help_text::REPOSITORIES_AND_TABS_7),
    Item(keys!("{home.favorite}"), msg::help_text::REPOSITORIES_AND_TABS_8),
    Item(keys!("{home.close_repo}"), msg::help_text::REPOSITORIES_AND_TABS_9),
    Item(keys!("{home.copy_path} / {home.open_in}"), msg::help_text::REPOSITORIES_AND_TABS_COPY_OPEN),
    Item(keys!("{*global.context_menu}"), msg::help_text::REPOSITORIES_AND_TABS_10),
    Item(Msg(msg::help_keys::REPOSITORIES_AND_TABS_11_KEYS), msg::help_text::REPOSITORIES_AND_TABS_11),
    Item(keys!("Esc"), msg::help_text::REPOSITORIES_AND_TABS_12),
    Item(keys!("Space / Left / Right"), msg::help_text::REPOSITORIES_AND_TABS_13),
    Item(keys!("{home.collapse_all} / {home.expand_all}"), msg::help_text::REPOSITORIES_AND_TABS_14),
    Item(keys!("{home.browse} / {home.clone} / {home.init}"), msg::help_text::REPOSITORIES_AND_TABS_15),
    Item(keys!("{global.workspace_switch}"), msg::workspaces::HELP_SWITCH),
    Item(keys!("{global.workspaces}"), msg::keymap_action::GLOBAL_WORKSPACES),
    Item(keys!("{home.workspace_new} / {home.workspace_from_tabs}"), msg::workspaces::HELP_NEW),
    Item(keys!("{home.workspace_edit}"), msg::workspaces::HELP_EDIT),
    Item(keys!("{home.workspace_fetch} / {home.workspace_pull}"), msg::workspaces::HELP_WALK),
    Item(keys!("{home.workspace_clone_missing} / {home.workspace_locate}"), msg::workspaces::HELP_MISSING),
    Item(keys!("{global.quit}"), msg::help_text::REPOSITORIES_AND_TABS_16),
    Item(Msg(msg::help_keys::REPOSITORIES_AND_TABS_17_KEYS), msg::help_text::REPOSITORIES_AND_TABS_17),
    Item(keys!("{global.palette}"), msg::help_text::REPOSITORIES_AND_TABS_18),
    Item(keys!("{global.git_prefix} / {global.close_tab} / {global.tab_menu}"), msg::help_text::REPOSITORIES_AND_TABS_19),
    Item(keys!(""), msg::help_text::REPOSITORIES_AND_TABS_20),
    Sub(msg::help_section::REPOSITORIES_AND_TABS_SUB_2),
    Item(keys!("Tab / Shift+Tab"), msg::help_text::REPOSITORIES_AND_TABS_21),
    Item(keys!("Enter"), msg::help_text::REPOSITORIES_AND_TABS_22),
    Item(keys!("Backspace / Alt+Up"), msg::help_text::REPOSITORIES_AND_TABS_23),
    Item(keys!("{folder_picker.choose}"), msg::help_text::REPOSITORIES_AND_TABS_24),
    Item(keys!("Esc"), msg::help_text::REPOSITORIES_AND_TABS_25),
    Item(keys!("{folder_picker.new_folder}"), msg::help_text::REPOSITORIES_AND_TABS_26),
    Item(keys!("{folder_picker.toggle_hidden}"), msg::help_text::REPOSITORIES_AND_TABS_27),
    Item(keys!("{folder_picker.toggle_siblings}"), msg::help_text::REPOSITORIES_AND_TABS_40),
    Item(keys!("{folder_picker.filter}"), msg::help_text::REPOSITORIES_AND_TABS_39),
    Item(keys!("Alt+Left / Alt+Right"), msg::help_text::REPOSITORIES_AND_TABS_28),
    Item(Msg(msg::help_keys::REPOSITORIES_AND_TABS_29_KEYS), msg::help_text::REPOSITORIES_AND_TABS_29),
    Sub(msg::help_section::REPOSITORIES_AND_TABS_SUB_3),
    Item(keys!("Tab"), msg::help_text::REPOSITORIES_AND_TABS_30),
    Item(keys!("Enter"), msg::help_text::REPOSITORIES_AND_TABS_31),
    Item(keys!("{dialog.browse}"), msg::help_text::REPOSITORIES_AND_TABS_32),
    Item(keys!("{dialog.submit}"), msg::help_text::REPOSITORIES_AND_TABS_33),
    Item(keys!("{dialog.web}"), msg::help_text::REPOSITORIES_AND_TABS_37),
    Item(keys!("{dialog.sources}"), msg::help_text::REPOSITORIES_AND_TABS_38),
    Item(keys!("Ctrl+U"), msg::help_text::REPOSITORIES_AND_TABS_34),
    Item(keys!("Esc"), msg::help_text::REPOSITORIES_AND_TABS_35),
    Item(keys!(""), msg::help_text::REPOSITORIES_AND_TABS_36),
    Sec(msg::help_section::MARKDOWN_EDITOR),
    Item(keys!("{md_editor.preview}"), msg::help_text::MARKDOWN_EDITOR_1),
    Item(keys!("{md_editor.split}"), msg::help_text::MARKDOWN_EDITOR_2),
    Item(keys!("{md_editor.submit}"), msg::help_text::MARKDOWN_EDITOR_3),
    Item(keys!("{md_editor.ai}"), msg::help_text::MARKDOWN_EDITOR_4),
    Sec(msg::help_section::AI_RESULT_POPUPS),
    Item(keys!("{ai_popup.copy}"), msg::help_text::AI_RESULT_POPUPS_1),
    Sec(msg::help_section::SETTINGS_INTEGRATIONS_THE),
    Item(keys!("{integrations.add}"), msg::help_text::SETTINGS_INTEGRATIONS_THE_1),
    Item(keys!("{integrations.edit}"), msg::help_text::SETTINGS_INTEGRATIONS_THE_2),
    Item(keys!("{integrations.open_token}"), msg::help_text::SETTINGS_INTEGRATIONS_THE_3),
    Item(keys!("{integrations.remove}"), msg::help_text::SETTINGS_INTEGRATIONS_THE_4),
    Sec(msg::help_section::SETTINGS_PREFERENCES_THE),
    Item(keys!("Tab / Shift+Tab"), msg::help_text::SETTINGS_PREFERENCES_THE_1),
    Item(keys!("h / l, Left/Right"), msg::help_text::SETTINGS_PREFERENCES_THE_2),
    Item(keys!("j / k, Up/Down"), msg::help_text::SETTINGS_PREFERENCES_THE_3),
    Item(keys!("Enter / Space"), msg::help_text::SETTINGS_PREFERENCES_THE_4),
    Item(Msg(msg::help_keys::SETTINGS_PREFERENCES_THE_5_KEYS), msg::help_text::SETTINGS_PREFERENCES_THE_5),
    Item(keys!("{preferences.search}"), msg::help_text::SETTINGS_PREFERENCES_THE_6),
    Item(keys!("{preferences.list_add}"), msg::help_text::SETTINGS_LIST_1),
    Item(keys!("{preferences.list_duplicate}"), msg::help_text::SETTINGS_LIST_2),
    Item(keys!("{preferences.list_remove}"), msg::help_text::SETTINGS_LIST_3),
    Item(keys!("{preferences.list_move_up}"), msg::help_text::SETTINGS_LIST_4),
    Item(keys!("{preferences.list_move_down}"), msg::help_text::SETTINGS_LIST_5),
    Item(keys!("{preferences.prev_tab} / {preferences.next_tab}"), msg::help_text::SETTINGS_LIST_6),
    Item(keys!("Esc / q"), msg::help_text::SETTINGS_PREFERENCES_THE_7),
    Item(keys!("?"), msg::help_text::SETTINGS_PREFERENCES_THE_8),
    Item(keys!(""), msg::help_text::SETTINGS_PREFERENCES_THE_9),
    Sec(msg::help_section::TEXT_FIELDS_FILTERS),
    Item(keys!("Left/Right, Home/End"), msg::help_text::TEXT_FIELDS_FILTERS_1),
    Item(Msg(msg::help_keys::TEXT_FIELDS_FILTERS_2_KEYS), msg::help_text::TEXT_FIELDS_FILTERS_2),
    Item(keys!("Ctrl+A"), msg::help_text::TEXT_FIELDS_FILTERS_3),
    Item(keys!("Ctrl+U / Ctrl+K"), msg::help_text::TEXT_FIELDS_FILTERS_4),
    Item(keys!("Ctrl+C / Ctrl+X"), msg::help_text::TEXT_FIELDS_FILTERS_5),
    Item(keys!(""), msg::help_text::TEXT_FIELDS_FILTERS_6),
    Sec(msg::help_section::MOUSE_GESTURES_IN),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_1_KEYS), msg::help_text::MOUSE_GESTURES_IN_1),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_2_KEYS), msg::help_text::MOUSE_GESTURES_IN_2),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_3_KEYS), msg::help_text::MOUSE_GESTURES_IN_3),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_4_KEYS), msg::help_text::MOUSE_GESTURES_IN_4),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_5_KEYS), msg::help_text::MOUSE_GESTURES_IN_5),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_6_KEYS), msg::help_text::MOUSE_GESTURES_IN_6),
    Item(Msg(msg::help_keys::MOUSE_GESTURES_IN_7_KEYS), msg::help_text::MOUSE_GESTURES_IN_7),
];

/// Text column of a top-level row (two spaces of indent, a 16-wide key
/// column); nested rows sit two cells further in.
const TEXT_COL: usize = 18;
/// The width `filter_help_text` lays out for (no terminal at hand).
#[cfg(test)]
const DEFAULT_WIDTH: usize = 100;
/// A key that overflows its column stays on the description's first line
/// only when at least this much room is left for the description there.
const MIN_FIRST_LINE: usize = 32;

/// Replaces every `{ctx.action}` in `s` with the action's effective label
/// under `km` (`{*..}` lists every binding, ` / `-separated). Anything else
/// in braces is left alone.
pub(crate) fn resolve_keys(km: &keymap::Keymap, s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}').map(|i| open + i) else {
            break;
        };
        let id = &rest[open + 1..close];
        let (all, id) = match id.strip_prefix('*') {
            Some(id) => (true, id),
            None => (false, id),
        };
        out.push_str(&rest[..open]);
        match keymap::ACTIONS.iter().find(|d| d.config_id() == id) {
            Some(def) => {
                let label = if all {
                    km.shown_labels(def.action).join(" / ")
                } else {
                    km.shown(def.action)
                };
                out.push_str(if label.is_empty() {
                    "(unbound)"
                } else {
                    &label
                });
            }
            None => out.push_str(&rest[open..=close]),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Greedy word wrap on ASCII spaces (a no-break space stays inside its
/// word), `first` cells on the first line and `rest` on the others. A word
/// wider than the line is cut at the column.
pub(crate) fn wrap(text: &str, first: usize, rest: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut used = 0usize;
    let mut limit = first.max(1);
    for word in text.split(' ').filter(|w| !w.is_empty()) {
        let mut word = word.to_string();
        let mut w = display_width(&word);
        loop {
            let sep = usize::from(used > 0);
            if used + sep + w <= limit {
                if sep == 1 {
                    line.push(' ');
                }
                line.push_str(&word);
                used += sep + w;
                break;
            }
            if used > 0 {
                lines.push(std::mem::take(&mut line));
                used = 0;
                limit = rest.max(1);
                continue;
            }
            // Alone on an empty line and still too wide: cut it.
            let mut cut = String::new();
            let mut cw = 0;
            let mut chars = word.chars();
            let mut taken = 0;
            for c in chars.by_ref() {
                let cwid = display_width(c.encode_utf8(&mut [0; 4]));
                if cw + cwid > limit && !cut.is_empty() {
                    break;
                }
                cut.push(c);
                taken += c.len_utf8();
                cw += cwid;
            }
            lines.push(cut);
            limit = rest.max(1);
            word = word[taken..].to_string();
            w = display_width(&word);
            if word.is_empty() {
                break;
            }
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// One paragraph: a section and its rows, with nesting levels.
struct Para {
    title: M,
    rows: Vec<Row>,
    /// Rows left out because this build lacks what they describe.
    hidden: usize,
}

enum Row {
    Sub(M),
    Item(HelpKeys, M),
}

fn paragraphs() -> Vec<Para> {
    let mut out: Vec<Para> = Vec::new();
    for e in HELP_TEXT {
        match *e {
            Sec(title) => out.push(Para {
                title,
                rows: Vec::new(),
                hidden: 0,
            }),
            Sub(m) => out
                .last_mut()
                .expect("a section comes first")
                .rows
                .push(Row::Sub(m)),
            Gated(feature, k, m) => {
                let para = out.last_mut().expect("a section comes first");
                if crate::build_features::offered(Some(feature)) {
                    para.rows.push(Row::Item(k, m));
                } else {
                    para.hidden += 1;
                }
            }
            Item(k, m) => {
                let para = out.last_mut().expect("a section comes first");
                // A row for an action this build lacks is not shown.
                if matches!(k, Lit(s) if built_parts(s).is_empty()) {
                    para.hidden += 1;
                } else {
                    para.rows.push(Row::Item(k, m));
                }
            }
        }
    }
    // A section whose every row belongs to a feature this build lacks (the
    // History workbench's) has no heading left either.
    out.retain(|p| !(p.rows.is_empty() && p.hidden > 0));
    out
}

/// The parts of a literal key column that this build offers: the
/// `{ctx.action}` placeholders of actions it lacks (`a / {b}` where `b` is
/// not built) are left out. Empty = the whole row is dropped.
fn built_parts(s: &str) -> Vec<&str> {
    let unbuilt = |part: &str| {
        let id = part.trim().strip_prefix('{')?.strip_suffix('}')?;
        let id = id.strip_prefix('*').unwrap_or(id);
        keymap::ACTIONS
            .iter()
            .find(|d| d.config_id() == id)
            .map(|d| !d.built())
    };
    s.split(" / ")
        .filter(|p| unbuilt(p) != Some(true))
        .collect()
}

fn key_text(km: &keymap::Keymap, keys: HelpKeys, lang: Option<Lang>) -> String {
    match keys {
        Lit(s) => resolve_keys(km, &built_parts(s).join(" / ")),
        Msg(m) => resolve_keys(km, lang.map_or_else(|| m.get(), |l| m.get_in(l))),
    }
}

/// Everything a paragraph says, lower-cased, in the current language and in
/// English: what the filter matches against.
fn haystack(km: &keymap::Keymap, p: &Para) -> String {
    let mut out = String::new();
    for lang in [None, Some(Lang::En)] {
        let one = |m: M| resolve_keys(km, lang.map_or_else(|| m.get(), |l| m.get_in(l)));
        out.push_str(&one(p.title));
        for r in &p.rows {
            out.push('\n');
            match r {
                Row::Sub(m) => out.push_str(&one(*m)),
                Row::Item(k, m) => {
                    out.push_str(&key_text(km, *k, lang));
                    out.push(' ');
                    out.push_str(&one(*m));
                }
            }
        }
        out.push('\n');
    }
    out.to_lowercase()
}

/// The help body laid out for `width` cells under `km`, narrowed to the
/// paragraphs that match `filter`.
pub(crate) fn help_lines(km: &keymap::Keymap, filter: &str, width: usize) -> Vec<String> {
    let width = width.max(24);
    let needle = filter.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for p in paragraphs() {
        if !needle.is_empty() && !haystack(km, &p).contains(&needle) {
            continue;
        }
        if !out.is_empty() {
            out.push(String::new());
        }
        for (i, l) in wrap(&resolve_keys(km, p.title.get()), width, width - 2)
            .into_iter()
            .enumerate()
        {
            out.push(if i == 0 { l } else { format!("  {l}") });
        }
        let mut nested = 0usize;
        for r in &p.rows {
            let (keys, text) = match r {
                Row::Sub(m) => {
                    nested = 2;
                    for l in wrap(&resolve_keys(km, m.get()), width - 2, width - 2) {
                        out.push(format!("  {l}"));
                    }
                    continue;
                }
                Row::Item(k, m) => (key_text(km, *k, None), resolve_keys(km, m.get())),
            };
            layout_row(&mut out, &keys, &text, nested, width);
        }
    }
    out
}

/// Appends one row: `keys` in the key column, `text` wrapped beside it.
fn layout_row(out: &mut Vec<String>, keys: &str, text: &str, nested: usize, width: usize) {
    let indent = 2 + nested;
    let col = TEXT_COL + nested;
    let pad = " ".repeat(indent);
    let hang = " ".repeat(col);
    let rest_w = width.saturating_sub(col).max(1);
    let key_w = display_width(keys);
    if keys.is_empty() {
        for (i, l) in wrap(text, width.saturating_sub(indent), rest_w)
            .into_iter()
            .enumerate()
        {
            out.push(if i == 0 {
                format!("{pad}{l}")
            } else {
                format!("{hang}{l}")
            });
        }
        return;
    }
    let (head_w, own_line) = if indent + key_w + 2 <= col {
        (rest_w, false)
    } else {
        let first = width.saturating_sub(indent + key_w + 2);
        (first, first < MIN_FIRST_LINE)
    };
    if own_line {
        for l in wrap(
            keys,
            width.saturating_sub(indent),
            width.saturating_sub(indent),
        ) {
            out.push(format!("{pad}{l}"));
        }
        for l in wrap(text, rest_w, rest_w) {
            out.push(format!("{hang}{l}"));
        }
        return;
    }
    let gap = if indent + key_w + 2 <= col {
        " ".repeat(col - indent - key_w)
    } else {
        "  ".to_string()
    };
    for (i, l) in wrap(text, head_w, rest_w).into_iter().enumerate() {
        out.push(if i == 0 {
            format!("{pad}{keys}{gap}{l}")
        } else {
            format!("{hang}{l}")
        });
    }
}

/// The help body for the overlay, `width` cells wide, as one string.
pub(crate) fn help_body(filter: &str, width: usize) -> String {
    help_lines(&keymap::current(), filter, width).join("\n")
}

#[cfg(test)]
/// Narrows the help to the paragraphs matching `filter` (an empty filter
/// keeps everything). A paragraph is a section title and its rows: it stays
/// whole when any of its text, in the current language or in English,
/// contains `filter` case-insensitively. Pure, so it is unit-tested here
/// rather than through a pty run.
pub(crate) fn filter_help_text(filter: &str) -> String {
    filter_help_text_with(&keymap::current(), filter)
}

#[cfg(test)]
/// [`filter_help_text`] under `km`'s bindings, laid out at a default width.
pub(crate) fn filter_help_text_with(km: &keymap::Keymap, filter: &str) -> String {
    help_lines(km, filter, DEFAULT_WIDTH).join("\n")
}
