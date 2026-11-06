//! What leaves the machine in an AI request, and what is taken out first.
//!
//! [`prepare`] is the one place a request is cleaned: it drops the diff of
//! every sensitive file (a private key, a `.env`) and replaces every
//! recognised secret with `[REDACTED:<rule>]`. [`PrivacyClient`] wraps every
//! client [`crate::client_for_in`] builds, so assist, review, conflict,
//! split, rewrite, the cloud clients and the local agent all go through it,
//! and a frontend can show the prepared text before anything is sent
//! ([`SendGate`]) or print it instead of sending ([`Privacy::print_only`]).
//!
//! **Limit.** A local agent that works in the repository folder can read
//! files itself: nothing here can stop that. Its prompt is cleaned like any
//! other, but a profile whose working folder is `dedicated` is the way to
//! keep it out of the repository.

use std::sync::{Arc, Mutex};

use gitpoulpi_config::AiPrivacy;
use gitpoulpi_core::privacy::{self, Redaction};

use crate::{AiClient, Error, Result};

/// A request after cleaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub system: String,
    pub user: String,
    /// What was replaced: a rule and a masked preview each, never a value.
    pub redactions: Vec<Redaction>,
    /// The paths whose diff was left out.
    pub excluded_files: Vec<String>,
}

/// Asked before a prepared request is sent. `false` cancels it.
#[async_trait::async_trait]
pub trait SendGate: Send + Sync {
    async fn confirm(&self, sent: &Prepared) -> bool;
}

/// What a run did to its requests, for the CLI/MCP `privacy` object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrivacySummary {
    pub redactions: usize,
    pub excluded_files: Vec<String>,
}

#[derive(Debug, Default)]
struct Record {
    redactions: Vec<Redaction>,
    excluded_files: Vec<String>,
    sent: Vec<(String, String)>,
}

/// Shared by every clone of a [`Privacy`] (and so of a job): what the
/// requests of one run were cleaned of.
#[derive(Debug, Default)]
pub struct Report(Mutex<Record>);

impl Report {
    fn record(&self, p: &Prepared) {
        let mut r = self.0.lock().unwrap_or_else(|e| e.into_inner());
        // A retry sends the same text again: keep the larger count, not the sum.
        if p.redactions.len() >= r.redactions.len() {
            r.redactions = p.redactions.clone();
        }
        for f in &p.excluded_files {
            if !r.excluded_files.contains(f) {
                r.excluded_files.push(f.clone());
            }
        }
        r.sent.push((p.system.clone(), p.user.clone()));
    }

    pub fn summary(&self) -> PrivacySummary {
        let r = self.0.lock().unwrap_or_else(|e| e.into_inner());
        PrivacySummary {
            redactions: r.redactions.len(),
            excluded_files: r.excluded_files.clone(),
        }
    }

    /// The masked previews of the redactions (`ghp_****abcd`), by rule.
    pub fn redactions(&self) -> Vec<Redaction> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .redactions
            .clone()
    }

    /// The prepared `(system, user)` of every request, in order.
    pub fn sent(&self) -> Vec<(String, String)> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .sent
            .clone()
    }
}

/// The settings of one run's cleaning. `Default` is the safe setting.
#[derive(Clone)]
pub struct Privacy {
    pub redact_secrets: bool,
    pub exclude_sensitive_files: bool,
    /// `[privacy] sensitive_patterns`.
    pub patterns: Vec<String>,
    /// Record the prepared request and fail with [`Error::PrintOnly`]
    /// instead of sending it (`--print-sent`).
    pub print_only: bool,
    /// Asked before the first request of a run goes out.
    pub gate: Option<Arc<dyn SendGate>>,
    report: Arc<Report>,
    /// The gate said yes once for this run (shared by every clone, so a retry
    /// of the same request is not asked again).
    approved: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for Privacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Privacy")
            .field("redact_secrets", &self.redact_secrets)
            .field("exclude_sensitive_files", &self.exclude_sensitive_files)
            .field("patterns", &self.patterns.len())
            .field("print_only", &self.print_only)
            .field("gate", &self.gate.is_some())
            .finish()
    }
}

impl Default for Privacy {
    fn default() -> Self {
        Self::from_config(
            &AiPrivacy::default(),
            &privacy::default_sensitive_patterns(),
        )
    }
}

impl Privacy {
    pub fn from_config(ai: &AiPrivacy, patterns: &[String]) -> Self {
        Self {
            redact_secrets: ai.redact_secrets,
            exclude_sensitive_files: ai.exclude_sensitive_files,
            patterns: patterns.to_vec(),
            print_only: false,
            gate: None,
            report: Arc::default(),
            approved: Arc::default(),
        }
    }

    /// What this run's requests were cleaned of.
    pub fn report(&self) -> Arc<Report> {
        Arc::clone(&self.report)
    }
}

