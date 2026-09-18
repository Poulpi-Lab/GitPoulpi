//! The Clone dialog in French at 100 columns, with and without the git
//! vocabulary: everything is translated and fits.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

#[test]
#[cfg(feature = "hosting")]
fn the_url_source_in_french_fits_at_100_columns() {
    let _fr = override_lang(Lang::Fr);
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    dialog.url = "https://example.com/team/app.git".into();
    dialog.sync_name();
    let (text, rect) = paint(&mut dialog, 100, 30);
    for needle in [
        "Cloner un dépôt",
        "CLONER DEPUIS",
        "Cloner par URL",
        "Cloner dans",
        "Parcourir...",
        "Nom du dossier",
        "Chemin complet\u{a0}:",
        "Clone superficiel (dernier commit seulement)",
        "Extraction partielle",
        "Cloner ",
        "Annuler",
        "suivant",
    ] {
        assert!(text.contains(needle), "{needle}\n{text}");
    }
    for english in [
        "Clone a repository",
        "Where to clone to",
        "Cancel",
        "Folder name",
    ] {
        assert!(!text.contains(english), "{english}\n{text}");
    }
    for (r, _) in &dialog.hits {
        assert!(rect.contains((r.x, r.y).into()), "{r:?}");
    }
}

#[test]
fn the_git_vocabulary_switches_the_shallow_label() {
    let _fr = override_lang(Lang::Fr);
    let _terms = override_git_terms(true);
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    dialog.url = "https://example.com/team/app.git".into();
    dialog.sync_name();
    let (text, _) = paint(&mut dialog, 100, 30);
    assert!(text.contains("dernière validation seulement"), "{text}");
}

#[test]
#[cfg(feature = "hosting")]
fn the_hosted_source_in_french_fits_and_explains_itself() {
    let _fr = override_lang(Lang::Fr);
    let mut dialog = with_hosted();
    dialog.pick_source(1);
    let (text, _) = paint(&mut dialog, 100, 40);
    assert!(text.contains("Dépôt"), "{text}");
    assert!(text.contains("chargement des dépôts..."), "{text}");
    let mut bare = CloneDialog::new(Some(&abs("clones")));
    let (note, _) = paint(&mut bare, 100, 30);
    assert!(note.contains("dans Paramètres >"), "{note}");
    assert!(note.contains("un hébergeur pour"), "{note}");
    assert!(note.contains("lister vos dépôts ici"), "{note}");
}

#[test]
fn the_validation_errors_follow_the_language() {
    let _fr = override_lang(Lang::Fr);
    let dialog = CloneDialog::new(Some(&abs("clones")));
    assert_eq!(
        dialog.plan().unwrap_err(),
        "saisissez l'URL du dépôt à cloner"
    );
    assert_eq!(
        validate_folder_name("a/b").unwrap_err(),
        "le nom du dossier ne peut pas contenir / ni \\"
    );
}
