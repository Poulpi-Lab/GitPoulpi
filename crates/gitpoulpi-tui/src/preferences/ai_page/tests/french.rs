//! The AI page in French at 100 columns, global and repository; the
//! sentences the chips insert stay English (they are sent to the model).

use super::*;
use crate::preferences::list_editor::ListOp;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

fn expect_all(screen: &str, wants: &[&str]) {
    for want in wants {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn the_page_is_in_french() {
    let _l = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    s.config.ai.enabled = true;
    s.config.ai.profiles.push(AiProfile::new(
        "second",
        "Second",
        AiProviderKind::LocalAgent,
    ));
    // Both tabs of the page: General, then the profiles.
    s.remember_sub_tab(PrefSection::Ai, SubTab::General);
    let mut screen = draw(&mut s, 100, 140).replace('\u{a0}', " ");
    s.remember_sub_tab(PrefSection::Ai, SubTab::List);
    screen.push_str(&draw(&mut s, 100, 140).replace('\u{a0}', " "));
    expect_all(
        &screen,
        &[
            "Activer l'IA",
            "Instructions globales",
            "Instructions",
            "Génération du message de commit",
            "Profils (connexions)",
            "Fournisseur",
            "Tester la connexion",
        ],
    );
    for gone in ["Enable AI", "Quick insert", "Test connection"] {
        assert!(
            !screen.contains(gone),
            "{gone:?} is still English:\n{screen}"
        );
    }
    let _t = override_git_terms(true);
    s.remember_sub_tab(PrefSection::Ai, SubTab::General);
    let screen = draw(&mut s, 100, 140).replace('\u{a0}', " ");
    expect_all(&screen, &["Génération du message de validation"]);
}

#[test]
fn the_rows_and_helps_are_french_and_the_chip_sentence_is_not() {
    let _l = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    s.config.ai.enabled = true;
    s.remember_sub_tab(PrefSection::Ai, SubTab::General);
    let fields = s.fields();
    let help_of = |label: &str| {
        fields
            .iter()
            .find(|f| f.label == label)
            .and_then(|f| f.help.clone())
            .unwrap_or_default()
    };
    assert!(help_of("Activer l'IA").starts_with("Désactivée par défaut"));
    assert!(help_of("Profil").starts_with("Le profil que GitPoulpi utilise pour"));
    assert!(fields.iter().all(|f| f.label != "Insertion rapide"));
    select(&mut s, "Instructions perso.");
    s.activate();
    let screen = draw(&mut s, 120, 40);
    assert!(
        screen.contains("Insertion rapide:  + Répondre en <langue> "),
        "{screen}"
    );
    s.on_key(key(KeyCode::Tab));
    s.on_key(key(KeyCode::Right));
    s.on_key(key(KeyCode::Enter));
    s.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    // The sentence is English whatever the UI language: it is sent to the model.
    assert!(
        file(dir.path()).contains("Keep responses concise."),
        "{}",
        file(dir.path())
    );
}

#[test]
fn the_repository_page_is_in_french() {
    let _l = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = repo_state_on_page(dir.path());
    let screen = draw(&mut s, 100, 60).replace('\u{a0}', " ");
    expect_all(
        &screen,
        &[
            "Instructions du dépôt",
            "Combiner avec le global",
            "Ajouter au global",
            "Utiliser le global",
            "(aucune instruction du dépôt)",
        ],
    );
}

#[test]
fn the_editor_overlay_is_in_french() {
    let _l = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    s.config.ai.enabled = true;
    select(&mut s, "Instructions perso.");
    s.on_key(key(KeyCode::Enter));
    assert!(s.ai.prompt_open());
    let screen = draw(&mut s, 100, 40).replace('\u{a0}', " ");
    expect_all(
        &screen,
        &[
            "Instructions globales - Instructions personnalisées",
            "Ctrl+S : enregistrer",
            "Enter : saut de ligne",
            "Enregistrer",
            "Annuler",
        ],
    );
}

#[test]
fn a_new_profile_is_named_in_english_whatever_the_language() {
    let _l = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    s.apply_list_op(ListOp::Add(None));
    assert_eq!(s.config.ai.profiles.last().unwrap().name, "New profile");
}
