use super::*;
use crate::keymap::{Action, Ctx, Keymap, ACTIONS};
use std::collections::HashSet;

fn km_with(entries: &[(&str, &str, &[&str])]) -> Keymap {
    Keymap::from_config(&gitpoulpi_config::KeybindingsConfig::from_entries(
        entries
            .iter()
            .map(|(c, a, v)| (*c, *a, v.iter().map(|s| s.to_string()).collect())),
    ))
}

fn row(command: PaletteCommand) -> &'static PaletteEntry {
    PALETTE_COMMANDS
        .iter()
        .find(|e| e.command == command)
        .expect("in the catalogue")
}

/// Every row has something to bind: a registered action or its own
/// `palette.<id>` one, so the key button shows on all of them.
#[test]
fn every_palette_row_has_a_bindable_action() {
    for e in PALETTE_COMMANDS {
        assert!(
            bind_action(e).is_some(),
            "`{}` has nothing to bind",
            e.label.get()
        );
    }
}

#[test]
fn command_ids_are_unique_and_round_trip() {
    let mut seen = HashSet::new();
    for e in PALETTE_COMMANDS {
        let id = command_id(&e.command);
        assert!(!id.is_empty());
        assert!(
            id.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "{id}"
        );
        assert!(seen.insert(id.clone()), "two commands share the id {id}");
    }
    let defs: Vec<_> = ACTIONS
        .iter()
        .filter(|d| matches!(d.action, Action::Palette(_)))
        .collect();
    assert!(defs.len() > 100, "{} palette actions", defs.len());
    for d in defs {
        let entry = palette_entry(d.action).expect("an index into the catalogue");
        assert_eq!(d.id, format!("palette.{}", command_id(&entry.command)));
        // The entry finds its own action again.
        assert_eq!(bind_action(entry), Some(d.action));
    }
}

/// The ids are the config file's vocabulary: renaming one silently drops
/// the user's binding, so a few of every shape are pinned.
#[test]
fn command_ids_are_pinned() {
    assert_eq!(command_id(&PaletteCommand::ToggleMinimap), "toggle_minimap");
    assert_eq!(
        command_id(&PaletteCommand::StashPop(StashTarget::Latest)),
        "stash_pop_latest"
    );
    // The sequence vocabulary names these steps by these ids.
    assert_eq!(command_id(&PaletteCommand::TabOpenIn), "tab_open_in");
    assert_eq!(
        command_id(&PaletteCommand::TabOpenAgentHere),
        "tab_open_agent_here"
    );
    assert_eq!(
        command_id(&PaletteCommand::TabOpenInDefault),
        "tab_open_in_default"
    );
    assert_eq!(
        command_id(&PaletteCommand::TabDir(
            crate::tab_dir::TabDirButton::Terminal
        )),
        "tab_dir_terminal"
    );
}

/// Every step of the sequence vocabulary that is a palette command resolves
/// to a catalogue entry, so a runner can always find what it must run.
#[test]
fn every_vocabulary_command_step_resolves_to_a_palette_entry() {
    let ids: Vec<String> = PALETTE_COMMANDS
        .iter()
        .map(|e| command_id(&e.command))
        .collect();
    for id in [
        "tab_dir_terminal",
        "tab_open_in",
        "tab_open_in_default",
        "tab_open_agent_here",
    ] {
        assert!(ids.iter().any(|i| i == id), "{id} has no palette entry");
        let kind = gitpoulpi_config::sequences::step_kind(id)
            .unwrap_or_else(|| panic!("{id} is not in the vocabulary"));
        assert!(!kind.pending, "{id} is still pending");
    }
}

#[test]
fn a_palette_command_binding_comes_from_config_and_shows_in_its_row() {
    let minimap = row(PaletteCommand::ToggleMinimap);
    let action = bind_action(minimap).unwrap();
    assert!(matches!(action, Action::Palette(_)));
    let before = entry_shortcut(&Keymap::defaults(), minimap);
    assert_ne!(before, "Ctrl+Alt+M");
    let km = km_with(&[("palette", "toggle_minimap", &["Ctrl+Alt+M"])]);
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    assert_eq!(entry_shortcut(&km, minimap), "Ctrl+Alt+M");
    assert!(km.is_modified(action));
    // The dispatch table finds it.
    let ev = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('m'),
        crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT,
    );
    assert_eq!(km.action_in(Ctx::Palette, &ev), Some(action));
    assert_eq!(Keymap::defaults().action_in(Ctx::Palette, &ev), None);
}

#[test]
fn an_unknown_palette_id_is_reported_and_ignored() {
    let km = km_with(&[("palette", "no_such_command", &["Ctrl+Alt+M"])]);
    assert_eq!(km.issues().len(), 1);
}

/// A palette command's key clashes with a global one like any other.
#[test]
fn a_palette_binding_clashes_with_a_global_key() {
    let minimap = row(PaletteCommand::ToggleMinimap);
    let action = bind_action(minimap).unwrap();
    // Ctrl+P opens the palette: the same scope, so an error.
    let hypothetical = Keymap::defaults().with_chords(
        action,
        vec![crate::keymap::KeyChord::parse("Ctrl+P").unwrap()],
    );
    let conflicts = hypothetical.conflicts();
    assert!(
        conflicts.iter().any(|c| {
            c.level == crate::keymap::conflict::Level::Error
                && (c.action == action || c.other == Some(action))
        }),
        "{conflicts:?}"
    );
    // A pane key on the same chord hides it, with a warning.
    let hypothetical = Keymap::defaults().with_chords(
        action,
        vec![crate::keymap::KeyChord::parse("Shift+O").unwrap()],
    );
    assert!(hypothetical.conflicts().iter().any(|c| {
        c.level == crate::keymap::conflict::Level::Warning
            && (c.other == Some(action) || c.action == action)
    }));
}

#[test]
fn a_config_clash_between_two_palette_commands_keeps_the_defaults() {
    let km = km_with(&[
        ("palette", "toggle_minimap", &["Ctrl+Alt+M"]),
        ("palette", "toggle_word_wrap", &["Ctrl+Alt+M"]),
    ]);
    assert!(!km.issues().is_empty());
}
