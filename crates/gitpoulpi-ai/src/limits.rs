//! The input caps of a request, and what happens when an input is over one.
//!
//! Every cap a user can reach is a setting under `[ai.limits]`
//! ([`gitpoulpi_config::AiLimits`]); this module turns them into one value
//! ([`Limits`]), describes an input that is over a cap ([`Cut`]) and decides
//! what to do about it ([`LimitPolicy::decide`]):
//!
//! * **Nothing silently cut.** With a [`LimitGate`] (the TUI) the user is
//!   asked: send everything, send it truncated, or cancel.
//! * Without a gate (the CLI, MCP and any run nobody is watching) an input
//!   over a cap is truncated like before, and every cut is recorded
//!   ([`LimitPolicy::truncated`]) so the caller can report it.
//!   `send_all` (`--send-all`) sends the whole input instead.
//!
//! Output sanity caps (`MAX_ANSWER_BYTES`), the 20 lines / 6 KB of text
//! around a conflict, and the counts of a history-rewrite sample (50 commits)
//! or its refs and authors (100) are not here: they are not an input the
//! user can reason about by size, or the user chooses them (the sample).

use std::sync::{Arc, Mutex};

use gitpoulpi_config::AiLimits;
use serde::Serialize;

use crate::{Error, Result};

/// Which input was cut or would be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Part {
    /// The diff of a commit message, explanation, stash name, pull request
    /// description or commit plan.
    Diff,
    /// The hunks of the commit a split plan is asked for.
    SplitHunks,
    /// The whole diff of a code review: files that did not fit are left out.
    ReviewDiff,
    /// One file's diff in a code review.
    ReviewFile,
    /// The description of the reviewed request.
    ReviewText,
    /// The comments that already exist on the reviewed request.
    ReviewThreads,
    /// One project context file of a review.
    ContextFile,
    /// The project context files of a review together.
    ContextTotal,
}

impl Part {
    /// The `ai.limits.*` key that raises this cap.
    pub fn setting(self) -> &'static str {
        match self {
            Part::Diff | Part::SplitHunks | Part::ReviewDiff => "ai.limits.diff_bytes",
            Part::ReviewFile => "ai.limits.review_file_bytes",
            Part::ReviewText => "ai.limits.review_text_bytes",
            Part::ReviewThreads => "ai.limits.review_threads_bytes",
            Part::ContextFile => "ai.limits.review_context_file_bytes",
            Part::ContextTotal => "ai.limits.review_context_bytes",
        }
    }
}

/// One input over its cap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cut {
    pub part: Part,
    /// What exactly: a file path, a context file's name, the left-out files;
    /// empty when the part says it all.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub detail: String,
    /// The input's size.
    pub bytes: usize,
    /// The cap it is over.
    pub limit: usize,
    /// The `ai.limits.*` key that raises the cap.
    pub setting: &'static str,
}

impl Cut {
    pub fn new(part: Part, detail: impl Into<String>, bytes: usize, limit: usize) -> Self {
        Self {
            part,
            detail: detail.into(),
            bytes,
            limit,
            setting: part.setting(),
        }
    }
}

/// `Some` when `bytes` is over `limit`.
pub fn over(part: Part, detail: &str, bytes: usize, limit: usize) -> Option<Cut> {
    (bytes > limit).then(|| Cut::new(part, detail, bytes, limit))
}

/// The caps of one run, in bytes (`usize::MAX`: no cap).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub diff: usize,
    pub review_file: usize,
    pub review_text: usize,
    pub review_threads: usize,
    pub context_file: usize,
    pub context_total: usize,
}

impl Limits {
    /// `[ai.limits]` with a hand-edited out-of-range value brought back.
    pub fn from_config(config: &AiLimits) -> Self {
        Self {
            diff: config.diff_bytes_clamped(),
            review_file: config.review_file_bytes_clamped(),
            review_text: config.review_text_bytes_clamped(),
            review_threads: config.review_threads_bytes_clamped(),
            context_file: config.review_context_file_bytes_clamped(),
            context_total: config.review_context_bytes_clamped(),
        }
    }

