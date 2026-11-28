//! The rules list of the workbench: what the Add menu offers, the list
//! operations (add, toggle, reorder, delete with restore, duplicate), one
//! line per rule, and the check that says whether the recipe can run.
//!
//! The operations work on the recipe's own `Vec<Rule>`, so what the list
//! shows is what Save writes and Preview sends. Order matters (rules run top
//! to bottom), hence the numbered rows and the reorder keys.

use gitpoulpi_core::domain::rewrite::{
    validate_recipe, ContentFind, PathMove, Recipe, RecipeIssue, RewriteRefusal, Rule, RuleKind,
    Tool, ToolRequirement,
};
use gitpoulpi_i18n::{msg::rewrite_view as t, M};

use super::rules_form::{
    blank_anonymise, blank_content, blank_identity, blank_lfs_export, blank_lfs_import,
    blank_message_replace, blank_script, blank_trailer, blank_window, format_duration,
    format_offset, format_size,
};

/// The groups of the Add menu, in menu order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuleGroup {
    Authors,
    Dates,
    Messages,
    Content,
    Paths,
    Lfs,
    Script,
    Signatures,
}

impl RuleGroup {
    pub(crate) const ALL: [RuleGroup; 8] = [
        RuleGroup::Authors,
        RuleGroup::Dates,
        RuleGroup::Messages,
        RuleGroup::Content,
        RuleGroup::Paths,
        RuleGroup::Lfs,
        RuleGroup::Script,
        RuleGroup::Signatures,
    ];

    pub(crate) const fn label(self) -> M {
        match self {
            RuleGroup::Authors => t::G_AUTHORS,
            RuleGroup::Dates => t::G_DATES,
            RuleGroup::Messages => t::G_MESSAGES,
            RuleGroup::Content => t::G_CONTENT,
            RuleGroup::Paths => t::G_PATHS,
            RuleGroup::Lfs => t::G_LFS,
            RuleGroup::Script => t::G_SCRIPT,
            RuleGroup::Signatures => t::G_SIGNATURES,
        }
    }
}

pub(crate) use super::vocab::RuleTemplate;

impl RuleTemplate {
    pub(crate) const ALL: [RuleTemplate; 22] = [
        RuleTemplate::Identity,
        RuleTemplate::Mailmap,
        RuleTemplate::Anonymise,
        RuleTemplate::DateShift,
        RuleTemplate::DateTimezone,
        RuleTemplate::DateWindow,
        RuleTemplate::CommitterFromAuthor,
        RuleTemplate::MessageReplace,
        RuleTemplate::MessageDeleteLines,
        RuleTemplate::Trailer,
        RuleTemplate::ContentReplace,
        RuleTemplate::RemovePaths,
        RuleTemplate::RemoveBlobsOver,
        RuleTemplate::MovePaths,
        RuleTemplate::LfsImport,
        RuleTemplate::LfsExport,
        RuleTemplate::Script,
        RuleTemplate::RemoveSignatures,
        RuleTemplate::LineEndings,
        RuleTemplate::StripExec,
        RuleTemplate::Reencode,
        RuleTemplate::MessageEncoding,
    ];

    pub(crate) const fn group(self) -> RuleGroup {
        match self {
            Self::Identity | Self::Mailmap | Self::Anonymise => RuleGroup::Authors,
            Self::DateShift | Self::DateTimezone | Self::DateWindow | Self::CommitterFromAuthor => {
                RuleGroup::Dates
            }
            Self::MessageReplace
            | Self::MessageDeleteLines
            | Self::Trailer
            | Self::MessageEncoding => RuleGroup::Messages,
            Self::ContentReplace | Self::LineEndings | Self::StripExec | Self::Reencode => {
                RuleGroup::Content
            }
            Self::RemovePaths | Self::RemoveBlobsOver | Self::MovePaths => RuleGroup::Paths,
            Self::LfsImport | Self::LfsExport => RuleGroup::Lfs,
            Self::Script => RuleGroup::Script,
            Self::RemoveSignatures => RuleGroup::Signatures,
        }
    }

