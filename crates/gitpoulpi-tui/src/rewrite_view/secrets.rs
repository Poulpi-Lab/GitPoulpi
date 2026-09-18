//! The Secrets tab (slice S14): scan the scope's history for secrets, list
//! what was found, turn the ticked findings into one rule.
//!
//! The TUI never holds a secret value. A finding carries a rule id, paths,
//! two commits, counts and a redacted preview (`ghp_****abcd`); the rule it
//! adds names findings by their id (`f-` and twelve hex digits), and the
//! daemon, which can read the history, maps ids to values for the length of
//! the preview call. Findings are stale after any rewrite (the ids of a
//! history that no longer holds the value find nothing, and the daemon says
//! `InvalidRule`), so the tab says so and offers to scan again.

use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    ContentFind, ContentReplace, RewriteProgress, Rule, RuleKind, Scope, SecretFinding, SecretScan,
    SecretText,
};
use gitpoulpi_i18n::msg::{rewrite_secrets as s, rewrite_view as t};
use gitpoulpi_i18n::M;
use gitpoulpi_ipc::rewrite::ScanSecretsArgs;

use super::render::{bad, bold, dim, plain, row_bg, warn, wrap, RewriteHit, Ui};
use super::{DetailForm, Focus, Notice, PreviewTab, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit;
use crate::th;

/// A scan on its way.
pub(crate) struct ScanRun {
    pub(super) reply: Reply<SecretScan>,
    pub since: Instant,
    /// The daemon job, known from the first progress report that is its own.
    pub job: Option<u64>,
    pub progress: Option<RewriteProgress>,
    /// The progress the connection already held when the scan was sent: the
    /// last report of an earlier job, which is not this scan's.
    baseline: Option<Option<(u64, RewriteProgress)>>,
    scope: Scope,
}

/// What a finished scan found, and which findings are ticked.
pub(crate) struct ScanResult {
    pub scan: SecretScan,
    /// The scope the scan read, to tell when the recipe's has moved on.
    pub scope: Scope,
    pub ticked: HashSet<String>,
}

#[derive(Default)]
pub(crate) struct SecretsState {
    pub running: Option<ScanRun>,
    pub result: Option<ScanResult>,
    pub cursor: usize,
    /// A rewrite ran (or the daemon refused an id) since the scan.
    pub stale: bool,
    /// Why the last scan failed.
    pub error: Option<String>,
}

/// The label of a scanner rule id; `None` for one this build does not know.
pub(super) fn rule_label(id: &str) -> Option<M> {
    Some(match id {
        "github-token" => s::R_GITHUB_TOKEN,
        "github-pat" => s::R_GITHUB_PAT,
        "gitlab-token" => s::R_GITLAB_TOKEN,
        "slack-token" => s::R_SLACK_TOKEN,
        "stripe-key" => s::R_STRIPE_KEY,
        "anthropic-key" => s::R_ANTHROPIC_KEY,
        "openai-key" => s::R_OPENAI_KEY,
        "google-api-key" => s::R_GOOGLE_API_KEY,
        "npm-token" => s::R_NPM_TOKEN,
        "aws-access-key" => s::R_AWS_ACCESS_KEY,
        "private-key" => s::R_PRIVATE_KEY,
        "jwt" => s::R_JWT,
        "url-credentials" => s::R_URL_CREDENTIALS,
        "generic-secret" => s::R_GENERIC_SECRET,
        _ => return None,
    })
}

fn rule_text(finding: &SecretFinding) -> String {
    rule_label(&finding.rule)
        .map(|m| m.get().to_string())
        .unwrap_or_else(|| finding.rule.clone())
}

fn short(oid: &gitpoulpi_core::domain::Oid) -> String {
    super::preview::short(oid)
}

/// Whether `rule` is a scanning secrets rule made of finding ids only (what
/// "Remove from history" writes, and may extend).
fn is_id_rule(rule: &Rule) -> bool {
    matches!(
        &rule.kind,
        RuleKind::ContentReplace(ContentReplace {
            paths,
            find: ContentFind::Secrets { literals, scan: true },
            with,
            ..
        }) if paths.is_empty()
            && with.is_empty()
            && !literals.is_empty()
            && literals.iter().all(|l| l.finding_id().is_some())
    )
}

impl RewriteView {
    pub(crate) fn scanning(&self) -> bool {
        self.secrets.running.is_some()
    }

    /// Why the findings on screen should not be trusted any more.
    pub(crate) fn secrets_stale_reason(&self) -> Option<M> {
        let result = self.secrets.result.as_ref()?;
        if self.secrets.stale {
            Some(s::STALE_REWRITE)
        } else if result.scope != self.recipe.scope {
            Some(s::STALE_SCOPE)
        } else {
            None
        }
    }

    // ------------------------------------------------------------- daemon

    /// "Scan for secrets": the scope of the workbench, as a job.
    pub(crate) fn start_scan(&mut self) {
        if self.busy() {
            return;
        }
        self.notice = None;
        self.secrets.error = None;
        let scope = self.recipe.scope.clone();
        let args = ScanSecretsArgs {
            scope: scope.clone(),
        };
        let reply = self.requester.call(
            gitpoulpi_core::domain::OperationKind::RewriteScanSecrets,
            serde_json::to_value(args).unwrap_or_default(),
        );
        self.secrets.running = Some(ScanRun {
            reply,
            since: Instant::now(),
            job: None,
            progress: None,
            baseline: None,
            scope,
        });
    }

    /// Stops the running scan (the job is the daemon's).
    pub(crate) fn cancel_scan(&mut self) {
        if let Some(job) = self.secrets.running.as_ref().and_then(|r| r.job) {
            crate::jobs_ui::cancel(job);
        }
    }

    /// Collects the scan's progress and answer. Never waits.
    pub(super) fn poll_secrets(&mut self, progress: Option<(u64, RewriteProgress)>) {
        let Some(run) = self.secrets.running.as_mut() else {
            return;
        };
        match run.baseline {
            None => run.baseline = Some(progress),
            Some(base) => {
                if let Some((job, report)) = progress {
                    if base != Some((job, report)) {
                        run.job = Some(job);
                        run.progress = Some(report);
                    }
                }
            }
        }
        let Some(answer) = run.reply.take() else {
            return;
        };
        let scope = run.scope.clone();
        self.secrets.running = None;
        match answer {
            Ok(scan) => {
                let ticked = scan.findings.iter().map(|f| f.id.clone()).collect();
                self.secrets.result = Some(ScanResult {
                    scan,
                    scope,
                    ticked,
                });
                self.secrets.cursor = 0;
                self.secrets.stale = false;
            }
            Err(error) => {
                use gitpoulpi_core::TypedError as T;
                if matches!(error.typed(), Some(T::Canceled { .. })) {
                    self.notice = Some(Notice {
                        text: s::SCAN_CANCELED.get().to_string(),
                        error: false,
                    });
                } else {
                    self.secrets.error = Some(crate::daemon_link::error_text::error_text(&error));
                }
            }
        }
    }

    // ----------------------------------------------------------- the list

    pub(crate) fn findings_len(&self) -> usize {
        self.findings().len()
    }

    fn findings(&self) -> &[SecretFinding] {
        self.secrets
            .result
            .as_ref()
            .map_or(&[][..], |r| &r.scan.findings)
    }

    pub(crate) fn ticked_findings(&self) -> usize {
        self.secrets.result.as_ref().map_or(0, |r| {
            r.scan
                .findings
                .iter()
                .filter(|f| r.ticked.contains(&f.id))
                .count()
        })
    }

    pub(crate) fn move_secret_cursor(&mut self, delta: isize) {
        let last = self.findings().len().saturating_sub(1);
        self.secrets.cursor =
            (self.secrets.cursor as isize + delta).clamp(0, last as isize) as usize;
    }

    /// Ticks or unticks finding `index`.
    pub(crate) fn toggle_finding(&mut self, index: usize) {
        let Some(result) = self.secrets.result.as_mut() else {
            return;
        };
        let Some(id) = result.scan.findings.get(index).map(|f| f.id.clone()) else {
            return;
        };
        if !result.ticked.remove(&id) {
            result.ticked.insert(id);
        }
    }

    pub(crate) fn check_all_findings(&mut self) {
        if let Some(r) = self.secrets.result.as_mut() {
            r.ticked = r.scan.findings.iter().map(|f| f.id.clone()).collect();
        }
    }

    pub(crate) fn uncheck_all_findings(&mut self) {
        if let Some(r) = self.secrets.result.as_mut() {
            r.ticked.clear();
        }
    }

    /// "Remove from history": the ticked findings become one scanning rule
    /// of the recipe (an earlier one made here is extended, not repeated).
    pub(crate) fn remove_findings(&mut self) {
        let ids: Vec<String> = match self.secrets.result.as_ref() {
            Some(r) => r
                .scan
                .findings
                .iter()
                .filter(|f| r.ticked.contains(&f.id))
                .map(|f| f.id.clone())
                .collect(),
            None => Vec::new(),
        };
        if ids.is_empty() {
            self.notice = Some(Notice {
                text: s::NONE_TICKED.get().to_string(),
                error: true,
            });
            return;
        }
        let existing = self.recipe.rules.iter().position(is_id_rule);
        let index = match existing {
            Some(i) => {
                if let RuleKind::ContentReplace(ContentReplace {
                    find: ContentFind::Secrets { literals, .. },
                    ..
                }) = &mut self.recipe.rules[i].kind
                {
                    for id in &ids {
                        if !literals.iter().any(|l| l.0 == *id) {
                            literals.push(SecretText(id.clone()));
                        }
                    }
                }
                i
            }
            None => {
                self.recipe
                    .rules
                    .push(Rule::new(RuleKind::ContentReplace(ContentReplace {
                        paths: Vec::new(),
                        find: ContentFind::Secrets {
                            literals: ids.iter().map(|id| SecretText(id.clone())).collect(),
                            scan: true,
                        },
                        with: String::new(),
                        max_blob_bytes: None,
                    })));
                self.recipe.rules.len() - 1
            }
        };
        self.rules.cursor = index;
        self.detail = DetailForm::default();
        self.sync_detail();
        self.touch();
        self.notice = Some(Notice {
            text: s::added_rule(ids.len() as u64, index + 1),
            error: false,
        });
    }

    // --------------------------------------------------------------- keys

    pub(super) fn on_secrets_key(&mut self, key: KeyEvent, plain: bool) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_secret_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_secret_cursor(1),
            KeyCode::PageUp => self.move_secret_cursor(-10),
            KeyCode::PageDown => self.move_secret_cursor(10),
            KeyCode::Char(' ') | KeyCode::Enter if plain => {
                self.toggle_finding(self.secrets.cursor)
            }
            KeyCode::Char('s') if plain => self.start_scan(),
            KeyCode::Char('r') if plain => self.remove_findings(),
            _ => {}
        }
    }

    // -------------------------------------------------------------- paint

    pub(super) fn paint_secrets(&self, ui: &mut Ui<'_>, body: Rect) {
        let x = body.x + 1;
        let w = body.width.saturating_sub(1);
        let width = body.width.saturating_sub(2) as usize;
        let mut y = body.y;
        let bottom = body.bottom();
        let lines = |ui: &mut Ui<'_>, y: &mut u16, text: &str, style: Style, max: usize| {
            for line in wrap(text, width).into_iter().take(max) {
                if *y >= bottom {
                    return;
                }
                ui.text(x, *y, w, &line, style);
                *y += 1;
            }
        };
        lines(ui, &mut y, s::ROTATE_FIRST.get(), warn(), 2);

        // The scan button (a Cancel while it runs) and what was read.
        if y < bottom {
            if self.scanning() {
                ui.button(x, y, t::BTN_CANCEL.get(), RewriteHit::Cancel, false, true);
            } else {
                ui.button(
                    x,
                    y,
                    s::BTN_SCAN.get(),
                    RewriteHit::SecretsScan,
                    self.secrets.result.is_none(),
                    !self.busy(),
                );
            }
            y += 1;
        }
        // What was read and what could not be: the counters that make "not
        // exhaustive" visible.
        if let Some(r) = &self.secrets.result {
            let mut parts = vec![s::summary(r.scan.commits, r.scan.blobs_scanned)];
            if r.scan.blobs_skipped_binary > 0 {
                parts.push(s::skipped_binary(r.scan.blobs_skipped_binary));
            }
            if r.scan.blobs_skipped_large > 0 {
                parts.push(s::skipped_large(r.scan.blobs_skipped_large));
            }
            lines(ui, &mut y, &parts.join("; "), dim(), 2);
        }
        lines(ui, &mut y, s::NOT_EXHAUSTIVE.get(), dim(), 2);
        if let Some(reason) = self.secrets_stale_reason() {
            lines(ui, &mut y, reason.get(), warn(), 1);
        }
        if let Some(why) = &self.secrets.error {
            lines(ui, &mut y, why, bad(), 2);
        }
        if y >= bottom {
            return;
        }

        let findings = self.findings();
        if self.secrets.result.is_none() {
            if !self.scanning() {
                lines(ui, &mut y, s::IDLE.get(), plain(), 3);
            }
            return;
        }
        if findings.is_empty() {
            lines(ui, &mut y, s::NONE_FOUND.get(), plain(), 1);
            return;
        }

        // List, then the detail of the finding under the cursor, then the
        // actions. The detail and the actions go when the pane is short.
        let actions_y = bottom - 1;
        let detail_h: u16 = if bottom.saturating_sub(y) >= 8 { 3 } else { 0 };
        let list_end = actions_y.saturating_sub(detail_h);
        let room = list_end.saturating_sub(y) as usize;
        let cursor = self.secrets.cursor.min(findings.len() - 1);
        let mut top = self.paint.secrets_top.get();
        if cursor < top {
            top = cursor;
        } else if room > 0 && cursor >= top + room {
            top = cursor + 1 - room;
        }
        top = top.min(findings.len().saturating_sub(room));
        self.paint.secrets_top.set(top);
        let focused = self.focus == Focus::Preview;
        let ticked = &self.secrets.result.as_ref().map(|r| &r.ticked);
        let label_w = (width / 4).clamp(12, 26);
        let preview_w = 22usize;
        for (slot, finding) in findings.iter().enumerate().skip(top).take(room) {
            let ry = y + (slot - top) as u16;
            let band = Rect::new(body.x, ry, body.width, 1);
            let hit = RewriteHit::SecretRow(slot);
            let bg = row_bg(slot == cursor && focused, ui.hot(hit));
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            let on = ticked.is_some_and(|t| t.contains(&finding.id));
            let style = Style::default().fg(th().ui.text).bg(bg);
            let mut cx = band.x + 1;
            cx += ui.text(cx, ry, 4, if on { "[x] " } else { "[ ] " }, style);
            let label = rule_text(finding);
            ui.text(cx, ry, label_w as u16, &label, style);
            cx += label_w as u16 + 1;
            ui.text(
                cx,
                ry,
                preview_w as u16,
                &finding.preview,
                Style::default().fg(th().ui.accent).bg(bg),
            );
            cx += preview_w as u16 + 1;
            let path = if finding.more_paths > 0 {
                s::finding_files(u64::from(finding.more_paths), &finding.path)
            } else {
                finding.path.clone()
            };
            ui.text(
                cx,
                ry,
                band.right().saturating_sub(cx + 1),
                &path,
                Style::default().fg(th().ui.muted).bg(bg),
            );
            ui.fine.push((band, hit));
        }
        if detail_h > 0 {
            let f = &findings[cursor];
            let dy = list_end;
            let path = if f.more_paths > 0 {
                s::finding_files(u64::from(f.more_paths), &f.path)
            } else {
                f.path.clone()
            };
            ui.text(x, dy, w, &path, bold(false));
            ui.text(
                x,
                dy + 1,
                w,
                &s::detail_span(short(&f.first_commit), short(&f.last_commit)),
                dim(),
            );
            ui.text(x, dy + 2, w, &s::detail_counts(f.commits, f.blobs), dim());
        }
        // Actions.
        let n = self.ticked_findings();
        let remove = if n == 0 {
            s::BTN_REMOVE.get().to_string()
        } else {
            s::btn_remove_n(n)
        };
        let mut ax = x;
        ax += ui.button(
            ax,
            actions_y,
            &remove,
            RewriteHit::SecretsRemove,
            n > 0,
            n > 0 && !self.busy(),
        ) + 1;
        ax += ui.button(
            ax,
            actions_y,
            t::PROTECTED_ALL.get(),
            RewriteHit::SecretsAll,
            false,
            true,
        ) + 1;
        ui.button(
            ax,
            actions_y,
            t::PROTECTED_NONE.get(),
            RewriteHit::SecretsNone,
            false,
            true,
        );
    }

    /// The tab a click on the Secrets hits belongs to.
    pub(super) fn on_secrets_press(&mut self, hit: RewriteHit) {
        self.focus = Focus::Preview;
        self.tab = PreviewTab::Secrets;
        match hit {
            RewriteHit::SecretsScan => self.start_scan(),
            RewriteHit::SecretsRemove => self.remove_findings(),
            RewriteHit::SecretsAll => self.check_all_findings(),
            RewriteHit::SecretsNone => self.uncheck_all_findings(),
            RewriteHit::SecretRow(i) => {
                self.secrets.cursor = i;
                self.toggle_finding(i);
            }
            _ => {}
        }
    }
}
