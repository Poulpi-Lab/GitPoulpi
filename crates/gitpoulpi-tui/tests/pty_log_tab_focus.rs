//! Tab (or Shift+Tab) into the graph, then the arrows and `j`/`k` move its
//! selection at once. The changed-files list is a sub-focus of the graph
//! pane: leaving it with Tab used to keep it, so after a lap of Tab the keys
//! drove the files list and the graph looked dead until a click.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const SESSION: &str = "log-tab-focus";

fn state(config: &Path) -> (Option<String>, Option<i64>) {
    let s = common::ui_state(config, SESSION);
    let row = s.log.as_ref().and_then(|l| l["selected_row"].as_i64());
    (s.focus, row)
}

fn wait_state(config: &Path, what: &str, want: impl Fn(&(Option<String>, Option<i64>)) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let got = state(config);
        if want(&got) {
            return;
        }
        assert!(Instant::now() < deadline, "{what}: stuck at {got:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn launch(config: &Path, repo: &Path) -> Terminal {
    let term = common::launch(repo)
        .session(SESSION)
        .config_dir(config)
        .size(180, 50)
        .ready("newest commit")
        .spawn();
    common::wait_for_viewer(config, SESSION);
    term
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base commit")
        .unwrap()
        .commit_file("b.txt", "b\n", "middle commit")
        .unwrap()
        .commit_file("c.txt", "c\n", "newest commit")
        .unwrap()
        .build()
}

fn press(term: &mut Terminal, config: &Path, key: &str, focus: &str) {
    term.send_keys(key).unwrap();
    wait_state(config, key, |(f, _)| f.as_deref() == Some(focus));
}

#[test]
fn keys_move_the_graph_after_tab_even_from_the_files_list() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(config.path(), repo.root());
    let c = config.path();

    // Into the changed-files list (a sub-focus of the graph), then a lap.
    term.wait_for("the newest commit's file", Duration::from_secs(30), |s| {
        s.contains("c.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    press(&mut term, c, "Tab", "status");
    press(&mut term, c, "Tab", "refs");
    press(&mut term, c, "Tab", "log");
    term.send_keys("j").unwrap();
    wait_state(c, "j after Tab", |(_, r)| *r == Some(1));
    term.send_keys("Down").unwrap();
    wait_state(c, "Down", |(_, r)| *r == Some(2));
    term.send_keys("k").unwrap();
    wait_state(c, "k", |(_, r)| *r == Some(1));
    term.send_keys("Up").unwrap();
    wait_state(c, "Up", |(_, r)| *r == Some(0));

    // The reverse lap, again from the files list.
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    press(&mut term, c, "BTab", "refs");
    press(&mut term, c, "BTab", "status");
    press(&mut term, c, "BTab", "log");
    term.send_keys("j").unwrap();
    wait_state(c, "j after Shift+Tab", |(_, r)| *r == Some(1));
}