    pub(crate) const fn label(self) -> M {
        match self {
            Self::Identity => t::K_IDENTITY,
            Self::Mailmap => t::K_MAILMAP,
            Self::Anonymise => t::K_ANONYMISE,
            Self::DateShift => t::K_DATE_SHIFT,
            Self::DateTimezone => t::K_DATE_TIMEZONE,
            Self::DateWindow => t::K_DATE_WINDOW,
            Self::CommitterFromAuthor => t::K_COMMITTER_FROM_AUTHOR,
            Self::MessageReplace => t::K_MESSAGE_REPLACE,
            Self::MessageDeleteLines => t::K_MESSAGE_DELETE_LINES,
            Self::Trailer => t::K_TRAILER,
            Self::ContentReplace => t::K_CONTENT_REPLACE,
            Self::RemovePaths => t::K_REMOVE_PATHS,
            Self::RemoveBlobsOver => t::K_REMOVE_BLOBS,
            Self::MovePaths => t::K_MOVE_PATHS,
            Self::LfsImport => t::K_LFS_IMPORT,
            Self::LfsExport => t::K_LFS_EXPORT,
            Self::Script => t::K_SCRIPT,
            Self::RemoveSignatures => t::K_REMOVE_SIGNATURES,
            Self::LineEndings => t::K_LINE_ENDINGS,
            Self::StripExec => t::K_STRIP_EXEC,
            Self::Reencode => t::K_REENCODE,
            Self::MessageEncoding => t::K_MESSAGE_ENCODING,
        }
    }

    /// Why this kind cannot be added in this build, when it cannot: a Script
    /// rule needs the embedded Lua runtime.
    pub(crate) fn disabled_reason(self) -> Option<M> {
        match self {
            Self::Script if !super::script::scripts_built() => Some(t::SCRIPT_OFF_REASON),
            _ => None,
        }
    }

    /// The kinds of one group, in menu order.
    pub(crate) fn of_group(group: RuleGroup) -> Vec<RuleTemplate> {
        Self::ALL
            .into_iter()
            .filter(|k| k.group() == group)
            .collect()
    }

    /// A new rule of this kind. Starts filled with something sensible where
    /// the kind has a natural default, else blank (and then the recipe says
    /// what is missing until the fields are filled).
    pub(crate) fn new_rule(self) -> Rule {
        use gitpoulpi_core::domain::rewrite as r;
        Rule::new(match self {
            Self::Identity => blank_identity(),
            Self::Mailmap => RuleKind::Mailmap {
                apply_to: r::Who::Both,
                file: None,
            },
            Self::Anonymise => blank_anonymise(),
            Self::DateShift => RuleKind::DateShift {
                by_seconds: 3_600,
                apply_to: r::WhichDate::Both,
            },
            Self::DateTimezone => RuleKind::DateTimezone {
                offset_minutes: 0,
                keep: r::KeepClock::Instant,
                apply_to: r::WhichDate::Both,
            },
            Self::DateWindow => blank_window(),
            Self::CommitterFromAuthor => RuleKind::CommitterFromAuthor,
            Self::MessageReplace => blank_message_replace(),
            Self::MessageDeleteLines => RuleKind::MessageDeleteLines {
                pattern: r::TextPattern {
                    pattern: String::new(),
                    regex: false,
                    case_sensitive: true,
                },
            },
            Self::Trailer => blank_trailer(),
            Self::ContentReplace => blank_content(),
            Self::RemovePaths => RuleKind::RemovePaths { globs: Vec::new() },
            Self::RemoveBlobsOver => RuleKind::RemoveBlobsOver { bytes: 5 << 20 },
            Self::MovePaths => RuleKind::MovePaths(PathMove::Rename {
                from: String::new(),
                to: String::new(),
            }),
            Self::LfsImport => blank_lfs_import(),
            Self::LfsExport => blank_lfs_export(),
            Self::Script => blank_script(),
            Self::RemoveSignatures => RuleKind::RemoveSignatures,
            Self::LineEndings => super::rules_form::blank_line_endings(),
            Self::StripExec => super::rules_form::blank_strip_exec(),
            Self::Reencode => super::rules_form::blank_reencode(),
            Self::MessageEncoding => RuleKind::MessageEncoding { names: false },
        })
    }
}

