//! The PULL REQUESTS / ISSUES sections, the chip tooltip and the palette's
//! "why not" texts in French, with GitHub (pull request) and GitLab (merge
//! request) terminology, git terms English and translated.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, override_merge_terms, with_lang, Lang};

fn prs() -> (HostingUi, tempfile::TempDir) {
    let mut mine = open_pr(1);
    mine.author = "kai".into();
    let mut other = pr(4, "d", PullRequestState::Open);
    other.author = "zed".into();
    ui_with(snapshot(vec![mine, other], vec![]))
}

fn pulls_screen(ui: &HostingUi) -> String {
    draw_section(ui, HostingList::Pulls, 46, 14, None).0
}

fn assert_has(screen: &str, wants: &[&str]) {
    for want in wants {
        assert!(screen.contains(want), "{want}\n{screen}");
    }
}

#[test]
fn english_follows_the_host_word_in_title_case() {
    let (ui, _d) = prs();
    let _m = override_merge_terms(true);
    assert_has(
        &pulls_screen(&ui),
        &[
            "MERGE REQUESTS (2)",
            "Search merge requests",
            "My Merge Requests",
            "All Merge Requests",
        ],
    );
    drop(_m);
    let _m = override_merge_terms(false);
    assert_has(
        &pulls_screen(&ui),
        &[
            "PULL REQUESTS (2)",
            "Search pull requests",
            "My Pull Requests",
            "All Pull Requests",
        ],
    );
}

#[test]
fn french_github_says_pull_request_by_default() {
    let (ui, _d) = prs();
    let _l = override_lang(Lang::Fr);
    let _m = override_merge_terms(false);
    assert_has(
        &pulls_screen(&ui),
        &[
            "PULL REQUESTS (2)",
            "Chercher des pull requests",
            "Ouvertes",
            "Mes pull requests",
            "Assignées à moi",
            "En attente de ma relecture",
            "Toutes les pull requests",
        ],
    );
}

#[test]
fn french_gitlab_says_merge_request_by_default() {
    let (ui, _d) = prs();
    let _l = override_lang(Lang::Fr);
    let _m = override_merge_terms(true);
    assert_has(
        &pulls_screen(&ui),
        &[
            "MERGE REQUESTS (2)",
            "Chercher des merge requests",
            "Mes merge requests",
        ],
    );
}

#[test]
fn french_with_translated_git_terms_uses_the_french_words() {
    let (ui, _d) = prs();
    let _l = override_lang(Lang::Fr);
    let _g = override_git_terms(true);
    let _m = override_merge_terms(false);
    assert_has(
        &pulls_screen(&ui),
        &[
            "DEMANDES DE TIRAGE (2)",
            "Mes demandes de tirage",
            "Toutes les demandes de tirage",
        ],
    );
    drop(_m);
    let _m = override_merge_terms(true);
    assert_has(
        &pulls_screen(&ui),
        &[
            "DEMANDES DE FUSION (2)",
            "Chercher des demandes de fusion",
            "Mes demandes de fusion",
        ],
    );
}

/// Polish keeps the English host words in git mode: a GitLab host's section is
/// MERGE REQUESTY, a GitHub host's PULL REQUESTY, in the hosting list and in the
/// sidebar header alike.
#[test]
fn polish_section_title_follows_the_host() {
    use crate::refs_sidebar::SidebarSection;
    let (ui, _d) = prs();
    let _l = override_lang(Lang::Pl);
    let _g = override_git_terms(false);
    let _m = override_merge_terms(true);
    assert_has(&pulls_screen(&ui), &["MERGE REQUESTY (2)"]);
    assert_eq!(
        SidebarSection::PullRequests.display_title(),
        "MERGE REQUESTY"
    );
    drop(_m);
    let _m = override_merge_terms(false);
    assert_has(&pulls_screen(&ui), &["PULL REQUESTY (2)"]);
    assert_eq!(
        SidebarSection::PullRequests.display_title(),
        "PULL REQUESTY"
    );
}

#[test]
fn french_issues_section_and_filter_rows() {
    let (ui, _d) = ui_with(snapshot(vec![], vec![issue(7)]));
    let _l = override_lang(Lang::Fr);
    let (screen, _, _) = draw_section(&ui, HostingList::Issues, 46, 12, None);
    assert_has(
        &screen,
        &["TICKETS GITLAB", "Chercher des tickets", "Ouverts"],
    );
    assert_eq!(ui.section_title(HostingList::Issues), "TICKETS GITLAB");
    assert_eq!(IssueView::Assigned.label(), "Qui me sont assignés");
    {
        let _g = override_git_terms(false);
        assert_eq!(PrView::Closed.label(), "Mergées / fermées");
    }
    let _g = override_git_terms(true);
    assert_eq!(PrView::Closed.label(), "Fusionnées / fermées");
}

