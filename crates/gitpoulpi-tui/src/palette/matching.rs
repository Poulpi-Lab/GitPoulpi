use super::*;

/// One row of the filtered list the popup shows.
pub(crate) struct PaletteMatch {
    pub(crate) entry: &'static PaletteEntry,
    pub(crate) unavailable: Option<&'static str>,
}

/// Score `label` against `query`, higher is better, `None` for no match.
///
/// The ranking is `gitpoulpi_core::fuzzy` (shared with the workspace search):
/// tier 1 is the query verbatim *after* the `Category: ` prefix, tier 2
/// verbatim anywhere, tier 3 the characters in order but apart; within a
/// tier an earlier position and a shorter label win.
pub(crate) fn score(label: &str, query: &str) -> Option<i32> {
    gitpoulpi_core::fuzzy::score(label, query, gitpoulpi_core::fuzzy::Anchor::AfterCategory)
}

/// [`score`] over an already lower-cased label (the palette caches these).
pub(crate) fn score_lower(lower: &str, query: &str) -> Option<i32> {
    gitpoulpi_core::fuzzy::score_lower(lower, query, gitpoulpi_core::fuzzy::Anchor::AfterCategory)
}

/// One entry's searchable text: the label in the current language and, when
/// that differs, in English, both lower-cased once.
struct Haystack {
    current: String,
    english: Option<String>,
}

/// Extra English words a command is found by, whatever the language: the
/// screen was called "Preferences" before the UI word became "Settings", and
/// people still type it.
fn search_alias(command: &PaletteCommand) -> &'static str {
    match command {
        PaletteCommand::OpenPreferences => "preferences",
        // The command was `Git: Set Upstream...`: the old name still finds it.
        PaletteCommand::SetUpstream => "git: set upstream", // i18n-english
        _ => "",
    }
}

struct HaystackCache {
    key: (u64, u8, bool, bool, u64),
    rows: Vec<Haystack>,
}

thread_local! {
    static HAYSTACKS: std::cell::RefCell<Option<HaystackCache>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `f` over the per-entry haystacks, built once per language epoch
/// (a switch of language, git vocabulary or host) rather than per keystroke.
/// The key also holds the resolved language and vocabulary bits, so a test's
/// thread-local overrides can never reuse another thread state's text.
fn with_haystacks<R>(seq: &'static [PaletteEntry], f: impl FnOnce(&[Haystack]) -> R) -> R {
    let key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        gitpoulpi_i18n::git_terms_translated(),
        gitpoulpi_i18n::merge_terms(),
        flow::signature() ^ jobs::signature() ^ (seq.as_ptr() as usize as u64).rotate_left(32),
    );
    HAYSTACKS.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.as_ref().is_none_or(|c| c.key != key) {
            let rows = PALETTE_COMMANDS
                .iter()
                .chain(settings::entries())
                .chain(flow::entries())
                .chain(jobs::entries())
                .chain(seq)
                .map(|e| {
                    let current = e.display_label().to_lowercase();
                    let en = e.english().to_lowercase();
                    let alias = search_alias(&e.command);
                    let english = if alias.is_empty() {
                        (!en.is_empty() && !current.contains(&en)).then_some(en)
                    } else {
                        Some(format!("{en} {alias}"))
                    };
                    Haystack { current, english }
                })
                .collect();
            *slot = Some(HaystackCache { key, rows });
        }
        f(&slot.as_ref().expect("filled above").rows)
    })
}