/// The Add-menu template whose kind `kind` is.
pub(crate) fn template_of(kind: &RuleKind) -> RuleTemplate {
    match kind {
        RuleKind::Identity(_) => RuleTemplate::Identity,
        RuleKind::Mailmap { .. } => RuleTemplate::Mailmap,
        RuleKind::Anonymise(_) => RuleTemplate::Anonymise,
        RuleKind::DateShift { .. } => RuleTemplate::DateShift,
        RuleKind::DateTimezone { .. } => RuleTemplate::DateTimezone,
        RuleKind::DateWindow(_) => RuleTemplate::DateWindow,
        RuleKind::CommitterFromAuthor => RuleTemplate::CommitterFromAuthor,
        RuleKind::MessageReplace(_) => RuleTemplate::MessageReplace,
        RuleKind::MessageDeleteLines { .. } => RuleTemplate::MessageDeleteLines,
        RuleKind::Trailer(_) => RuleTemplate::Trailer,
        RuleKind::ContentReplace(_) => RuleTemplate::ContentReplace,
        RuleKind::RemovePaths { .. } => RuleTemplate::RemovePaths,
        RuleKind::RemoveBlobsOver { .. } => RuleTemplate::RemoveBlobsOver,
        RuleKind::MovePaths(_) => RuleTemplate::MovePaths,
        RuleKind::LfsImport(_) => RuleTemplate::LfsImport,
        RuleKind::LfsExport(_) => RuleTemplate::LfsExport,
        RuleKind::Script(_) => RuleTemplate::Script,
        RuleKind::RemoveSignatures => RuleTemplate::RemoveSignatures,
        RuleKind::LineEndings(_) => RuleTemplate::LineEndings,
        RuleKind::StripExec(_) => RuleTemplate::StripExec,
        RuleKind::Reencode(_) => RuleTemplate::Reencode,
        RuleKind::MessageEncoding { .. } => RuleTemplate::MessageEncoding,
    }
}

/// One row of the rules list: the kind's name, then what it is set to in
/// the user's own words (never translated: it is their data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleRow {
    pub number: usize,
    pub enabled: bool,
    pub label: M,
    /// The user's own name for the rule, when they gave one.
    pub custom: Option<String>,
    pub detail: String,
}

