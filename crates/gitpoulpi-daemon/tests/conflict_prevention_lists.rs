//! The Base branches lists in the daemon: the default list, a named list and
//! the repository's own patterns decide the base, and an edit of
//! `config.toml` or of the repository's choice applies to the next check with
//! no restart (settings are read with every request). Its own test binary: it
//! rewrites the scratch config, which the other conflict tests must not see.

use std::path::PathBuf;
use std::sync::{Arc, Once};

use gitpoulpi_core::domain::conflict_prevention::PredictionStatus;
use gitpoulpi_core::GitBackend;
use gitpoulpi_daemon::conflict_prevention::Predictor;
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::prediction_fixture::conflicting_feature;
use gitpoulpi_testkit::FixtureRepo;

fn config_dir() -> PathBuf {
    static ONCE: Once = Once::new();
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap().keep());
    ONCE.call_once(|| {
        std::env::set_var("GITPOULPI_CONFIG_DIR", dir);
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("HOME", dir);
    });
    dir.clone()
}

/// The tests share one config file: one at a time.
static CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn backend() -> Arc<dyn GitBackend> {
    config_dir();
    Arc::new(gitpoulpi_git2_backend::Git2Backend::new())
}

fn write_config(text: &str) {
    std::fs::write(config_dir().join("config.toml"), text).unwrap();
}

fn set_prefs(repo: &FixtureRepo, edit: impl FnOnce(&mut gitpoulpi_session::RepoPreferences)) {
    let path = RepoState::path_for(repo.root(), gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION);
    let mut state = RepoState::load(&path).unwrap_or_default();
    edit(&mut state.preferences);
    state.save(&path).unwrap();
}

fn status(predictor: &Predictor, repo: &FixtureRepo) -> PredictionStatus {
    predictor
        .predict_now(backend().as_ref(), &repo.repo_ref())
        .unwrap()
        .status
}

#[test]
fn a_changed_default_list_or_list_applies_to_the_next_check_without_a_restart() {
    let _one_at_a_time = CONFIG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let repo = conflicting_feature();
    let predictor = Predictor::default();

    // Nothing configured: the built-in `**` finds origin/main.
    write_config("");
    assert_eq!(status(&predictor, &repo), PredictionStatus::Conflicts);

    // A default list that allows no branch of this repository: no base.
    write_config(
        "[conflict_prevention]\ndefault_list = \"elsewhere\"\n\
         [conflict_prevention.lists.elsewhere]\npatterns = [\"upstream/**\"]\n",
    );
    assert_eq!(status(&predictor, &repo), PredictionStatus::NoBase);

    // Editing that list (not the default's name) applies as well.
    write_config(
        "[conflict_prevention]\ndefault_list = \"elsewhere\"\n\
         [conflict_prevention.lists.elsewhere]\npatterns = [\"origin/**\"]\n",
    );
    assert_eq!(status(&predictor, &repo), PredictionStatus::Conflicts);

    // The default is dropped (reset to the built-in): `**` again.
    write_config("[conflict_prevention.lists.elsewhere]\npatterns = [\"upstream/**\"]\n");
    assert_eq!(status(&predictor, &repo), PredictionStatus::Conflicts);
}

#[test]
fn own_patterns_beat_a_followed_list_which_beats_the_default() {
    let _one_at_a_time = CONFIG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let repo = conflicting_feature();
    let predictor = Predictor::default();
    write_config(
        "[conflict_prevention]\ndefault_list = \"nothing\"\n\
         [conflict_prevention.lists.nothing]\npatterns = [\"upstream/**\"]\n\
         [conflict_prevention.lists.trunk]\npatterns = [\"origin/main\"]\n",
    );
    // No choice: the default list, which allows nothing here.
    assert_eq!(status(&predictor, &repo), PredictionStatus::NoBase);

    // Following a list: it wins over the default.
    set_prefs(&repo, |p| {
        p.conflict_base_source = Some("list".into());
        p.conflict_base_list = Some("trunk".into());
    });
    assert_eq!(status(&predictor, &repo), PredictionStatus::Conflicts);

    // Own patterns win over the list the repository still remembers.
    set_prefs(&repo, |p| {
        p.conflict_base_source = Some("own".into());
        p.conflict_base_branches = Some("upstream/**".into());
    });
    assert_eq!(status(&predictor, &repo), PredictionStatus::NoBase);

    // An explicit "default" ignores both.
    set_prefs(&repo, |p| p.conflict_base_source = Some("default".into()));
    assert_eq!(status(&predictor, &repo), PredictionStatus::NoBase);

    // A file from before the choice existed: own patterns count as "own".
    set_prefs(&repo, |p| {
        p.conflict_base_source = None;
        p.conflict_base_list = None;
        p.conflict_base_branches = Some("origin/**".into());
    });
    assert_eq!(status(&predictor, &repo), PredictionStatus::Conflicts);

    // A followed list that was deleted falls back to the default.
    set_prefs(&repo, |p| {
        p.conflict_base_source = Some("list".into());
        p.conflict_base_list = Some("deleted".into());
        p.conflict_base_branches = None;
    });
    assert_eq!(status(&predictor, &repo), PredictionStatus::NoBase);
}
