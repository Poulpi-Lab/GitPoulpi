//! The commit composer under the Status pane: summary/description
//! fields, amend, rendering it and submitting the commit.
//!
//! A bordered box holds Summary and Description as two
//! visibly distinct input fields (placeholder text when empty, a caret when
//! focused, a highlighted border while either has focus), and underneath it
//! a full-width Commit button that is either a real green "commit"
//! affordance or a grayed-out one naming the reason it cannot be clicked yet.

use super::*;
use gitpoulpi_i18n::msg::composer as msg;

/// The shortest the composer ever renders at: separator, amend row, the
/// box's top border, one Summary line, the box's bottom border (no
/// Description line at all), and the button row. Below this the caller
/// hides the composer entirely rather than clipping it mid-row.
pub(crate) const COMPOSER_MIN_HEIGHT: u16 = 6;

/// The tallest the composer ever asks for: [`COMPOSER_MIN_HEIGHT`] plus a
/// thin rule separating Summary from Description and up to three
/// Description lines. `composer_height_for` never returns more than this,
/// however much room the pane has - the file list above it always keeps
/// some space of its own.
pub(crate) const COMPOSER_MAX_HEIGHT: u16 = COMPOSER_MIN_HEIGHT + 1 + MAX_DESCRIPTION_LINES;

const MAX_DESCRIPTION_LINES: u16 = 3;

/// The conventional Git subject-line limit (GitHub, `git`
/// itself all treat 72 columns as the point a summary is "too long" to
/// read whole in a log) - and what the Summary
/// counter counts *down from* rather than up to.
pub(crate) const SUMMARY_CHAR_LIMIT: usize = 72;
/// Once this few characters remain before [`SUMMARY_CHAR_LIMIT`], the
/// counter turns amber as a subtle heads-up before it goes red.
pub(crate) const SUMMARY_WARN_REMAINING: i32 = 10;

/// How tall to render the composer given `available` rows of room below
/// the file list, capped so the file list never loses all of its own
/// space to a growing box. Returns `0` (hide the composer) below
/// [`COMPOSER_MIN_HEIGHT`].
pub(crate) fn composer_height_for(available: u16) -> u16 {
    if available < COMPOSER_MIN_HEIGHT {
        0
    } else {
        available.min(COMPOSER_MAX_HEIGHT)
    }
}

/// Whether the composer is ready to commit, and if not, the reason to
/// show as the (disabled) button's own label - the same
/// register-a-hit-target-and-say-why convention as
/// `DiffBarButton::pending_reason`.
pub(crate) fn commit_readiness(
    summary: &str,
    amend: bool,
    staged_count: usize,
    merging: bool,
) -> Result<(), gitpoulpi_i18n::M> {
    // `merging`: a merge is stopped, and its commit needs nothing staged
    // (the resolved result may equal HEAD, "take ours" everywhere).
    if !amend && !merging && staged_count == 0 {
        return Err(msg::NEED_STAGED);
    }
    if summary.trim().is_empty() {
        return Err(msg::NEED_SUMMARY);
    }
    Ok(())
}

/// The Commit button's own label: the reason it's disabled, "Amend
/// previous commit" in amend mode, or "Commit changes to N files"
/// otherwise.
pub(crate) fn commit_button_label(
    amend: bool,
    staged_count: usize,
    merging: bool,
    ready: Result<(), gitpoulpi_i18n::M>,
) -> String {
    match ready {
        Err(reason) => reason.get().to_string(),
        Ok(()) if amend => msg::AMEND.get().to_string(),
        Ok(()) if merging && staged_count == 0 => gitpoulpi_i18n::msg::merge::BANNER_COMMIT_MERGE
            .get()
            .to_string(),
        Ok(()) => msg::commit_files(staged_count as u64),
    }
}

/// Submits the composer's contents as a commit.
///
/// Summary and description are joined git's own way - summary, blank
/// line, body - so `git log` renders them as it would any other commit.
pub(crate) fn submit_commit(
    link: &DaemonLink,
    summary: &str,
    description: &str,
    amend: bool,
    no_verify: bool,
    settings: &crate::commit_settings::CommitSettings,
    last_message: &mut Option<String>,
) {
    if summary.trim().is_empty() {
        *last_message = Some(msg::ERR_EMPTY_SUMMARY.get().into());
        return;
    }
    let message = if description.trim().is_empty() {
        summary.to_string()
    } else {
        format!("{summary}\n\n{description}")
    };
    // "Drop comment lines when making a commit": the commit is made with `-m`,
    // which git does not clean up, so the comment lines go here.
    let message = settings.prepared(&message);
    if message.trim().is_empty() {
        *last_message = Some(msg::ERR_EMPTY_SUMMARY.get().into());
        return;
    }
    // An agent's proposed message the user is committing (unchanged or
    // edited): its record says so.
    crate::ui_intent::proposal::confirmed(
        crate::ui_intent::proposal::Dialog::CommitMessage,
        &message,
    );
    link.submit_action(Action::Commit {
        message: message.clone(),
        amend,
        no_verify,
        push: settings.push_after_commit,
    });
    *last_message = Some(if amend {
        format!("Amending: {summary}")
    } else {
        format!("Committing: {summary}")
    });
}

/// Which composer field is taking keystrokes, if any. `None` means the
/// pane's file list has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposerField {
    Summary,
    Description,
    /// The Summary again, edited from the Log's WIP row instead of from
    /// this pane. **The same buffer** - `commit_summary` - so the two
    /// fields cannot disagree; only where the caret is drawn differs.
    /// Enter moves on to the Description here, exactly as it does from
    /// `Summary`, and Esc leaves the field with the text kept.
    WipSummary,
}