    /// No cap at all: "Send all".
    pub fn unlimited() -> Self {
        Self {
            diff: usize::MAX,
            review_file: usize::MAX,
            review_text: usize::MAX,
            review_threads: usize::MAX,
            context_file: usize::MAX,
            context_total: usize::MAX,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::from_config(&AiLimits::default())
    }
}

/// What the user chose when told an input is over a cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitAnswer {
    /// Send the whole input, this request only.
    SendAll,
    /// Send what fits, as before.
    SendTruncated,
    /// Send nothing (also what "Open setting" ends as: the frontend opens
    /// the setting, the request stops).
    Cancel,
}

/// Asked, before a request is built, when an input is over a cap.
#[async_trait::async_trait]
pub trait LimitGate: Send + Sync {
    async fn ask(&self, cuts: &[Cut]) -> LimitAnswer;
}

/// A run's caps, whom to ask when one is exceeded, and what was cut.
#[derive(Clone)]
pub struct LimitPolicy {
    pub limits: Limits,
    /// Asked once per decision; `None` means nobody can answer.
    pub gate: Option<Arc<dyn LimitGate>>,
    /// Nobody can answer and the caller wants everything sent (`--send-all`).
    pub send_all: bool,
    truncated: Arc<Mutex<Vec<Cut>>>,
}

impl std::fmt::Debug for LimitPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LimitPolicy")
            .field("limits", &self.limits)
            .field("gate", &self.gate.is_some())
            .field("send_all", &self.send_all)
            .finish()
    }
}

impl Default for LimitPolicy {
    fn default() -> Self {
        Self::new(Limits::default())
    }
}

