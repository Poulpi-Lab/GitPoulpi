//! The remote UI in French, with "Translate git terms" off and on: the
//! Add / Edit popup and the dropdowns fit, and the git words follow the
//! setting.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn lines(draw: impl Fn(&mut Frame)) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|f| draw(f)).unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect())
        .collect()
}

fn menu_lines(menu: &ContextMenu<RemoteUiAction>) -> Vec<String> {
    lines(|f| drop(menu.render(f, None)))
}

/// Every row of the menu is drawn whole: its line ends on the border.
fn assert_rows_fit(menu: &ContextMenu<RemoteUiAction>) -> String {
    let lines = menu_lines(menu);
    for r in 0..menu.row_count() {
        let label = menu.row_label(r);
        if label.is_empty() {
            continue;
        }
        let line = lines
            .iter()
            .find(|l| l.contains(label.as_str()))
            .unwrap_or_else(|| panic!("row {label:?} is cut:\n{}", lines.join("\n")));
        assert!(line.trim_end().ends_with('\u{2502}'), "{line:?}");
    }
    lines.join("\n")
}

fn popup(form: &RemoteForm) -> String {
    lines(|f| drop(form.render(f, f.area(), None))).join("\n")
}

#[test]
fn the_add_popup_fits_in_french_and_the_labels_follow_the_vocabulary() {
    let _l = override_lang(Lang::Fr);
    let form = RemoteForm::add(vec![]);
    let screen = popup(&form);
    for want in [
        "Ajouter un distant",
        "Nom",
        "URL fetch",
        "URL push",
        "(comme l'URL fetch)",
        "Annuler",
        "Ajouter",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    let _t = override_git_terms(true);
    let screen = popup(&form);
    for want in ["URL récup.", "URL envoi", "(comme l'URL de récupération)"] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn the_edit_popup_and_its_errors_are_in_french() {
    let _l = override_lang(Lang::Fr);
    let remote = Remote {
        name: "origin".into(),
        fetch_url: "https://host/o/r.git".into(),
        push_url: None,
    };
    let mut form = RemoteForm::edit(&remote, vec!["origin".into()]);
    let screen = popup(&form);
    for want in ["Modifier le distant origin", "Enregistrer"] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    form.fetch_url = "nope".into();
    let err = form.submit().unwrap_err();
    assert!(err.starts_with("URL fetch\u{a0}: "), "{err}");
    let _t = override_git_terms(true);
    let err = form.submit().unwrap_err();
    assert!(err.starts_with("URL de récupération\u{a0}: "), "{err}");
    form.name = String::new();
    assert_eq!(form.submit().unwrap_err(), "le nom est vide");
}

#[test]
fn the_pull_and_push_dropdowns_fit_in_french() {
    let _l = override_lang(Lang::Fr);
    for local in [false, true] {
        let _t = override_git_terms(local);
        let pull = ContextMenu::new(msg::PULL_TITLE.get(), pull_mode_entries(), 2, 1);
        let screen = assert_rows_fit(&pull);
        let want = if local {
            "Tirer (avance rapide si possible)"
        } else {
            "Pull (fast-forward si possible)"
        };
        assert!(screen.contains(want), "{screen}");
        let push = ContextMenu::new(
            msg::push_to_title("main"),
            push_to_entries(&["origin".into(), "fork".into()], Some("origin")),
            2,
            1,
        );
        let screen = assert_rows_fit(&push);
        assert!(screen.contains("origin  (par défaut)"), "{screen}");
    }
}

#[test]
fn the_remote_menu_and_the_diverged_dialog_fit_in_french() {
    let _l = override_lang(Lang::Fr);
    for local in [false, true] {
        let _t = override_git_terms(local);
        let menu = ContextMenu::new(
            msg::menu_title("origin"),
            remote_menu_entries("origin", false),
            2,
            1,
        );
        let screen = assert_rows_fit(&menu);
        assert!(screen.contains("Copier le nom du distant"), "{screen}");
        assert!(
            screen.contains("Définir comme distant par défaut"),
            "{screen}"
        );
        let rejection = PushRejection {
            remote: "origin".into(),
            branch: "main".into(),
            reason: PushRejectReason::NonFastForward,
            detail: None,
        };
        let diverged = ContextMenu::new(
            msg::diverged_title("origin", "main"),
            diverged_entries(&rejection),
            2,
            1,
        );
        let screen = assert_rows_fit(&diverged);
        assert!(screen.contains("origin/main a divergé"), "{screen}");
        let stash = ContextMenu::new(msg::autostash_title("merge"), autostash_entries(), 2, 1);
        assert_rows_fit(&stash);
    }
}

#[test]
fn the_tooltips_and_statuses_follow_the_vocabulary() {
    let _l = override_lang(Lang::Fr);
    let tip = RemoteUiAction::RunPull(PullButtonMode::Rebase)
        .tooltip()
        .unwrap();
    assert_eq!(tip, "Fetch, puis rebase de vos commits locaux sur l'amont");
    let tip = RemoteUiAction::SetDefaultRemote("fork".into())
        .tooltip()
        .unwrap();
    assert_eq!(
        tip,
        "Faire de fork le distant utilisé par défaut par Push et Fetch"
    );
    let _t = override_git_terms(true);
    let tip = RemoteUiAction::SetDefaultRemote("fork".into())
        .tooltip()
        .unwrap();
    assert_eq!(
        tip,
        "Faire de fork le distant utilisé par défaut pour l'envoi et la récupération"
    );
    let head = HeadBranch {
        name: "main".into(),
        upstream: None,
    };
    let plan = push_plan(PushBehavior::Direct, Some(&head), Some("origin"), true);
    let PushPlan::Direct(_, said) = plan else {
        panic!("not direct")
    };
    assert_eq!(said, "poussée de main vers origin...");
    assert_eq!(
        msg::branches_hidden(1, "origin"),
        "1 branche de origin masquée du graphe"
    );
}