#[test]
fn french_chip_tooltip_and_unavailable_reasons() {
    let _l = override_lang(Lang::Fr);
    let _m = override_merge_terms(true);
    let (mut unlinked, _d) = ui_with(snapshot(vec![], vec![]));
    unlinked.snapshot.as_mut().unwrap().head_pr = None;
    unlinked.snapshot.as_mut().unwrap().head_branch = Some("feature".into());
    let tip = unlinked.chip_tooltip().join("\n");
    assert!(
        tip.contains("Pas encore de merge request pour cette branche"),
        "{tip}"
    );
    assert!(tip.contains("clic\u{a0}: créer une merge request"), "{tip}");
    assert_eq!(unlinked.chip().unwrap().text, "+ MR");

    let (ui, _d2) = ui_with(snapshot(vec![open_pr(5)], vec![]));
    let tip = ui.chip_tooltip().join("\n");
    assert!(
        tip.contains("ouverte  ·  feature → main  ·  par kai"),
        "{tip}"
    );

    let none = HostingAvailability::default();
    let why = command_unavailable_from(HostingCommand::CheckoutPullRequest, none).unwrap();
    assert_eq!(
        why,
        "aucune intégration ne correspond au distant de ce dépôt"
    );
    drop(_m);
    let _m = override_merge_terms(false);
    let linked = HostingAvailability {
        linked: true,
        ..HostingAvailability::default()
    };
    let why = command_unavailable_from(HostingCommand::CheckoutPullRequest, linked).unwrap();
    assert_eq!(why, "aucune pull request dans la liste");
}

#[test]
fn the_row_menus_and_status_messages_are_french() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(3)], vec![]));
    let _l = override_lang(Lang::Fr);
    let _m = override_merge_terms(false);
    assert!(ui.open_chip_menu(2, 2));
    let menu = ui.menu.as_ref().unwrap();
    let labels: Vec<String> = (0..menu.row_count()).map(|r| menu.row_label(r)).collect();
    let all = labels.join("|");
    for want in [
        "Ouvrir dans le navigateur",
        "Copier l'URL",
        "Actualiser",
        "Changer de profil (corp)",
    ] {
        assert!(all.contains(want), "{want} in {all}");
    }
    let mut clip = crate::clipboard::Clipboard::new();
    let effect = ui.run(HostingAction::Refresh, &mut clip);
    match effect {
        HostingEffect::Message(m) => {
            assert_eq!(m, "actualisation des pull requests et des tickets...")
        }
        _ => panic!("expected a message"),
    }
}

#[test]
fn hosting_errors_keep_the_host_detail_and_translate_the_frame() {
    use gitpoulpi_hosting::Error;
    let forbidden = Error::Forbidden {
        what: "listing".into(),
        detail: "403 Forbidden".into(),
    };
    let api = |status: u16| Error::Api {
        what: "creating".into(),
        status,
        detail: "bad".into(),
    };
    with_lang(Lang::En, || {
        assert_eq!(
            crate::hosting_task::error_text(&forbidden),
            "the token lacks the required scope (403 Forbidden)"
        );
        assert_eq!(
            crate::hosting_task::error_text(&api(404)),
            "not found on the host (bad)"
        );
        assert_eq!(
            crate::hosting_task::error_text(&api(422)),
            "the host refused the request (bad)"
        );
    });
    with_lang(Lang::Fr, || {
        assert_eq!(
            crate::hosting_task::error_text(&forbidden),
            "le jeton n'a pas la portée requise (403 Forbidden)"
        );
        assert_eq!(
            crate::hosting_task::error_text(&api(404)),
            "introuvable sur l'hébergeur (bad)"
        );
        assert_eq!(
            crate::hosting_task::error_text(&api(422)),
            "l'hébergeur a refusé la requête (bad)"
        );
        let limited = Error::RateLimited {
            host: "gitlab.com".into(),
            retry_in_secs: 300,
        };
        assert_eq!(
            crate::hosting_task::error_text(&limited),
            "gitlab.com\u{a0}: limite de requêtes atteinte, réessayez dans 5 min"
        );
    });
}