impl LimitPolicy {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            gate: None,
            send_all: false,
            truncated: Arc::default(),
        }
    }

    /// What was truncated by the decisions made so far (shared by every clone
    /// of the job), for a frontend that cannot ask.
    pub fn truncated(&self) -> Vec<Cut> {
        self.truncated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn record(&self, found: Vec<Cut>) {
        let mut truncated = self.truncated.lock().unwrap_or_else(|e| e.into_inner());
        for cut in found {
            if !truncated.contains(&cut) {
                truncated.push(cut);
            }
        }
    }

    /// What to do about `found`, the inputs over their caps. `Ok(true)`: the
    /// caps are lifted for this request, build it whole. `Ok(false)`: build
    /// it with the caps (what is cut was recorded). `Err(Declined)`: the user
    /// chose Cancel.
    pub async fn decide(&self, found: Vec<Cut>) -> Result<bool> {
        if found.is_empty() {
            return Ok(false);
        }
        if self.send_all {
            return Ok(true);
        }
        let Some(gate) = &self.gate else {
            self.record(found);
            return Ok(false);
        };
        match gate.ask(&found).await {
            LimitAnswer::SendAll => Ok(true),
            LimitAnswer::SendTruncated => {
                self.record(found);
                Ok(false)
            }
            LimitAnswer::Cancel => Err(Error::Declined),
        }
    }

    /// The byte budget of a diff of `bytes` bytes: its cap, or no cap once
    /// the caps are lifted.
    pub async fn diff_budget(&self, part: Part, bytes: usize) -> Result<usize> {
        let limit = self.limits.diff;
        let found: Vec<Cut> = over(part, "", bytes, limit).into_iter().collect();
        Ok(if self.decide(found).await? {
            usize::MAX
        } else {
            limit
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Answers(LimitAnswer, AtomicUsize);

    #[async_trait::async_trait]
    impl LimitGate for Answers {
        async fn ask(&self, _: &[Cut]) -> LimitAnswer {
            self.1.fetch_add(1, Ordering::SeqCst);
            self.0
        }
    }

    fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn policy(answer: Option<LimitAnswer>) -> (LimitPolicy, Option<Arc<Answers>>) {
        let mut policy = LimitPolicy::default();
        policy.limits.diff = 100;
        let gate = answer.map(|a| Arc::new(Answers(a, AtomicUsize::new(0))));
        if let Some(g) = &gate {
            policy.gate = Some(g.clone());
        }
        (policy, gate)
    }

    #[test]
    fn under_the_cap_nothing_is_asked_or_recorded() {
        let (p, gate) = policy(Some(LimitAnswer::Cancel));
        assert_eq!(block_on(p.diff_budget(Part::Diff, 100)), Ok(100));
        assert_eq!(gate.unwrap().1.load(Ordering::SeqCst), 0);
        assert!(p.truncated().is_empty());
    }

    #[test]
    fn the_gate_decides_over_the_cap() {
        let (p, gate) = policy(Some(LimitAnswer::SendAll));
        assert_eq!(block_on(p.diff_budget(Part::Diff, 101)), Ok(usize::MAX));
        assert_eq!(gate.unwrap().1.load(Ordering::SeqCst), 1);
        assert!(p.truncated().is_empty(), "send all cuts nothing");

        let (p, _) = policy(Some(LimitAnswer::SendTruncated));
        assert_eq!(block_on(p.diff_budget(Part::Diff, 101)), Ok(100));
        assert_eq!(
            p.truncated(),
            [Cut::new(Part::Diff, "", 101, 100)],
            "the cut is recorded"
        );

        let (p, _) = policy(Some(LimitAnswer::Cancel));
        assert_eq!(
            block_on(p.diff_budget(Part::Diff, 101)),
            Err(Error::Declined)
        );
        assert!(p.truncated().is_empty());
    }

    #[test]
    fn nobody_to_ask_truncates_and_records_unless_send_all() {
        let (p, _) = policy(None);
        assert_eq!(block_on(p.diff_budget(Part::Diff, 500)), Ok(100));
        assert_eq!(p.truncated(), [Cut::new(Part::Diff, "", 500, 100)]);
        // The same cut twice is reported once.
        assert_eq!(block_on(p.diff_budget(Part::Diff, 500)), Ok(100));
        assert_eq!(p.truncated().len(), 1);

        let (mut p, _) = policy(None);
        p.send_all = true;
        assert_eq!(block_on(p.diff_budget(Part::Diff, 500)), Ok(usize::MAX));
        assert!(p.truncated().is_empty());
    }

    #[test]
    fn a_cut_names_the_setting_that_raises_its_cap() {
        for (part, key) in [
            (Part::Diff, "ai.limits.diff_bytes"),
            (Part::SplitHunks, "ai.limits.diff_bytes"),
            (Part::ReviewDiff, "ai.limits.diff_bytes"),
            (Part::ReviewFile, "ai.limits.review_file_bytes"),
            (Part::ReviewText, "ai.limits.review_text_bytes"),
            (Part::ReviewThreads, "ai.limits.review_threads_bytes"),
            (Part::ContextFile, "ai.limits.review_context_file_bytes"),
            (Part::ContextTotal, "ai.limits.review_context_bytes"),
        ] {
            assert_eq!(Cut::new(part, "", 2, 1).setting, key);
            assert!(AiLimits::KEYS.contains(&key));
        }
        let json = serde_json::to_value(Cut::new(Part::ReviewFile, "src/a.rs", 5, 3)).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "part": "review_file", "detail": "src/a.rs", "bytes": 5, "limit": 3,
                "setting": "ai.limits.review_file_bytes"
            })
        );
    }

    #[test]
    fn the_default_caps_are_todays_values() {
        let l = Limits::default();
        assert_eq!(
            (l.diff, l.review_file, l.review_text, l.review_threads),
            (60_000, 12_000, 4_000, 6_000)
        );
        assert_eq!((l.context_file, l.context_total), (8_192, 16_384));
    }
}