impl ComposerField {
    /// Whether this field edits the commit's summary line - in the
    /// composer or in the WIP row, which are one buffer.
    pub(crate) fn edits_summary(self) -> bool {
        matches!(self, ComposerField::Summary | ComposerField::WipSummary)
    }
}

/// What Enter (or the commit chord) does in a composer field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposerEnter {
    /// Move on to this field.
    Next(ComposerField),
    /// Type a line break (the Description is multi-line).
    Newline,
    /// Commit now.
    Commit,
}

/// Where Enter takes the composer from `field`: from either spelling of
/// the Summary on to the Description; in the Description, a new line
/// `commit_chord` ([`is_commit_chord`]) commits
/// from any of them.
pub(crate) fn composer_enter(field: ComposerField, commit_chord: bool) -> ComposerEnter {
    if commit_chord {
        return ComposerEnter::Commit;
    }
    match field {
        ComposerField::Summary | ComposerField::WipSummary => {
            ComposerEnter::Next(ComposerField::Description)
        }
        ComposerField::Description => ComposerEnter::Newline,
    }
}

/// The "commit now" chord of the composer: `Ctrl+Enter`, the chord the Edit
/// commit message popup uses to apply. Terminals disagree on how it
/// arrives, so its other spellings count too: `Alt+Enter`, and `Ctrl+J`
/// (what most Unix terminals send for `Ctrl+Enter`: a line feed, which
/// crossterm reads as `Ctrl+J` in raw mode), bare or as a `\n` character.
pub(crate) fn is_commit_chord(key: &KeyEvent) -> bool {
    keymap::current().is(keymap::Action::ComposerCommit, key)
}

/// Which of the composer's own clickable parts the mouse is over: the same
/// lighten-on-hover affordance every other button uses, as one small struct
/// since both live exactly as long as the composer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ComposerHover {
    pub(crate) amend_checkbox: bool,
    /// The Skip hooks checkbox is checked (`--no-verify` for this commit)...
    pub(crate) skip_hooks: bool,
    /// ...and the mouse is over it.
    pub(crate) skip_hooks_checkbox: bool,
    pub(crate) commit_button: bool,
    /// The AI Generate button: whether it exists at all (`ai.enabled`),
    /// whether a request is running, whether the mouse is over it.
    pub(crate) ai: ComposerAi,
    /// The `#` (insert issue reference) button.
    pub(crate) issue_ref: ComposerIssueRef,
}

/// State of the `#` button at the right of the Summary row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ComposerIssueRef {
    /// Drawn at all (a build with a hosting provider).
    pub(crate) shown: bool,
    /// An issue list is linked, so there is something to insert.
    pub(crate) enabled: bool,
    /// The mouse is over it.
    pub(crate) hovered: bool,
}

/// State of the composer's two AI buttons (Generate in the summary row,
/// Compose commits on the commit-options row). With AI off neither is
/// drawn nor registered as a hit target.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ComposerAi {
    /// AI is turned on.
    pub(crate) enabled: bool,
    pub(crate) running: bool,
    /// The mouse is over Generate.
    pub(crate) hovered: bool,
    /// The mouse is over Compose.
    pub(crate) compose_hovered: bool,
}

