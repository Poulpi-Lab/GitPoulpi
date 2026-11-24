use super::*;
use crate::chrome::HELP_TEXT;

/// The help overlay must cover every keybinding, and
/// drift between it and the palette's own catalogue is exactly the
/// kind of gap that's invisible until an owner goes looking for a
/// chord and finds nothing. A byte-for-byte check across every one
/// of the catalogue's 140-odd shortcuts isn't feasible - many are
/// deliberately bare single letters (`s`, `q`, `m`, ...) that would
/// "match" any help line trivially and prove nothing - but every
/// `Ctrl+...` chord is distinctive enough to be a real assertion:
/// if `HELP_TEXT` doesn't contain the literal chord anywhere, the
/// help overlay has no entry for it. A shortcut listing several
/// alternatives (`"Ctrl+, / Ctrl+N ,"`) passes if *any* alternative
/// is documented - the other one doesn't need its own separate line.
#[test]
fn every_ctrl_chord_shortcut_has_a_matching_help_line() {
    let _ = HELP_TEXT;
    let flattened = crate::chrome::filter_help_text_with(&crate::keymap::Keymap::defaults(), "");
    for e in PALETTE_COMMANDS {
        // A command of a feature this build lacks has no help row either.
        if !crate::build_features::offered(e.requires)
            || !crate::build_features::offered(crate::palette::command_requires(&e.command))
        {
            continue;
        }
        for alt in e.shortcut.split(" / ") {
            let alt = alt.trim();
            if !alt.contains("Ctrl+") {
                continue;
            }
            let documented = e
                .shortcut
                .split(" / ")
                .map(str::trim)
                .any(|candidate| flattened.contains(candidate));
            assert!(
                documented,
                "`{}`'s shortcut `{}` (chord `{alt}`) has no matching line in \
                 chrome::HELP_TEXT - add one to the relevant section",
                e.label.get(),
                e.shortcut
            );
            break;
        }
    }
}

/// Pins the registry labels: every palette
/// shortcut naming a Ctrl+G / Ctrl+N second key, or one of the global
/// window chords, equals the registry's default label for some action
/// (`Shift+P` normalized to the `P` the palette prints).
#[test]
fn palette_shortcut_labels_agree_with_registry() {
    use crate::keymap::{Keymap, ACTIONS};
    let km = Keymap::defaults();
    let normalise = |label: String| -> String {
        match label.rsplit_once("Shift+") {
            Some((head, key)) if key.chars().count() == 1 => format!("{head}{key}"),
            _ => label,
        }
    };
    let known: Vec<String> = ACTIONS
        .iter()
        .flat_map(|d| km.labels(d.action))
        .map(normalise)
        .collect();
    for e in PALETTE_COMMANDS {
        for alt in e.shortcut.split(" / ").map(str::trim) {
            let names_key = alt
                .strip_prefix("Ctrl+G ")
                .or_else(|| alt.strip_prefix("Ctrl+N "))
                .is_some_and(|k| !k.is_empty() && k != "<badge>");
            if names_key {
                assert!(
                    known.iter().any(|k| k == alt),
                    "`{}`: palette shortcut `{alt}` is not a registry default",
                    e.label.get()
                );
            }
        }
    }
}

fn km_with(entries: &[(&str, &str, &[&str])]) -> crate::keymap::Keymap {
    crate::keymap::Keymap::from_config(&gitpoulpi_config::KeybindingsConfig::from_entries(
        entries
            .iter()
            .map(|(c, a, v)| (*c, *a, v.iter().map(|s| s.to_string()).collect())),
    ))
}