/// The one-line summary of a rule's settings.
pub(crate) fn detail_of(kind: &RuleKind) -> String {
    match kind {
        RuleKind::Identity(r) => {
            let from = r
                .when
                .email
                .clone()
                .or_else(|| r.when.name.clone())
                .unwrap_or_else(|| "*".into());
            let to = r
                .set
                .email
                .clone()
                .or_else(|| r.set.name.clone())
                .unwrap_or_default();
            format!("{from} -> {to}")
        }
        RuleKind::Mailmap { .. } => ".mailmap".into(),
        RuleKind::Anonymise(r) => r.name_pattern.clone(),
        RuleKind::DateShift { by_seconds, .. } => format_duration(*by_seconds),
        RuleKind::DateTimezone { offset_minutes, .. } => format_offset(*offset_minutes),
        RuleKind::DateWindow(w) => super::rules_form::format_windows(&w.windows),
        RuleKind::CommitterFromAuthor
        | RuleKind::RemoveSignatures
        | RuleKind::MessageEncoding { .. } => String::new(),
        RuleKind::MessageReplace(r) => {
            format!("\"{}\" -> \"{}\"", r.find.pattern, r.with)
        }
        RuleKind::MessageDeleteLines { pattern } => format!("\"{}\"", pattern.pattern),
        RuleKind::Trailer(r) => format!("{}: {}", r.key, r.value),
        RuleKind::ContentReplace(c) => match &c.find {
            ContentFind::Pattern { pattern } => {
                format!("\"{}\" -> \"{}\"", pattern.pattern, c.with)
            }
            // Never the values: the row only counts them.
            ContentFind::Secrets { literals, scan } => {
                format!(
                    "{}{}",
                    "*".repeat(literals.len().min(3)),
                    if *scan { " +" } else { "" }
                )
            }
        },
        RuleKind::RemovePaths { globs } => globs.join(", "),
        RuleKind::RemoveBlobsOver { bytes } => format!("> {}", format_size(*bytes)),
        RuleKind::MovePaths(PathMove::Rename { from, to }) => format!("{from} -> {to}"),
        RuleKind::MovePaths(PathMove::IntoSubdir { dir }) => format!("-> {dir}/"),
        RuleKind::MovePaths(PathMove::ExtractSubdir { dir, .. }) => format!("{dir}/ ->"),
        RuleKind::LfsImport(l) => match l.min_size {
            Some(min) => format!("{} (>= {})", l.patterns.join(", "), format_size(min)),
            None => l.patterns.join(", "),
        },
        RuleKind::LfsExport(l) => l.patterns.join(", "),
        RuleKind::LineEndings(e) => format!("{} {}", e.target.name(), e.paths.join(", ")),
        RuleKind::StripExec(e) => e.paths.join(", "),
        RuleKind::Reencode(e) => format!("{} {}", e.target.name(), e.paths.join(", ")),
        RuleKind::Script(s) => match &s.source {
            gitpoulpi_core::domain::rewrite::ScriptSource::File { path } => path.clone(),
            gitpoulpi_core::domain::rewrite::ScriptSource::Inline { text } => {
                text.lines().next().unwrap_or("").to_string()
            }
        },
    }
}

/// The rows of the list.
pub(crate) fn rows(rules: &[Rule]) -> Vec<RuleRow> {
    rules
        .iter()
        .enumerate()
        .map(|(i, rule)| RuleRow {
            number: i + 1,
            enabled: rule.enabled,
            label: template_of(&rule.kind).label(),
            custom: rule.label.clone().filter(|l| !l.trim().is_empty()),
            detail: detail_of(&rule.kind),
        })
        .collect()
}

/// The list's cursor and the rules removed since the workbench opened.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuleList {
    pub cursor: usize,
    /// Removed rules with the position they had, most recent last.
    removed: Vec<(usize, Rule)>,
}

impl RuleList {
    /// Keeps the cursor on a rule (or at 0 for an empty list).
    pub(crate) fn clamp(&mut self, rules: &[Rule]) {
        self.cursor = self.cursor.min(rules.len().saturating_sub(1));
    }

    pub(crate) fn move_cursor(&mut self, rules: &[Rule], delta: isize) {
        let last = rules.len().saturating_sub(1) as isize;
        self.cursor = (self.cursor as isize + delta).clamp(0, last) as usize;
    }

    /// Appends a rule of `template` and puts the cursor on it.
    pub(crate) fn add(&mut self, rules: &mut Vec<Rule>, template: RuleTemplate) -> usize {
        rules.push(template.new_rule());
        self.cursor = rules.len() - 1;
        self.cursor
    }

    /// Switches the rule under the cursor on or off.
    pub(crate) fn toggle(&mut self, rules: &mut [Rule]) -> bool {
        match rules.get_mut(self.cursor) {
            Some(rule) => {
                rule.enabled = !rule.enabled;
                true
            }
            None => false,
        }
    }

    /// Moves the rule under the cursor by `delta` places (the cursor
    /// follows). `false` at either end.
    pub(crate) fn move_rule(&mut self, rules: &mut [Rule], delta: isize) -> bool {
        let to = self.cursor as isize + delta;
        if rules.is_empty() || to < 0 || to as usize >= rules.len() {
            return false;
        }
        rules.swap(self.cursor, to as usize);
        self.cursor = to as usize;
        true
    }