/// Draws the commit composer and registers its clickable parts.
///
/// Pulled out of the draw closure because it is self-contained (it reads
/// its inputs and paints a fixed band) and the closure is already long
/// enough to make an inline version hard to follow.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_commit_composer(
    frame: &mut Frame,
    area: Rect,
    hits: &mut Vec<(Rect, HitTarget)>,
    summary: &str,
    description: &str,
    amend: bool,
    focus: Option<ComposerField>,
    staged_count: usize,
    merging: bool,
    hover: ComposerHover,
    editor: &text_edit::FieldEditor<MainField>,
) {
    if area.height < COMPOSER_MIN_HEIGHT {
        return;
    }

    let [sep, amend_row, rest] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);
    let [box_area, button_row] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(rest);

    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(area.width as usize),
            Style::default().fg(th().ui.muted),
        ))),
        sep,
    );

    // -- Amend checkbox --------------------------------------------------
    let check = if amend { "[x]" } else { "[ ]" };
    let amend_label = format!("{check} {}", msg::AMEND.get());
    let amend_style = if amend {
        Style::default().fg(th().status.warning)
    } else {
        Style::default().fg(th().ui.muted)
    };
    let amend_style = if hover.amend_checkbox {
        amend_style.patch(crate::theme::plain_button_style(true))
    } else {
        amend_style
    };
    let amend_w = display_width(&amend_label) as u16;
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(amend_label, amend_style))),
        amend_row,
    );
    // Skip hooks sits after the Amend label when the row has room for both
    // (the palette and `Shift+H` reach it either way).
    let skip_label = format!(
        "{} {}",
        if hover.skip_hooks { "[x]" } else { "[ ]" },
        msg::SKIP_HOOKS.get()
    );
    let skip_w = display_width(&skip_label) as u16;
    // The AI Compose button keeps its (long) label before this one is drawn.
    let compose_w = if hover.ai.enabled {
        display_width(&format!(" {SPARK} {} ", msg::COMPOSE_LONG.get())) as u16
    } else {
        0
    };
    let skip_rect = (amend_row.width > amend_w + 2 + skip_w + compose_w)
        .then(|| Rect::new(amend_row.x + amend_w + 2, amend_row.y, skip_w, 1));
    if let Some(rect) = skip_rect {
        let style = if hover.skip_hooks {
            Style::default().fg(th().status.warning)
        } else {
            Style::default().fg(th().ui.muted)
        };
        let style = if hover.skip_hooks_checkbox {
            style.patch(crate::theme::plain_button_style(true))
        } else {
            style
        };
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(skip_label, style))),
            rect,
        );
    }
    // First registered wins a hit: the AI button sits on the amend row's
    // right end and must precede the amend hit; the amend hit is its label's
    // width (the Skip hooks checkbox has its own).
    let reserved = skip_rect.map_or(amend_w, |r| r.right() - amend_row.x);
    render_compose_button(frame, amend_row, hits, hover.ai, reserved as usize);
    if let Some(rect) = skip_rect {
        hits.push((rect, HitTarget::SkipHooksCheckbox));
    }
    hits.push((
        Rect::new(amend_row.x, amend_row.y, amend_w.min(amend_row.width), 1),
        HitTarget::AmendCheckbox,
    ));

    // -- The bordered box holding Summary and Description ---------------
    let box_focused = matches!(
        focus,
        Some(ComposerField::Summary) | Some(ComposerField::Description)
    );
    let block = Block::default()
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(if box_focused {
            Style::default().fg(th().ui.accent)
        } else {
            Style::default().fg(th().ui.muted)
        });
    let inner = block.inner(box_area);
    frame.render_clipped(block, box_area);

    if inner.height == 0 {
        return;
    }

    let [summary_row, desc_band] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);

    render_summary_row(
        hits,
        frame,
        summary_row,
        summary,
        focus,
        editor,
        hover.ai,
        hover.issue_ref,
        staged_count > 0,
    );

    // How much of `desc_band` is a thin rule versus Description lines: 0
    // rows of slack collapses Description away, 1 row shows a single line
    // with no separator, and 2+ rows spend the first on a rule and the rest
    // (capped at `MAX_DESCRIPTION_LINES`) on Description.
    if desc_band.height >= 2 {
        let [rule_row, desc_rows] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(desc_band);
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(
                "\u{2500}".repeat(rule_row.width as usize),
                Style::default().fg(th().ui.muted),
            ))),
            rule_row,
        );
        let desc_lines = desc_rows.height.min(MAX_DESCRIPTION_LINES);
        let desc_rows = Rect::new(desc_rows.x, desc_rows.y, desc_rows.width, desc_lines);
        render_description(hits, frame, desc_rows, description, focus, editor);
    } else if desc_band.height == 1 {
        render_description(hits, frame, desc_band, description, focus, editor);
    }

    // -- Commit button, full pane width -----------------------------------
    let ready = commit_readiness(summary, amend, staged_count, merging);
    let label = commit_button_label(amend, staged_count, merging, ready);
    hits.push((button_row, HitTarget::CommitButton));
    let style = match (ready.is_ok(), hover.commit_button) {
        (true, false) => Style::default()
            .bg(th().refs.local)
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD),
        (true, true) => Style::default()
            .bg(lighten(th().refs.local, 24))
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD),
        (false, false) => Style::default()
            .bg(th().chrome.disabled_button_bg)
            .fg(th().ui.disabled),
        (false, true) => Style::default()
            .bg(lighten(th().chrome.disabled_button_bg, 14))
            .fg(th().ui.disabled),
    };
    frame.render_clipped(
        Paragraph::new(Line::from(Span::styled(label, style)))
            .alignment(Alignment::Center)
            .style(style),
        button_row,
    );
}

/// Columns of the Generate glyph button in the summary row: space,
/// sparkle, space.
const GENERATE_W: usize = 3;
/// The sparkle glyph (text presentation, one column).
const SPARK: &str = "\u{2726}";
/// The summary row is too narrow for the Generate button below this.
const GENERATE_MIN_ROW: u16 = 24;
/// Columns of the `#` button: space, `#`, the dropdown arrow.
const ISSUE_REF_W: usize = 3;

/// "Compose commits with AI" at the right end of the commit-options row.
/// Not drawn when AI is off; the long label gives way to a short one, then to
/// nothing, rather than overlapping the 24-column amend label.
fn render_compose_button(
    frame: &mut Frame,
    row: Rect,
    hits: &mut Vec<(Rect, HitTarget)>,
    ai: ComposerAi,
    reserved_w: usize,
) {
    if !ai.enabled {
        return;
    }
    let long = format!(" {SPARK} {} ", msg::COMPOSE_LONG.get());
    let short = format!(" {SPARK} {} ", msg::COMPOSE_SHORT.get());
    // The checkboxes to its left ("[x] " and their text) keep the room they need.
    let amend_w = reserved_w.max(display_width(&format!("[x] {}", msg::AMEND.get())));
    let label = [long, short]
        .into_iter()
        .find(|l| row.width as usize > amend_w + display_width(l));
    let Some(label) = label else { return };
    let width = display_width(&label) as u16;
    let rect = Rect::new(row.right() - width, row.y, width, 1);
    hits.push((rect, HitTarget::AiCompose));
    let style = if ai.enabled {
        Style::default().fg(th().status.ai)
    } else {
        Style::default().fg(th().ui.muted)
    };
    let style = if ai.compose_hovered {
        style.patch(crate::theme::plain_button_style(true))
    } else {
        style
    };
    frame.render_clipped(Paragraph::new(Line::from(Span::styled(label, style))), rect);
}

