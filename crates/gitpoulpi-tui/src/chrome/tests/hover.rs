use super::*;

/// What a `HitTarget` variant's hover feedback is. `Required` names the
/// mechanism that lights it; `NotApplicable` requires the reason a clickable
/// target has no hover of its own. There is no "known gap": a hover that is
/// missing is a bug to fix, not an entry to add (maintainer U1).
///
/// `hover_style_for` is an exhaustive match over `HitTarget` (no `_ =>`):
/// a new variant without an arm is a compile error in the test build, so
/// every new clickable target must be classified.
enum HoverKind {
    /// Has a real hover treatment, named here (and buffer-tested where its
    /// owner module has a test).
    Required(&'static str),
    /// Clickable but deliberately without a fill of its own, with why.
    NotApplicable(&'static str),
}

fn hover_style_for(target: &HitTarget) -> HoverKind {
    use HoverKind::{NotApplicable, Required};
    match target {
        HitTarget::Toolbar(_) => Required("event_loop.rs toolbar button fill, hand-verified"),
        HitTarget::ToolbarDropdown(_) => Required("event_loop.rs dropdown chip fill"),
        HitTarget::RemoteMenuItem(_) => Required("context_menu::Menu::render(hovered)"),
        HitTarget::RemoteForm(_) => Required("remotes_ui::RemoteForm::render(hovered)"),
        HitTarget::RewordForm(_) => {
            Required("reword_ui::RewordForm::render(hovered): Cancel / Apply buttons")
        }
        HitTarget::AddRemoteButton => Required("plain_button_style, plain at rest (item 15)"),
        HitTarget::LfsInstallLink => Required("event_loop.rs hover_style on the banner rows"),
        HitTarget::Gear => Required("render_gear -> plain_button_style"),
        HitTarget::About => Required("render_about_button -> plain_button_style"),
        HitTarget::ProfileField => Required("profile_bar::render_field -> Combo::render_field(hovered)"),
        HitTarget::ProfileItem(_) => Required("profile_bar::render_popup -> Combo::set_hover"),
        HitTarget::WorkspaceChip => Required("repo_switcher::tab_bar_with new_tab.chip.hovered"),
        HitTarget::WorkspaceItem(_) => Required("workspace_ui::chip::render_popup -> Combo::set_hover"),
        HitTarget::WorkspacePopup => NotApplicable(
            "the popup's frame: a shield so nothing behind it hovers, no action of its own",
        ),
        HitTarget::ProfilePopup => NotApplicable(
            "the popup's frame: a shield so nothing behind it hovers, no action of its own",
        ),
        HitTarget::CloseDiff => Required("event_loop.rs hover_style on the title span"),
        HitTarget::OpenRepositoryButton => Required("repo_switcher::tab_bar_with folder_hovered"),
        HitTarget::NewTabButton => Required("repo_switcher::tab_bar_with new_tab.plus_hovered"),
        HitTarget::TabOverflow => Required("repo_switcher::tab_bar_with new_tab.more_hovered"),
        HitTarget::RadarTab => Required("repo_switcher::tab_bar_with new_tab.radar.hovered"),
        HitTarget::RadarClose => {
            Required("repo_switcher::tab_bar_with new_tab.radar.close_hovered")
        }
        HitTarget::WorkspacesTab => {
            Required("repo_switcher::tab_bar_with new_tab.workspaces.hovered")
        }
        HitTarget::WorkspacesClose => {
            Required("repo_switcher::tab_bar_with new_tab.workspaces.close_hovered")
        }
        HitTarget::NotesTab => NotApplicable(
            "the Release notes tab is always the selected one while it exists: no hover state to light",
        ),
        HitTarget::NotesClose => {
            Required("repo_switcher::tab_bar_with new_tab.notes.close_hovered")
        }
        HitTarget::GlobalSearchButton => Required("draw_chrome -> plain_button_style"),
        HitTarget::TabDir(_) => Required("render_tab_dir_buttons -> Combo::render_field(hovered)"),
        HitTarget::OpenInItem(_) => Required("open_in_bar::render_popup -> Combo::set_hover"),
        HitTarget::OpenWith => Required(
            "open_with::view::render -> popup::render_button hovered, buffer-tested in open_with/tests.rs",
        ),
        HitTarget::OpenWithDefault => Required(
            "open_with::view::render_card_actions -> input_bg(hovered), buffer-tested in open_with/tests.rs",
        ),
        HitTarget::OpenInPopup => NotApplicable(
            "the popup's frame: a shield so nothing behind it hovers, no action of its own",
        ),
        HitTarget::LogRefPill { .. } => NotApplicable(
            "a double-click target inside a Log row, which lights as a whole; the pill is \
             not a button and a separate fill would fight the lane color",
        ),
        HitTarget::RefsSectionHeader(_) => {
            Required("event_loop.rs hover_style on the section header's title span")
        }
        HitTarget::StopSoloing | HitTarget::ShowAllHiddenRefs => {
            Required("event_loop.rs hover_style on the title button span (item 15)")
        }
        HitTarget::TooltipRefPill { .. } => {
            NotApplicable("a line of a tooltip, which never outlives the pointer's dwell")
        }
        HitTarget::CommitReference(_) => Required(
            "event_loop.rs: the token is painted underlined + accent; the pointing hand and \
             the hint tooltip are the hover",
        ),
        HitTarget::CommitAuthorName => NotApplicable(
            "plain text that only carries the email tooltip, no action to invite a click",
        ),
        HitTarget::CommitDetailPart(_) => NotApplicable(
            "text bands of the commit message and author: right-click and double-click \
             targets with no single-click action, so no hover fill to invite one",
        ),
        HitTarget::CommitSortButton => Required("event_loop.rs hover_style on the sort label"),
        HitTarget::CommitIdLine => Required("commit_detail.rs light() on the `commit <id>` line"),
        HitTarget::CompareSwap => Required("right_panel/commit_detail.rs plain_button_style on the Swap label"),
        HitTarget::CompareCard(_) => Required("right_panel/commit_detail.rs: the hovered card gets the hover background"),
        HitTarget::LogColumnBoundary(_)
        | HitTarget::SidebarSectionBoundary(_)
        | HitTarget::CommitMessageBoundary => Required("hovered_resize_strip highlight"),
        HitTarget::LogColumnHeader(_) => NotApplicable(
            "deliberately no hover fill: the header cell has no click action (drag-to-reorder \
             only); the resize handle's own highlight is LogColumnBoundary's",
        ),
        HitTarget::ColumnMenuItem(_) | HitTarget::ColumnMenuAction(_) => {
            Required("event_loop.rs hover_style (item 15)")
        }
        HitTarget::ColumnMenuClose | HitTarget::ConfirmClose => {
            Required("close_cross::paint's plain_button_style")
        }
        HitTarget::RefContextMenuItem(_)
        | HitTarget::LogContextMenuItem(_)
        | HitTarget::WipContextMenuItem(_)
        | HitTarget::PaneContextMenuItem(_)
        | HitTarget::DiffHunkContextMenuItem(_)
        | HitTarget::HistoryContextMenuItem(_) => {
            Required("context_menu::Menu::render(hovered) (item 15)")
        }
        HitTarget::History(_) => Required(
            "history_view::render_list's hovered_row fill; the splitter is hovered_resize_strip",
        ),
        HitTarget::CommitColumn(_) => Required(
            "commit_column::render: the row's hover fill, the cross's plain_button_style, the splitter's accent",
        ),
        HitTarget::DiffBarButton(_) => {
            Required("diff_view::diff_bar's own ComposerHover-like param")
        }
        HitTarget::DiffBodyButton(_) => {
            Required("diff_view::visible_window fills the hovered button, buffer-tested in diff_view.rs")
        }
        HitTarget::DiffRow(_) => Required("diff_view's cursor-row highlight"),
        HitTarget::SearchBarButton(_) => {
            Required("event_loop/draw/log.rs paints plain_button_style(true) on the hovered find-bar button")
        }
        HitTarget::FileListModeButton(_) => Required("theme::segmented_style(active, hovered)"),
        HitTarget::StatusBulkButton(_) => Required("event_loop.rs hover_style (item 15)"),
        HitTarget::StatusSectionHeader(_) => {
            Required("event_loop.rs hover_style on the section header's title span")
        }
        HitTarget::StatusRowButton { .. } | HitTarget::StatusFolderButton { .. } => {
            Required("overlay_menus.rs fills the hovered Stage / Unstage button, over the lit row")
        }
        HitTarget::StatusFolderRow(_) => {
            Required("status_list::status_row_item's hover_style on the folder name span")
        }
        HitTarget::CollapseAllButton { .. } => Required("event_loop.rs hover_style (item 15)"),
        HitTarget::FileContextMenuItem(_) => Required("event_loop.rs per-row th().ui.hover (item 15)"),
        HitTarget::FolderContextMenuItem(_) => {
            Required("event_loop.rs hovered_menu_row, same as FileContextMenuItem (item 18)")
        }
        HitTarget::StatusDiscardAllButton => {
            Required("status_list.rs render_status_header_buttons' hover_style")
        }
        HitTarget::StatusExplainButton => {
            Required("status_list.rs render_status_header_buttons' hover_style")
        }
        HitTarget::ViewAllFilesCheckbox => {
            Required("event_loop.rs hover_style on the checkbox label span")
        }
        HitTarget::CommitFilterField => {
            Required("event_loop.rs hover_style on the unfocused filter field's text span")
        }
        HitTarget::AmendCheckbox
        | HitTarget::SkipHooksCheckbox
        | HitTarget::AiGenerate
        | HitTarget::AiCompose
        | HitTarget::ComposerIssueRef
        | HitTarget::CommitButton => {
            Required("composer::ComposerHover, buffer-tested in composer.rs")
        }
        HitTarget::CommitSummaryField | HitTarget::CommitDescriptionField => {
            NotApplicable("a text field: focus is its state and the I-beam pointer its hover cue")
        }
        HitTarget::WipMessageField => {
            NotApplicable("a text field: focus is its state and the I-beam pointer its hover cue")
        }
        HitTarget::ConfirmButton(_) => {
            Required("event_loop.rs lighten() on the filled band (item 15)")
        }
        HitTarget::CollapsePane(_) | HitTarget::RestorePane(_) => {
            Required("render_pane_collapse_button / render_collapsed_pane_strip, this file")
        }
        HitTarget::CloseApp => Required("render_close_app_button -> plain_button_style"),
        HitTarget::Image(_) => Required(
            "image_view::header::layout(hovered) lights the button with button_hover; \
             header's hover test",
        ),
        HitTarget::ScrollbarThumb(_) => {
            Required("scrollbar::render lights the thumb, buffer-tested in scrollbar.rs")
        }
        HitTarget::ScrollbarTrack(_) => Required(
            "scrollbar::render_buf lights the gutter and the end arrows from the pointer, \
             buffer-tested in scrollbar.rs",
        ),
        HitTarget::BarOverflow(_) => Required(
            "chip_overflow_style / hover_style on the overflow button, buffer-tested in \
             toolbar_render's, diff_bar's and status_toolbar_row's tests (item 22)",
        ),
        HitTarget::BarOverflowMenuItem(_) => Required("context_menu::Menu::render(hovered)"),
        HitTarget::Editor(_) => Required(
            "editor::Editor::render(hovered) - bar and prompt buttons, the close, the thumbs",
        ),
        HitTarget::Blame(_) => Required(
            "blame_view::render(hovered) - the hovered hunk's annotation is tinted with ui.hover, the \
             border lights up, a card or menu row is lit; blame_view's render tests",
        ),
        HitTarget::Merge(_) => Required(
            "merge_view::MergeView::render(hovered) - toolbar and tab buttons through \
             dialog_kit::button, the header arrows, chunk checkbox, badge and close through \
             plain_button_style(true), a line or Output row with th().ui.hover, the splitter rule \
             and scrollbar thumbs lit; merge_view render_tests' hover tests",
        ),
        HitTarget::RebaseForm(_) => Required(
            "rebase_ui::RebaseForm::render(hovered) - every pill, row handle, checkbox and              button lights with plain_button_style(true); rebase_ui's hover test",
        ),
        HitTarget::ComposeForm(_) => Required(
            "compose_ui::ComposeForm::render(hovered) - rows, the per-row controls and the three buttons light with plain_button_style(true); compose_ui's render test",
        ),
        HitTarget::SplitForm(_) => Required(
            "split_ui::SplitForm::render(hovered) - the close cross, parts and their controls, every row's [N v] field, the sign checkbox and the three buttons light; split_ui's render tests",
        ),
        HitTarget::Rewrite(_) => Required(
            "rewrite_view::RewriteView::render(hovered) - the close cross, every button, tab, combo, checkbox, row and rule control light; rewrite_view's render tests",
        ),
        HitTarget::RebaseBanner(_) => Required(
            "rebase_banner::render(hovered) - Continue/Skip/Abort light with              plain_button_style(true); rebase_banner's hover test",
        ),
        HitTarget::StuckBanner(_) => Required(
            "stuck_banner::paint - every button lights with plain_button_style(true); \
             stuck_banner's hover test",
        ),
        HitTarget::MergeEditorButton => Required(
            "event_loop/draw/central.rs draws it through dialog_kit::button with the hovered state",
        ),
        HitTarget::OperationBanner(_) => Required(
            "operation_banner::render(hovered) - Resolve/Commit/Abort light with \
             plain_button_style(true); operation_banner's tests",
        ),
        HitTarget::AutoStashBanner(_) => Required(
            "autostash_banner::render(hovered) - Pop now/Show stash/Drop light with              plain_button_style(true); autostash_banner's render test",
        ),
        HitTarget::ConflictBadge => Required(
            "conflict_badge::render(hovered) - the button lights with plain_button_style(true); conflict_badge's render test",
        ),
        HitTarget::Hosting(_) => Required(
            "hosting_ui: plain_button_style(true) on rows, chips and the toolbar chip,              buffer-tested in hosting_ui.rs and chrome.rs",
        ),
        HitTarget::RepoTab(_) => Required("repo_switcher::tab_style hovered_tab_index"),
        HitTarget::ListRow | HitTarget::VirtualHand => NotApplicable(
            "pointer-only target: Log, status and folder rows light through their hovered_* state",
        ),
    }
}

/// The guard itself: every `HitTarget` variant is classified by
/// `hover_style_for` (an exhaustive match, so a new variant fails to
/// compile until someone decides how it lights), every classification
/// carries its mechanism or its reason, and `sample_hit_targets` is
/// proven to list every variant, so none escapes the loop.
#[test]
fn every_clickable_hit_target_is_classified_for_hover() {
    let samples = sample_hit_targets();
    for t in &samples {
        match hover_style_for(t) {
            HoverKind::Required(mechanism) => {
                assert!(!mechanism.is_empty(), "{t:?} must name its hover path");
            }
            HoverKind::NotApplicable(reason) => {
                assert!(!reason.is_empty(), "{t:?} must say why it has no hover");
            }
        }
    }
    let source = include_str!("../hit.rs");
    let start = source.find("pub(crate) enum HitTarget {").unwrap();
    let body = &source[start..];
    let body = &body[..body.find("\n}\n").unwrap()];
    let declared: std::collections::BTreeSet<&str> = body
        .lines()
        .filter(|l| l.starts_with("    ") && !l.starts_with("     "))
        .filter_map(|l| {
            let name = l.trim();
            name.chars().next().filter(|c| c.is_ascii_uppercase())?;
            Some(
                name.split(|c: char| !c.is_ascii_alphanumeric())
                    .next()
                    .unwrap(),
            )
        })
        .collect();
    let sampled: std::collections::BTreeSet<String> = samples
        .iter()
        .map(|t| {
            format!("{t:?}")
                .split(|c: char| !c.is_ascii_alphanumeric())
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    for name in &declared {
        assert!(
            sampled.contains(*name),
            "HitTarget::{name} is missing from sample_hit_targets"
        );
    }
    assert!(declared.len() > 40, "the enum scan found {declared:?}");
}

/// Every toolbar button, both split-button chevrons and the chrome
/// buttons carry a tooltip (Undo / Redo have their own live text).
#[test]
fn every_toolbar_button_and_dropdown_has_a_tooltip() {
    for (_, _, button) in TOOLBAR_BUTTONS {
        let target = HitTarget::Toolbar(*button);
        if matches!(button, ToolbarButton::Undo | ToolbarButton::Redo) {
            assert!(crate::tooltip::hint_for(&target).is_none());
            continue;
        }
        assert!(
            crate::tooltip::hint_text(&target).is_some_and(|t| t.len() > 8),
            "{button:?} needs a tooltip"
        );
    }
    for button in [
        ToolbarButton::Pull,
        ToolbarButton::Push,
        ToolbarButton::Undo,
    ] {
        assert!(
            crate::tooltip::hint_for(&HitTarget::ToolbarDropdown(button)).is_some(),
            "{button:?} dropdown needs a tooltip"
        );
    }
    for target in [
        HitTarget::Gear,
        HitTarget::About,
        HitTarget::ProfileField,
        HitTarget::WorkspaceChip,
        HitTarget::CloseApp,
        HitTarget::OpenRepositoryButton,
        HitTarget::NewTabButton,
        HitTarget::TabOverflow,
        HitTarget::RadarTab,
        HitTarget::RadarClose,
        HitTarget::NotesTab,
        HitTarget::NotesClose,
        HitTarget::GlobalSearchButton,
        HitTarget::TabDir(crate::tab_dir::TabDirButton::QuitCd),
        HitTarget::TabDir(crate::tab_dir::TabDirButton::Terminal),
    ] {
        assert!(
            crate::tooltip::hint_for(&target).is_some(),
            "{target:?} needs a tooltip"
        );
    }
    // The Pull / Push dropdown items explain themselves too.
    for entry in crate::remotes_ui::pull_mode_entries()
        .iter()
        .chain(crate::remotes_ui::push_to_entries(&["origin".into()], None).iter())
    {
        let crate::context_menu::Entry::Action { action, label, .. } = entry else {
            continue;
        };
        assert!(action.tooltip().is_some(), "{label} needs a tooltip");
    }
}

/// One representative value per `HitTarget` variant, for
/// `hover_coverage_is_recorded_for_every_hit_target_and_covers_most_of_them`
/// to iterate. Kept separate from that test so the list is easy to
/// scan on its own.
fn sample_hit_targets() -> Vec<HitTarget> {
    vec![
        HitTarget::Toolbar(ToolbarButton::Pop),
        HitTarget::ToolbarDropdown(ToolbarButton::Push),
        HitTarget::RemoteMenuItem(0),
        HitTarget::RemoteForm(RemoteFormHit::Part(RemoteFormPart::Cancel)),
        HitTarget::RewordForm(RewordPart::Cancel),
        HitTarget::AddRemoteButton,
        HitTarget::LfsInstallLink,
        HitTarget::Gear,
        HitTarget::About,
        HitTarget::ProfileField,
        HitTarget::ProfileItem(0),
        HitTarget::ProfilePopup,
        HitTarget::WorkspaceChip,
        HitTarget::WorkspaceItem(0),
        HitTarget::WorkspacePopup,
        HitTarget::CloseDiff,
        HitTarget::OpenRepositoryButton,
        HitTarget::NewTabButton,
        HitTarget::TabOverflow,
        HitTarget::RadarTab,
        HitTarget::RadarClose,
        HitTarget::WorkspacesTab,
        HitTarget::WorkspacesClose,
        HitTarget::NotesTab,
        HitTarget::NotesClose,
        HitTarget::GlobalSearchButton,
        HitTarget::TabDir(crate::tab_dir::TabDirButton::Terminal),
        HitTarget::OpenInItem(0),
        HitTarget::OpenInPopup,
        HitTarget::OpenWith,
        HitTarget::OpenWithDefault,
        HitTarget::LogRefPill {
            name: "main".into(),
            kind: RefRowKind::Local,
        },
        HitTarget::RefsSectionHeader(SidebarSection::Local),
        HitTarget::StopSoloing,
        HitTarget::ShowAllHiddenRefs,
        HitTarget::TooltipRefPill {
            name: "main".into(),
            kind: RefRowKind::Local,
        },
        HitTarget::LogColumnBoundary(LogColumn::Author),
        HitTarget::SidebarSectionBoundary(SidebarSection::Local),
        HitTarget::CommitMessageBoundary,
        HitTarget::CommitSortButton,
        HitTarget::CommitIdLine,
        HitTarget::CompareSwap,
        HitTarget::CompareCard(0),
        HitTarget::CommitReference(0),
        HitTarget::CommitAuthorName,
        HitTarget::CommitDetailPart(AmendScope::Summary),
        HitTarget::LogColumnHeader(LogColumn::Author),
        HitTarget::ColumnMenuItem(LogColumn::Author),
        HitTarget::ColumnMenuAction(ChooserRow::FitAll),
        HitTarget::ColumnMenuClose,
        HitTarget::RefContextMenuItem(0),
        HitTarget::LogContextMenuItem(0),
        HitTarget::WipContextMenuItem(0),
        HitTarget::PaneContextMenuItem(0),
        HitTarget::DiffHunkContextMenuItem(0),
        HitTarget::HistoryContextMenuItem(0),
        HitTarget::History(crate::history_view::HistoryHit::Row(0)),
        HitTarget::CommitColumn(crate::commit_column::ColumnHit::Close),
        HitTarget::CommitColumn(crate::commit_column::ColumnHit::Reopen),
        HitTarget::DiffBarButton(DiffBarButton::PrevHunk),
        HitTarget::DiffBodyButton(diff_view::DiffBodyButton::Hunk(HunkId(0))),
        HitTarget::DiffRow(0),
        HitTarget::SearchBarButton(SearchBarButton::Close),
        HitTarget::FileListModeButton(FileListMode::Tree),
        HitTarget::StatusBulkButton(StatusSection::Unstaged),
        HitTarget::StatusSectionHeader(StatusSection::Unstaged),
        HitTarget::StatusRowButton {
            section: StatusSection::Unstaged,
            path: PathBuf::from("a"),
        },
        HitTarget::StatusFolderRow(PathBuf::from("a")),
        HitTarget::StatusFolderButton {
            section: StatusSection::Unstaged,
            path: PathBuf::from("a"),
        },
        HitTarget::CollapseAllButton { collapse: true },
        HitTarget::FileContextMenuItem(0),
        HitTarget::FolderContextMenuItem(0),
        HitTarget::StatusDiscardAllButton,
        HitTarget::AmendCheckbox,
        HitTarget::SkipHooksCheckbox,
        HitTarget::AiGenerate,
        HitTarget::AiCompose,
        HitTarget::ComposerIssueRef,
        HitTarget::StatusExplainButton,
        HitTarget::RebaseForm(crate::rebase_ui::RebasePart::Cancel),
        HitTarget::ComposeForm(crate::compose_ui::ComposePart::Toggle(0)),
        HitTarget::SplitForm(crate::split_ui::SplitHit::Apply),
        HitTarget::Rewrite(crate::rewrite_view::RewriteHit::Apply),
        HitTarget::RebaseBanner(crate::rebase_banner::RebaseBannerButton::Abort),
        HitTarget::AutoStashBanner(crate::autostash_banner::AutoStashBannerButton::Drop),
        HitTarget::ConflictBadge,
        HitTarget::OperationBanner(crate::operation_banner::OperationBannerButton::Abort),
        HitTarget::StuckBanner(crate::stuck_banner::StuckBannerButton::AmAbort),
        HitTarget::MergeEditorButton,
        HitTarget::CommitSummaryField,
        HitTarget::CommitDescriptionField,
        HitTarget::CommitButton,
        HitTarget::WipMessageField,
        HitTarget::CommitFilterField,
        HitTarget::ViewAllFilesCheckbox,
        HitTarget::ConfirmButton(true),
        HitTarget::ConfirmClose,
        HitTarget::CollapsePane(PaneSide::Refs),
        HitTarget::RestorePane(PaneSide::Refs),
        HitTarget::CloseApp,
        HitTarget::ScrollbarThumb(ScrollBar::DiffVertical),
        HitTarget::Image(crate::image_view::pane::ImageHit::Fit),
        HitTarget::ScrollbarTrack(ScrollBar::DiffVertical),
        HitTarget::BarOverflow(OverflowBar::Toolbar),
        HitTarget::BarOverflowMenuItem(0),
        HitTarget::Editor(crate::editor::EditorHit::Save),
        HitTarget::Blame(crate::blame_view::BlameHit::Handle),
        HitTarget::Merge(crate::merge_view::MergeHit::Save),
        HitTarget::Hosting(HostingHit::LinkedPrChip),
        HitTarget::RepoTab(0),
        HitTarget::ListRow,
        HitTarget::VirtualHand,
    ]
}
