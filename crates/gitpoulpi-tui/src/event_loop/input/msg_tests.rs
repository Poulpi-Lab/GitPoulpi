//! The status-line messages of the input handlers in French: plurals,
//! the git vocabulary switch, typed arguments, and English staying the default.

use gitpoulpi_i18n::{msg, override_git_terms, override_lang, Lang};

#[test]
fn english_is_the_default_and_reads_as_before() {
    assert_eq!(
        msg::input::SELECT_CHANGED_FILE.get(),
        "select a changed file in the Status pane first"
    );
    assert_eq!(msg::input::popping_stash(3), "popping stash@{3}...");
    assert_eq!(msg::input::drop_stash(0), "Drop stash@{0}?");
    assert_eq!(
        msg::input::commits_selected(2),
        "2 commits selected (Esc clears)"
    );
    assert_eq!(msg::input::message_rows(1), "commit message: 1 row");
    assert_eq!(msg::input::ran("Fetch"), "ran: Fetch");
    assert_eq!(
        msg::input::branch_not_here("dev", " (Ctrl+F: fetch)"),
        "dev is not in this repository yet - fetch first (Ctrl+F: fetch)"
    );
    assert_eq!(
        msg::undo_ui::tip_cant_undo("Commit x", "why"),
        "Can't undo Commit x: why"
    );
}

#[test]
fn french_plurals_treat_zero_and_one_as_singular() {
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(false);
    assert_eq!(
        msg::input::commits_selected(1),
        "1 commit sélectionné (Esc efface)"
    );
    assert_eq!(
        msg::input::commits_selected(5),
        "5 commits sélectionnés (Esc efface)"
    );
    assert_eq!(msg::input::solo_folder(1, "src"), "solo\u{a0}: src (1 ref)");
    assert_eq!(
        msg::input::solo_folder(4, "src"),
        "solo\u{a0}: src (4 refs)"
    );
    assert_eq!(
        msg::input::section_rows(7, "LOCAL"),
        "section LOCAL\u{a0}: 7 lignes"
    );
}

#[test]
fn git_words_follow_the_vocabulary_setting() {
    let _l = override_lang(Lang::Fr);
    {
        let _t = override_git_terms(false);
        assert_eq!(msg::input::NO_STASH_TO_POP.get(), "aucun stash à dépiler");
        assert_eq!(
            msg::input::delete_tag("v1"),
            "Supprimer le tag «\u{a0}v1\u{a0}»\u{a0}?"
        );
        assert_eq!(
            msg::input::commits_selected(2),
            "2 commits sélectionnés (Esc efface)"
        );
    }
    let _t = override_git_terms(true);
    assert_eq!(msg::input::NO_STASH_TO_POP.get(), "aucune remise à dépiler");
    assert_eq!(
        msg::input::delete_tag("v1"),
        "Supprimer l'étiquette «\u{a0}v1\u{a0}»\u{a0}?"
    );
    assert_eq!(
        msg::input::commits_selected(2),
        "2 validations sélectionnées (Esc efface)"
    );
    assert_eq!(
        msg::input::commits_selected(1),
        "1 validation sélectionnée (Esc efface)"
    );
}

#[test]
fn arguments_land_where_the_translation_puts_them() {
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(false);
    assert_eq!(msg::input::renaming("a", "b"), "renommage de a en b...");
    assert_eq!(
        msg::input::copied("short SHA", "abc1234"),
        "copié\u{a0}: short SHA (abc1234)"
    );
    assert_eq!(
        msg::undo_ui::tip_cant_undo("Commit x", "raison"),
        "Impossible d'annuler Commit x\u{a0}: raison"
    );
    assert_eq!(msg::undo_ui::UNDOING.get(), "annulation...");
}