/// The Summary row: placeholder when empty and unfocused, a scrolling
/// window with a trailing caret while focused, and a right-aligned
/// character counter, counting *down* from
/// [`SUMMARY_CHAR_LIMIT`] as you type, going negative past it (shown in
/// red) and amber once only [`SUMMARY_WARN_REMAINING`] or fewer remain.
#[allow(clippy::too_many_arguments)]
fn render_summary_row(
    hits: &mut Vec<(Rect, HitTarget)>,
    frame: &mut Frame,
    row: Rect,
    summary: &str,
    focus: Option<ComposerField>,
    editor: &text_edit::FieldEditor<MainField>,
    ai: ComposerAi,
    issue_ref: ComposerIssueRef,
    has_staged: bool,
) {
    let gen_w = if ai.enabled && row.width >= GENERATE_MIN_ROW {
        GENERATE_W
    } else {
        0
    };
    let ref_w = if issue_ref.shown && row.width >= GENERATE_MIN_ROW {
        ISSUE_REF_W
    } else {
        0
    };
    let gen_w = gen_w + ref_w;
    let focused = matches!(
        focus,
        Some(ComposerField::Summary) | Some(ComposerField::WipSummary)
    );
    let n = summary.chars().count();
    let remaining = SUMMARY_CHAR_LIMIT as i32 - n as i32;
    let count = remaining.to_string();
    // Right-aligned counter -- reserve its columns (plus one gap column)
    // before laying out the text so the two never overlap at a narrow
    // width. `count`'s own length already accounts for a leading `-` once
    // `remaining` goes negative (e.g. "-123"), so this stays correct
    // however long the summary gets.
    let count_width = count.len() + 1;
    let avail = (row.width as usize).saturating_sub(count_width + 1 + gen_w);
    editor.drawn(
        MainField::Summary,
        Rect::new(row.x, row.y, avail as u16, 1),
        false,
    );

    let mut spans = if focused {
        // The shared editor's rendering: caret anywhere, selection,
        // scrolling. Padding stays unstyled so the field's fill ends
        // with its text, as it always has.
        let look = text_field::EditLook::new(
            Style::default()
                .fg(th().ui.text)
                .bg(crate::input_focus_bg()),
            "_",
        );
        text_field::edit_line(summary, editor.caret(MainField::Summary), avail, true, look).spans
    } else if summary.is_empty() {
        let bg = Style::default().bg(crate::input_rest_bg());
        vec![
            Span::styled(msg::SUMMARY.get(), bg.fg(th().ui.muted)),
            Span::styled(
                " ".repeat(avail.saturating_sub(display_width(msg::SUMMARY.get()))),
                bg,
            ),
        ]
    } else {
        text_field::resting_spans(
            summary,
            avail,
            Style::default().fg(th().ui.text).bg(crate::input_rest_bg()),
        )
    };
    let used: usize = spans.iter().map(|s| s.width()).sum();
    let pad = (row.width as usize).saturating_sub(used + count_width + gen_w);
    spans.push(Span::raw(" ".repeat(pad)));
    if ref_w > 0 {
        // Registered before the row-wide summary hit (the first registered
        // wins), like Generate.
        let x = row.x + (used + pad) as u16;
        hits.push((
            Rect::new(x, row.y, ref_w as u16, 1),
            HitTarget::ComposerIssueRef,
        ));
        let base = Style::default().fg(if issue_ref.enabled {
            th().ui.accent
        } else {
            th().ui.disabled
        });
        let style = if issue_ref.hovered {
            base.patch(crate::theme::plain_button_style(true))
        } else {
            base
        };
        spans.push(Span::styled(" #\u{25BE}", style));
    }
    let gen_w = gen_w - ref_w;
    if gen_w > 0 {
        // First registered wins a hit: Generate precedes the row-wide
        // summary-field hit.
        let x = row.x + (used + pad + ref_w) as u16;
        hits.push((Rect::new(x, row.y, gen_w as u16, 1), HitTarget::AiGenerate));
        let base = if ai.running {
            Style::default().fg(th().status.warning)
        } else if ai.enabled && has_staged {
            Style::default().fg(th().status.ai)
        } else {
            Style::default().fg(th().ui.muted)
        };
        let style = if ai.hovered {
            base.patch(crate::theme::plain_button_style(true))
        } else {
            base
        };
        spans.push(Span::styled(format!(" {SPARK} "), style));
    }
    spans.push(Span::styled(
        count,
        Style::default().fg(if remaining < 0 {
            th().status.danger
        } else if remaining <= SUMMARY_WARN_REMAINING {
            th().status.warning
        } else {
            th().ui.muted
        }),
    ));
    hits.push((row, HitTarget::CommitSummaryField));
    frame.render_clipped(Paragraph::new(Line::from(spans)), row);
}

