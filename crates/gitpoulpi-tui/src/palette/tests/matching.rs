use super::*;

/// An empty query lists everything, in catalogue order, with the
/// currently-unavailable ones pushed to the end.
#[test]
fn an_empty_query_lists_every_command() {
    let ai_on = PaletteContext {
        ai_enabled: true,
        ..PaletteContext::default()
    };
    let all = palette_matches("", ai_on);
    let built = PALETTE_COMMANDS
        .iter()
        .filter(|e| crate::build_features::offered(e.requires))
        .count();
    assert_eq!(all.len(), built + settings_listed_count());
}

/// AI is off by default and every AI entry point is *hidden* (not
/// grayed out) while it is.
#[cfg(feature = "ai")]
#[test]
fn ai_commands_are_hidden_while_ai_is_off() {
    let off = palette_matches("", PaletteContext::default());
    assert!(off.iter().all(|m| !m.entry.command.is_ai()));
    assert!(off.len() < PALETTE_COMMANDS.len() + settings_listed_count());
    let on = palette_matches(
        "generate commit",
        PaletteContext {
            ai_enabled: true,
            ..PaletteContext::default()
        },
    );
    assert_eq!(on[0].entry.command, PaletteCommand::AiGenerateCommitMessage);
    assert!(on[0].unavailable.is_none());
}

/// The case that motivated the three-tier scoring: a short, exact
/// command must not be buried under longer ones that merely contain
/// the same letters.
#[test]
fn a_short_exact_command_outranks_longer_incidental_matches() {
    let matches = palette_matches("push", PaletteContext::default());
    // The catalogue's ranking; a setting that mentions "push" is always
    // available and is not what this test is about.
    let labels: Vec<&str> = matches
        .iter()
        .filter(|m| !matches!(m.entry.command, PaletteCommand::Setting(_)))
        .map(|m| m.entry.name())
        .collect();
    assert_eq!(
        labels.first().copied(),
        Some("Git: Push"),
        "got: {labels:?}"
    );
    // "Git: Stash All Changes" has no "push" substring, so the stash
    // push command is found by its label, not by accident.
    assert!(labels.contains(&"Git: Push"));
}

/// Typing a category name gathers that whole group.
#[test]
fn typing_a_category_gathers_its_group() {
    let matches = palette_matches("stash", PaletteContext::default());
    let labels: Vec<&str> = matches.iter().map(|m| m.entry.name()).collect();
    for expected in [
        "Git: Stash All Changes",
        "Git: Stash Pop",
        "Git: Stash Apply",
        "Git: Stash Drop...",
    ] {
        assert!(
            labels.contains(&expected),
            "missing {expected} in {labels:?}"
        );
    }
}

/// Available commands sort ahead of unavailable ones, so Enter on a
/// fresh query runs something rather than reporting a reason.
#[test]
fn available_commands_sort_before_unavailable_ones() {
    let ctx = PaletteContext::default();
    let matches = palette_matches("diff", ctx);
    let first_unavailable = matches.iter().position(|m| m.unavailable.is_some());
    let last_available = matches.iter().rposition(|m| m.unavailable.is_none());
    if let (Some(fu), Some(la)) = (first_unavailable, last_available) {
        assert!(
            la < fu,
            "an unavailable command sorted before an available one"
        );
    }
}

/// Guards the subsequence tier against matching everything.
#[test]
fn a_query_that_matches_nothing_returns_nothing() {
    assert!(palette_matches("zzzqqq", PaletteContext::default()).is_empty());
}

/// The owner searched "settings" and found nothing: the screen is
/// called Preferences in code and Settings in the UI; the UI word
/// must reach it, and it must rank first.
#[test]
fn settings_finds_the_settings_screen() {
    let found = palette_matches("settings", PaletteContext::default());
    assert!(matches!(
        found.first().map(|m| &m.entry.command),
        Some(PaletteCommand::OpenPreferences)
    ));
}

/// The screen used to be called "Preferences"; that word still finds it, in
/// English and in any other language.
#[test]
fn preferences_still_finds_the_settings_screen() {
    use gitpoulpi_i18n::{override_lang, Lang};
    for lang in [Lang::En, Lang::Fr, Lang::Ja] {
        let _l = override_lang(lang);
        let found = palette_matches("preferences", PaletteContext::default());
        assert!(
            found
                .iter()
                .any(|m| m.entry.command == PaletteCommand::OpenPreferences),
            "{lang:?}"
        );
    }
}