    /// Removes the rule under the cursor, keeping it for [`Self::restore`].
    pub(crate) fn remove(&mut self, rules: &mut Vec<Rule>) -> bool {
        if self.cursor >= rules.len() {
            return false;
        }
        let rule = rules.remove(self.cursor);
        self.removed.push((self.cursor, rule));
        self.clamp(rules);
        true
    }

    /// Puts back the most recently removed rule where it was.
    pub(crate) fn restore(&mut self, rules: &mut Vec<Rule>) -> bool {
        let Some((at, rule)) = self.removed.pop() else {
            return false;
        };
        let at = at.min(rules.len());
        rules.insert(at, rule);
        self.cursor = at;
        true
    }

    /// Whether there is a removed rule to restore.
    pub(crate) fn can_restore(&self) -> bool {
        !self.removed.is_empty()
    }

    /// Copies the rule under the cursor right below itself.
    pub(crate) fn duplicate(&mut self, rules: &mut Vec<Rule>) -> bool {
        let Some(rule) = rules.get(self.cursor).cloned() else {
            return false;
        };
        rules.insert(self.cursor + 1, rule);
        self.cursor += 1;
        true
    }

    /// Forgets the removed rules (a recipe was loaded: they belonged to the
    /// previous one).
    pub(crate) fn reset(&mut self) {
        self.cursor = 0;
        self.removed.clear();
    }
}

// ------------------------------------------------------------ validation

/// What the footer says about the recipe as it stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Validation {
    Ok,
    /// An "extract a subdirectory" rule has no mode yet: a question, not an
    /// error (the answer is asked each time).
    NeedsExtractMode,
    /// The first problem: the rule it is about (`None` for the recipe) and
    /// the sentence.
    Invalid {
        rule: Option<usize>,
        text: String,
    },
}

impl Validation {
    pub(crate) fn is_ok(&self) -> bool {
        matches!(self, Validation::Ok)
    }
}

/// [`validate_recipe`], worded for the footer.
pub(crate) fn validation(recipe: &Recipe) -> Validation {
    match validate_recipe(recipe) {
        Ok(()) => match script_issue(recipe) {
            Some((rule, text)) => Validation::Invalid {
                rule: Some(rule),
                text,
            },
            None => Validation::Ok,
        },
        Err(RecipeIssue {
            refusal: RewriteRefusal::ExtractModeNeeded,
            ..
        }) => Validation::NeedsExtractMode,
        Err(issue) => Validation::Invalid {
            rule: issue.rule,
            text: secrets_text(recipe, &issue).unwrap_or_else(|| issue_text(&issue)),
        },
    }
}

/// The first enabled script rule whose header is unreadable or whose text
/// parameters have neither a value nor a default (what the rule form shows
/// under the field, said once more where Preview is refused).
fn script_issue(recipe: &Recipe) -> Option<(usize, String)> {
    use gitpoulpi_core::domain::rewrite::script::parse_header_for;
    use gitpoulpi_core::domain::rewrite::ScriptSource;
    recipe.rules.iter().enumerate().find_map(|(i, rule)| {
        let RuleKind::Script(s) = &rule.kind else {
            return None;
        };
        let ScriptSource::Inline { text } = &s.source else {
            return None;
        };
        if !rule.enabled {
            return None;
        }
        let reason = match parse_header_for(s.language, text) {
            Err(why) => why,
            Ok(header) => {
                let missing = header.missing_params(&s.params);
                if missing.is_empty() {
                    return None;
                }
                t::script_test_params_first(missing.join(", "))
            }
        };
        Some((i, t::issue_rule(i + 1, reason)))
    })
}

/// A secret rule without values: the core's sentence mentions the scanner,
/// which this form does not offer (the Secrets tab adds scanning rules).
fn secrets_text(recipe: &Recipe, issue: &RecipeIssue) -> Option<String> {
    let i = issue.rule?;
    match &recipe.rules.get(i)?.kind {
        RuleKind::ContentReplace(c) if matches!(&c.find, ContentFind::Secrets { literals, .. } if literals.is_empty()) => {
            Some(t::issue_rule(i + 1, t::E_SECRETS_EMPTY.get().to_string()))
        }
        _ => None,
    }
}