/// The palette's filtered, ranked rows for `query` in the given context.
///
/// Unavailable commands are kept (grayed out by the caller) rather than
/// filtered out, but they sort after available ones at equal relevance so
/// the thing you can actually run lands under the cursor first.
pub(crate) fn palette_matches(query: &str, ctx: PaletteContext) -> Vec<PaletteMatch> {
    // A sequence added outside the window: before anything is keyed on it.
    sequences::sync();
    let query = query.trim().to_lowercase();
    let reason_of = |e: &PaletteEntry| {
        // On Home the missing repository is the real reason, whatever
        // selection-based reason the command would otherwise give.
        (ctx.in_repository_management && needs_repository(e.english()))
            .then_some(msg::palette::WHY_NEEDS_REPOSITORY.get())
            .or_else(|| unavailable_reason(e.command, ctx))
    };
    let prefixes = settings::prefixes();
    // One snapshot per call: the rows and the haystacks must be the same list.
    let seq = sequences::entries();
    let mut scored: Vec<(i32, bool, usize, &'static PaletteEntry)> = with_haystacks(seq, |rows| {
        PALETTE_COMMANDS
            .iter()
            .chain(settings::entries())
            .chain(flow::entries())
            .chain(jobs::entries())
            .chain(seq)
            .enumerate()
            .filter(|(_, e)| settings::listed(e, &query, prefixes))
            .filter(|(_, e)| ctx.ai_enabled || !e.command.needs_ai())
            // A command of a feature this build lacks is not offered at all.
            .filter(|(_, e)| {
                crate::build_features::offered(e.requires)
                    && crate::build_features::offered(super::command_requires(&e.command))
            })
            // Turned off, "Edit: Undo" and "Edit: Redo" are not offered.
            .filter(|(_, e)| {
                !(matches!(
                    e.command,
                    PaletteCommand::Undo
                        | PaletteCommand::Redo
                        | PaletteCommand::UndoHistory
                        | PaletteCommand::ReflogHead
                        | PaletteCommand::LostCommits
                        | PaletteCommand::UndoOffer
                        | PaletteCommand::Safety(_)
                ) && ctx.undo == crate::undo_ui::Avail::Off)
            })
            .filter_map(|(i, e)| {
                // A query finds the entry by its label in either language.
                let h = &rows[i];
                let a = score_lower(&h.current, &query);
                let b = h.english.as_deref().and_then(|en| score_lower(en, &query));
                // A generated settings label is long: a scattered subsequence
                // match on one is noise, so a setting must contain the query (tier 1 or 2).
                a.max(b)
                    .filter(|s| {
                        !matches!(e.command, PaletteCommand::Setting(_))
                            || *s > 10_000
                            || query.is_empty()
                    })
                    .map(|s| {
                        // A setting ranks below every catalogue command that
                        // matches as well as it does: "graph color theme"
                        // must keep landing on the command, not its setting.
                        let s = if matches!(e.command, PaletteCommand::Setting(_))
                            && !query.is_empty()
                        {
                            s - 12_000
                        } else {
                            s
                        };
                        (s, reason_of(e).is_some(), i, e)
                    })
            })
            .collect()
    });
    // Available before unavailable, then by score, then by catalogue order
    // so the grouping stays stable and predictable for an empty query.
    scored.sort_by_key(|(s, unavailable, i, _)| (*unavailable, -*s, *i));
    scored
        .into_iter()
        .map(|(_, _, _, entry)| PaletteMatch {
            entry,
            unavailable: reason_of(entry),
        })
        .collect()
}

/// Whether a command acts on an open repository (its graph, refs, working
/// tree or remotes). On the Home tab there is none, so these are grayed
/// out with "needs an open repository" rather than offered as live.
pub(crate) fn needs_repository(label: &str) -> bool {
    let (category, rest) = label.split_once(':').unwrap_or((label, ""));
    // i18n-scan: off
    match category {
        "Navigation" | "Refs" | "Git" | "Edit" | "Commit" | "Commit Details" | "Diff"
        | "Working Changes" | "Files" | "File" | "Copy" | "Composer" | "LFS" | "Remote"
        | "Pull Requests" | "Merge Requests" | "Issues" | "Flow" => true,
        "Integrations" => {
            !(rest.contains("Connect Integration") || rest.contains("Review security check"))
        }
        "Repository" => rest.contains("SSH & Credentials") || rest.contains("Sparse Checkout"),
        _ => false,
    }
    // i18n-scan: on
}
