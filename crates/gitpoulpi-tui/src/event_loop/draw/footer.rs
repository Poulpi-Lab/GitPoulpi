//! The footer line of the draw closure: status breadcrumb, prompts and key hints.

use super::super::*;
use gitpoulpi_i18n::msg::footer as f;

pub(in crate::event_loop) struct FooterDrawCtx<'a> {
    pub(in crate::event_loop) ai_ui: &'a AiUi,
    pub(in crate::event_loop) field_editor: &'a crate::text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) commit_total: graph::TotalShown,
    pub(in crate::event_loop) footer_area: Rect,
    pub(in crate::event_loop) in_flight_text: &'a Option<String>,
    pub(in crate::event_loop) inline_ref_drawn: bool,
    pub(in crate::event_loop) input_mode: &'a InputMode,
    pub(in crate::event_loop) last_message: &'a Option<String>,
    pub(in crate::event_loop) log_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(in crate::event_loop) new_file_error: &'a Option<String>,
    pub(in crate::event_loop) pending_count: Option<u32>,
    pub(in crate::event_loop) ref_drag: &'a Option<RefDrag>,
    pub(in crate::event_loop) refs_result:
        &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(in crate::event_loop) repo_open_started_at: Instant,
    pub(in crate::event_loop) session: &'a str,
    /// The working tree is a sparse checkout: a small badge in the breadcrumb.
    pub(in crate::event_loop) sparse: bool,
    pub(in crate::event_loop) status_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::WorkingTree>,
}