/// One issue as a sentence: the rule's number, then the reason. The reason is
/// the core's own words for what a rule field must look like; a refusal that
/// has a catalog sentence uses it.
pub(crate) fn issue_text(issue: &RecipeIssue) -> String {
    let reason = match issue.refusal {
        RewriteRefusal::NoRule | RewriteRefusal::EmptyScope => {
            crate::daemon_link::error_text::rewrite_refusal(issue.refusal)
                .get()
                .to_string()
        }
        _ => issue.detail.clone(),
    };
    match issue.rule {
        Some(i) => t::issue_rule(i + 1, reason),
        None => reason,
    }
}

/// The tools a rule needs that are not met, as the greyed-out reason.
pub(crate) fn rule_tools_unmet(
    rule: &Rule,
    tools: &gitpoulpi_core::domain::rewrite::RewriteTools,
) -> Vec<Tool> {
    let needs: Vec<ToolRequirement> = rule.kind.required_tools();
    tools
        .unmet(&needs)
        .iter()
        .flat_map(|r| r.candidates())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::Scope;

    fn rules(kinds: &[RuleTemplate]) -> Vec<Rule> {
        kinds.iter().map(|k| k.new_rule()).collect()
    }

    fn names(rules: &[Rule]) -> Vec<&'static str> {
        rules.iter().map(|r| r.kind.name()).collect()
    }

    #[test]
    fn adding_appends_and_puts_the_cursor_on_the_new_rule() {
        let mut list = RuleList::default();
        let mut r = Vec::new();
        list.add(&mut r, RuleTemplate::Identity);
        list.add(&mut r, RuleTemplate::DateShift);
        assert_eq!(names(&r), ["identity", "date_shift"]);
        assert_eq!(list.cursor, 1);
    }

    #[test]
    fn toggling_flips_only_the_rule_under_the_cursor() {
        let mut list = RuleList::default();
        let mut r = rules(&[RuleTemplate::Identity, RuleTemplate::DateShift]);
        list.cursor = 1;
        assert!(list.toggle(&mut r));
        assert!(r[0].enabled && !r[1].enabled);
        assert!(list.toggle(&mut r));
        assert!(r[1].enabled);
        assert!(!RuleList::default().toggle(&mut []));
    }

    #[test]
    fn reordering_moves_the_rule_and_the_cursor_and_stops_at_the_ends() {
        let mut list = RuleList::default();
        let mut r = rules(&[
            RuleTemplate::Identity,
            RuleTemplate::DateShift,
            RuleTemplate::Trailer,
        ]);
        assert!(!list.move_rule(&mut r, -1));
        assert!(list.move_rule(&mut r, 1));
        assert_eq!(names(&r), ["date_shift", "identity", "trailer"]);
        assert_eq!(list.cursor, 1);
        list.cursor = 2;
        assert!(!list.move_rule(&mut r, 1));
        assert!(list.move_rule(&mut r, -1));
        assert!(list.move_rule(&mut r, -1));
        assert_eq!(names(&r), ["trailer", "date_shift", "identity"]);
        assert_eq!(list.cursor, 0);
    }

    #[test]
    fn deleting_keeps_the_rule_and_restore_puts_it_back_where_it_was() {
        let mut list = RuleList::default();
        let mut r = rules(&[
            RuleTemplate::Identity,
            RuleTemplate::DateShift,
            RuleTemplate::Trailer,
        ]);
        list.cursor = 1;
        r[1].enabled = false;
        assert!(list.remove(&mut r));
        assert_eq!(names(&r), ["identity", "trailer"]);
        assert!(list.can_restore());
        // A second deletion stacks: restores come back newest first.
        list.cursor = 0;
        assert!(list.remove(&mut r));
        assert!(list.restore(&mut r));
        assert_eq!(names(&r), ["identity", "trailer"]);
        assert!(list.restore(&mut r));
        assert_eq!(names(&r), ["identity", "date_shift", "trailer"]);
        assert!(
            !r[1].enabled,
            "the restored rule is exactly the removed one"
        );
        assert_eq!(list.cursor, 1);
        assert!(!list.restore(&mut r));
    }

    #[test]
    fn deleting_the_last_rule_leaves_a_cursor_on_the_new_last() {
        let mut list = RuleList::default();
        let mut r = rules(&[RuleTemplate::Identity, RuleTemplate::DateShift]);
        list.cursor = 1;
        list.remove(&mut r);
        assert_eq!(list.cursor, 0);
        list.remove(&mut r);
        assert!(r.is_empty());
        assert!(!list.remove(&mut r));
    }

    #[test]
    fn duplicating_copies_below_and_moves_the_cursor_to_the_copy() {
        let mut list = RuleList::default();
        let mut r = rules(&[RuleTemplate::DateShift, RuleTemplate::Trailer]);
        assert!(list.duplicate(&mut r));
        assert_eq!(names(&r), ["date_shift", "date_shift", "trailer"]);
        assert_eq!(list.cursor, 1);
    }

    #[test]
    fn every_template_makes_a_rule_of_its_own_kind_and_group() {
        for template in RuleTemplate::ALL {
            let rule = template.new_rule();
            assert_eq!(template_of(&rule.kind), template);
            assert!(RuleTemplate::of_group(template.group()).contains(&template));
        }
        let total: usize = RuleGroup::ALL
            .iter()
            .map(|g| RuleTemplate::of_group(*g).len())
            .sum();
        assert_eq!(total, RuleTemplate::ALL.len());
    }

    #[test]
    fn the_recipe_check_names_the_rule_and_an_empty_recipe_is_a_recipe_problem() {
        let mut recipe = Recipe::new("t", Scope::AllLocal, Vec::new());
        assert!(matches!(
            validation(&recipe),
            Validation::Invalid { rule: None, .. }
        ));
        // A blank "remove files" rule names no path.
        recipe.rules.push(RuleTemplate::DateShift.new_rule());
        recipe.rules.push(RuleTemplate::RemovePaths.new_rule());
        match validation(&recipe) {
            Validation::Invalid { rule, text } => {
                assert_eq!(rule, Some(1));
                assert!(text.contains('2'), "{text}");
            }
            other => panic!("{other:?}"),
        }
        recipe.rules.pop();
        assert!(validation(&recipe).is_ok());
    }

    #[test]
    fn a_disabled_broken_rule_does_not_block() {
        let mut recipe = Recipe::new(
            "t",
            Scope::AllLocal,
            rules(&[RuleTemplate::DateShift, RuleTemplate::RemovePaths]),
        );
        recipe.rules[1].enabled = false;
        assert!(validation(&recipe).is_ok());
    }

    #[test]
    fn an_extract_rule_without_a_mode_is_a_question_not_an_error() {
        let mut recipe = Recipe::new("t", Scope::AllLocal, Vec::new());
        recipe
            .rules
            .push(Rule::new(RuleKind::MovePaths(PathMove::ExtractSubdir {
                dir: "docs".into(),
                mode: None,
            })));
        assert_eq!(validation(&recipe), Validation::NeedsExtractMode);
    }

    #[test]
    fn rows_number_from_one_and_never_show_secret_values() {
        let mut r = rules(&[RuleTemplate::DateShift, RuleTemplate::ContentReplace]);
        if let RuleKind::ContentReplace(c) = &mut r[1].kind {
            c.find = ContentFind::Secrets {
                literals: vec![gitpoulpi_core::domain::rewrite::SecretText(
                    "hunter2".into(),
                )],
                scan: false,
            };
        }
        let shown = rows(&r);
        assert_eq!(shown[0].number, 1);
        assert_eq!(shown[1].number, 2);
        assert_eq!(shown[0].detail, "+1h");
        assert!(!shown[1].detail.contains("hunter2"));
    }
}
