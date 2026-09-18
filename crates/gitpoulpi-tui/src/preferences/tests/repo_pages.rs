//! The Sparse checkout and LFS pages of the repository group: what they list,
//! the requests their buttons make, the greyed buttons, and the paths editor.

use super::*;
use crate::preferences::repo_pages::{RpButton, RpTarget, SparseRequest};
use crossterm::event::{KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{LfsBinary, LfsPattern, LfsRepoStatus, SparseState};

fn sparse_page(dir: &Path) -> PreferencesState {
    state_at(dir, "SparseCheckout")
}

fn lfs_page(dir: &Path) -> PreferencesState {
    state_at(dir, "Lfs")
}

fn on(dirs: &[&str]) -> SparseState {
    SparseState {
        enabled: true,
        cone: true,
        dirs: dirs.iter().map(|d| d.to_string()).collect(),
    }
}

fn key(st: &mut PreferencesState, code: KeyCode, mods: KeyModifiers) {
    st.on_key(KeyEvent::new(code, mods));
}

/// Types `text` in the paths editor (Enter is a new line) and saves it.
fn edit_paths(st: &mut PreferencesState, text: &str) {
    st.rp_open_editor(RpTarget::SparsePaths);
    assert!(st.rp_editor_open());
    for c in text.chars() {
        let code = if c == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(c)
        };
        key(st, code, KeyModifiers::NONE);
    }
    key(st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(!st.rp_editor_open());
}

fn field(fields: &[Field], label: &str) -> Field {
    let found = fields.iter().find(|f| f.label == label);
    let found = found.unwrap_or_else(|| {
        panic!(
            "no `{label}` in {:?}",
            fields.iter().map(|f| &f.label).collect::<Vec<_>>()
        )
    });
    Field {
        label: found.label.clone(),
        control: found.control.clone(),
        help: found.help.clone(),
        warning: found.warning.clone(),
        target: found.target.clone(),
        live: found.live,
        coming: found.coming.clone(),
        placeholder: found.placeholder.clone(),
    }
}

fn status(hooks: bool, uses: bool, patterns: &[&str]) -> LfsRepoStatus {
    LfsRepoStatus {
        binary: LfsBinary::Installed {
            version: "3.5.1".into(),
        },
        uses_lfs: uses,
        filters_configured: hooks,
        hooks_installed: hooks,
        patterns: patterns
            .iter()
            .map(|p| LfsPattern {
                pattern: p.to_string(),
                lockable: false,
                source: ".gitattributes".into(),
            })
            .collect(),
    }
}

/// The sparse page's one row of buttons.
fn sparse_row(fields: &[Field]) -> &Field {
    fields
        .iter()
        .find(|f| {
            matches!(&f.control, Control::Buttons(b)
            if b[0].action == PrefAction::RepoPage(RpButton::Apply))
        })
        .expect("the sparse buttons row")
}

fn button_labels(field: &Field) -> Vec<&str> {
    match &field.control {
        Control::Buttons(b) => b.iter().map(|b| b.label.as_str()).collect(),
        other => panic!("not a buttons row: {other:?}"),
    }
}

fn has(fields: &[Field], label: &str) -> bool {
    fields.iter().any(|f| f.label == label)
}

#[test]
fn both_pages_are_in_the_repository_group_with_stable_ids() {
    assert!(PrefSection::SparseCheckout.is_repo_scoped());
    assert!(PrefSection::Lfs.is_repo_scoped());
    assert_eq!(PrefSection::SparseCheckout.id(), "sparse-checkout");
    assert_eq!(PrefSection::Lfs.id(), "lfs");
    for id in ["sparse-checkout", "lfs"] {
        let page = gitpoulpi_ipc::ui::SettingsPage::parse(id).unwrap();
        assert_eq!(PrefSection::from_page(page).id(), id);
    }
}

#[test]
fn off_the_page_offers_enable_and_greys_what_does_not_apply() {
    let dir = tempfile::tempdir().unwrap();
    let st = sparse_page(dir.path());
    let fields = st.fields();
    assert_eq!(field(&fields, "State").value_text(), "disabled");
    let paths = field(&fields, "Directories to keep");
    assert_eq!(paths.control, Control::Area(String::new()));
    assert!(paths.placeholder.as_deref().unwrap().starts_with("e.g."));
    // An empty list: only Enable shows, greyed with the reason. Disable and
    // Reapply do not apply to a full checkout, so they are not offered.
    let row = sparse_row(&fields);
    assert_eq!(button_labels(row), ["Start sparse checkout"]);
    assert!(!row.selectable(), "Enable must be greyed out");
    assert!(row.coming.is_some());
}

#[test]
fn enable_sends_the_edited_paths_one_per_line() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    edit_paths(&mut st, "src/\ndocs/guide\n\n");
    let fields = st.fields();
    assert!(sparse_row(&fields).selectable());
    st.run_action(PrefAction::RepoPage(RpButton::Apply));
    assert_eq!(
        st.take_sparse_request(),
        Some(SparseRequest::Set(vec!["src".into(), "docs/guide".into()]))
    );
    assert_eq!(st.take_sparse_request(), None, "a request is taken once");
}

