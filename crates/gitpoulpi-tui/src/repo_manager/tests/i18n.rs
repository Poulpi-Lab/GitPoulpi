//! The Home tab in French at 100 columns, with and without the git
//! vocabulary.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

fn screen(m: &mut RepoManager, width: u16) -> Vec<String> {
    let area = Rect::new(0, 0, width, 30);
    render(m, area).lines.iter().map(text).collect()
}

#[test]
fn home_in_french_fits_at_100_columns() {
    let _fr = override_lang(Lang::Fr);
    let mut m = manager();
    let lines = screen(&mut m, 100);
    let all = lines.join("\n");
    for needle in [
        "Gestion des dépôts",
        "Parcourir (b)",
        "Cloner (c)",
        "Init (i)",
        "Tout replier",
        "Tout déplier",
        "DÉPÔTS OUVERTS",
        "FAVORIS",
        "DÉPÔTS RÉCENTS",
        "TOUS LES DÉPÔTS",
        "Fermer tous les onglets",
        "Tout retirer",
        "absent du disque",
        "Ouvrir",
        "Aller",
        "Fermer",
        "Retirer",
        "Chercher un dépôt",
        "retour",
    ] {
        assert!(all.contains(needle), "{needle}\n{all}");
    }
    for english in [
        "Repository Management",
        "Collapse all",
        "Open repositories",
        "Remove all",
    ] {
        assert!(!all.contains(english), "{english}\n{all}");
    }
    for line in &lines {
        assert!(display_width(line.trim_end()) <= 100, "{line:?}");
    }
}

#[test]
fn the_empty_sections_and_messages_are_french() {
    let _fr = override_lang(Lang::Fr);
    let mut m = RepoManager::new(Vec::new());
    let all = screen(&mut m, 100).join("\n");
    assert!(all.contains("aucun dépôt ouvert dans un onglet"), "{all}");
    assert!(all.contains("rien d'ouvert récemment"), "{all}");
    assert!(all.contains("Parcourir, Cloner ou Init"), "{all}");

    let mut gone = manager();
    let index = gone.repos.iter().position(|r| !r.exists).unwrap();
    gone.open_repo(index);
    let message = gone.message.clone().unwrap();
    assert!(message.contains("a disparu du disque"), "{message}");
}

#[test]
fn git_terms_change_the_unborn_placeholder() {
    let _fr = override_lang(Lang::Fr);
    assert_eq!(msg::home::UNBORN.get(), "(sans commit)");
    let _terms = override_git_terms(true);
    assert_eq!(msg::home::UNBORN.get(), "(sans validation)");
}