/// The Description area: placeholder on its first row when empty and
/// unfocused, word-wrapped across every row it was given while unfocused,
/// and (while focused) a single scrolling line, the same
/// horizontal-scroll-with-caret behavior as the Summary field (tracking a
/// caret's row *and* column inside wrapped text is out of scope). Growing or
/// shrinking the rows it is handed is `render_commit_composer`'s job.
fn render_description(
    hits: &mut Vec<(Rect, HitTarget)>,
    frame: &mut Frame,
    rows: Rect,
    description: &str,
    focus: Option<ComposerField>,
    editor: &text_edit::FieldEditor<MainField>,
) {
    if rows.height == 0 {
        return;
    }
    let focused = focus == Some(ComposerField::Description);
    // The scrollbar of a description longer than its box, over the
    // field's rightmost column, while it is being typed in (the text is
    // then scrolled lines, see `text_field::edit_lines`). Registered
    // before the field's own hit so a press on the bar wins.
    let track = if focused {
        let top = editor
            .caret(MainField::Description)
            .map_or(0, |c| c.top.get());
        text_field::field_track(rows, description, top)
    } else {
        None
    };
    let text_rows = if track.is_some() {
        Rect::new(rows.x, rows.y, rows.width - 1, rows.height)
    } else {
        rows
    };
    if let Some(track) = track {
        hits.push((
            track.rect,
            HitTarget::ScrollbarTrack(crate::scrollbar::ScrollBar::CommitDescription),
        ));
    }
    hits.push((rows, HitTarget::CommitDescriptionField));
    editor.drawn(MainField::Description, text_rows, true);

    if focused {
        // Multi-line while typing (Enter is a newline here): one row per
        // line, scrolled to keep the caret's line and column in view.
        let look = text_field::EditLook::new(
            Style::default()
                .fg(th().ui.text)
                .bg(crate::input_focus_bg()),
            "_",
        );
        let lines = text_field::edit_lines(
            description,
            editor.caret(MainField::Description),
            text_rows.width as usize,
            text_rows.height as usize,
            true,
            look,
        );
        frame.render_clipped(Paragraph::new(lines), text_rows);
        // The scroll `edit_lines` just settled on is what the thumb shows.
        let top = editor
            .caret(MainField::Description)
            .map_or(0, |c| c.top.get());
        if let Some(track) = text_field::field_track(rows, description, top) {
            crate::scrollbar::render(frame, track, true, false);
        }
        return;
    }

    let field_bg = Style::default().bg(crate::input_rest_bg());
    if description.is_empty() {
        let line = Line::from(Span::styled(
            msg::DESCRIPTION.get(),
            field_bg.fg(th().ui.muted),
        ));
        frame.render_clipped(
            Paragraph::new(line)
                .style(field_bg)
                .wrap(ratatui::widgets::Wrap { trim: false }),
            rows,
        );
        return;
    }

    frame.render_clipped(
        // `Text::styled` splits on the newlines Enter types, one `Line`
        // each; a single `Span` would carry them raw into the buffer.
        Paragraph::new(ratatui::text::Text::styled(
            description,
            field_bg.fg(th().ui.text),
        ))
        .style(field_bg)
        .wrap(ratatui::widgets::Wrap { trim: false }),
        rows,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint(
        width: u16,
        height: u16,
        summary: &str,
        description: &str,
        amend: bool,
        focus: Option<ComposerField>,
        staged_count: usize,
    ) -> ratatui::buffer::Buffer {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
                render_commit_composer(
                    frame,
                    Rect::new(0, 0, width, height),
                    &mut hits,
                    summary,
                    description,
                    amend,
                    focus,
                    staged_count,
                    false,
                    ComposerHover::default(),
                    &text_edit::FieldEditor::default(),
                );
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn paint_ai(
        width: u16,
        staged: usize,
        enabled: bool,
    ) -> (ratatui::buffer::Buffer, Vec<(Rect, HitTarget)>) {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(width, 12)).unwrap();
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        terminal
            .draw(|frame| {
                render_commit_composer(
                    frame,
                    Rect::new(0, 0, width, 12),
                    &mut hits,
                    "",
                    "",
                    false,
                    None,
                    staged,
                    false,
                    ComposerHover {
                        ai: ComposerAi {
                            enabled,
                            ..ComposerAi::default()
                        },
                        ..ComposerHover::default()
                    },
                    &text_edit::FieldEditor::default(),
                );
            })
            .unwrap();
        (terminal.backend().buffer().clone(), hits)
    }

    /// Generate sits in the summary row (right of it, by the counter);
    /// Compose sits on the amend row. Both registered from the paint walk;
    /// Generate is purple only with AI on AND something staged.
    #[test]
    fn generate_is_in_the_summary_row_and_compose_on_the_options_row() {
        let (buf, hits) = paint_ai(60, 1, true);
        let gen_rect = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::AiGenerate)
            .expect("generate hit")
            .0;
        let compose_rect = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::AiCompose)
            .expect("compose hit")
            .0;
        assert_eq!(
            gen_rect.y, 3,
            "summary row: sep, amend, box border, summary"
        );
        assert_eq!(compose_rect.y, 1, "the amend (commit-options) row");
        assert_eq!(buf[(gen_rect.x + 1, gen_rect.y)].symbol(), SPARK);
        assert_eq!(buf[(gen_rect.x + 1, gen_rect.y)].fg, th().status.ai);
        assert!(row_text(&buf, 1, 60).contains("Compose commits with AI"));
        // Generate's hit precedes the row-wide summary hit.
        let gi = hits.iter().position(|(_, t)| *t == HitTarget::AiGenerate);
        let si = hits
            .iter()
            .position(|(_, t)| *t == HitTarget::CommitSummaryField);
        assert!(gi < si);
    }

    #[test]
    fn generate_is_grayed_with_nothing_staged() {
        let (buf, hits) = paint_ai(60, 0, true);
        let r = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::AiGenerate)
            .unwrap()
            .0;
        assert_eq!(buf[(r.x + 1, r.y)].fg, th().ui.muted);
    }

    /// AI off: no sparkle drawn and no hit target left behind.
    #[test]
    fn with_ai_off_neither_ai_button_is_drawn_or_clickable() {
        let (buf, hits) = paint_ai(60, 1, false);
        assert!(hits
            .iter()
            .all(|(_, t)| !matches!(t, HitTarget::AiGenerate | HitTarget::AiCompose)));
        assert!(!buffer_contains(&buf, SPARK));
    }

    fn row_text(buf: &ratatui::buffer::Buffer, y: u16, width: u16) -> String {
        (0..width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    }

    fn buffer_contains(buf: &ratatui::buffer::Buffer, needle: &str) -> bool {
        let mut all = String::new();
        for y in 0..buf.area.height {
            all.push_str(&row_text(buf, y, buf.area.width));
            all.push('\n');
        }
        all.contains(needle)
    }

    /// The Summary row is a fixed-width field: its character count is pinned
    /// to the row's right edge, and the caret has to stay on screen however
    /// long the summary gets. Truncating the focused field from the right
    /// would hide both the caret and the characters just typed.
    #[test]
    fn the_composer_summary_field_keeps_its_caret_and_its_count_column() {
        let paint_at = |summary: &str| -> ratatui::buffer::Buffer {
            paint(
                60,
                COMPOSER_MIN_HEIGHT,
                summary,
                "",
                false,
                Some(ComposerField::Summary),
                1,
            )
        };

        let short = paint_at("fix");
        let long = paint_at(&"fix: a summary far longer than sixty columns of row".repeat(3));
        // Row 3 of the composer at COMPOSER_MIN_HEIGHT: sep(0), amend(1),
        // box-top(2), summary(3). One entry per screen *column* (not per
        // char): a byte or char offset into the joined row string would
        // drift once a multi-byte glyph like the field's own left-elision
        // ellipsis appears earlier in the row.
        let row_cells = |buf: &ratatui::buffer::Buffer, y: u16| -> Vec<String> {
            (0..60).map(|x| buf[(x, y)].symbol().to_string()).collect()
        };
        let short_row = row_cells(&short, 3);
        let long_row = row_cells(&long, 3);
        let count_col = |cells: &[String]| {
            cells
                .iter()
                .rposition(|s| s.chars().all(|c| c.is_ascii_digit()) && s != " ")
        };
        assert_eq!(
            count_col(&short_row),
            count_col(&long_row),
            "the character count moved when the summary grew"
        );
        let short_joined: String = short_row.concat();
        let long_joined: String = long_row.concat();
        assert!(
            short_joined.contains('_') && long_joined.contains('_'),
            "the caret fell off the summary field"
        );
        assert!(
            long_joined.contains("columns of row"),
            "the end of the summary isn't on screen; the row reads {long_joined:?}"
        );
    }

    /// Falsifiable: an empty, unfocused Summary shows the dim
    /// "Summary" placeholder rather than a blank field indistinguishable
    /// from chrome.
    #[test]
    fn empty_summary_shows_a_placeholder_when_unfocused() {
        let buf = paint(60, COMPOSER_MIN_HEIGHT, "", "", false, None, 0);
        assert!(
            buffer_contains(&buf, "Summary"),
            "no placeholder text painted for an empty, unfocused Summary"
        );
    }

    /// Same for Description, and only once there's a row to paint it in.
    #[test]
    fn empty_description_shows_a_placeholder_when_there_is_room() {
        let buf = paint(60, COMPOSER_MIN_HEIGHT + 2, "fix", "", false, None, 1);
        assert!(
            buffer_contains(&buf, "Description"),
            "no placeholder text painted for an empty Description with room to show it"
        );
    }

    /// The counter counts down from [`SUMMARY_CHAR_LIMIT`]: neutral with
    /// plenty left, amber once [`SUMMARY_WARN_REMAINING`] or fewer remain,
    /// red once it goes negative - three distinct colors on three
    /// distinct counter cells.
    #[test]
    fn the_summary_counter_changes_color_past_its_warn_and_hard_limits() {
        let color_of_last_digit = |buf: &ratatui::buffer::Buffer| {
            let row = 3u16; // summary row at COMPOSER_MIN_HEIGHT
            (0..buf.area.width)
                .rev()
                .find_map(|x| {
                    let cell = &buf[(x, row)];
                    cell.symbol()
                        .chars()
                        .next()
                        .filter(|c| c.is_ascii_digit())
                        .map(|_| cell.fg)
                })
                .expect("no digit painted on the summary row")
        };

        let neutral = paint(60, COMPOSER_MIN_HEIGHT, "short", "", false, None, 1);
        let amber = paint(
            60,
            COMPOSER_MIN_HEIGHT,
            &"x".repeat(SUMMARY_CHAR_LIMIT - SUMMARY_WARN_REMAINING as usize),
            "",
            false,
            None,
            1,
        );
        let red = paint(
            60,
            COMPOSER_MIN_HEIGHT,
            &"x".repeat(SUMMARY_CHAR_LIMIT + 1),
            "",
            false,
            None,
            1,
        );
        let c_neutral = color_of_last_digit(&neutral);
        let c_amber = color_of_last_digit(&amber);
        let c_red = color_of_last_digit(&red);
        assert_eq!(c_neutral, th().ui.muted);
        assert_eq!(c_amber, th().status.warning);
        assert_eq!(c_red, th().status.danger);
        assert_ne!(c_amber, c_red);
    }

    /// The counter starts at [`SUMMARY_CHAR_LIMIT`] and counts down as you
    /// type, not up from zero.
    #[test]
    fn the_summary_counter_starts_at_the_limit_and_counts_down() {
        let empty = paint(60, COMPOSER_MIN_HEIGHT, "", "", false, None, 1);
        assert!(
            buffer_contains(&empty, &SUMMARY_CHAR_LIMIT.to_string()),
            "an empty summary must show the full limit, not 0"
        );

        let one_char = paint(60, COMPOSER_MIN_HEIGHT, "a", "", false, None, 1);
        assert!(
            buffer_contains(&one_char, &(SUMMARY_CHAR_LIMIT - 1).to_string()),
            "one typed character must count down by one"
        );
    }

    /// Past the limit the counter goes negative - and the reserved width
    /// must grow to fit the minus sign and extra digit ("-123") without
    /// the counter clipping into the summary text beside it.
    #[test]
    fn the_summary_counter_goes_negative_without_overlapping_the_text() {
        let long = "x".repeat(SUMMARY_CHAR_LIMIT + 51); // remaining = -51
        let buf = paint(60, COMPOSER_MIN_HEIGHT, &long, "", false, None, 1);
        assert!(
            buffer_contains(&buf, "-51"),
            "expected a negative counter for a summary 51 chars past the limit"
        );
        let row = 3u16;
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, row)].symbol().to_string())
            .collect();
        assert!(
            !line.contains("x-51") && !line.contains("-51x"),
            "counter and summary text must not run into each other: {line:?}"
        );
    }

    /// The disabled button shows the reason it's inert, not a generic
    /// "Commit" label - and the two reasons are distinguishable.
    #[test]
    fn the_disabled_button_names_its_reason() {
        let no_staged = paint(60, COMPOSER_MIN_HEIGHT, "fix: x", "", false, None, 0);
        assert!(
            buffer_contains(&no_staged, "Stage changes to commit"),
            "button didn't say why it's disabled with nothing staged"
        );

        let no_summary = paint(60, COMPOSER_MIN_HEIGHT, "", "", false, None, 1);
        assert!(
            buffer_contains(&no_summary, "Enter a commit summary"),
            "button didn't say why it's disabled with an empty summary"
        );
    }

    /// Committable, non-amend: the button names the file count.
    #[test]
    fn the_ready_button_names_the_file_count() {
        let buf = paint(60, COMPOSER_MIN_HEIGHT, "fix: x", "", false, None, 3);
        assert!(
            buffer_contains(&buf, "Commit changes to 3 files"),
            "ready button doesn't name the staged file count"
        );
        let one = paint(60, COMPOSER_MIN_HEIGHT, "fix: x", "", false, None, 1);
        assert!(
            buffer_contains(&one, "Commit changes to 1 file"),
            "singular form missing for exactly one staged file"
        );
    }

    /// Amend mode never demands staged files, and its ready label is
    /// "Amend previous commit" rather than a file count.
    #[test]
    fn amend_mode_is_committable_with_nothing_staged_and_says_so() {
        assert_eq!(commit_readiness("fix: x", true, 0, false), Ok(()));
        let buf = paint(60, COMPOSER_MIN_HEIGHT, "fix: x", "", true, None, 0);
        assert!(
            buffer_contains(&buf, "Amend previous commit"),
            "amend button doesn't carry the amend label"
        );
    }

    /// While a merge is stopped its commit needs nothing staged (the
    /// resolved result may equal HEAD); a plain commit still does.
    #[test]
    fn a_merge_commit_is_ready_with_nothing_staged() {
        assert_eq!(commit_readiness("Merge x", false, 0, true), Ok(()));
        assert_eq!(
            commit_readiness("Merge x", false, 0, false),
            Err(msg::NEED_STAGED)
        );
        assert_eq!(commit_readiness("", false, 0, true), Err(msg::NEED_SUMMARY));
    }

    /// Small panes degrade rather than panic: below `COMPOSER_MIN_HEIGHT`
    /// nothing is drawn (and nothing panics), and `composer_height_for`
    /// never asks for more than the pane can spare above the file list.
    #[test]
    fn tiny_areas_draw_nothing_and_never_panic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        for height in 0..COMPOSER_MIN_HEIGHT {
            let mut terminal = Terminal::new(TestBackend::new(40, COMPOSER_MIN_HEIGHT)).unwrap();
            terminal
                .draw(|frame| {
                    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
                    render_commit_composer(
                        frame,
                        Rect::new(0, 0, 40, height),
                        &mut hits,
                        "a summary",
                        "a description",
                        false,
                        Some(ComposerField::Description),
                        2,
                        false,
                        ComposerHover::default(),
                        &text_edit::FieldEditor::default(),
                    );
                    assert!(
                        hits.is_empty(),
                        "a too-short composer still registered hit targets"
                    );
                })
                .unwrap();
        }
        assert_eq!(composer_height_for(0), 0);
        assert_eq!(composer_height_for(COMPOSER_MIN_HEIGHT - 1), 0);
        assert_eq!(
            composer_height_for(COMPOSER_MIN_HEIGHT),
            COMPOSER_MIN_HEIGHT
        );
        assert_eq!(composer_height_for(1000), COMPOSER_MAX_HEIGHT);
    }

    /// Description degrades before it disappears: given exactly one
    /// spare row over the minimum, it still shows on one line with no
    /// separating rule; given none, it isn't drawn (and isn't clickable)
    /// at all.
    #[test]
    fn description_collapses_to_one_line_then_hides() {
        let one_line = paint(60, COMPOSER_MIN_HEIGHT + 1, "fix", "", false, None, 1);
        assert!(
            buffer_contains(&one_line, "Description"),
            "one spare row over the minimum should still show a Description line"
        );

        // At the bare minimum, Description has no row at all.
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(60, COMPOSER_MIN_HEIGHT)).unwrap();
        terminal
            .draw(|frame| {
                let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
                render_commit_composer(
                    frame,
                    Rect::new(0, 0, 60, COMPOSER_MIN_HEIGHT),
                    &mut hits,
                    "fix",
                    "",
                    false,
                    None,
                    1,
                    false,
                    ComposerHover::default(),
                    &text_edit::FieldEditor::default(),
                );
                assert!(
                    !hits
                        .iter()
                        .any(|(_, t)| *t == HitTarget::CommitDescriptionField),
                    "Description field is clickable even with no room to show it"
                );
            })
            .unwrap();
    }

    /// Every clickable part of the composer (the amend checkbox, both fields
    /// and the button) registers a hit target at two sizes, and each rect
    /// sits fully inside the area the composer was given: a control that
    /// paints inside the pane but registers its hit rect outside it is
    /// clickable where nothing is drawn.
    #[test]
    fn every_composer_control_has_a_hit_target_inside_the_pane_at_two_sizes() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        for (width, height) in [(60, COMPOSER_MIN_HEIGHT), (80, COMPOSER_MAX_HEIGHT)] {
            let area = Rect::new(0, 0, width, height);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
            terminal
                .draw(|frame| {
                    render_commit_composer(
                        frame,
                        area,
                        &mut hits,
                        "a summary",
                        "a description",
                        false,
                        None,
                        2,
                        false,
                        ComposerHover::default(),
                        &text_edit::FieldEditor::default(),
                    );
                })
                .unwrap();

            for target in [
                HitTarget::AmendCheckbox,
                HitTarget::CommitSummaryField,
                HitTarget::CommitButton,
            ] {
                let rect = hits
                    .iter()
                    .find(|(_, t)| *t == target)
                    .unwrap_or_else(|| panic!("no hit target for {target:?} at {width}x{height}"))
                    .0;
                assert!(
                    area.intersection(rect) == rect,
                    "{target:?}'s hit rect {rect:?} falls outside the composer's own area {area:?} at {width}x{height}"
                );
            }
        }
    }

    /// The composer in French (I8b): placeholders, the commit button and the
    /// AI button fit a 40-column pane, git terms on or off.
    #[test]
    fn the_french_composer_fits_with_git_terms_on_and_off() {
        use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
        for terms in [false, true] {
            let _l = override_lang(Lang::Fr);
            let _t = override_git_terms(terms);
            let buf = paint(40, COMPOSER_MIN_HEIGHT.max(12), "", "", false, None, 3);
            assert!(buffer_contains(&buf, "Résumé"), "terms={terms}");
            assert!(buffer_contains(&buf, "Description"), "terms={terms}");
            let verb = if terms {
                "Valider 3 fichiers"
            } else {
                "Commiter 3 fichiers"
            };
            assert!(buffer_contains(&buf, "Saisissez un"), "terms={terms}");
            let ready = paint(
                40,
                COMPOSER_MIN_HEIGHT.max(12),
                "fix: x",
                "",
                false,
                None,
                3,
            );
            assert!(buffer_contains(&ready, verb), "terms={terms}");
            let amend = paint(60, 12, "fix: x", "", true, None, 0);
            assert!(buffer_contains(&amend, "Amender"), "terms={terms}");
            // A wide pane has room for the AI button beside the amend label.
            let (_, hits) = paint_ai(80, 3, true);
            assert!(hits.iter().any(|(_, t)| *t == HitTarget::AiCompose));
            let (_, narrow) = paint_ai(40, 3, true);
            for (rect, _) in narrow {
                assert!(rect.x + rect.width <= 40);
            }
        }
    }

    /// Skip hooks: a checkbox beside Amend with its own hit rect and hover
    /// (the click is `HitTarget::SkipHooksCheckbox`, the keyboard twin
    /// `Shift+H` and the palette command), checked or not as the state says,
    /// and gone on a row too narrow for it so Amend and Compose keep their room.
    #[test]
    fn skip_hooks_sits_beside_amend_with_its_own_hit_and_state() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let paint_with = |width: u16, skip: bool, hovered: bool| {
            let mut terminal = Terminal::new(TestBackend::new(width, 12)).unwrap();
            let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
            terminal
                .draw(|frame| {
                    render_commit_composer(
                        frame,
                        Rect::new(0, 0, width, 12),
                        &mut hits,
                        "",
                        "",
                        false,
                        None,
                        1,
                        false,
                        ComposerHover {
                            skip_hooks: skip,
                            skip_hooks_checkbox: hovered,
                            ..ComposerHover::default()
                        },
                        &text_edit::FieldEditor::default(),
                    );
                })
                .unwrap();
            (terminal.backend().buffer().clone(), hits)
        };
        let (buf, hits) = paint_with(80, false, false);
        assert!(row_text(&buf, 1, 80).contains("[ ] Skip hooks"));
        let rect = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::SkipHooksCheckbox)
            .expect("skip hooks hit")
            .0;
        assert_eq!(rect.y, 1, "the amend row");
        assert_eq!(buf[(rect.x, rect.y)].symbol(), "[");
        let amend = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::AmendCheckbox)
            .unwrap()
            .0;
        assert!(amend.right() <= rect.x, "the two hits do not overlap");

        let (buf, _) = paint_with(80, true, false);
        let row = row_text(&buf, 1, 80);
        assert!(
            row.contains("[x] Skip hooks") && row.contains("[ ] Amend"),
            "{row}"
        );
        let (hover, _) = paint_with(80, true, true);
        assert_ne!(
            buf[(rect.x, rect.y)].style(),
            hover[(rect.x, rect.y)].style(),
            "hover lights the label"
        );

        // Too narrow for both: Amend keeps the row, no Skip hooks hit.
        let (buf, hits) = paint_with(30, false, false);
        assert!(!row_text(&buf, 1, 30).contains("Skip hooks"));
        assert!(!hits.iter().any(|(_, t)| *t == HitTarget::SkipHooksCheckbox));
        assert!(hits.iter().any(|(_, t)| *t == HitTarget::AmendCheckbox));
    }
}
