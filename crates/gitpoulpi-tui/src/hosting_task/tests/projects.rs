//! The Clone dialog's repository fetch: the saved copy first, the host's
//! list after it, saved for next time; a dropped receiver cancels it.

use super::*;
use gitpoulpi_hosting::list_cache::{self, CachedList, ListKey};
use std::time::Duration;

fn project(full: &str) -> gitpoulpi_hosting::HostedProject {
    gitpoulpi_hosting::HostedProject {
        integration_id: "corp".into(),
        id: full.into(),
        name: full.rsplit('/').next().unwrap().into(),
        full_name: full.into(),
        description: None,
        default_branch: None,
        ssh_url: None,
        https_url: None,
        web_url: None,
        private: false,
    }
}

fn job(dir: &std::path::Path) -> ListCacheJob {
    ListCacheJob {
        dir: dir.to_path_buf(),
        key: ListKey::of(&gitlab_profile(), None).unwrap(),
    }
}

fn next(rx: &ProjectsReceiver) -> ProjectsAnswer {
    rx.rx
        .recv_timeout(Duration::from_secs(10))
        .expect("an answer")
}

fn seed(dir: &std::path::Path, names: &[&str]) {
    let key = job(dir).key;
    let list = CachedList::new(key, 1, names.iter().map(|n| project(n)).collect());
    list_cache::store(dir, &list).unwrap();
}

#[test]
fn the_saved_copy_comes_first_then_the_host_list_which_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &["g/a"]);
    let mut factory = FakeFactory::new();
    factory.projects = vec![project("g/a"), project("g/b")];
    let rx = start_projects(
        Arc::new(factory),
        gitlab_profile(),
        vec![gitlab_profile()],
        Some(job(dir.path())),
    );
    let first = next(&rx);
    assert_eq!(first.phase, ListPhase::Cached);
    assert_eq!(first.result.unwrap().len(), 1);
    let last = next(&rx);
    assert_eq!(last.phase, ListPhase::Final);
    assert_eq!(last.result.unwrap().len(), 2);
    let saved = list_cache::load(dir.path(), &job(dir.path()).key).unwrap();
    assert_eq!(saved.projects.len(), 2, "the fresh list replaces the copy");
}

#[test]
fn without_a_cache_job_only_the_final_answer_comes_and_nothing_is_written() {
    let mut factory = FakeFactory::new();
    factory.projects = vec![project("g/a")];
    let rx = start_projects(Arc::new(factory), gitlab_profile(), vec![], None);
    let only = next(&rx);
    assert_eq!(only.phase, ListPhase::Final);
    assert!(only.result.is_ok());
}

#[test]
fn a_rejected_token_deletes_the_saved_copy_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &["g/a"]);
    let mut factory = FakeFactory::new();
    factory.fail = Some(|| HostingError::Unauthorized {
        what: "list".into(),
        detail: "401".into(),
    });
    let rx = start_projects(
        Arc::new(factory),
        gitlab_profile(),
        vec![gitlab_profile()],
        Some(job(dir.path())),
    );
    assert_eq!(next(&rx).phase, ListPhase::Cached);
    let last = next(&rx);
    assert_eq!(last.phase, ListPhase::Final);
    assert!(last.result.is_err());
    assert!(last.unauthorized);
    assert!(list_cache::load(dir.path(), &job(dir.path()).key).is_none());
}

#[test]
fn another_failure_keeps_the_saved_copy() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &["g/a"]);
    let mut factory = FakeFactory::new();
    factory.fail = Some(|| HostingError::Unsupported("offline".into()));
    let rx = start_projects(
        Arc::new(factory),
        gitlab_profile(),
        vec![gitlab_profile()],
        Some(job(dir.path())),
    );
    assert_eq!(next(&rx).phase, ListPhase::Cached);
    let last = next(&rx);
    assert!(last.result.is_err() && !last.unauthorized);
    assert!(list_cache::load(dir.path(), &job(dir.path()).key).is_some());
}

#[test]
fn a_dropped_receiver_cancels_the_fetch_and_nothing_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut factory = FakeFactory::new();
    factory.projects = vec![project("g/a")];
    // Building the client (the token read) takes a while: the receiver is
    // gone by the time the list could be fetched.
    factory.client_cost = Duration::from_millis(300);
    let built = Arc::clone(&factory.built);
    let rx = start_projects(
        Arc::new(factory),
        gitlab_profile(),
        vec![gitlab_profile()],
        Some(job(dir.path())),
    );
    drop(rx);
    for _ in 0..200 {
        if built.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(600));
    assert!(
        list_cache::load(dir.path(), &job(dir.path()).key).is_none(),
        "a cancelled fetch stores nothing"
    );
}
