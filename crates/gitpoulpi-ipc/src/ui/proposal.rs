//! Propose intents (see the mcp-show-ui design section 11):
//! what an agent suggests, and how the window reports what the user decided.

use gitpoulpi_core::domain::rewrite::{Recipe, RuleKind};
use gitpoulpi_core::domain::{ConflictPicks, Oid, SplitPart};
use serde::{Deserialize, Serialize};

use super::{sanitize_with_limit, DISPLAY_LIMIT};

/// What a `Propose` intent suggests (plan section 11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "what", rename_all = "snake_case")]
pub enum Proposal {
    /// The merge editor open on `path` with the agent's resolution in it
    /// (`picks`, or `content` as the whole Output), for the user to review
    /// and confirm with Save & Mark Resolved. Refused when the stage oids
    /// are no longer `expected`.
    ConflictResolution {
        path: String,
        /// The stage oids (base, ours, theirs) the agent decided against.
        expected: [Option<Oid>; 3],
        /// One choice per conflict, in file order.
        #[serde(default)]
        picks: Option<Vec<ConflictPicks>>,
        /// The whole file as it should read.
        #[serde(default)]
        content: Option<String>,
    },
    /// The Reword popup over `commit`, its fields set to `message`.
    Reword { commit: Oid, message: String },
    /// The split-commit editor over `commit`, its parts filled from `parts`
    /// (oldest first). The user reviews, edits and applies it; nothing is
    /// split until then.
    SplitCommit { commit: Oid, parts: Vec<SplitPart> },
    /// The history rewrite workbench open with this recipe in it (S25).
    /// The user previews and applies; nothing runs, is purged or pushed
    /// until then. Never carries a script rule (see
    /// [`Proposal::problem`]).
    Rewrite { recipe: Box<Recipe> },
    /// The commit composer's Summary and Description, filled.
    CommitMessage {
        summary: String,
        #[serde(default)]
        body: String,
    },
    /// The New Branch box on `start_point`'s row, named `name`.
    Branch { name: String, start_point: Oid },
    /// The New Tag box on `target`'s row; with a message, the annotated
    /// two-step box with its message step pre-filled.
    Tag {
        name: String,
        target: Oid,
        #[serde(default)]
        message: Option<String>,
    },
    /// Run the sequence `id` (see `gitpoulpi_config::sequences`) in this
    /// repository: the window shows "Proposed by <client>: run sequence
    /// <name>" with Run and Dismiss, and nothing runs until the user presses
    /// Run. Only a sequence the user can run themselves (valid, and trusted
    /// when it comes from the repository) is offered.
    SequenceRun { id: String },
}