#[test]
fn a_bad_path_is_refused_with_the_dialogs_wording_and_sends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    edit_paths(&mut st, "src\n../etc");
    st.run_action(PrefAction::RepoPage(RpButton::Apply));
    assert_eq!(st.take_sparse_request(), None);
    let said = st.take_message().unwrap();
    assert!(said.contains("not a valid directory"), "{said}");
}

#[test]
fn when_on_the_field_shows_the_current_paths_and_the_button_updates() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.set_sparse(&on(&["src", "docs"]));
    let fields = st.fields();
    assert_eq!(field(&fields, "State").value_text(), "enabled");
    assert_eq!(
        field(&fields, "Directories to keep").control,
        Control::Area("src\ndocs".into())
    );
    let row = sparse_row(&fields);
    assert_eq!(
        button_labels(row),
        [
            "Apply directory list",
            "Refresh sparse checkout",
            "Stop sparse checkout"
        ]
    );
    assert!(row.selectable());
    // Update applies the edit (the caret opens at the end of the text).
    edit_paths(&mut st, "\nlib");
    st.run_action(PrefAction::RepoPage(RpButton::Apply));
    assert_eq!(
        st.take_sparse_request(),
        Some(SparseRequest::Set(vec![
            "src".into(),
            "docs".into(),
            "lib".into()
        ]))
    );
}

#[test]
fn reapply_sends_the_current_paths_not_the_draft_and_disable_disables() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.set_sparse(&on(&["src"]));
    edit_paths(&mut st, "\nunsent");
    st.run_action(PrefAction::RepoPage(RpButton::Reapply));
    assert_eq!(
        st.take_sparse_request(),
        Some(SparseRequest::Set(vec!["src".into()]))
    );
    st.run_action(PrefAction::RepoPage(RpButton::Disable));
    assert_eq!(st.take_sparse_request(), Some(SparseRequest::Disable));
}

#[test]
fn a_greyed_button_that_is_triggered_anyway_does_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.run_action(PrefAction::RepoPage(RpButton::Disable));
    st.run_action(PrefAction::RepoPage(RpButton::Reapply));
    assert_eq!(st.take_sparse_request(), None);
}

#[test]
fn a_new_state_from_the_repository_drops_the_unsent_draft() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    edit_paths(&mut st, "typed");
    st.set_sparse(&SparseState::default());
    assert_eq!(
        field(&st.fields(), "Directories to keep").control,
        Control::Area("typed".into()),
        "the same state keeps the draft"
    );
    st.set_sparse(&on(&["other"]));
    assert_eq!(
        field(&st.fields(), "Directories to keep").control,
        Control::Area("other".into()),
        "a changed state wins"
    );
}