/// Leader commands show the full chord, never the bare prefix,
/// including after a rebinding.
#[test]
fn leader_rows_show_the_full_chord_not_the_bare_prefix() {
    use crate::keymap::Keymap;
    let km = Keymap::defaults();
    let rebound = km_with(&[
        ("global", "nav_prefix", &["Alt+N"]),
        ("global", "git_prefix", &["Ctrl+B"]),
    ]);
    for e in PALETTE_COMMANDS {
        let shown = entry_shortcut(&km, e);
        assert!(
            !matches!(shown.as_str(), "Ctrl+N" | "Ctrl+G"),
            "`{}` shows a bare prefix",
            e.label.get()
        );
    }
    let focus = PALETTE_COMMANDS
        .iter()
        .find(|e| e.label.get() == "Navigation: Focus Log Pane")
        .unwrap();
    assert_eq!(entry_shortcut(&km, focus), "Ctrl+N <badge>");
    assert_eq!(entry_shortcut(&rebound, focus), "Alt+N <badge>");
    let collapse = PALETTE_COMMANDS
        .iter()
        .find(|e| e.label.get() == "Navigation: Toggle Refs Pane Collapsed")
        .unwrap();
    assert_eq!(entry_shortcut(&km, collapse), "Ctrl+N H");
    assert_eq!(entry_shortcut(&rebound, collapse), "Alt+N H");
    let stage = PALETTE_COMMANDS
        .iter()
        .find(|e| e.label.get().contains("Stage Selected"))
        .unwrap();
    assert_eq!(entry_shortcut(&rebound, stage), "Ctrl+B s");
}

#[test]
fn palette_rows_show_the_effective_binding() {
    use crate::keymap::Keymap;
    let stage = PALETTE_COMMANDS
        .iter()
        .find(|e| e.label.get().contains("Stage Selected"))
        .expect("a stage-selected row");
    let km = km_with(&[("global", "git_prefix", &["Ctrl+B"])]);
    assert_eq!(effective_shortcut(&km, stage.shortcut), "Ctrl+B s");
    // falsified by the defaults
    assert_eq!(
        effective_shortcut(&Keymap::defaults(), stage.shortcut),
        "Ctrl+G s"
    );
    // Descriptions are not keys: left alone.
    assert_eq!(
        effective_shortcut(&km, "Enter on the search row"),
        "Enter on the search row"
    );
}

/// No palette row still shows the old prefix once it is rebound, and
/// an unbound action shows nothing.
#[test]
fn no_palette_row_keeps_the_old_prefixes() {
    let km = km_with(&[
        ("global", "git_prefix", &["Ctrl+B"]),
        ("global", "nav_prefix", &["Alt+N"]),
        ("global", "palette", &[]),
    ]);
    for e in PALETTE_COMMANDS
        .iter()
        .filter(|e| crate::build_features::offered(e.requires))
    {
        let shown = effective_shortcut(&km, e.shortcut);
        assert!(
            !shown.contains("Ctrl+G")
                && !shown.starts_with("Ctrl+N")
                && !shown.split(" / ").any(|a| a == "Ctrl+P"),
            "`{}` still shows `{shown}`",
            e.label.get()
        );
    }
}

/// Actions of the palette-reachable contexts that have no row of their
/// own showing their key, each with the reason. Everything else must be
/// shown by some palette row: a new action without one fails the test
/// below instead of silently missing from the place the owner learns keys.
const NO_OWN_PALETTE_ROW: &[crate::keymap::Action] = {
    use crate::keymap::Action as A;
    &[
        // The chords that open the palette / the prefixes: nothing to run.
        A::GlobalGitPrefix,
        A::GlobalNavPrefix,
        A::GlobalPalette,
        // Second spellings of an action whose row shows the first.
        A::NavToggleHelp,
        A::NavSelectTab,
        A::MergeSideUnpick,
        A::MergeSideResetAll,
        A::MergeSidePrevConflict,
        A::MergeSideToggleBase,
        // Rows whose text names the key together with its context.
        A::StatusOpenDiff,
        A::HomeInit,
        A::DiffMergeEditor,
        // Cursor movement, no command.
        A::FileInsightLastRow,
        // A toggle whose two halves have their own rows (Git: Stage / Unstage
        // Selected File, Ctrl+G s / u).
        A::StatusStageToggle,
    ]
};