impl Proposal {
    /// The snake_case name of the variant (`ui state`'s `kind`).
    pub fn kind(&self) -> &'static str {
        match self {
            Proposal::ConflictResolution { .. } => "conflict_resolution",
            Proposal::Reword { .. } => "reword",
            Proposal::SplitCommit { .. } => "split_commit",
            Proposal::Rewrite { .. } => "rewrite",
            Proposal::CommitMessage { .. } => "commit_message",
            Proposal::Branch { .. } => "branch",
            Proposal::Tag { .. } => "tag",
            Proposal::SequenceRun { .. } => "sequence_run",
        }
    }

    /// Why this proposal cannot be shown, when it cannot. A rewrite
    /// proposal carries at most [`REWRITE_RULES`] rules and no script: a
    /// script runs code when previewed, so the user adds it themselves.
    pub fn problem(&self) -> Option<String> {
        let Proposal::Rewrite { recipe } = self else {
            return None;
        };
        if recipe.rules.is_empty() {
            return Some("a rewrite proposal needs at least one rule".to_string());
        }
        if recipe.rules.len() > REWRITE_RULES {
            return Some(format!(
                "a rewrite proposal has at most {REWRITE_RULES} rules"
            ));
        }
        if recipe
            .rules
            .iter()
            .any(|r| matches!(r.kind, RuleKind::Script(_)))
        {
            return Some(
                "a rewrite proposal cannot carry a script: the user adds scripts themselves"
                    .to_string(),
            );
        }
        None
    }

    /// Strips controls and bidi characters and caps every text field;
    /// multi-line fields keep their newlines.
    pub fn sanitize(&mut self) {
        match self {
            Proposal::ConflictResolution { path, content, .. } => {
                *path = sanitize_with_limit(path, DISPLAY_LIMIT);
                if let Some(text) = content {
                    *text = sanitize_file_text(text);
                }
            }
            Proposal::Reword { message, .. } => *message = sanitize_message(message),
            Proposal::SplitCommit { parts, .. } => {
                for part in parts {
                    part.message = sanitize_message(&part.message);
                }
            }
            Proposal::Rewrite { recipe } => {
                recipe.name = sanitize_with_limit(&recipe.name, DISPLAY_LIMIT);
                for rule in &mut recipe.rules {
                    if let Some(label) = &mut rule.label {
                        *label = sanitize_with_limit(label, DISPLAY_LIMIT);
                    }
                }
                // What the daemon fills in itself; whatever came is dropped.
                recipe.script_policy = Default::default();
            }
            Proposal::CommitMessage { summary, body } => {
                *summary = sanitize_with_limit(summary, DISPLAY_LIMIT);
                *body = sanitize_message(body);
            }
            Proposal::Branch { name, .. } => *name = sanitize_name(name),
            Proposal::SequenceRun { id } => *id = sanitize_name(id),
            Proposal::Tag { name, message, .. } => {
                *name = sanitize_name(name);
                if let Some(m) = message {
                    *m = sanitize_message(m);
                }
            }
        }
    }

    /// The text the user is asked to confirm, as one string: what "edited"
    /// is measured against (`summary\n\nbody`, `name\n\nmessage`).
    pub fn text(&self) -> String {
        let join = |a: &str, b: &str| {
            if b.trim().is_empty() {
                a.trim_end().to_string()
            } else {
                format!("{}\n\n{}", a.trim_end(), b.trim_matches('\n').trim_end())
            }
        };
        match self {
            Proposal::ConflictResolution { content, .. } => content.clone().unwrap_or_default(),
            Proposal::Reword { message, .. } => {
                let (summary, body) = message.split_once('\n').unwrap_or((message, ""));
                join(summary, body.trim_start_matches('\n'))
            }
            Proposal::SplitCommit { parts, .. } => parts
                .iter()
                .map(|p| p.message.trim_end())
                .collect::<Vec<_>>()
                .join("\n\n"),
            Proposal::Rewrite { recipe } => serde_json::to_string(recipe).unwrap_or_default(),
            Proposal::CommitMessage { summary, body } => join(summary, body),
            Proposal::Branch { name, .. } => name.clone(),
            Proposal::SequenceRun { id } => id.clone(),
            Proposal::Tag { name, message, .. } => join(name, message.as_deref().unwrap_or("")),
        }
    }
}

/// A branch or tag name: one line, no spaces at its ends, capped.
fn sanitize_name(name: &str) -> String {
    sanitize_with_limit(name, DISPLAY_LIMIT).trim().to_string()
}

/// The most rules a rewrite proposal carries.
pub const REWRITE_RULES: usize = 50;

/// The longest multi-line proposal text (a commit or tag message).
pub const MESSAGE_LIMIT: usize = 5_000;

/// [`sanitize_display`] for multi-line text: newlines are kept (`\r\n` and
/// `\r` become `\n`), tabs become a space, every other control and the
/// bidi characters are removed, and the result is capped at
/// [`MESSAGE_LIMIT`] characters.
pub fn sanitize_message(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .chars()
        .filter_map(|c| match c {
            '\n' => Some('\n'),
            '\t' => Some(' '),
            c if c.is_control() => None,
            '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => None,
            '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{FEFF}' => None,
            c => Some(c),
        })
        .take(MESSAGE_LIMIT)
        .collect()
}

/// The longest file content a conflict proposal may carry (1 MiB).
pub const FILE_TEXT_LIMIT: usize = 1024 * 1024;

/// [`sanitize_message`] for the content of a file: tabs are kept, line
/// breaks become `\n`, every other control character and the bidi
/// overrides are removed, and the result is capped at [`FILE_TEXT_LIMIT`]
/// bytes (on a character boundary).
pub fn sanitize_file_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len().min(FILE_TEXT_LIMIT));
    for c in text.replace("\r\n", "\n").replace('\r', "\n").chars() {
        let keep = match c {
            '\n' | '\t' => true,
            c if c.is_control() => false,
            '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => false,
            '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{FEFF}' => false,
            _ => true,
        };
        if !keep {
            continue;
        }
        if out.len() + c.len_utf8() > FILE_TEXT_LIMIT {
            break;
        }
        out.push(c);
    }
    out
}

