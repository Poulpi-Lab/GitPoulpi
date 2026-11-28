//! The detail pane of one rule: its fields, and how an edit lands in the
//! rule.
//!
//! [`DetailForm`] lists the fields of a rule kind ([`fields`]) and applies an
//! edit with [`apply`]. A text that does not parse (a shift of `2x`) keeps the
//! user's draft on screen with the reason beside it and leaves the rule as it
//! was; the recipe never holds a half-typed value. A field marked `secret`
//! (a literal password to remove) is masked by the renderer and is the one
//! thing a saved recipe leaves out (`Recipe::saved`).

use std::collections::HashMap;

use gitpoulpi_core::domain::rewrite::script::{parse_header_for, ParamKind, ScriptHeader};
use gitpoulpi_core::domain::rewrite::{
    validate_glob, AnonymiseRule, CompiledPattern, ContentFind, ContentReplace, Direction, EolRule,
    IdentityMatch, IdentityRule, IdentitySet, KeepClock, LfsExportRule, LfsImportRule, LineEnding,
    PathMove, PruneEmpty, ReencodeRule, Rule, RuleKind, ScriptLanguage, ScriptRef, ScriptSource,
    SecretText, StripExecRule, TextEncoding, TextPattern, TextReplace, TrailerAction,
    TrailerIfExists, TrailerRule, WeekWindow, Weekday, WhichDate, Who, WindowOffset,
    WorkWindowRule, MAX_OFFSET_MINUTES,
};
use gitpoulpi_i18n::{msg::rewrite_view as t, M};

/// Which field of a rule's detail pane. Stable across rebuilds, so a draft
/// and a cursor survive the pane being rebuilt after a choice changed what
/// is listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldId {
    MatchName,
    MatchEmail,
    MatchRegex,
    SetName,
    SetEmail,
    ApplyTo,
    IncludeTaggers,
    Keep,
    NamePattern,
    EmailPattern,
    Shift,
    DateApplyTo,
    Offset,
    KeepClock,
    Windows,
    Direction,
    KeepOrder,
    Seed,
    Find,
    FindRegex,
    FindCase,
    With,
    TrailerAction,
    TrailerKey,
    TrailerValue,
    TrailerIfExists,
    TrailerFind,
    MapTrailers,
    /// S24: also fix the names of the message-encoding rule.
    FixNames,
    /// The run option `update_message_hashes` (the recipe's, shown in the
    /// panes of the rules that edit messages).
    UpdateHashes,
    Paths,
    ContentMode,
    SecretValues,
    ScanSecrets,
    PruneEmpty,
    MaxBlob,
    /// S21: the smallest file an LFS import moves.
    MinSize,
    Globs,
    Size,
    MoveKind,
    MoveFrom,
    MoveTo,
    MoveDir,
    Patterns,
    ScriptLanguage,
    ScriptSource,
    ScriptText,
    ScriptPath,
    /// Parameter `.0` of the script's header (a typed row).
    ScriptParam(usize),
    /// S22 / S24bis: the files a rule leaves alone.
    Exclude,
    EolTarget,
    FollowAttrs,
    EncTarget,
    EncFrom,
    EncFallback,
}

/// What a field holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    Text(String),
    /// One of `options`; `index` is the current one.
    Choice {
        options: Vec<M>,
        index: usize,
    },
    Toggle(bool),
}

/// What the user did to a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldInput {
    Text(String),
    Choice(usize),
    Toggle(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub id: FieldId,
    pub label: M,
    /// A label that is the user's own words (a script parameter's), shown
    /// instead of `label`.
    pub label_text: Option<String>,
    pub value: FieldValue,
    /// Masked on screen, never saved.
    pub secret: bool,
    /// Several lines (a script, a list of secret values).
    pub multiline: bool,
    /// An example or a format reminder, dim beside the value.
    pub hint: Option<M>,
    /// Why the draft was not accepted.
    pub error: Option<String>,
}

impl Field {
    fn text(id: FieldId, label: M, value: impl Into<String>) -> Self {
        Self {
            id,
            label,
            label_text: None,
            value: FieldValue::Text(value.into()),
            secret: false,
            multiline: false,
            hint: None,
            error: None,
        }
    }

    fn toggle(id: FieldId, label: M, on: bool) -> Self {
        Self {
            value: FieldValue::Toggle(on),
            ..Self::text(id, label, "")
        }
    }

    fn choice(id: FieldId, label: M, options: Vec<M>, index: usize) -> Self {
        Self {
            value: FieldValue::Choice { options, index },
            ..Self::text(id, label, "")
        }
    }

    fn titled(mut self, text: impl Into<String>) -> Self {
        self.label_text = Some(text.into());
        self
    }

    /// The label on screen.
    pub fn title(&self) -> &str {
        self.label_text
            .as_deref()
            .unwrap_or_else(|| self.label.get())
    }

    fn hint(mut self, hint: M) -> Self {
        self.hint = Some(hint);
        self
    }

    fn secret(mut self) -> Self {
        self.secret = true;
        self.multiline = true;
        self
    }

    fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    /// The text of a text field.
    pub fn as_text(&self) -> Option<&str> {
        match &self.value {
            FieldValue::Text(s) => Some(s),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------- choices

const WHO: [(Who, M); 3] = [
    (Who::Both, t::O_WHO_BOTH),
    (Who::Author, t::O_WHO_AUTHOR),
    (Who::Committer, t::O_WHO_COMMITTER),
];
const WHICH_DATE: [(WhichDate, M); 3] = [
    (WhichDate::Both, t::O_DATE_BOTH),
    (WhichDate::AuthorDate, t::O_DATE_AUTHOR),
    (WhichDate::CommitterDate, t::O_DATE_COMMITTER),
];
const KEEP_CLOCK: [(KeepClock, M); 2] = [
    (KeepClock::Instant, t::O_CLOCK_INSTANT),
    (KeepClock::WallClock, t::O_CLOCK_WALL),
];
const DIRECTION: [(Direction, M); 3] = [
    (Direction::Nearest, t::O_DIRECTION_NEAREST),
    (Direction::LaterOnly, t::O_DIRECTION_LATER),
    (Direction::EarlierOnly, t::O_DIRECTION_EARLIER),
];
const TRAILER: [(TrailerAction, M); 3] = [
    (TrailerAction::Add, t::O_TRAILER_ADD),
    (TrailerAction::Remove, t::O_TRAILER_REMOVE),
    (TrailerAction::Map, t::O_TRAILER_MAP),
];
const TRAILER_IF_EXISTS: [(TrailerIfExists, M); 3] = [
    (TrailerIfExists::Add, t::O_TRAILER_IF_ADD),
    (TrailerIfExists::Replace, t::O_TRAILER_IF_REPLACE),
    (TrailerIfExists::Skip, t::O_TRAILER_IF_SKIP),
];
pub const LANGUAGE: [(ScriptLanguage, M); 4] = [
    (ScriptLanguage::Lua, t::O_SCRIPT_LUA),
    (ScriptLanguage::Shell, t::O_SCRIPT_SHELL),
    (ScriptLanguage::Python, t::O_SCRIPT_PYTHON),
    (ScriptLanguage::LuaExternal, t::O_SCRIPT_LUA_EXTERNAL),
];

const EOL: [(LineEnding, M); 3] = [
    (LineEnding::Lf, t::O_EOL_LF),
    (LineEnding::Crlf, t::O_EOL_CRLF),
    (LineEnding::Cr, t::O_EOL_CR),
];

const ENCODING: [(TextEncoding, M); 6] = [
    (TextEncoding::Utf8, t::O_ENC_UTF8),
    (TextEncoding::Utf8Bom, t::O_ENC_UTF8_BOM),
    (TextEncoding::Utf16Le, t::O_ENC_UTF16LE),
    (TextEncoding::Utf16Be, t::O_ENC_UTF16BE),
    (TextEncoding::Windows1252, t::O_ENC_CP1252),
    (TextEncoding::Latin1, t::O_ENC_LATIN1),
];

/// The "Files are in" choice: the detection first, then each encoding.
fn from_options() -> Vec<M> {
    std::iter::once(t::O_ENC_DETECT)
        .chain(ENCODING.iter().map(|(_, m)| *m))
        .collect()
}

fn from_index(from: Option<TextEncoding>) -> usize {
    from.map_or(0, |e| 1 + index_of(&ENCODING, e))
}

fn options<T: Copy>(table: &[(T, M)]) -> Vec<M> {
    table.iter().map(|(_, m)| *m).collect()
}

fn index_of<T: Copy + PartialEq>(table: &[(T, M)], value: T) -> usize {
    table.iter().position(|(v, _)| *v == value).unwrap_or(0)
}

fn pick<T: Copy>(table: &[(T, M)], index: usize) -> Option<T> {
    table.get(index).map(|(v, _)| *v)
}

/// The kind of path move, as the choice lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveKind {
    Rename,
    IntoSubdir,
    ExtractSubdir,
}

const MOVE_KIND: [(MoveKind, M); 3] = [
    (MoveKind::Rename, t::O_MOVE_RENAME),
    (MoveKind::IntoSubdir, t::O_MOVE_INTO),
    (MoveKind::ExtractSubdir, t::O_MOVE_EXTRACT),
];

fn move_kind_of(m: &PathMove) -> MoveKind {
    match m {
        PathMove::Rename { .. } => MoveKind::Rename,
        PathMove::IntoSubdir { .. } => MoveKind::IntoSubdir,
        PathMove::ExtractSubdir { .. } => MoveKind::ExtractSubdir,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    Pattern,
    Secrets,
}

const CONTENT_MODE: [(ContentMode, M); 2] = [
    (ContentMode::Pattern, t::O_CONTENT_PATTERN),
    (ContentMode::Secrets, t::O_CONTENT_SECRETS),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Inline,
    File,
}

const SOURCE: [(SourceKind, M); 2] = [
    (SourceKind::Inline, t::O_SOURCE_INLINE),
    (SourceKind::File, t::O_SOURCE_FILE),
];

// ------------------------------------------------------------ the fields

fn opt(s: &Option<String>) -> String {
    s.clone().unwrap_or_default()
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// The fields of `rule`'s detail pane, in order, each with the problem its
/// current value has (a pattern that does not compile, a bad glob), so the
/// reason shows beside the field while it is typed.
pub fn fields(rule: &Rule, header: Option<&ScriptHeader>) -> Vec<Field> {
    let mut list = raw_fields(rule, header);
    for f in &mut list {
        f.error = live_error(rule, f.id, header);
    }
    list
}

/// The reason the value of field `id` cannot run, from the core's own checks
/// (`CompiledPattern`, `validate_glob`). An empty value is not an error here:
/// the footer says what is missing.
fn live_error(rule: &Rule, id: FieldId, known: Option<&ScriptHeader>) -> Option<String> {
    use FieldId as F;
    let globs = |list: &[String]| list.iter().find_map(|g| validate_glob(g).err());
    match (&rule.kind, id) {
        (RuleKind::ContentReplace(c), F::Find) => match &c.find {
            ContentFind::Pattern { pattern } if !pattern.pattern.is_empty() => {
                CompiledPattern::new(pattern).err().map(one_line_reason)
            }
            _ => None,
        },
        (RuleKind::ContentReplace(c), F::Paths) => globs(&c.paths),
        (RuleKind::RemovePaths { globs: g }, F::Globs) => globs(g),
        (RuleKind::LfsImport(l), F::Patterns) => globs(&l.patterns),
        (RuleKind::LfsExport(l), F::Patterns) => globs(&l.patterns),
        (RuleKind::LineEndings(e), F::Paths) => globs(&e.paths),
        (RuleKind::LineEndings(e), F::Exclude) => globs(&e.exclude),
        (RuleKind::StripExec(e), F::Paths) => globs(&e.paths),
        (RuleKind::StripExec(e), F::Exclude) => globs(&e.exclude),
        (RuleKind::Reencode(e), F::Paths) => globs(&e.paths),
        (RuleKind::Reencode(e), F::Exclude) => globs(&e.exclude),
        (RuleKind::Reencode(e), F::EncFrom) if e.from == Some(e.target) => {
            Some(t::E_ENC_SAME.get().to_string())
        }
        (
            RuleKind::LfsImport(LfsImportRule {
                min_size: Some(0), ..
            }),
            F::MinSize,
        ) => Some(t::E_SIZE_ZERO.get().to_string()),
        (RuleKind::Script(s), F::ScriptParam(i)) => {
            let header = script_header(s, known)?;
            let p = header.params.get(i)?;
            let value = param_value(s, p);
            match p.kind {
                ParamKind::Number if value.trim().parse::<f64>().is_err() => {
                    Some(t::E_PARAM_NUMBER.get().to_string())
                }
                ParamKind::Text if value.is_empty() && p.default.is_none() => {
                    Some(t::E_PARAM_REQUIRED.get().to_string())
                }
                _ => None,
            }
        }
        (RuleKind::RemoveBlobsOver { bytes: 0 }, F::Size) => Some(t::E_SIZE_ZERO.get().to_string()),
        (RuleKind::ContentReplace(c), F::MaxBlob) if c.max_blob_bytes == Some(0) => {
            Some(t::E_SIZE_ZERO.get().to_string())
        }
        _ => None,
    }
}

/// A regex engine's multi-line complaint as the one line that says what is
/// wrong (`error: unclosed group`).
fn one_line_reason(reason: String) -> String {
    reason
        .lines()
        .rev()
        .find_map(|l| l.trim().strip_prefix("error: "))
        .or_else(|| reason.lines().next())
        .unwrap_or("")
        .to_string()
}

/// The dim lines under the fields: what a kind does that the fields do not
/// say. Empty for most kinds.
pub fn notes(rule: &Rule) -> Vec<M> {
    match &rule.kind {
        RuleKind::ContentReplace(c) => {
            let mut n = vec![match c.find {
                ContentFind::Pattern { .. } => t::N_CONTENT_TEXT,
                ContentFind::Secrets { .. } => t::N_SECRETS,
            }];
            n.push(t::N_GLOB_DEPTH);
            n
        }
        RuleKind::Script(s) if s.language.is_external() => {
            vec![t::SCRIPT_UNSANDBOXED, t::N_SCRIPT_EDIT]
        }
        RuleKind::Script(_) => vec![t::N_SCRIPT_EDIT],
        RuleKind::RemovePaths { .. } => vec![t::N_GLOB_DEPTH],
        RuleKind::RemoveBlobsOver { .. } => vec![t::N_BLOBS_OVER],
        RuleKind::LfsImport(_) => vec![t::N_LFS_IMPORT, t::N_GLOB_DEPTH],
        RuleKind::LfsExport(_) => vec![t::N_LFS_EXPORT, t::N_GLOB_DEPTH],
        RuleKind::LineEndings(_) => vec![t::N_LINE_ENDINGS, t::N_GLOB_DEPTH],
        RuleKind::StripExec(_) => vec![t::N_STRIP_EXEC, t::N_GLOB_DEPTH],
        RuleKind::Reencode(_) => vec![t::N_REENCODE, t::N_GLOB_DEPTH],
        RuleKind::MessageEncoding { .. } => vec![t::N_MESSAGE_ENCODING],
        _ => Vec::new(),
    }
}

/// Whether the kind can leave a commit empty, so the run option "empty
/// commits" belongs in its pane.
pub fn can_empty_commits(kind: &RuleKind) -> bool {
    matches!(
        kind,
        RuleKind::ContentReplace(_)
            | RuleKind::RemovePaths { .. }
            | RuleKind::RemoveBlobsOver { .. }
    )
}

/// The run option field for [`PruneEmpty`].
pub fn prune_field(prune: PruneEmpty) -> Field {
    Field::choice(
        FieldId::PruneEmpty,
        t::F_PRUNE_EMPTY,
        vec![t::O_PRUNE_AUTO, t::O_PRUNE_NEVER],
        usize::from(prune == PruneEmpty::Never),
    )
}

/// The rules whose pane carries the `update_message_hashes` run option.
pub fn edits_messages(kind: &RuleKind) -> bool {
    matches!(
        kind,
        RuleKind::MessageReplace(_) | RuleKind::MessageDeleteLines { .. } | RuleKind::Trailer(_)
    )
}

/// [`fields`] plus the run options a rule's pane carries: what to do with
/// commits a content rule empties, and whether ids in messages follow the
/// rewrite.
pub fn fields_with_run(
    rule: &Rule,
    prune: PruneEmpty,
    hashes: bool,
    header: Option<&ScriptHeader>,
) -> Vec<Field> {
    let mut list = fields(rule, header);
    if can_empty_commits(&rule.kind) {
        list.push(prune_field(prune));
    }
    if edits_messages(&rule.kind) {
        list.push(Field::toggle(
            FieldId::UpdateHashes,
            t::F_UPDATE_HASHES,
            hashes,
        ));
    }
    list
}

fn raw_fields(rule: &Rule, known: Option<&ScriptHeader>) -> Vec<Field> {
    use FieldId as F;
    match &rule.kind {
        RuleKind::Identity(r) => vec![
            Field::text(F::MatchName, t::F_MATCH_NAME, opt(&r.when.name)).hint(t::H_ANY),
            Field::text(F::MatchEmail, t::F_MATCH_EMAIL, opt(&r.when.email)).hint(t::H_ANY),
            Field::toggle(F::MatchRegex, t::F_MATCH_REGEX, r.when.regex),
            Field::text(F::SetName, t::F_SET_NAME, opt(&r.set.name)).hint(t::H_KEEP),
            Field::text(F::SetEmail, t::F_SET_EMAIL, opt(&r.set.email)).hint(t::H_KEEP),
            Field::choice(
                F::ApplyTo,
                t::F_APPLY_TO,
                options(&WHO),
                index_of(&WHO, r.apply_to),
            ),
            Field::toggle(F::IncludeTaggers, t::F_INCLUDE_TAGGERS, r.include_taggers),
            Field::toggle(F::MapTrailers, t::F_MAP_TRAILERS, r.map_trailers),
        ],
        RuleKind::Mailmap { apply_to, .. } => vec![Field::choice(
            F::ApplyTo,
            t::F_APPLY_TO,
            options(&WHO),
            index_of(&WHO, *apply_to),
        )],
        RuleKind::Anonymise(r) => vec![
            Field::text(F::NamePattern, t::F_NAME_PATTERN, &r.name_pattern),
            Field::text(F::EmailPattern, t::F_EMAIL_PATTERN, &r.email_pattern),
            Field::text(F::Keep, t::F_KEEP_LIST, keep_text(&r.keep)).hint(t::H_KEEP_LIST),
            Field::choice(
                F::ApplyTo,
                t::F_APPLY_TO,
                options(&WHO),
                index_of(&WHO, r.apply_to),
            ),
            Field::toggle(F::IncludeTaggers, t::F_INCLUDE_TAGGERS, r.include_taggers),
        ],
        RuleKind::DateShift {
            by_seconds,
            apply_to,
        } => vec![
            Field::text(F::Shift, t::F_SHIFT, format_duration(*by_seconds)).hint(t::H_SHIFT),
            Field::choice(
                F::DateApplyTo,
                t::F_DATE_APPLY_TO,
                options(&WHICH_DATE),
                index_of(&WHICH_DATE, *apply_to),
            ),
        ],
        RuleKind::DateTimezone {
            offset_minutes,
            keep,
            apply_to,
        } => vec![
            Field::text(F::Offset, t::F_OFFSET, format_offset(*offset_minutes)).hint(t::H_OFFSET),
            Field::choice(
                F::KeepClock,
                t::F_KEEP_CLOCK,
                options(&KEEP_CLOCK),
                index_of(&KEEP_CLOCK, *keep),
            ),
            Field::choice(
                F::DateApplyTo,
                t::F_DATE_APPLY_TO,
                options(&WHICH_DATE),
                index_of(&WHICH_DATE, *apply_to),
            ),
        ],
        RuleKind::DateWindow(w) => vec![
            Field::text(F::Windows, t::F_WINDOWS, format_windows(&w.windows))
                .hint(t::H_WINDOWS)
                .multiline(),
            Field::choice(
                F::Direction,
                t::F_DIRECTION,
                options(&DIRECTION),
                index_of(&DIRECTION, w.direction),
            ),
            Field::toggle(F::KeepOrder, t::F_KEEP_ORDER, w.keep_order),
            Field::text(F::Seed, t::F_SEED, w.seed.to_string()),
            Field::choice(
                F::DateApplyTo,
                t::F_DATE_APPLY_TO,
                options(&WHICH_DATE),
                index_of(&WHICH_DATE, w.apply_to),
            ),
        ],
        RuleKind::CommitterFromAuthor | RuleKind::RemoveSignatures => Vec::new(),
        RuleKind::MessageEncoding { names } => {
            vec![Field::toggle(F::FixNames, t::F_FIX_NAMES, *names)]
        }
        RuleKind::MessageReplace(r) => vec![
            Field::text(F::Find, t::F_FIND, &r.find.pattern),
            Field::toggle(F::FindRegex, t::F_REGEX, r.find.regex),
            Field::toggle(F::FindCase, t::F_CASE, r.find.case_sensitive),
            Field::text(F::With, t::F_WITH, &r.with).hint(t::H_DELETE_IF_EMPTY),
        ],
        RuleKind::MessageDeleteLines { pattern } => vec![
            Field::text(F::Find, t::F_LINE_PATTERN, &pattern.pattern),
            Field::toggle(F::FindRegex, t::F_REGEX, pattern.regex),
            Field::toggle(F::FindCase, t::F_CASE, pattern.case_sensitive),
        ],
        RuleKind::Trailer(r) => {
            let mut f = vec![
                Field::choice(
                    F::TrailerAction,
                    t::F_TRAILER_ACTION,
                    options(&TRAILER),
                    index_of(&TRAILER, r.action),
                ),
                Field::text(F::TrailerKey, t::F_TRAILER_KEY, &r.key).hint(t::H_TRAILER_KEY),
            ];
            let find = r.find.as_ref().map(|p| p.pattern.as_str()).unwrap_or("");
            match r.action {
                TrailerAction::Add => {
                    f.push(Field::text(F::TrailerValue, t::F_TRAILER_VALUE, &r.value));
                    f.push(Field::choice(
                        F::TrailerIfExists,
                        t::F_TRAILER_IF_EXISTS,
                        options(&TRAILER_IF_EXISTS),
                        index_of(&TRAILER_IF_EXISTS, r.if_exists),
                    ));
                }
                TrailerAction::Remove => f.push(
                    Field::text(F::TrailerFind, t::F_TRAILER_FIND_REMOVE, find)
                        .hint(t::H_TRAILER_ANY_VALUE),
                ),
                TrailerAction::Map => {
                    f.push(Field::text(F::TrailerFind, t::F_TRAILER_FIND_MAP, find));
                    f.push(Field::text(F::With, t::F_WITH, &r.with));
                }
            }
            f
        }
        RuleKind::ContentReplace(c) => {
            let mode = match c.find {
                ContentFind::Pattern { .. } => ContentMode::Pattern,
                ContentFind::Secrets { .. } => ContentMode::Secrets,
            };
            let mut f = vec![
                Field::text(F::Paths, t::F_PATHS, c.paths.join(", ")).hint(t::H_ALL_FILES),
                Field::choice(
                    F::ContentMode,
                    t::F_CONTENT_MODE,
                    options(&CONTENT_MODE),
                    index_of(&CONTENT_MODE, mode),
                ),
            ];
            match &c.find {
                ContentFind::Pattern { pattern } => {
                    f.push(Field::text(F::Find, t::F_FIND, &pattern.pattern));
                    f.push(Field::toggle(F::FindRegex, t::F_REGEX, pattern.regex));
                    f.push(Field::toggle(
                        F::FindCase,
                        t::F_CASE,
                        pattern.case_sensitive,
                    ));
                    f.push(Field::text(F::With, t::F_WITH, &c.with).hint(t::H_DELETE_IF_EMPTY));
                }
                ContentFind::Secrets { literals, .. } => {
                    let joined = literals
                        .iter()
                        .map(|l| l.0.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    f.push(
                        Field::text(F::SecretValues, t::F_SECRET_VALUES, joined)
                            .secret()
                            .hint(t::H_SECRET_VALUES),
                    );
                }
            }
            f.push(
                Field::text(
                    F::MaxBlob,
                    t::F_MAX_BLOB,
                    c.max_blob_bytes.map(format_size).unwrap_or_default(),
                )
                .hint(t::H_MAX_BLOB_DEFAULT),
            );
            f
        }
        RuleKind::RemovePaths { globs } => {
            vec![Field::text(F::Globs, t::F_GLOBS, globs.join(", ")).hint(t::H_GLOBS)]
        }
        RuleKind::RemoveBlobsOver { bytes } => {
            vec![Field::text(F::Size, t::F_SIZE, format_size(*bytes)).hint(t::H_SIZE)]
        }
        RuleKind::MovePaths(m) => {
            let mut f = vec![Field::choice(
                F::MoveKind,
                t::F_MOVE_KIND,
                options(&MOVE_KIND),
                index_of(&MOVE_KIND, move_kind_of(m)),
            )];
            match m {
                PathMove::Rename { from, to } => {
                    f.push(Field::text(F::MoveFrom, t::F_MOVE_FROM, from));
                    f.push(Field::text(F::MoveTo, t::F_MOVE_TO, to));
                }
                PathMove::IntoSubdir { dir } | PathMove::ExtractSubdir { dir, .. } => {
                    f.push(Field::text(F::MoveDir, t::F_MOVE_DIR, dir));
                }
            }
            f
        }
        RuleKind::LfsImport(l) => vec![
            Field::text(F::Patterns, t::F_PATTERNS, l.patterns.join(", ")).hint(t::H_GLOBS),
            Field::text(
                F::MinSize,
                t::F_MIN_SIZE,
                l.min_size.map(format_size).unwrap_or_default(),
            )
            .hint(t::H_MIN_SIZE_ALL),
        ],
        RuleKind::LfsExport(l) => {
            vec![
                Field::text(F::Patterns, t::F_PATTERNS_EXPORT, l.patterns.join(", "))
                    .hint(t::H_GLOBS),
            ]
        }
        RuleKind::LineEndings(e) => vec![
            Field::text(F::Paths, t::F_PATHS, e.paths.join(", ")).hint(t::H_ALL_FILES),
            Field::text(F::Exclude, t::F_EXCLUDE, e.exclude.join(", ")).hint(t::H_EXCLUDE_NONE),
            Field::choice(
                F::EolTarget,
                t::F_CONVERT_TO,
                options(&EOL),
                index_of(&EOL, e.target),
            ),
            Field::toggle(F::FollowAttrs, t::F_FOLLOW_ATTRS, e.follow_attributes),
            max_blob_field(e.max_blob_bytes),
        ],
        RuleKind::StripExec(e) => vec![
            Field::text(F::Paths, t::F_PATHS, e.paths.join(", ")).hint(t::H_GLOBS),
            Field::text(F::Exclude, t::F_EXCLUDE, e.exclude.join(", ")).hint(t::H_EXCLUDE_NONE),
        ],
        RuleKind::Reencode(e) => vec![
            Field::text(F::Paths, t::F_PATHS, e.paths.join(", ")).hint(t::H_ALL_FILES),
            Field::text(F::Exclude, t::F_EXCLUDE, e.exclude.join(", ")).hint(t::H_EXCLUDE_NONE),
            Field::choice(
                F::EncTarget,
                t::F_CONVERT_TO,
                options(&ENCODING),
                index_of(&ENCODING, e.target),
            ),
            Field::choice(
                F::EncFrom,
                t::F_ENC_FROM,
                from_options(),
                from_index(e.from),
            ),
            Field::choice(
                F::EncFallback,
                t::F_ENC_FALLBACK,
                options(&ENCODING),
                index_of(&ENCODING, e.fallback),
            ),
            max_blob_field(e.max_blob_bytes),
        ],
        RuleKind::Script(s) => {
            let source = match s.source {
                ScriptSource::Inline { .. } => SourceKind::Inline,
                ScriptSource::File { .. } => SourceKind::File,
            };
            let mut f = vec![
                Field::choice(
                    F::ScriptLanguage,
                    t::F_SCRIPT_LANGUAGE,
                    options(&LANGUAGE),
                    index_of(&LANGUAGE, s.language),
                ),
                Field::choice(
                    F::ScriptSource,
                    t::F_SCRIPT_SOURCE,
                    options(&SOURCE),
                    index_of(&SOURCE, source),
                ),
            ];
            match &s.source {
                ScriptSource::Inline { text } => {
                    f.push(Field::text(F::ScriptText, t::F_SCRIPT_TEXT, text).multiline())
                }
                ScriptSource::File { path } => {
                    f.push(Field::text(F::ScriptPath, t::F_SCRIPT_PATH, path))
                }
            }
            if let Some(header) = script_header(s, known) {
                for (i, p) in header.params.iter().enumerate() {
                    let value = param_value(s, p);
                    let field = match p.kind {
                        ParamKind::Bool => {
                            Field::toggle(F::ScriptParam(i), t::F_SCRIPT_PARAMS, value == "true")
                        }
                        _ => Field::text(F::ScriptParam(i), t::F_SCRIPT_PARAMS, value),
                    };
                    f.push(field.titled(p.label.clone()));
                }
            }
            f
        }
    }
}

fn max_blob_field(max: Option<u64>) -> Field {
    Field::text(
        FieldId::MaxBlob,
        t::F_MAX_BLOB,
        max.map(format_size).unwrap_or_default(),
    )
    .hint(t::H_MAX_BLOB_DEFAULT)
}

// --------------------------------------------------------------- applying

/// Puts `input` into `rule`. `Err` is the reason the text is not a value
/// (the rule is then unchanged).
pub fn apply(
    rule: &mut Rule,
    id: FieldId,
    input: FieldInput,
    known: Option<&ScriptHeader>,
) -> Result<(), String> {
    use FieldId as F;
    let text = match &input {
        FieldInput::Text(s) => Some(s.as_str()),
        _ => None,
    };
    let flag = match &input {
        FieldInput::Toggle(b) => Some(*b),
        _ => None,
    };
    let index = match &input {
        FieldInput::Choice(i) => Some(*i),
        _ => None,
    };
    let bad = |m: M| m.get().to_string();
    match (&mut rule.kind, id) {
        (RuleKind::Identity(r), F::MatchName) => r.when.name = non_empty(text.unwrap_or("")),
        (RuleKind::Identity(r), F::MatchEmail) => r.when.email = non_empty(text.unwrap_or("")),
        (RuleKind::Identity(r), F::MatchRegex) => r.when.regex = flag.unwrap_or(false),
        (RuleKind::Identity(r), F::SetName) => r.set.name = non_empty(text.unwrap_or("")),
        (RuleKind::Identity(r), F::SetEmail) => r.set.email = non_empty(text.unwrap_or("")),
        (RuleKind::Identity(r), F::ApplyTo) => {
            r.apply_to = index.and_then(|i| pick(&WHO, i)).unwrap_or(r.apply_to)
        }
        (RuleKind::Identity(r), F::IncludeTaggers) => r.include_taggers = flag.unwrap_or(false),
        (RuleKind::Mailmap { apply_to, .. }, F::ApplyTo) => {
            *apply_to = index.and_then(|i| pick(&WHO, i)).unwrap_or(*apply_to)
        }
        (RuleKind::Anonymise(r), F::NamePattern) => r.name_pattern = text.unwrap_or("").into(),
        (RuleKind::Anonymise(r), F::EmailPattern) => r.email_pattern = text.unwrap_or("").into(),
        (RuleKind::Anonymise(r), F::Keep) => r.keep = parse_keep(text.unwrap_or("")),
        (RuleKind::Anonymise(r), F::ApplyTo) => {
            r.apply_to = index.and_then(|i| pick(&WHO, i)).unwrap_or(r.apply_to)
        }
        (RuleKind::Anonymise(r), F::IncludeTaggers) => r.include_taggers = flag.unwrap_or(false),
        (RuleKind::DateShift { by_seconds, .. }, F::Shift) => {
            *by_seconds = parse_duration(text.unwrap_or("")).ok_or_else(|| bad(t::E_SHIFT))?
        }
        (RuleKind::DateShift { apply_to, .. }, F::DateApplyTo)
        | (RuleKind::DateTimezone { apply_to, .. }, F::DateApplyTo) => {
            *apply_to = index
                .and_then(|i| pick(&WHICH_DATE, i))
                .unwrap_or(*apply_to)
        }
        (RuleKind::DateTimezone { offset_minutes, .. }, F::Offset) => {
            *offset_minutes = parse_offset(text.unwrap_or("")).ok_or_else(|| bad(t::E_OFFSET))?
        }
        (RuleKind::DateTimezone { keep, .. }, F::KeepClock) => {
            *keep = index.and_then(|i| pick(&KEEP_CLOCK, i)).unwrap_or(*keep)
        }
        (RuleKind::DateWindow(w), F::Windows) => {
            w.windows = parse_windows(text.unwrap_or("")).ok_or_else(|| bad(t::E_WINDOWS))?
        }
        (RuleKind::DateWindow(w), F::Direction) => {
            w.direction = index
                .and_then(|i| pick(&DIRECTION, i))
                .unwrap_or(w.direction)
        }
        (RuleKind::DateWindow(w), F::KeepOrder) => w.keep_order = flag.unwrap_or(false),
        (RuleKind::DateWindow(w), F::Seed) => {
            w.seed = text
                .unwrap_or("")
                .trim()
                .parse()
                .map_err(|_| bad(t::E_NUMBER))?
        }
        (RuleKind::DateWindow(w), F::DateApplyTo) => {
            w.apply_to = index
                .and_then(|i| pick(&WHICH_DATE, i))
                .unwrap_or(w.apply_to)
        }
        (RuleKind::MessageReplace(r), F::Find) => r.find.pattern = text.unwrap_or("").into(),
        (RuleKind::MessageReplace(r), F::FindRegex) => r.find.regex = flag.unwrap_or(false),
        (RuleKind::MessageReplace(r), F::FindCase) => r.find.case_sensitive = flag.unwrap_or(true),
        (RuleKind::MessageReplace(r), F::With) => r.with = text.unwrap_or("").into(),
        (RuleKind::MessageDeleteLines { pattern }, F::Find) => {
            pattern.pattern = text.unwrap_or("").into()
        }
        (RuleKind::MessageDeleteLines { pattern }, F::FindRegex) => {
            pattern.regex = flag.unwrap_or(false)
        }
        (RuleKind::MessageDeleteLines { pattern }, F::FindCase) => {
            pattern.case_sensitive = flag.unwrap_or(true)
        }
        (RuleKind::Trailer(r), F::TrailerAction) => {
            r.action = index.and_then(|i| pick(&TRAILER, i)).unwrap_or(r.action)
        }
        (RuleKind::Trailer(r), F::TrailerKey) => r.key = text.unwrap_or("").trim().into(),
        (RuleKind::Trailer(r), F::TrailerValue) => r.value = text.unwrap_or("").into(),
        (RuleKind::Trailer(r), F::TrailerIfExists) => {
            r.if_exists = index
                .and_then(|i| pick(&TRAILER_IF_EXISTS, i))
                .unwrap_or(r.if_exists)
        }
        // A literal text; the regex form is for recipe files.
        (RuleKind::Trailer(r), F::TrailerFind) => {
            let text = text.unwrap_or("");
            r.find = (!text.is_empty()).then(|| TextPattern::literal(text));
        }
        (RuleKind::Trailer(r), F::With) => r.with = text.unwrap_or("").into(),
        (RuleKind::MessageEncoding { names }, F::FixNames) => *names = flag.unwrap_or(false),
        (RuleKind::Identity(r), F::MapTrailers) => r.map_trailers = flag.unwrap_or(false),
        (RuleKind::ContentReplace(c), F::Paths) => c.paths = split_list(text.unwrap_or("")),
        (RuleKind::ContentReplace(c), F::ContentMode) => {
            let wanted = index.and_then(|i| pick(&CONTENT_MODE, i));
            match (wanted, &c.find) {
                (Some(ContentMode::Secrets), ContentFind::Pattern { .. }) => {
                    c.find = ContentFind::Secrets {
                        literals: Vec::new(),
                        // Scanning arrives with S14: a rule never asks for it.
                        scan: false,
                    };
                    c.with.clear();
                }
                (Some(ContentMode::Pattern), ContentFind::Secrets { .. }) => {
                    c.find = ContentFind::Pattern {
                        pattern: TextPattern {
                            pattern: String::new(),
                            regex: false,
                            case_sensitive: true,
                        },
                    };
                }
                _ => {}
            }
        }
        (RuleKind::ContentReplace(c), F::Find) => {
            if let ContentFind::Pattern { pattern } = &mut c.find {
                pattern.pattern = text.unwrap_or("").into();
            }
        }
        (RuleKind::ContentReplace(c), F::FindRegex) => {
            if let ContentFind::Pattern { pattern } = &mut c.find {
                pattern.regex = flag.unwrap_or(false);
            }
        }
        (RuleKind::ContentReplace(c), F::FindCase) => {
            if let ContentFind::Pattern { pattern } = &mut c.find {
                pattern.case_sensitive = flag.unwrap_or(true);
            }
        }
        (RuleKind::ContentReplace(c), F::With) => c.with = text.unwrap_or("").into(),
        (RuleKind::ContentReplace(c), F::SecretValues) => {
            if let ContentFind::Secrets { literals, .. } = &mut c.find {
                *literals = text
                    .unwrap_or("")
                    .lines()
                    .map(|l| l.trim_end_matches('\r'))
                    .filter(|l| !l.is_empty())
                    .map(|l| SecretText(l.to_string()))
                    .collect();
            }
        }
        (RuleKind::ContentReplace(c), F::ScanSecrets) => {
            if let ContentFind::Secrets { scan, .. } = &mut c.find {
                *scan = flag.unwrap_or(false);
            }
        }
        (RuleKind::ContentReplace(c), F::MaxBlob) => {
            let s = text.unwrap_or("").trim();
            c.max_blob_bytes = if s.is_empty() {
                None
            } else {
                Some(parse_size(s).ok_or_else(|| bad(t::E_SIZE))?)
            };
        }
        (RuleKind::RemovePaths { globs }, F::Globs) => *globs = split_list(text.unwrap_or("")),
        (RuleKind::RemoveBlobsOver { bytes }, F::Size) => {
            *bytes = parse_size(text.unwrap_or("")).ok_or_else(|| bad(t::E_SIZE))?
        }
        (RuleKind::MovePaths(m), F::MoveKind) => {
            if let Some(kind) = index.and_then(|i| pick(&MOVE_KIND, i)) {
                if kind != move_kind_of(m) {
                    let dir = match m {
                        PathMove::IntoSubdir { dir } | PathMove::ExtractSubdir { dir, .. } => {
                            dir.clone()
                        }
                        PathMove::Rename { to, .. } => to.clone(),
                    };
                    *m = match kind {
                        MoveKind::Rename => PathMove::Rename {
                            from: String::new(),
                            to: dir,
                        },
                        MoveKind::IntoSubdir => PathMove::IntoSubdir { dir },
                        MoveKind::ExtractSubdir => PathMove::ExtractSubdir { dir, mode: None },
                    };
                }
            }
        }
        (RuleKind::MovePaths(PathMove::Rename { from, .. }), F::MoveFrom) => {
            *from = text.unwrap_or("").trim().into()
        }
        (RuleKind::MovePaths(PathMove::Rename { to, .. }), F::MoveTo) => {
            *to = text.unwrap_or("").trim().into()
        }
        (
            RuleKind::MovePaths(PathMove::IntoSubdir { dir } | PathMove::ExtractSubdir { dir, .. }),
            F::MoveDir,
        ) => *dir = text.unwrap_or("").trim().into(),
        (RuleKind::LfsImport(l), F::Patterns) => l.patterns = split_list(text.unwrap_or("")),
        (RuleKind::LfsExport(l), F::Patterns) => l.patterns = split_list(text.unwrap_or("")),
        (RuleKind::LineEndings(e), F::Paths) => e.paths = split_list(text.unwrap_or("")),
        (RuleKind::LineEndings(e), F::Exclude) => e.exclude = split_list(text.unwrap_or("")),
        (RuleKind::LineEndings(e), F::EolTarget) => {
            e.target = index.and_then(|i| pick(&EOL, i)).unwrap_or(e.target)
        }
        (RuleKind::LineEndings(e), F::FollowAttrs) => e.follow_attributes = flag.unwrap_or(false),
        (RuleKind::StripExec(e), F::Paths) => e.paths = split_list(text.unwrap_or("")),
        (RuleKind::StripExec(e), F::Exclude) => e.exclude = split_list(text.unwrap_or("")),
        (RuleKind::Reencode(e), F::Paths) => e.paths = split_list(text.unwrap_or("")),
        (RuleKind::Reencode(e), F::Exclude) => e.exclude = split_list(text.unwrap_or("")),
        (RuleKind::Reencode(e), F::EncTarget) => {
            e.target = index.and_then(|i| pick(&ENCODING, i)).unwrap_or(e.target)
        }
        (RuleKind::Reencode(e), F::EncFrom) => {
            e.from = index
                .and_then(|i| i.checked_sub(1))
                .and_then(|i| pick(&ENCODING, i))
        }
        (RuleKind::Reencode(e), F::EncFallback) => {
            e.fallback = index.and_then(|i| pick(&ENCODING, i)).unwrap_or(e.fallback)
        }
        (RuleKind::LineEndings(e), F::MaxBlob) => e.max_blob_bytes = max_blob(text, &bad)?,
        (RuleKind::Reencode(e), F::MaxBlob) => e.max_blob_bytes = max_blob(text, &bad)?,
        (RuleKind::LfsImport(l), F::MinSize) => {
            let s = text.unwrap_or("").trim();
            l.min_size = if s.is_empty() {
                None
            } else {
                Some(parse_size(s).ok_or_else(|| bad(t::E_SIZE))?)
            };
        }
        (RuleKind::Script(s), F::ScriptLanguage) => {
            let new = index.and_then(|i| pick(&LANGUAGE, i)).unwrap_or(s.language);
            // A script nobody wrote yet follows the language.
            if let ScriptSource::Inline { text } = &mut s.source {
                if text.trim().is_empty() || text == starter_for(s.language) {
                    *text = starter_for(new).to_string();
                }
            }
            s.language = new;
        }
        (RuleKind::Script(s), F::ScriptSource) => {
            match (index.and_then(|i| pick(&SOURCE, i)), &s.source) {
                (Some(SourceKind::File), ScriptSource::Inline { .. }) => {
                    s.source = ScriptSource::File {
                        path: String::new(),
                    }
                }
                (Some(SourceKind::Inline), ScriptSource::File { .. }) => {
                    s.source = ScriptSource::Inline {
                        text: String::new(),
                    }
                }
                _ => {}
            }
        }
        (RuleKind::Script(s), F::ScriptText) => {
            s.source = ScriptSource::Inline {
                text: text.unwrap_or("").into(),
            }
        }
        (RuleKind::Script(s), F::ScriptPath) => {
            s.source = ScriptSource::File {
                path: text.unwrap_or("").trim().into(),
            }
        }
        (RuleKind::Script(s), F::ScriptParam(i)) => {
            let header = script_header(s, known);
            let Some(p) = header.as_ref().and_then(|h| h.params.get(i)) else {
                return Ok(());
            };
            let value = match &input {
                FieldInput::Toggle(on) => on.to_string(),
                _ => text.unwrap_or("").trim().to_string(),
            };
            if p.kind == ParamKind::Number && !value.is_empty() && value.parse::<f64>().is_err() {
                return Err(bad(t::E_PARAM_NUMBER));
            }
            // An empty value is the header's default again.
            if value.is_empty() {
                s.params.remove(&p.name);
            } else {
                s.params.insert(p.name.clone(), value);
            }
        }
        _ => {}
    }
    Ok(())
}

// ----------------------------------------------------------- the form

/// The detail pane of the rule under the cursor.
#[derive(Debug, Clone, Default)]
pub struct DetailForm {
    /// The rule's position in the list.
    pub rule: usize,
    pub fields: Vec<Field>,
    pub cursor: usize,
    /// The run option shown at the end of a content rule's pane (it belongs
    /// to the recipe, not to the rule; `RewriteView` writes it back).
    pub run_prune: PruneEmpty,
    /// The run option `update_message_hashes`, shown in the panes of the
    /// rules that edit messages (like `run_prune`).
    pub run_hashes: bool,
    /// The header of a script read from a file, as the daemon's check last
    /// answered it (an inline script's header is parsed from its text).
    pub file_header: Option<ScriptHeader>,
    /// Texts the rule did not accept, kept so they stay on screen.
    drafts: HashMap<FieldId, (String, String)>,
}

impl DetailForm {
    pub fn for_rule(index: usize, rule: &Rule) -> Self {
        Self {
            rule: index,
            fields: fields_with_run(rule, PruneEmpty::Auto, false, None),
            cursor: 0,
            run_prune: PruneEmpty::Auto,
            run_hashes: false,
            file_header: None,
            drafts: HashMap::new(),
        }
    }

    pub fn current(&self) -> Option<&Field> {
        self.fields.get(self.cursor)
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let last = self.fields.len().saturating_sub(1) as isize;
        self.cursor = (self.cursor as isize + delta).clamp(0, last) as usize;
    }

    /// Applies `input` to the field at `at`. Returns `true` when the rule
    /// changed.
    pub fn edit(&mut self, rule: &mut Rule, at: usize, input: FieldInput) -> bool {
        let Some(id) = self.fields.get(at).map(|f| f.id) else {
            return false;
        };
        let before = rule.clone();
        let draft = match &input {
            FieldInput::Text(s) => Some(s.clone()),
            _ => None,
        };
        match apply(rule, id, input, self.file_header.as_ref()) {
            Ok(()) => {
                self.drafts.remove(&id);
                self.rebuild(rule);
            }
            Err(why) => {
                if let Some(text) = draft {
                    self.drafts.insert(id, (text, why));
                }
                self.rebuild(rule);
            }
        }
        *rule != before
    }

    /// Re-lists the fields from the rule (a choice may add or remove some),
    /// keeping the cursor on the same field when it is still there and the
    /// drafts of the texts that were refused.
    pub fn rebuild(&mut self, rule: &Rule) {
        let keep = self.fields.get(self.cursor).map(|f| f.id);
        let mut fresh = fields_with_run(
            rule,
            self.run_prune,
            self.run_hashes,
            self.file_header.as_ref(),
        );
        self.drafts
            .retain(|id, _| fresh.iter().any(|f| f.id == *id));
        for f in &mut fresh {
            if let Some((text, why)) = self.drafts.get(&f.id) {
                f.value = FieldValue::Text(text.clone());
                f.error = Some(why.clone());
            }
        }
        self.cursor = keep
            .and_then(|id| fresh.iter().position(|f| f.id == id))
            .unwrap_or_else(|| self.cursor.min(fresh.len().saturating_sub(1)));
        self.fields = fresh;
    }

    /// `true` while a refused text is on screen.
    pub fn has_draft(&self) -> bool {
        !self.drafts.is_empty()
    }
}

// -------------------------------------------------------------- parsing

fn split_list(text: &str) -> Vec<String> {
    text.split([',', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn keep_text(keep: &[IdentityMatch]) -> String {
    keep.iter()
        .filter_map(|m| m.email.clone().or_else(|| m.name.clone()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_keep(text: &str) -> Vec<IdentityMatch> {
    split_list(text)
        .into_iter()
        .map(|entry| {
            if entry.contains('@') {
                IdentityMatch {
                    email: Some(entry),
                    ..IdentityMatch::default()
                }
            } else {
                IdentityMatch {
                    name: Some(entry),
                    ..IdentityMatch::default()
                }
            }
        })
        .collect()
}

/// `+2h`, `-1d 3h`, `90m`, `45s`: a sign for the whole value, then
/// `<number><d|h|m|s>` parts. `None` for anything else (and for no part).
pub fn parse_duration(text: &str) -> Option<i64> {
    let text = text.trim();
    let (sign, rest) = match text.chars().next()? {
        '-' => (-1i64, &text[1..]),
        '+' => (1, &text[1..]),
        _ => (1, text),
    };
    let mut total: i64 = 0;
    let mut parts = 0;
    for part in rest.split_whitespace() {
        let unit = part.chars().last()?;
        let number: i64 = part[..part.len() - unit.len_utf8()].parse().ok()?;
        let seconds = match unit {
            'd' => 86_400,
            'h' => 3_600,
            'm' => 60,
            's' => 1,
            _ => return None,
        };
        total = total.checked_add(number.checked_mul(seconds)?)?;
        parts += 1;
    }
    (parts > 0).then_some(sign * total)
}

/// The text [`parse_duration`] reads back to the same value.
pub fn format_duration(seconds: i64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }
    let sign = if seconds < 0 { '-' } else { '+' };
    let mut rest = seconds.unsigned_abs();
    let mut out = String::new();
    for (unit, size) in [('d', 86_400u64), ('h', 3_600), ('m', 60), ('s', 1)] {
        if rest >= size {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&format!("{}{unit}", rest / size));
            rest %= size;
        }
    }
    format!("{sign}{out}")
}

/// `+02:00`, `-0530`, `+2`, `Z`, `UTC`: minutes east of UTC.
pub fn parse_offset(text: &str) -> Option<i16> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("z") || text.eq_ignore_ascii_case("utc") {
        return Some(0);
    }
    let (sign, rest) = match text.chars().next()? {
        '-' => (-1i32, &text[1..]),
        '+' => (1, &text[1..]),
        _ => (1, text),
    };
    let (h, m) = match rest.split_once(':') {
        Some((h, m)) => (h, m),
        None if rest.len() == 4 => rest.split_at(2),
        None => (rest, "0"),
    };
    let hours: i32 = h.parse().ok()?;
    let minutes: i32 = m.parse().ok()?;
    if !(0..60).contains(&minutes) || hours < 0 {
        return None;
    }
    let total = sign * (hours * 60 + minutes);
    (total.abs() <= i32::from(MAX_OFFSET_MINUTES)).then_some(total as i16)
}

pub fn format_offset(minutes: i16) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let abs = minutes.unsigned_abs();
    format!("{sign}{:02}:{:02}", abs / 60, abs % 60)
}

/// `5M`, `500k`, `1G`, `5 MiB`, `500 KiB`, `1 GiB`, `2048`: bytes, binary
/// units (a K, M or G with or without `iB`/`B`, any case).
pub fn parse_size(text: &str) -> Option<u64> {
    let lower = text.trim().to_ascii_lowercase();
    let body = lower
        .strip_suffix("ib")
        .or_else(|| lower.strip_suffix('b'))
        .unwrap_or(&lower)
        .trim_end();
    let (digits, factor) = match body.chars().last()? {
        'k' => (&body[..body.len() - 1], 1u64 << 10),
        'm' => (&body[..body.len() - 1], 1 << 20),
        'g' => (&body[..body.len() - 1], 1 << 30),
        c if c.is_ascii_digit() => (body, 1),
        _ => return None,
    };
    let number: u64 = digits.trim().parse().ok()?;
    number.checked_mul(factor)
}

// i18n-scan: off
/// The size with its unit, the form [`parse_size`] reads back (`5 MiB`).
pub fn format_size(bytes: u64) -> String {
    for (unit, size) in [("GiB", 1u64 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)] {
        if bytes >= size && bytes.is_multiple_of(size) {
            return format!("{} {unit}", bytes / size);
        }
    }
    format!("{bytes} B")
}
// i18n-scan: on

// The day names of the `Mon-Fri 18:00-23:30` hours syntax, the same in every
// language (a format, not text shown to the user).
// i18n-scan: off
const DAY_NAMES: [(&str, Weekday); 7] = [
    ("Mon", Weekday::Mon),
    ("Tue", Weekday::Tue),
    ("Wed", Weekday::Wed),
    ("Thu", Weekday::Thu),
    ("Fri", Weekday::Fri),
    ("Sat", Weekday::Sat),
    ("Sun", Weekday::Sun),
];
// i18n-scan: on

fn day_named(name: &str) -> Option<Weekday> {
    DAY_NAMES
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name.trim()))
        .map(|(_, d)| *d)
}

fn parse_clock(text: &str) -> Option<u16> {
    let (h, m) = text.trim().split_once(':')?;
    let (h, m): (u16, u16) = (h.parse().ok()?, m.parse().ok()?);
    (m < 60 && (h < 24 || (h == 24 && m == 0))).then_some(h * 60 + m)
}

/// `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`: day ranges and lists, then the
/// hours. `None` for a syntax error or an empty list.
pub fn parse_windows(text: &str) -> Option<Vec<WeekWindow>> {
    let mut out = Vec::new();
    for part in text
        .split([';', '\n'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        let (days, hours) = part.rsplit_once(' ')?;
        let (from, to) = hours.split_once('-')?;
        let mut weekdays: Vec<Weekday> = Vec::new();
        for piece in days.split(',') {
            match piece.split_once('-') {
                Some((a, b)) => {
                    let (a, b) = (day_named(a)?.index(), day_named(b)?.index());
                    weekdays.extend(DAY_NAMES.iter().take(b + 1).skip(a).map(|(_, d)| *d));
                }
                None => weekdays.push(day_named(piece)?),
            }
        }
        if weekdays.is_empty() {
            return None;
        }
        out.push(WeekWindow::new(
            &weekdays,
            parse_clock(from)?,
            parse_clock(to)?,
        ));
    }
    (!out.is_empty()).then_some(out)
}

pub fn format_windows(windows: &[WeekWindow]) -> String {
    let clock = |m: u16| format!("{:02}:{:02}", m / 60, m % 60);
    windows
        .iter()
        .map(|w| {
            let days = w
                .days
                .iter()
                .map(|d| DAY_NAMES[d.index()].0)
                .collect::<Vec<_>>()
                .join(",");
            format!("{days} {}-{}", clock(w.start_minute), clock(w.end_minute))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

// ------------------------------------------------------ default values

/// `true` when `kind` keeps its secrets in a field the saved recipe drops.
pub fn holds_secret(kind: &RuleKind) -> bool {
    matches!(
        kind,
        RuleKind::ContentReplace(ContentReplace {
            find: ContentFind::Secrets { .. },
            ..
        })
    )
}

/// Fresh, still-unfilled kinds the Add menu offers (see `model::RuleTemplate`).
pub fn blank_identity() -> RuleKind {
    RuleKind::Identity(IdentityRule {
        when: IdentityMatch::default(),
        set: IdentitySet::default(),
        apply_to: Who::Both,
        include_taggers: true,
        map_trailers: false,
    })
}

pub fn blank_anonymise() -> RuleKind {
    RuleKind::Anonymise(AnonymiseRule {
        keep: Vec::new(),
        name_pattern: "Contributor {n}".into(), // i18n-english: written into commits, like the core default
        email_pattern: "contributor-{n}@anonymous.invalid".into(),
        apply_to: Who::Both,
        include_taggers: false,
    })
}

pub fn blank_window() -> RuleKind {
    RuleKind::DateWindow(WorkWindowRule {
        windows: vec![WeekWindow::new(&Weekday::WEEKDAYS, 9 * 60, 18 * 60)],
        offset: WindowOffset::Commit,
        direction: Direction::Nearest,
        keep_order: true,
        min_gap_seconds: 60,
        seed: 0,
        apply_to: WhichDate::Both,
    })
}

pub fn blank_message_replace() -> RuleKind {
    RuleKind::MessageReplace(TextReplace {
        find: TextPattern {
            pattern: String::new(),
            regex: false,
            case_sensitive: true,
        },
        with: String::new(),
    })
}

pub fn blank_trailer() -> RuleKind {
    RuleKind::Trailer(TrailerRule::add("Signed-off-by", ""))
}

pub fn blank_content() -> RuleKind {
    RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Pattern {
            pattern: TextPattern {
                pattern: String::new(),
                regex: false,
                case_sensitive: true,
            },
        },
        with: String::new(),
        max_blob_bytes: None,
    })
}

pub fn blank_lfs_import() -> RuleKind {
    RuleKind::LfsImport(LfsImportRule {
        patterns: Vec::new(),
        min_size: None,
    })
}

/// Linux to Windows is the one people ask for.
pub fn blank_line_endings() -> RuleKind {
    RuleKind::LineEndings(EolRule {
        target: LineEnding::Crlf,
        paths: Vec::new(),
        exclude: Vec::new(),
        follow_attributes: false,
        max_blob_bytes: None,
    })
}

pub fn blank_strip_exec() -> RuleKind {
    RuleKind::StripExec(StripExecRule {
        paths: Vec::new(),
        exclude: Vec::new(),
    })
}

pub fn blank_reencode() -> RuleKind {
    RuleKind::Reencode(ReencodeRule {
        target: TextEncoding::Utf8,
        from: None,
        fallback: TextEncoding::Windows1252,
        paths: Vec::new(),
        exclude: Vec::new(),
        max_blob_bytes: None,
    })
}

/// The size limit a field holds: empty is the default.
fn max_blob(text: Option<&str>, bad: &dyn Fn(M) -> String) -> Result<Option<u64>, String> {
    let s = text.unwrap_or("").trim();
    if s.is_empty() {
        Ok(None)
    } else {
        parse_size(s).map(Some).ok_or_else(|| bad(t::E_SIZE))
    }
}

pub fn blank_lfs_export() -> RuleKind {
    RuleKind::LfsExport(LfsExportRule {
        patterns: Vec::new(),
    })
}

/// What a new Script rule starts with: a header, the one function the
/// engine calls, and a body that does something visible so "Test on
/// selection" shows a result at once.
// Lua source, not interface text:
// i18n-scan: off
pub const STARTER_SCRIPT: &str = "\
--! name: Uppercase author names
--! kinds: commit
--! window: 0
--! param suffix: text \"Text added to each message\" = \"\"
-- Called once per commit. Return only what changes: author, committer, message.
function rewrite_commit(c, ctx)
  return {
    author = { name = string.upper(c.author.name) },
    message = c.message .. ctx.params.suffix,
  }
end
";

/// The starter of a POSIX shell script: one JSON request per line on stdin,
/// one JSON answer per line on stdout, diagnostics on stderr.
pub const STARTER_SHELL: &str = "\
#!/bin/sh
#! name: Uppercase author names
#! kinds: commit
# Not sandboxed: this runs with your rights. One JSON request per line on
# stdin, one JSON answer per line on stdout; print diagnostics on stderr.
while IFS= read -r line; do
  name=$(printf '%s' \"$line\" | sed -n 's/.*\"author\":{[^}]*\"name\":\"\\([^\"]*\\)\".*/\\1/p' | tr 'a-z' 'A-Z')
  printf '{\"author\":{\"name\":\"%s\"}}\\n' \"$name\"
done
";

/// The starter of a PowerShell script (the shell on Windows).
pub const STARTER_POWERSHELL: &str = "\
#! name: Uppercase author names
#! kinds: commit
# Not sandboxed: this runs with your rights. One JSON request per line on
# stdin, one JSON answer per line on stdout; print diagnostics on stderr.
while ($null -ne ($line = [Console]::In.ReadLine())) {
  $request = $line | ConvertFrom-Json
  $answer = @{ author = @{ name = $request.commit.author.name.ToUpper() } }
  [Console]::Out.WriteLine(($answer | ConvertTo-Json -Compress))
}
";

pub const STARTER_PYTHON: &str = "\
#!/usr/bin/env python3
#! name: Uppercase author names
#! kinds: commit
# Not sandboxed: this runs with your rights. One JSON request per line on
# stdin, one JSON answer per line on stdout; print diagnostics on stderr.
import json
import sys

for line in sys.stdin:
    request = json.loads(line)
    name = request[\"commit\"][\"author\"][\"name\"]
    print(json.dumps({\"author\": {\"name\": name.upper()}}), flush=True)
";

pub const STARTER_LUA_EXTERNAL: &str = "\
--! name: Uppercase author names
--! kinds: commit
-- Not sandboxed: this runs with your rights. A stock Lua has no JSON library,
-- so this reads the line with a pattern. One JSON answer per line on stdout.
for line in io.lines() do
  local name = line:match('\"author\":{[^}]*\"name\":\"([^\"]*)\"')
  io.write('{\"author\":{\"name\":\"', (name or ''):upper(), '\"}}\\n')
  io.flush()
end
";
// i18n-scan: on

/// What a script of `language` starts with.
pub fn starter_for(language: ScriptLanguage) -> &'static str {
    match language {
        ScriptLanguage::Lua => STARTER_SCRIPT,
        ScriptLanguage::Shell if cfg!(windows) => STARTER_POWERSHELL,
        ScriptLanguage::Shell => STARTER_SHELL,
        ScriptLanguage::Python => STARTER_PYTHON,
        ScriptLanguage::LuaExternal => STARTER_LUA_EXTERNAL,
    }
}

pub fn blank_script() -> RuleKind {
    RuleKind::Script(ScriptRef {
        language: ScriptLanguage::Lua,
        source: ScriptSource::Inline {
            text: STARTER_SCRIPT.into(),
        },
        params: Default::default(),
    })
}

/// The header a Script rule declares: an inline script's own, parsed from its
/// text; a file's as the daemon's check last returned it.
pub fn script_header(s: &ScriptRef, known: Option<&ScriptHeader>) -> Option<ScriptHeader> {
    match &s.source {
        ScriptSource::Inline { text } => parse_header_for(s.language, text).ok(),
        ScriptSource::File { .. } => known.cloned(),
    }
}

/// The value a parameter row shows: the rule's, else the header's default.
pub fn param_value(
    s: &ScriptRef,
    p: &gitpoulpi_core::domain::rewrite::script::ScriptParam,
) -> String {
    s.params
        .get(&p.name)
        .cloned()
        .or_else(|| p.default.clone())
        .unwrap_or_else(|| match p.kind {
            ParamKind::Bool => "false".into(),
            _ => String::new(),
        })
}
