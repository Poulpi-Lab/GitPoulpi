//! The history rewrite entry points and rule kinds.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// One kind the Add menu offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTemplate {
    Identity,
    Mailmap,
    Anonymise,
    DateShift,
    DateTimezone,
    DateWindow,
    CommitterFromAuthor,
    MessageReplace,
    MessageDeleteLines,
    Trailer,
    ContentReplace,
    RemovePaths,
    RemoveBlobsOver,
    MovePaths,
    LfsImport,
    LfsExport,
    Script,
    RemoveSignatures,
    LineEndings,
    StripExec,
    Reencode,
    MessageEncoding,
}

/// What a palette row or a menu entry asks the workbench to open with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteEntry {
    /// The workbench as the chord opens it.
    Workbench,
    /// With one rule of this kind added and its fields focused.
    Rule(RuleTemplate),
    /// With the Log's selected commits as the scope.
    SelectedCommits,
    /// With the selected commit and everything after it as the scope.
    FromSelectedCommit,
    /// On the purge screen: delete the old history of a rewrite for good.
    Purge,
    /// On the Identities tab: merge names and emails through `.mailmap`.
    Identities,
    /// A new Script rule, its editor and the ✦ "Write with AI" dialog.
    ScriptAi,
    /// The ✦ "Create Rules with AI" dialog over the workbench.
    RulesAi,
}