/// Where a proposal stands (`ui state`'s `proposals[].status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    /// On the chip (deferred), waiting for the user's Show.
    Offered,
    /// The dialog is open, pre-filled.
    Open,
    /// The user confirmed it unchanged.
    Accepted,
    /// The user changed the text, then confirmed.
    AcceptedEdited,
    /// The user canceled the dialog, dismissed the chip, emptied the
    /// composer or left the tab.
    Rejected,
    /// The chip expired, or another intent replaced it.
    Expired,
    /// The dialog could not open.
    Failed,
}

impl ProposalStatus {
    /// Whether the user has decided (nothing more will happen to it).
    pub fn is_final(self) -> bool {
        !matches!(self, ProposalStatus::Offered | ProposalStatus::Open)
    }
}

/// One proposal as `ui state` lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRecord {
    pub id: String,
    /// `reword | commit_message | branch | tag | conflict_resolution`
    pub kind: String,
    pub client: String,
    pub status: ProposalStatus,
    /// Unix seconds of the last status change.
    #[serde(default)]
    pub at: i64,
}

#[cfg(test)]
mod tests {
    use super::super::{UiIntent, UiOutcome, UiState};
    use super::*;
    use gitpoulpi_core::domain::RepoId;

    fn repo() -> RepoId {
        RepoId("/r".into())
    }
    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn every_proposal() -> Vec<Proposal> {
        vec![
            Proposal::Reword {
                commit: oid(1),
                message: "a\n\nb".into(),
            },
            Proposal::CommitMessage {
                summary: "s".into(),
                body: "b".into(),
            },
            Proposal::Branch {
                name: "feat/x".into(),
                start_point: oid(2),
            },
            Proposal::Tag {
                name: "v1".into(),
                target: oid(3),
                message: Some("release".into()),
            },
            Proposal::ConflictResolution {
                path: "src/a.rs".into(),
                expected: [Some(oid(1)), Some(oid(2)), None],
                picks: Some(vec![ConflictPicks::default()]),
                content: None,
            },
            Proposal::SequenceRun {
                id: "backup".into(),
            },
        ]
    }

    #[test]
    fn every_proposal_round_trips_inside_a_propose_intent() {
        let all = every_proposal();
        let kinds: std::collections::HashSet<_> = all.iter().map(|p| p.kind()).collect();
        assert_eq!(kinds.len(), 6);
        for proposal in all {
            let intent = UiIntent::Propose {
                repo: repo(),
                id: "p-7".into(),
                proposal,
            };
            let json = serde_json::to_value(&intent).unwrap();
            assert_eq!(json["kind"], "propose");
            assert!(json["proposal"]["what"].is_string(), "{json}");
            let back: UiIntent = serde_json::from_value(json).unwrap();
            assert_eq!(back, intent);
        }
        // Optional fields default.
        let p: Proposal =
            serde_json::from_value(serde_json::json!({"what": "commit_message", "summary": "s"}))
                .unwrap();
        assert_eq!(
            p,
            Proposal::CommitMessage {
                summary: "s".into(),
                body: String::new()
            }
        );
    }

    #[test]
    fn a_rewrite_proposal_has_rules_and_never_a_script() {
        use gitpoulpi_core::domain::rewrite::{
            Recipe, Rule, RuleKind, Scope, ScriptLanguage, ScriptRef, ScriptSource,
        };
        let with = |rules: Vec<Rule>| Proposal::Rewrite {
            recipe: Box::new(Recipe::new("r", Scope::default(), rules)),
        };
        let fine = Rule::new(RuleKind::MessageEncoding { names: false });
        assert_eq!(with(vec![fine.clone()]).problem(), None);
        assert_eq!(with(vec![fine.clone()]).kind(), "rewrite");
        assert!(with(Vec::new()).problem().is_some());
        assert!(with(vec![fine.clone(); REWRITE_RULES + 1])
            .problem()
            .is_some());
        let script = Rule::new(RuleKind::Script(ScriptRef {
            language: ScriptLanguage::Lua,
            source: ScriptSource::Inline { text: "x".into() },
            params: Default::default(),
        }));
        assert!(with(vec![fine, script])
            .problem()
            .unwrap()
            .contains("script"));
        // Only a rewrite proposal can be refused.
        assert_eq!(every_proposal().iter().find_map(Proposal::problem), None);
        // The recipe's text fields are cleaned and the daemon's policy reset.
        let mut p = with(vec![Rule::new(RuleKind::RemoveSignatures)]);
        if let Proposal::Rewrite { recipe } = &mut p {
            recipe.name = "n\u{1b}[0m\u{202e}x".into();
            recipe.rules[0].label = Some("l\u{7}".into());
        }
        p.sanitize();
        let Proposal::Rewrite { recipe } = p else {
            unreachable!()
        };
        assert_eq!(recipe.name, "n[0mx");
        assert_eq!(recipe.rules[0].label.as_deref(), Some("l"));
    }

