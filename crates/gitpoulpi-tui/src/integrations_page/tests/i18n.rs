//! The Integrations page in French, with "Translate git terms" off and on:
//! every view fits 100 columns and shows the translated text.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

/// Every non-empty line of the drawn page, so a clipped word fails the test.
fn french(page: &mut IntegrationsPage, h: &H) -> String {
    text(&draw(page, h, 100, 34, None))
}

fn expect_all(screen: &str, wants: &[&str]) {
    for want in wants {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn the_empty_page_is_in_french_and_the_term_follows_the_setting() {
    let _l = override_lang(Lang::Fr);
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Hébergement",
            "Aucun profil GitHub pour l'instant.",
            "+ Ajouter un profil",
            " Se connecter ",
            "dépôts, pull requests, tickets, clés SSH",
            "Connectez un compte pour lier des pull requests et des tickets à vos",
        ],
    );
    let _t = override_git_terms(true);
    let screen = french(&mut page, &h);
    expect_all(&screen, &["dépôts, demandes de tirage, tickets, clés SSH"]);
}

#[test]
fn the_connect_form_is_in_french() {
    let _l = override_lang(Lang::Fr);
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Domaine de l'hôte",
            "p. ex. gitlab.example.com ou http://nas:8080 (https:// par défaut)",
            "Jeton d'accès personnel",
            "collez votre jeton",
            "Portées : api, read_user",
            "Générer un jeton (saisissez d'abord l'hôte)",
            " Se connecter ",
            " Annuler ",
        ],
    );
    page.activate(Item::Connect, &mut h.env());
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &["Saisissez un domaine d'hôte valide, p. ex. gitlab.example.com"],
    );
    type_str(&mut page, &mut h, "gitlab.example.com");
    let screen = french(&mut page, &h);
    expect_all(&screen, &["Générer un jeton sur gitlab.example.com"]);
}

#[test]
fn the_connected_card_and_the_ssh_section_are_in_french() {
    let _l = override_lang(Lang::Fr);
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Kai G (@kai)  Connecté",
            "Hôte",
            "Jeton",
            "enregistré",
            "Auth.",
            " Modifier ",
            " Déconnecter ",
            " Supprimer ",
            "Clé SSH",
            "Clé par défaut",
            "Générer une clé",
            "Clé personnalisée",
            "Ce profil utilise votre clé SSH par défaut ou votre agent.",
        ],
    );
    page.activate(Item::SshGenerate, &mut h.env());
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Titre de la clé",
            "[x] Ajouter la clé publique à gitlab.example.com",
            " Générer la clé ",
        ],
    );
    page.activate(Item::SshCustom, &mut h.env());
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Fichier de clé privée",
            " Parcourir... ",
            " Utiliser cette clé ",
        ],
    );
}

#[test]
fn an_attached_key_and_its_deletion_dialog_fit_in_french() {
    let _l = override_lang(Lang::Fr);
    let mut h = H::new();
    let id = h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let at = h
        .config
        .integrations
        .iter()
        .position(|i| i.id == id)
        .unwrap();
    h.config.integrations[at].ssh = Some(IntegrationSsh {
        mode: SshMode::Generated,
        private_key_path: Some("/nonexistent/id_x".into()),
        public_key_path: None,
        title: Some("laptop".into()),
        remote_key_id: Some("42".into()),
    });
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "clé générée",
            "ajoutée à gitlab.example.com (clé 42)",
            " Retirer ",
            " Supprimer ",
            "Retirer détache la clé",
        ],
    );
    page.activate(Item::SshDelete, &mut h.env());
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &[
            "Supprimer cette clé et ses fichiers\u{a0}?",
            "privée",
            "supprimée sur gitlab.example.com. Action irréversible.",
            " Supprimer les fichiers ",
            " Garder ",
        ],
    );
}

#[test]
fn the_remove_profile_question_is_in_french() {
    let _l = override_lang(Lang::Fr);
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.activate(Item::Remove, &mut h.env());
    let screen = french(&mut page, &h);
    expect_all(
        &screen,
        &["Supprimer le profil", "Son jeton enregistré", " Garder "],
    );
}