/// Every bindable action of a pane, view or global context is shown, as
/// its effective key, by some palette row - under the defaults and after
/// a rebinding (the keymap is the source, not a copied string).
#[test]
fn every_bindable_action_with_a_palette_row_shows_its_key() {
    use crate::keymap::{Ctx, Keymap, ACTIONS};
    let shown_by_rows = |km: &Keymap| -> Vec<String> {
        PALETTE_COMMANDS
            .iter()
            .flat_map(|e| {
                entry_shortcut(km, e)
                    .split(" / ")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let km = Keymap::defaults();
    let shown = shown_by_rows(&km);
    for d in ACTIONS.iter().filter(|d| d.built()) {
        // Palette commands ship unbound; `palette_bindings` covers them.
        if d.ctx == Ctx::Palette
            || d.ctx.is_popup()
            || matches!(
                d.ctx,
                Ctx::Prompt | Ctx::Editor | Ctx::Composer | Ctx::Dialog | Ctx::FolderPicker
            )
            || NO_OWN_PALETTE_ROW.contains(&d.action)
        {
            continue;
        }
        let labels = km.shown_labels(d.action);
        assert!(
            labels.iter().any(|l| shown.contains(l)),
            "no palette row shows `{}` ({:?}): give its command an entry in \
             `command_action` (or a catalogue shortcut), or list the action in \
             NO_OWN_PALETTE_ROW with a reason",
            d.id,
            labels
        );
    }
}

/// "Tab: Open in" and "Tab: Open in Default" show their keys, and a
/// rebinding shows up in the palette.
#[test]
fn open_in_rows_follow_the_keymap() {
    let row = |c: PaletteCommand| PALETTE_COMMANDS.iter().find(|e| e.command == c).unwrap();
    let popup = row(PaletteCommand::TabOpenIn);
    let default = row(PaletteCommand::TabOpenInDefault);
    let km = crate::keymap::Keymap::defaults();
    assert_eq!(entry_shortcut(&km, popup), "Alt+O");
    assert_eq!(entry_shortcut(&km, default), "Ctrl+Alt+O");
    let km = km_with(&[
        ("global", "tab_open_in_popup", &["Alt+Y"]),
        ("global", "tab_open_in_default", &[]),
    ]);
    assert_eq!(entry_shortcut(&km, popup), "Alt+Y");
    assert_eq!(entry_shortcut(&km, default), "");
}

/// Every Ctrl+G / Ctrl+N second-key shortcut of the catalogue is
/// recognized as an action (so it follows a rebinding).
#[test]
fn every_prefix_shortcut_in_the_catalogue_is_an_action() {
    let km = km_with(&[("global", "git_prefix", &["Ctrl+B"])]);
    for e in PALETTE_COMMANDS
        .iter()
        .filter(|e| crate::build_features::offered(e.requires))
    {
        for alt in e.shortcut.split(" / ").map(str::trim) {
            if alt.starts_with("Ctrl+G ") {
                assert!(
                    effective_shortcut(&km, alt).starts_with("Ctrl+B "),
                    "`{}`: `{alt}` is not tied to an action",
                    e.label.get()
                );
            }
        }
    }
}

/// A multi-bound action shows every binding in the palette, not only the
/// first: "Tab: next" lists Ctrl+PageDown and Ctrl+Tab.
#[test]
fn tab_cycle_rows_show_every_binding() {
    let row = |c: PaletteCommand| PALETTE_COMMANDS.iter().find(|e| e.command == c).unwrap();
    let km = crate::keymap::Keymap::defaults();
    assert_eq!(
        entry_shortcut(&km, row(PaletteCommand::CycleRepositoryTab(true))),
        "Ctrl+PageDown / Ctrl+Tab"
    );
    assert_eq!(
        entry_shortcut(&km, row(PaletteCommand::CycleRepositoryTab(false))),
        "Ctrl+PageUp / Ctrl+Shift+Tab"
    );
}