    #[test]
    fn proposed_outcome_and_records_round_trip() {
        let mut state = UiState::default();
        state.proposals.push(ProposalRecord {
            id: "p-1".into(),
            kind: "reword".into(),
            client: "claude-code".into(),
            status: ProposalStatus::AcceptedEdited,
            at: 5,
        });
        let out = UiOutcome::Proposed {
            id: "p-1".into(),
            state,
        };
        assert_eq!(out.exit_code(), 0);
        let json = serde_json::to_value(&out).unwrap();
        assert_eq!(json["outcome"], "proposed");
        assert_eq!(json["state"]["proposals"][0]["status"], "accepted_edited");
        let back: UiOutcome = serde_json::from_value(json).unwrap();
        assert_eq!(back, out);
        // A state without the field (an older window) still parses.
        let old: UiState = serde_json::from_value(serde_json::json!({"repo": "/r"})).unwrap();
        assert!(old.proposals.is_empty());
        assert!(!ProposalStatus::Open.is_final() && ProposalStatus::Rejected.is_final());
    }

    #[test]
    fn proposals_are_sanitized_keeping_message_newlines() {
        let mut intent = UiIntent::Propose {
            repo: repo(),
            id: "p-\u{1b}1".into(),
            proposal: Proposal::Tag {
                name: "  v1\u{1b}[2J ".into(),
                target: oid(1),
                message: Some("line1\r\nline2\u{202e}\tx\u{7}".into()),
            },
        };
        intent.sanitize();
        let UiIntent::Propose { id, proposal, .. } = intent else {
            unreachable!()
        };
        assert_eq!(id, "p-1");
        assert_eq!(
            proposal,
            Proposal::Tag {
                name: "v1[2J".into(),
                target: oid(1),
                message: Some("line1\nline2 x".into()),
            }
        );
        assert_eq!(sanitize_message(&"x".repeat(9000)).len(), MESSAGE_LIMIT);
        let mut cm = Proposal::CommitMessage {
            summary: "a\nb".into(),
            body: "c\n\nd".into(),
        };
        cm.sanitize();
        assert_eq!(cm.text(), "a b\n\nc\n\nd");
    }

    #[test]
    fn file_text_keeps_tabs_and_lines_and_drops_escapes_and_caps() {
        assert_eq!(
            sanitize_file_text("a\tb\r\nc\u{1b}[2J\u{202e}d\r"),
            "a\tb\nc[2Jd\n"
        );
        assert_eq!(
            sanitize_file_text(&"é".repeat(FILE_TEXT_LIMIT)).len(),
            FILE_TEXT_LIMIT
        );
        let mut p = Proposal::ConflictResolution {
            path: "a\u{1b}b".into(),
            expected: [None; 3],
            picks: None,
            content: Some("x\ty\u{7}".into()),
        };
        p.sanitize();
        assert_eq!(p.text(), "x\ty");
        let Proposal::ConflictResolution { path, .. } = p else {
            unreachable!()
        };
        assert_eq!(path, "ab");
    }

    #[test]
    fn proposal_text_joins_like_the_dialogs() {
        let reword = Proposal::Reword {
            commit: oid(1),
            message: "subject\n\nbody line\n".into(),
        };
        assert_eq!(reword.text(), "subject\n\nbody line");
        let tag = Proposal::Tag {
            name: "v1".into(),
            target: oid(1),
            message: None,
        };
        assert_eq!(tag.text(), "v1");
    }
}