#[test]
fn escape_closes_the_editor_without_keeping_the_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.rp_open_editor(RpTarget::SparsePaths);
    key(&mut st, KeyCode::Char('x'), KeyModifiers::NONE);
    key(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(st.rp_editor_open(), "edited text asks first");
    key(&mut st, KeyCode::Right, KeyModifiers::NONE);
    key(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!st.rp_editor_open());
    assert_eq!(
        field(&st.fields(), "Directories to keep").control,
        Control::Area(String::new())
    );
}

#[test]
fn the_sparse_page_draws_its_field_help_and_buttons() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    let screen = draw_screen(&mut st);
    for want in [
        "Sparse checkout",
        "Directories to keep",
        "e.g. src/",
        "Start sparse checkout",
        "enter at least one directory above",
    ] {
        assert!(screen.contains(want), "{want} missing:\n{screen}");
    }
    assert!(!screen.contains("Stop sparse checkout"), "{screen}");
    st.set_sparse(&on(&["src"]));
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Apply directory list"), "{screen}");
    assert!(screen.contains("Stop sparse checkout"), "{screen}");
    assert!(!screen.contains("Start sparse checkout"), "{screen}");
}

#[test]
fn the_editor_is_drawn_with_clickable_save_and_cancel() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.rp_open_editor(RpTarget::SparsePaths);
    let screen = draw_screen(&mut st);
    assert!(
        screen.contains("Save") && screen.contains("Cancel"),
        "{screen}"
    );
    let (col, row) = screen
        .lines()
        .enumerate()
        .find_map(|(y, l)| l.find("Cancel").map(|x| (x as u16, y as u16)))
        .unwrap();
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), col + 1, row);
    assert!(!st.rp_editor_open(), "Cancel by mouse closes it");
}

#[test]
fn the_in_flight_text_shows_on_the_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = sparse_page(dir.path());
    st.set_in_flight(Some("applying the sparse checkout...".into()));
    assert!(draw_screen(&mut st).contains("applying the sparse checkout..."));
}

#[test]
fn lfs_page_says_it_is_not_initialized_and_the_button_asks_for_the_install() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = lfs_page(dir.path());
    let name = st.repo_label.clone();
    st.set_lfs_status(Some(&status(false, false, &[])));
    let fields = st.fields();
    assert!(has(&fields, &format!("Git LFS is not set up in {name}")));
    let button = field(&fields, "Set up LFS here");
    assert_eq!(button.target, Target::Action(PrefAction::InitializeLfs));
    assert!(button.selectable());
    st.run_action(PrefAction::InitializeLfs);
    assert!(st.take_lfs_init_request());
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Set up LFS here"), "{screen}");
}

#[test]
fn lfs_page_when_initialized_has_no_button_and_lists_the_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = lfs_page(dir.path());
    let name = st.repo_label.clone();
    st.set_lfs_status(Some(&status(true, true, &["*.psd", "*.bin"])));
    let fields = st.fields();
    assert!(has(&fields, &format!("Git LFS is set up in {name}")));
    assert!(!has(&fields, "Set up LFS here"));
    assert!(has(&fields, "*.psd") && has(&fields, "*.bin"));
}

#[test]
fn lfs_page_without_a_state_or_with_lfs_off_shows_a_note_and_no_button() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = lfs_page(dir.path());
    let fields = st.fields();
    assert!(has(&fields, "reading the LFS state..."));
    assert!(fields
        .iter()
        .all(|f| !matches!(f.control, Control::Buttons(_))));
    st.repo_state.preferences.lfs_enabled = Some(false);
    st.set_lfs_status(Some(&status(true, true, &[])));
    let fields = st.fields();
    assert!(fields
        .iter()
        .any(|f| f.label.starts_with("LFS is turned off")));
    assert!(fields
        .iter()
        .all(|f| !matches!(f.control, Control::Buttons(_))));
}

#[test]
fn a_repo_that_uses_lfs_without_hooks_is_not_initialized_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = lfs_page(dir.path());
    st.set_lfs_status(Some(&status(false, true, &["*.bin"])));
    let fields = st.fields();
    assert!(fields
        .iter()
        .any(|f| f.label.contains("hooks are not installed")));
    assert!(has(&fields, "Set up LFS here"));
}