pub(in crate::event_loop) fn draw_footer(
    frame: &mut Frame,
    ctx: &mut FooterDrawCtx<'_>,
) -> Option<String> {
    // Painted again below when this frame shows the idle line.
    crate::activity_log::clear_status_button();
    crate::jobs_ui::clear_indicator();
    crate::activity_log::status_error::begin_frame();
    let ai_ui = ctx.ai_ui;
    let field_editor = ctx.field_editor;
    let footer_area = ctx.footer_area;
    let in_flight_text = ctx.in_flight_text;
    let inline_ref_drawn = ctx.inline_ref_drawn;
    let input_mode = ctx.input_mode;
    let last_message = ctx.last_message;
    let log_result = ctx.log_result;
    let new_file_error = ctx.new_file_error;
    let pending_count = ctx.pending_count;
    let ref_drag = ctx.ref_drag;
    let refs_result = ctx.refs_result;
    let repo_open_started_at = ctx.repo_open_started_at;
    let session = ctx.session;
    let sparse = ctx.sparse;
    let status_result = ctx.status_result;
    let empty_refs_for_head = Vec::new();
    let checked_out = checked_out_branch_name(refs_result.as_ref().unwrap_or(&empty_refs_for_head));
    // The footer's own one-line prompts are text fields too:
    // `footer_text` is then just the label, and
    // the field and its key hint are drawn after it below.
    let mut footer_prompt: Option<(MainField, &str, &str)> = None;
    let mut ref_name_hint = String::new();
    let footer_text = match &input_mode {
        InputMode::FilteringRefs(_) => f::FILTER_REFS_HINT.get().to_string(),
        // Named inline, or in the name box (`ref_name_box.rs`): never here.
        InputMode::ComposingRefName { op, .. } if is_inline_name(op) => String::new(),
        InputMode::ComposingRefName { op, value } => {
            let label = match op {
                RefNameOp::NewBranch => f::NEW_BRANCH_NAME.get().to_string(),
                RefNameOp::NewTag => f::NEW_TAG_NAME.get().to_string(),
                RefNameOp::NewBranchAt(at) => f::new_branch_at(&at.to_hex()[..SHORT_SHA_LEN]),
                RefNameOp::NewTagAt(at) => f::new_tag_at(&at.to_hex()[..SHORT_SHA_LEN]),
                RefNameOp::NewAnnotatedTagAt(at) => {
                    f::new_annotated_tag_at(&at.to_hex()[..SHORT_SHA_LEN])
                }
                RefNameOp::AnnotatedTagMessage { name, .. } => f::tag_message(name),
                RefNameOp::RenameBranch { name: old, .. } => f::rename_to(old),
                RefNameOp::AnnotateTag(tag) => f::tag_message(tag),
                RefNameOp::SetUpstream(branch) => f::set_upstream(branch),
                RefNameOp::LfsTrack => f::LFS_TRACK.get().to_string(),
                RefNameOp::LfsUntrack => f::LFS_UNTRACK.get().to_string(),
                RefNameOp::StashMessage => f::STASH_MESSAGE.get().to_string(),
                RefNameOp::CreateFile(folder) => {
                    let place = if folder.as_os_str().is_empty() {
                        String::new()
                    } else {
                        f::new_file_in(folder.to_string_lossy().replace('\\', "/"))
                    };
                    match &new_file_error {
                        Some(why) => f::new_file_error(&place, why),
                        None => f::new_file_name(&place),
                    }
                }
            };
            ref_name_hint.push_str(
                &if matches!(op, RefNameOp::StashMessage) && ai_ui.enabled() {
                    f::stash_hint_ai(
                        keymap::current().shown(Bind::PromptAiStashMessage),
                        "\u{2726}",
                    )
                } else if matches!(op, RefNameOp::StashMessage) {
                    f::STASH_HINT.get().to_string()
                } else {
                    f::CONFIRM_HINT.get().to_string()
                },
            );
            if inline_ref_drawn {
                // The box is on the commit's row; nothing here.
                String::new()
            } else {
                footer_prompt = Some((MainField::RefName, value, ref_name_hint.as_str()));
                format!("{label}: ")
            }
        }
        InputMode::Leader => {
            chrome::prefix_hint(&keymap::current(), keymap::Ctx::GitPrefix, ai_ui.enabled())
        }
        InputMode::Navigation => {
            chrome::prefix_hint(&keymap::current(), keymap::Ctx::NavPrefix, ai_ui.enabled())
        }
        // Rendered as its own centered popup (see the
        // `InputMode::Confirm` block after the command palette),
        // not a status-bar line -- a destructive action deserves a
        // real modal, not text a user could miss.
        InputMode::Confirm { .. } => String::new(),
        // The dialog names its own keys on its buttons.
        InputMode::Guide(_) => String::new(),
        // The popup names its own keys.
        InputMode::EditingCommitMessage(_) => String::new(),
        // The dialog names its own keys on its buttons.
        InputMode::Prompt(_) => String::new(),
        InputMode::InteractiveRebase(_) => String::new(),
        InputMode::ComposeCommits(_) => String::new(),
        InputMode::SplitCommit(_) => String::new(),
        InputMode::Rewrite(_) => String::new(),
        // Rendered as its own centered popup below instead of a
        // bottom-bar line -- see the `InputMode::CommandPalette`
        // block after the help overlay.
        InputMode::CommandPalette { .. } => String::new(),
        // Rendered as its own centered popup below instead of a
        // bottom-bar line -- see the `InputMode::TabPicker` block
        // after the command palette's own.
        InputMode::TabPicker { .. } => String::new(),
        // Its own popup too (`theme_picker::render`).
        InputMode::ThemePicker(_)
        | InputMode::LanguagePicker(_)
        | InputMode::WorkspaceSearch(_) => String::new(),
        // The find bar carries its own query, counter and buttons
        // in the overlay -- repeating it in the footer would be
        // noise. The footer just names the keys.
        InputMode::SearchingLog(_) => f::search_hint("\u{2193}", "\u{2191}"),
        InputMode::Normal => String::new(),
    };
    let home_mode_hint = if crate::dialog_kit::frame::armed() {
        Some(form_badges_hint(&keymap::current()))
    } else {
        matches!(input_mode, InputMode::Navigation | InputMode::Leader).then(|| footer_text.clone())
    };

    if matches!(input_mode, InputMode::Normal) && !crate::dialog_kit::frame::armed() {
        // gh-dash-style breadcrumb: dim icon+dot-separated status
        // segments, with any hint/error (bright) tacked on last.
        // The Activity button takes the right end of the line when there is room.
        let button_w = crate::activity_log::status_button_width();
        // The jobs indicator sits left of the Activity button: it takes its
        // room first, and is dropped (never the breadcrumb) when too narrow.
        let jobs_w = crate::jobs_ui::width_for();
        let show_jobs = jobs_w > 0 && footer_area.width >= button_w + jobs_w + 40;
        let reserved = button_w + if show_jobs { jobs_w } else { 0 };
        let text_area = if footer_area.width >= reserved + 40 {
            Rect::new(
                footer_area.x,
                footer_area.y,
                footer_area.width - reserved,
                footer_area.height,
            )
        } else {
            footer_area
        };
        let dim = Style::default().fg(th().ui.muted);
        let commit_count = log_result.as_ref().map(|g| g.nodes.len()).unwrap_or(0);
        let dirty = matches!(&status_result, Ok(t) if !t.files.is_empty());
        let mut segs: Vec<Span> = vec![
            Span::styled(
                // A branch glyph, not `LOCAL_ICON` (💻) -- this is
                // "which branch is checked out", not "this is a
                // local-kind ref as opposed to remote/tag", so the
                // computer icon (correct for a LOCAL-section ref
                // row/pill) doesn't apply here.
                format!(
                    " \u{2387} {} ",
                    checked_out.unwrap_or(gitpoulpi_i18n::msg::sidebar::DETACHED.get())
                ),
                dim,
            ),
            Span::styled("\u{2022}", dim),
            Span::styled(
                format!(" {} ", commit_count_text(commit_count, ctx.commit_total)),
                dim,
            ),
            Span::styled("\u{2022}", dim),
            Span::styled(
                if dirty {
                    f::DIRTY.get()
                } else {
                    f::CLEAN.get()
                },
                if dirty {
                    Style::default().fg(th().status.warning)
                } else {
                    dim
                },
            ),
            Span::styled("\u{2022}", dim),
            Span::styled(format!(" {} ", f::session(session)), dim),
        ];
        if sparse {
            segs.push(Span::styled("\u{2022}", dim));
            segs.push(Span::styled(
                format!(" {} ", gitpoulpi_i18n::msg::sparse::BADGE.get()),
                Style::default().fg(th().ui.accent),
            ));
        }
        if let Some(count) = pending_count {
            // The vim-style count prefix in progress (`4…`) --
            // shown ahead of everything else in this slot because
            // it is live keyboard state the user is mid-typing,
            // same priority as a drag in progress just below.
            segs.push(Span::styled("\u{2022}", dim));
            segs.push(Span::styled(
                format!(" {count}\u{2026} "),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if let Some(drag) = ref_drag.as_ref().filter(|d| d.started) {
            // A drag in flight owns this slot ahead of the last
            // message: it is live state, not a past event, and it
            // is the affordance that survives a terminal with no
            // `OSC 22` support (see `cursor.rs`). The color is
            // the third statement of the same valid/invalid fact
            // the lit drop target and the wording already make.
            let drag_fg = match &drag.over {
                Some(c) if c.valid => th().ui.accent,
                Some(_) => th().status.warning,
                None => th().ui.text,
            };
            segs.push(Span::styled("\u{2022}", dim));
            segs.push(Span::styled(
                format!(" {} ", drag.banner()),
                Style::default().fg(drag_fg).add_modifier(Modifier::BOLD),
            ));
        } else if let Some(busy) = &in_flight_text {
            segs.push(Span::styled("\u{2022}", dim));
            segs.push(Span::styled(
                format!(
                    " {} {busy} ",
                    activity::spinner_frame(repo_open_started_at.elapsed())
                ),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if let Some(msg) = &last_message {
            segs.push(Span::styled("\u{2022}", dim));
            // A push or pull failure: the error style, and a click on it
            // opens the Activity Log on its entry.
            let quiet_error = crate::activity_log::status_error::is_showing(msg);
            let shown = format!(" {} ", terminology::tr(msg));
            if quiet_error {
                // On a narrow window the error comes first: the branch,
                // the session and the rest would push it out of sight.
                let before: usize = segs.iter().map(|s| display_width(&s.content)).sum();
                if before + display_width(&shown) > usize::from(text_area.width) {
                    segs.clear();
                    segs.push(Span::styled("\u{2022}", dim));
                }
                let used: usize = segs.iter().map(|s| display_width(&s.content)).sum();
                let x = text_area.x + u16::try_from(used).unwrap_or(u16::MAX);
                let room = text_area.right().saturating_sub(x);
                let w = u16::try_from(display_width(&shown)).unwrap_or(u16::MAX);
                crate::activity_log::status_error::note_rect(Rect::new(
                    x,
                    text_area.y,
                    w.min(room),
                    1,
                ));
            }
            segs.push(Span::styled(
                shown,
                Style::default()
                    .fg(if quiet_error {
                        th().status.danger
                    } else {
                        th().ui.text
                    })
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            segs.push(Span::styled("\u{2022}", dim));
            let km = keymap::current();
            let menu_key = menu_hint_keys(&km.shown_labels(Bind::GlobalContextMenu));
            let used: usize = segs.iter().map(|s| display_width(&s.content)).sum();
            let room = (text_area.width as usize).saturating_sub(used);
            segs.push(Span::styled(
                footer_hints(
                    &km.shown(Bind::GlobalGitPrefix),
                    &km.shown(Bind::GlobalNavPrefix),
                    &km.shown(Bind::GlobalPalette),
                    &menu_key,
                    room,
                ),
                dim,
            ));
        }
        frame.render_clipped(Paragraph::new(Line::from(segs)), text_area);
        crate::activity_log::paint_status_button(frame.buffer_mut(), footer_area);
        if show_jobs {
            crate::jobs_ui::paint_indicator(
                frame.buffer_mut(),
                footer_area,
                footer_area.right() - button_w,
                footer_area.x,
            );
        }
    } else if let Some((id, value, hint)) = footer_prompt {
        let label_w = display_width(&footer_text) as u16;
        let room = footer_area
            .width
            .saturating_sub(label_w + display_width(hint) as u16);
        let field_w = room.min(display_width(value) as u16 + 1);
        field_editor.drawn(
            id,
            Rect::new(footer_area.x + label_w, footer_area.y, field_w, 1),
            false,
        );
        let mut spans = vec![Span::raw(footer_text)];
        spans.extend(
            text_field::edit_line(
                value,
                field_editor.caret(id),
                field_w as usize,
                true,
                text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
            )
            .spans,
        );
        spans.push(Span::raw(hint));
        frame.render_clipped(Paragraph::new(Line::from(spans)), footer_area);
    } else if matches!(input_mode, InputMode::Leader | InputMode::Navigation)
        || form_badges(input_mode)
    {
        let rows = prefix_hint_rows(input_mode, ai_ui.enabled(), footer_area.width);
        frame.render_clipped(
            Paragraph::new(rows.into_iter().map(Line::from).collect::<Vec<_>>()),
            footer_area,
        );
    } else {
        frame.render_clipped(Paragraph::new(footer_text), footer_area);
    }
    home_mode_hint
}

/// The most rows the pending-prefix hint may take before the last one is clipped.
const MAX_HINT_ROWS: usize = 4;

/// `text` word-wrapped to `width` columns, so no key hint is cut off at the
/// right edge: it carries on the next row instead.
pub(in crate::event_loop) fn wrap_hint(text: &str, width: usize) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match rows.last_mut() {
            Some(row) if display_width(row) + 1 + display_width(word) <= width => {
                row.push(' ');
                row.push_str(word);
            }
            _ => rows.push(word.to_string()),
        }
    }
    rows
}

/// The commit message form, the rewrite page or any framed dialog or popup
/// waits for the key after its `Ctrl+N` (a badge, or `m` to arrange the dialog).
fn form_badges(input_mode: &InputMode) -> bool {
    if crate::dialog_kit::frame::armed() {
        return true;
    }
    match input_mode {
        InputMode::EditingCommitMessage(form) => form.badges && form.badge_ready(),
        InputMode::Prompt(dialog) => dialog.badges && dialog.badge_ready(),
        InputMode::Rewrite(view) => view.badges && view.badge_ready(),
        _ => false,
    }
}

/// The hint while [`form_badges`]: only what those two surfaces answer to.
fn form_badges_hint(km: &keymap::Keymap) -> String {
    use gitpoulpi_i18n::msg::footer_hint as h;
    let mut parts = vec![format!("<badge>={}", h::FOCUS_SUBPANE.get())];
    let key = km.shown_key(keymap::Action::NavArrangeDialog);
    if !key.is_empty() {
        parts.push(format!("{key}={}", h::ARRANGE.get()));
    }
    parts.push(h::ESC_CANCEL.get().to_string());
    format!(
        "{}{}",
        h::lead(km.shown(keymap::Action::GlobalNavPrefix)),
        parts.join("  ")
    )
}

/// The rows of the Ctrl+G / Ctrl+N hint wrapped to `width`; empty in every
/// other input mode, where the footer stays one row.
pub(in crate::event_loop) fn prefix_hint_rows(
    input_mode: &InputMode,
    ai_on: bool,
    width: u16,
) -> Vec<String> {
    if form_badges(input_mode) {
        return wrap_hint(&form_badges_hint(&keymap::current()), width.max(1) as usize);
    }
    let ctx = match input_mode {
        InputMode::Leader => keymap::Ctx::GitPrefix,
        InputMode::Navigation => keymap::Ctx::NavPrefix,
        _ => return Vec::new(),
    };
    let km = keymap::current();
    let width = width.max(1) as usize;
    if ctx == keymap::Ctx::NavPrefix {
        return nav_hint_rows(&km, width);
    }
    let mut rows = wrap_hint(&chrome::prefix_hint(&km, ctx, ai_on), width);
    rows.truncate(MAX_HINT_ROWS);
    rows
}

/// The Ctrl+N hint in at most [`MAX_HINT_ROWS`] rows of `width`: every bound
/// action while they fit, else the first ones (the most useful come first)
/// and a closing `?=all keys` where the help key lists the rest. Whole items
/// only, so nothing is cut mid-word at the right edge.
pub(in crate::event_loop) fn nav_hint_rows(km: &keymap::Keymap, width: usize) -> Vec<String> {
    use gitpoulpi_i18n::msg::footer_hint as h;
    let items = chrome::nav_hint_items(km);
    let lead = h::lead(km.shown(keymap::Action::GlobalNavPrefix));
    let build = |n: usize| {
        let truncated = n < items.len();
        let mut parts: Vec<String> = items[..n].to_vec();
        let help = chrome::nav_help_item(km, truncated);
        if !help.is_empty() {
            parts.push(help);
        }
        parts.push(h::ESC_CANCEL.get().to_string());
        wrap_hint(&format!("{lead}{}", parts.join("  ")), width)
    };
    for n in (0..=items.len()).rev() {
        let rows = build(n);
        if rows.len() <= MAX_HINT_ROWS {
            return rows;
        }
    }
    let mut rows = build(0);
    rows.truncate(MAX_HINT_ROWS);
    rows
}

/// The idle key hints, most useful first; when the line is too narrow the
/// least useful ones go, whole, instead of being cut mid-word: the menu
/// key first, then `q: quit`, then `nav`.
/// The keys the footer names for the context menu: the modified chord first
/// (`Shift+F10`), then the single-character ones (`.`), joined by `/`. A
/// named key such as `Menu` rarely reaches the app, so it is not advertised.
pub(in crate::event_loop) fn menu_hint_keys(labels: &[String]) -> String {
    let chords = labels.iter().filter(|l| l.contains('+') && l.len() > 1);
    let bare = labels.iter().filter(|l| l.chars().count() == 1);
    let all: Vec<&str> = chords.chain(bare).map(String::as_str).collect();
    if all.is_empty() {
        labels.first().cloned().unwrap_or_default()
    } else {
        all.join("/")
    }
}

pub(in crate::event_loop) fn footer_hints(
    git: &str,
    nav: &str,
    palette: &str,
    menu: &str,
    room: usize,
) -> String {
    let all = [
        f::HINT_HELP.get().to_string(),
        f::hint_git(git),
        f::hint_nav(nav),
        f::hint_commands(palette),
        if menu.is_empty() {
            String::new()
        } else {
            f::hint_menu(menu)
        },
        f::HINT_QUIT.get().to_string(),
    ];
    // Drop order, first to go first (indices into `all`).
    let drop_order = [4usize, 5, 2];
    let mut keep = [true; 6];
    keep[4] = !menu.is_empty();
    let render = |keep: &[bool; 6]| {
        let parts: Vec<&str> = all
            .iter()
            .zip(keep)
            .filter(|(_, k)| **k)
            .map(|(h, _)| h.as_str())
            .collect();
        format!(" {} ", parts.join("  "))
    };
    let mut text = render(&keep);
    for i in drop_order {
        if display_width(&text) <= room {
            break;
        }
        keep[i] = false;
        text = render(&keep);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{footer_hints, nav_hint_rows, wrap_hint};
    use crate::display_width;
    use crate::keymap::Keymap;
    use gitpoulpi_i18n::{override_lang, Lang};

    fn assert_fits(rows: &[String], width: usize, what: &str) {
        assert!(
            !rows.is_empty() && rows.len() <= super::MAX_HINT_ROWS,
            "{what}: {rows:?}"
        );
        for r in rows {
            assert!(
                display_width(r) <= width,
                "{what}: {r:?} wider than {width}"
            );
        }
    }

    /// The Ctrl+N line names the dialog keys (theme, language, Settings, tab,
    /// Activity ...) from the keymap, never overflows, and when something
    /// does not fit ends with the help key's `all keys` hint instead of
    /// cutting an item. Red if the items fall back to the old four.
    #[test]
    fn the_nav_line_lists_the_dialog_keys_and_never_overflows() {
        let km = Keymap::defaults();
        let wide = nav_hint_rows(&km, 240).join(" ");
        for want in [
            "y=theme",
            "g=language",
            ",=settings",
            "t=select tab",
            "A=activity",
            "u=updates",
            "a=about",
            "h=collapse Refs",
            "?=help",
        ] {
            assert!(wide.contains(want), "{want} missing: {wide}");
        }
        assert!(!wide.contains("all keys"), "everything fits at 240: {wide}");
        for lang in [Lang::En, Lang::Fr, Lang::De] {
            let _l = override_lang(lang);
            for width in [80usize, 60, 40] {
                let rows = nav_hint_rows(&km, width);
                assert_fits(&rows, width, &format!("{lang:?} {width}"));
                let text = rows.join(" ");
                assert!(text.contains("y="), "{lang:?} {width}: theme key: {text}");
                if width <= 60 {
                    assert!(text.contains("?="), "{lang:?} {width}: no help: {text}");
                }
            }
        }
        let _l = override_lang(Lang::En);
        let narrow = nav_hint_rows(&km, 40).join(" ");
        assert!(narrow.contains("?=all keys"), "{narrow}");
    }

    #[test]
    fn a_long_hint_wraps_between_words_and_loses_nothing() {
        let text = "Ctrl+G : s=stage u=unstage a=stage-all x=discard-all";
        let rows = wrap_hint(text, 24);
        assert!(rows.len() > 1);
        assert!(
            rows.iter().all(|r| crate::display_width(r) <= 24),
            "{rows:?}"
        );
        assert_eq!(rows.join(" "), text);
    }

    #[test]
    fn french_stage_hints_follow_the_git_terms_setting() {
        use gitpoulpi_i18n::{msg::footer_hint as h, override_git_terms, override_lang, Lang};
        let _l = override_lang(Lang::Fr);
        {
            let _t = override_git_terms(false);
            assert_eq!(h::STAGE_ALL.get(), "stage-all");
        }
        let _t = override_git_terms(true);
        assert_eq!(h::STAGE_ALL.get(), "tout-indexer");
    }

    #[test]
    fn full_line_lists_the_menu_key_before_quit() {
        let t = footer_hints("Ctrl+G", "Ctrl+N", "Ctrl+P", "Shift+F10", 200);
        assert_eq!(
            t,
            " ?: help  Ctrl+G: git  Ctrl+N: nav  Ctrl+P: commands  Shift+F10: menu  q: quit "
        );
    }

    #[test]
    fn narrow_widths_drop_menu_then_quit_then_nav() {
        let w = |room| footer_hints("Ctrl+G", "Ctrl+N", "Ctrl+P", "Shift+F10", room);
        assert!(w(72).ends_with("Ctrl+P: commands  q: quit "));
        assert!(!w(72).contains("menu"));
        assert!(!w(55).contains("quit") && w(55).contains("nav"));
        assert!(!w(40).contains("nav") && w(40).contains("commands"));
    }

    #[test]
    fn french_hints_fit_at_100_and_160_columns_in_both_vocabularies() {
        use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
        for translated in [false, true] {
            let _l = override_lang(Lang::Fr);
            let _t = override_git_terms(translated);
            for room in [100usize, 160] {
                let t = footer_hints("Ctrl+G", "Ctrl+N", "Ctrl+P", "Shift+F10", room);
                assert!(
                    crate::display_width(&t) <= room,
                    "{} cells in {room}: {t:?}",
                    crate::display_width(&t)
                );
            }
            let t = footer_hints("Ctrl+G", "Ctrl+N", "Ctrl+P", "Shift+F10", 160);
            assert_eq!(
                t,
                " ?\u{a0}: aide  Ctrl+G\u{a0}: git  Ctrl+N\u{a0}: nav  Ctrl+P\u{a0}: commandes  Shift+F10\u{a0}: menu  q\u{a0}: quitter "
            );
        }
    }
}

/// `500 / 2500 commits` while more history exists than is loaded,
/// `500 / ... commits` while that total is being counted, plain `500
/// commits` once everything is held.
pub(in crate::event_loop) fn commit_count_text(loaded: usize, total: graph::TotalShown) -> String {
    match total {
        graph::TotalShown::Whole => f::commit_count(loaded as u64),
        graph::TotalShown::Counting => f::commit_total_pending(loaded),
        graph::TotalShown::Of(total) => f::commit_total(total, loaded),
    }
}

#[cfg(test)]
mod commit_total_tests {
    use super::*;

    #[test]
    fn the_status_bar_reads_loaded_over_total_and_a_placeholder_while_counting() {
        assert_eq!(
            commit_count_text(500, graph::TotalShown::Of(48213)),
            "500 / 48213 commits"
        );
        assert_eq!(
            commit_count_text(500, graph::TotalShown::Counting),
            "500 / \u{2026} commits"
        );
        assert_eq!(
            commit_count_text(2500, graph::TotalShown::Whole),
            "2500 commits"
        );
        assert_eq!(commit_count_text(1, graph::TotalShown::Whole), "1 commit");
    }
}
