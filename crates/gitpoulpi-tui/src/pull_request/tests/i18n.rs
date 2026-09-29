//! The Create Pull / Merge Request form in French, both hosts, both vocabularies.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, override_merge_terms, Lang};

fn screen() -> String {
    let mut d = dialog();
    d.title.clear();
    paint(&mut d, 110, 34).0
}

fn assert_has(screen: &str, wants: &[&str]) {
    for want in wants {
        assert!(screen.contains(want), "{want}\n{screen}");
    }
}

#[test]
fn english_title_case_follows_the_host() {
    let _m = override_merge_terms(true);
    assert_has(
        &screen(),
        &["Create Merge Request", "Merge request description"],
    );
    drop(_m);
    let _m = override_merge_terms(false);
    assert_has(
        &screen(),
        &["Create Pull Request", "Pull request description"],
    );
}

#[test]
fn french_form_on_github_and_gitlab() {
    let _l = override_lang(Lang::Fr);
    let _m = override_merge_terms(false);
    assert_has(
        &screen(),
        &[
            "Créer une pull request",
            "Dépôt source",
            "Dépôt cible",
            "Branche",
            "Titre",
            "Description",
            "Créer en brouillon",
            "Annuler",
            "Tab suivant",
        ],
    );
    drop(_m);
    let _m = override_merge_terms(true);
    assert_has(&screen(), &["Créer une merge request"]);
}

#[test]
fn french_form_with_translated_git_terms() {
    let _l = override_lang(Lang::Fr);
    let _g = override_git_terms(true);
    let _m = override_merge_terms(false);
    assert_has(&screen(), &["Créer une demande de tirage"]);
    drop(_m);
    let _m = override_merge_terms(true);
    assert_has(&screen(), &["Créer une demande de fusion"]);
}

#[test]
fn french_validation_messages_and_the_loading_line() {
    let _l = override_lang(Lang::Fr);
    let mut d = dialog();
    d.title.clear();
    assert_eq!(d.plan().unwrap_err(), "saisissez un titre");
    let id = d.current_tab().unwrap().integration.id.clone();
    d.projects.insert(id, ProjectsState::Loading);
    d.message = None;
    let (text, _) = paint(&mut d, 110, 34);
    assert!(
        text.contains("Chargement des dépôts et des branches"),
        "{text}"
    );
}
