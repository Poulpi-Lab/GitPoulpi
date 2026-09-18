use super::*;

fn oid_of(prefix: &str) -> Oid {
    Oid::from_hex(&format!("{prefix}{}", "0".repeat(40 - prefix.len()))).unwrap()
}

/// Every status line shows an `Oid` as its first seven hex characters.
#[test]
fn messages_name_an_oid_by_its_seven_character_prefix() {
    let message = reword_done_message(
        oid_of("1234567"),
        &RewordOutcome {
            reworded: oid_of("abcdef0"),
            previous_head: oid_of("fedcba9"),
            new_head: oid_of("abcdef0"),
            replayed: 2,
        },
    );
    assert_eq!(
        message,
        "message of 1234567 edited, now abcdef0, 2 commits on top re-created \
         (previous tip fedcba9: reset --soft to it to undo)"
    );
}

#[test]
fn a_rebase_stopped_or_conflicted_names_its_commit_by_prefix() {
    let describe = describe_rebase(gitpoulpi_i18n::msg::link::REBASED);
    assert_eq!(describe(RebaseOutcome::Completed), "rebased");
    let at = oid_of("0a1b2c3");
    let conflicted =
        describe_rebase(gitpoulpi_i18n::msg::link::REBASED)(RebaseOutcome::Conflicted {
            paused_at: at,
            remaining: Vec::new(),
        });
    assert_eq!(conflicted, "rebase paused on a conflict at 0a1b2c3");
}

use gitpoulpi_i18n::{msg, with_lang, Lang};

#[test]
fn outcome_lines_agree_in_plural_and_language() {
    let two = RewordOutcome {
        reworded: oid_of("abcdef0"),
        previous_head: oid_of("fedcba9"),
        new_head: oid_of("abcdef0"),
        replayed: 1,
    };
    assert_eq!(
        reword_done_message(oid_of("1234567"), &two),
        "message of 1234567 edited, now abcdef0, 1 commit on top re-created \
         (previous tip fedcba9: reset --soft to it to undo)"
    );
    with_lang(Lang::Fr, || {
        assert_eq!(
            reword_done_message(oid_of("1234567"), &two),
            "message de 1234567 modifié, devenu abcdef0, 1 commit au-dessus recréé \
             (ancien sommet fedcba9\u{a0}: reset --soft dessus pour annuler)"
        );
        assert_eq!(msg::link::created_commits(1), "1 commit créé");
        assert_eq!(msg::link::created_commits(3), "3 commits créés");
        assert_eq!(
            msg::link::fetched(0, "origin"),
            "fetch de origin\u{a0}: 0 référence mise à jour"
        );
        assert_eq!(
            msg::link::fetched(2, "origin"),
            "fetch de origin\u{a0}: 2 références mises à jour"
        );
        assert_eq!(
            describe_pull(gitpoulpi_core::domain::PullOutcome::UpToDate),
            "déjà à jour"
        );
        assert_eq!(
            msg::link::pushed_refs("main", "origin"),
            "push de main vers origin terminé"
        );
    });
    with_lang(Lang::En, || {
        assert_eq!(msg::link::created_commits(1), "created 1 commit");
        assert_eq!(msg::link::created_commits(2), "created 2 commits");
        assert_eq!(
            msg::link::fetched(1, "origin"),
            "fetched origin: 1 ref updated"
        );
        assert_eq!(
            msg::link::orig_cleaned(2, "merged, new commit 1234567"),
            "merged, new commit 1234567, deleted 2 .orig files"
        );
    });
}

#[test]
fn in_flight_labels_follow_the_language_and_the_git_vocabulary() {
    use crate::daemon_link::Action;
    assert_eq!(Action::FetchAll.activity_label(), "fetching...");
    assert_eq!(Action::StageAll.activity_label(), "staging all changes...");
    with_lang(Lang::Fr, || {
        assert_eq!(Action::FetchAll.activity_label(), "fetch en cours...");
        let _t = gitpoulpi_i18n::override_git_terms(true);
        assert_eq!(
            Action::FetchAll.activity_label(),
            "r\u{e9}cup\u{e9}ration..."
        );
        assert_eq!(
            Action::DiscardPaths(vec!["a".into()]).activity_label(),
            "abandon des modifications de 1 fichier..."
        );
    });
}