/// CJK catalogs: a query typed in the language finds its command, whether the
/// category is followed by an ASCII or a full-width colon.
#[test]
fn cjk_queries_find_their_commands() {
    use gitpoulpi_i18n::{override_lang, Lang};
    let ctx = PaletteContext {
        ai_enabled: true,
        ..PaletteContext::default()
    };
    for lang in [Lang::ZhCn, Lang::Ja, Lang::Ko] {
        let _l = override_lang(lang);
        let mut checked = 0;
        for e in PALETTE_COMMANDS
            .iter()
            .filter(|e| crate::build_features::offered(e.requires))
            .step_by(7)
        {
            let label = e.display_label().to_string();
            let body = label
                .split_once([':', '\u{ff1a}'])
                .map_or(label.as_str(), |(_, rest)| rest.trim_start());
            let query: String = body
                .chars()
                .filter(|c| !c.is_whitespace())
                .take(2)
                .collect();
            if query.chars().count() < 2 {
                continue;
            }
            let found = palette_matches(&query, ctx);
            assert!(
                found.iter().any(|m| m.entry.command == e.command),
                "{}: {query:?} does not find {label:?}",
                lang.code()
            );
            checked += 1;
        }
        assert!(
            checked > 10,
            "{}: only {checked} labels checked",
            lang.code()
        );
    }
}

/// The category prefix ends at a full-width colon too.
#[test]
fn the_category_may_end_with_a_full_width_colon() {
    let in_body = score(
        "\u{5206}\u{652f}\u{ff1a}\u{65b0}\u{5efa}",
        "\u{65b0}\u{5efa}",
    )
    .unwrap();
    let in_category = score(
        "\u{5206}\u{652f}\u{ff1a}\u{65b0}\u{5efa}",
        "\u{5206}\u{652f}",
    )
    .unwrap();
    assert!(in_body > 29_000, "{in_body}");
    assert!(in_category < 29_000, "{in_category}");
}

#[test]
fn a_tagged_entry_is_left_out_when_its_feature_is_not_built() {
    use crate::build_features::{offered_with, Feature};
    let tagged =
        entry(PALETTE_COMMANDS[0].label, "", PaletteCommand::FocusRefs).requires(Feature::Ai);
    assert_eq!(tagged.requires, Some(Feature::Ai));
    assert!(offered_with(tagged.requires, |f| f == Feature::Ai));
    assert!(!offered_with(tagged.requires, |_| false));
    // Untagged entries are listed in every build.
    assert!(PALETTE_COMMANDS[0].requires.is_none());
}

/// Every worktree / submodule command carries its feature tag, so a build
/// without the feature lists none of them (and Focus Refs: Worktrees /
/// Submodules go with them).
#[test]
fn worktree_and_submodule_commands_are_tagged() {
    use crate::build_features::{offered, Feature};
    use PaletteCommand as C;
    let worktree = [
        C::CreateWorktreeFromSelectedRef,
        C::OpenWorktreeInTab,
        C::WorktreeOpenInSelected,
        C::WorktreeMergeSelected,
        C::WorktreeRemoveSelected,
        C::CreateWorktreeMenu,
        C::CreateWorktreeFromCurrentBranch,
        C::CreateWorktreeChooseBranch,
        C::ShowWorktreesSection,
        C::CreateWorktreeAtSelectedCommit,
        C::FocusRefsSection(SidebarSection::Worktrees),
    ];
    let submodule = [
        C::SubmoduleUpdateSelected,
        C::SubmoduleInitSelected,
        C::SubmoduleOpenSelected,
        C::SubmoduleUpdateAll,
        C::FocusRefsSection(SidebarSection::Submodules),
    ];
    for (commands, feature) in [
        (&worktree[..], Feature::Worktree),
        (&submodule[..], Feature::Submodule),
    ] {
        for command in commands {
            let entries: Vec<_> = PALETTE_COMMANDS
                .iter()
                .filter(|e| e.command == *command)
                .collect();
            assert!(!entries.is_empty(), "{command:?} is in the catalogue");
            for e in entries {
                assert_eq!(e.requires, Some(feature), "{command:?}");
                assert_eq!(offered(e.requires), feature.built(), "{command:?}");
            }
        }
    }
}