/// One piece of a prompt: a per-file diff (with its path) or plain text.
struct Section {
    path: Option<String>,
    text: String,
}

const STATUSES: [&str; 6] = [
    " (modified)",
    " (new file)",
    " (new file (untracked))",
    " (deleted file)",
    " (renamed)",
    " (conflicted)",
];

/// The path a per-file header line names, when it is one: `diff -- <path>
/// (<status>)`, `diff --git a/<p> b/<p>`, or a split piece's `### H1 <path>
/// hunk ...`.
fn header_path(line: &str) -> Option<String> {
    let line = line.trim_end();
    if let Some(rest) = line.strip_prefix("diff --git ") {
        let (_, b) = rest.rsplit_once(" b/")?;
        return Some(b.to_string());
    }
    if let Some(rest) = line.strip_prefix("diff -- ") {
        let path = STATUSES
            .iter()
            .find_map(|s| rest.strip_suffix(s))
            .unwrap_or(rest);
        return Some(path.to_string());
    }
    if let Some(rest) = line.strip_prefix("### ") {
        let (id, rest) = rest.split_once(' ')?;
        let split_id = id.len() > 1
            && matches!(id.as_bytes()[0], b'H' | b'F')
            && id[1..].bytes().all(|b| b.is_ascii_digit());
        if !split_id {
            return None;
        }
        let path = match rest.split_once(" hunk ") {
            Some((p, _)) => p,
            None => rest.split_once(" (").map_or(rest, |(p, _)| p),
        };
        return Some(path.to_string());
    }
    None
}

/// Splits `text` into per-file sections. A section ends at the next header
/// or at a closing code fence, so the fence of a dropped diff stays.
fn sections(text: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    for line in text.split_inclusive('\n') {
        if let Some(path) = header_path(line) {
            out.push(Section {
                path: Some(path),
                text: line.to_string(),
            });
            continue;
        }
        // The closing fence is plain text after a diff section.
        let ends_diff = line.trim_end() == "```" && out.last().is_some_and(|s| s.path.is_some());
        match out.last_mut() {
            Some(last) if !ends_diff => last.text.push_str(line),
            _ => out.push(Section {
                path: None,
                text: line.to_string(),
            }),
        }
    }
    out
}

/// Cleans one request: drops the diff of sensitive files, then redacts.
pub fn prepare(system: &str, user: &str, settings: &Privacy) -> Prepared {
    let mut excluded: Vec<String> = Vec::new();
    let mut redactions: Vec<Redaction> = Vec::new();
    let mut clean = |text: &str, per_file: bool| -> String {
        let mut out = String::with_capacity(text.len());
        for s in sections(text) {
            if per_file && settings.exclude_sensitive_files {
                if let Some(p) = &s.path {
                    if privacy::matching_pattern(p, &settings.patterns).is_some() {
                        if !excluded.contains(p) {
                            excluded.push(p.clone());
                        }
                        continue;
                    }
                }
            }
            if settings.redact_secrets {
                let (t, found) = privacy::redact_in(&s.text, s.path.as_deref().unwrap_or(""));
                redactions.extend(found);
                out.push_str(&t);
            } else {
                out.push_str(&s.text);
            }
        }
        out
    };
    let system = clean(system, false);
    let user = clean(user, true);
    Prepared {
        system,
        user,
        redactions,
        excluded_files: excluded,
    }
}

/// Stands in for the provider when nothing may be sent (`--print-sent`).
struct Nothing;

#[async_trait::async_trait]
impl AiClient for Nothing {
    async fn complete(&self, _: &str, _: &str) -> Result<String> {
        Err(Error::PrintOnly)
    }
}

/// Wraps a client: every request is cleaned, recorded, shown to the gate and
/// only then sent.
pub struct PrivacyClient {
    inner: Box<dyn AiClient>,
    privacy: Privacy,
}

impl PrivacyClient {
    pub fn new(inner: Box<dyn AiClient>, privacy: Privacy) -> Self {
        Self { inner, privacy }
    }

    /// A client that only records what it would send.
    pub fn print_only(privacy: Privacy) -> Self {
        let mut privacy = privacy;
        privacy.print_only = true;
        Self::new(Box::new(Nothing), privacy)
    }
}

#[async_trait::async_trait]
impl AiClient for PrivacyClient {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        use std::sync::atomic::Ordering;
        let prepared = prepare(system, user, &self.privacy);
        self.privacy.report.record(&prepared);
        if self.privacy.print_only {
            return Err(Error::PrintOnly);
        }
        if let Some(gate) = &self.privacy.gate {
            // One decision per run: a retry of the same request is not asked again.
            if !self.privacy.approved.load(Ordering::SeqCst) {
                if !gate.confirm(&prepared).await {
                    return Err(Error::Declined);
                }
                self.privacy.approved.store(true, Ordering::SeqCst);
            }
        }
        self.inner.complete(&prepared.system, &prepared.user).await
    }
}
