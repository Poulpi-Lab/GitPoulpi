//! The Clone dialog against a **fake hosting service** (`xtask::mock_host`:
//! a GitHub Enterprise shaped side and a GitLab self-managed side on
//! 127.0.0.1, several owners each). The real binary, daemon and git, driven
//! through a pty; no network - the clone URLs are `file://` bare
//! repositories the mock seeds in a temp directory.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 60;

const HUB: &str = "GitHub \u{00B7} hub";
const LAB: &str = "GitLab \u{00B7} lab";

struct Setup {
    term: Terminal,
    mock: MockHost,
    clone_root: tempfile::TempDir,
    _bare: tempfile::TempDir,
    _config: tempfile::TempDir,
    _start: gitpoulpi_testkit::FixtureRepo,
}

/// The mock, a profile per side, and the TUI on the management tab with the
/// Clone dialog open (both sources listed). `token` is what the GitHub
/// profile's token is (`revoked` makes the mock answer 401).
fn open_clone(session: &str, token: &str) -> Setup {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .unwrap();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-hosted-start-commit")
        .unwrap()
        .build();
    let clone_root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n\n\
             [[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{gh}\" }}\n\n\
             [[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{gl}\" }}\n",
            root = clone_root.path().display(),
            port = mock.port,
            gh = mock_host::GITHUB_LOGIN,
            gl = mock_host::GITLAB_LOGIN,
        ),
    )
    .unwrap();
    let mut term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_HUB", token)
        .env("GITPOULPI_TOKEN_LAB", "lab-token")
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("clone-hosted-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog with both sources", STEP, |s| {
        s.contains("Clone a repository") && s.contains(HUB) && s.contains(LAB)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        clone_root,
        _bare: bare,
        _config: config,
        _start: start,
    }
}

/// Clicks the source labeled `label`.
fn pick_source(term: &mut Terminal, label: &str) {
    let (col, row) = term.screen().find(label).unwrap();
    term.click(col + 2, row).expect("pick the source");
}

fn row_of(screen: &Screen, text: &str) -> u16 {
    screen
        .find(text)
        .unwrap_or_else(|| panic!("no {text:?}:\n{}", screen.text()))
        .1
}

/// Focuses the search box (four rows under the "Where to clone to" label:
/// path, gap, "Repository" label, then the box) and replaces its text.
fn search(term: &mut Terminal, text: &str) {
    let (col, row) = term.wait_for_text("Where to clone to", STEP).unwrap();
    term.click(col + 2, row + 4).expect("focus the search");
    // The click can land the caret inside the previous query (C-u then left
    // "umen" of "lumen" behind): go to its end before clearing.
    term.send_keys("End").unwrap();
    term.send_keys("C-u").unwrap();
    term.send_text(text).unwrap();
}

/// One section per owner inside a source, the connected user's own first,
/// the others alphabetical; the two providers' repositories never mix.
/// (The list shows six rows at a time: later sections are reached by search.)
#[test]
fn each_source_lists_its_own_owners_as_sections_and_the_providers_never_merge() {
    let mut s = open_clone("pty-clone-hosted-groups", "hub-token");
    pick_source(&mut s.term, HUB);
    let github = s
        .term
        .wait_for("the first GitHub sections", STEP, |x| {
            x.contains("mara-quinn (2)") && x.contains("harbor-guild (2)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        row_of(&github, "mara-quinn (2)") < row_of(&github, "harbor-guild (2)"),
        "the connected account's section comes first:\n{}",
        github.text()
    );
    assert!(
        github
            .line(row_of(&github, "mara-quinn/notes-vault"))
            .contains('\u{1F512}'),
        "lock on a private repository:\n{}",
        github.text()
    );
    assert!(
        !github
            .line(row_of(&github, "mara-quinn/dotfiles"))
            .contains('\u{1F512}'),
        "no lock on a public one:\n{}",
        github.text()
    );
    assert!(
        !github.contains("northwind"),
        "no GitLab owner in the GitHub source:\n{}",
        github.text()
    );
    // A GitLab group searched in the GitHub source finds nothing.
    search(&mut s.term, "northwind");
    s.term
        .wait_for("no GitHub match", STEP, |x| {
            x.contains("no repository matches") && !x.contains("mara-quinn")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    pick_source(&mut s.term, LAB);
    let gitlab = s
        .term
        .wait_for("the GitLab sections", STEP, |x| {
            x.contains("mara (2)") && x.contains("northwind (3)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        row_of(&gitlab, "mara (2)") < row_of(&gitlab, "northwind (3)")
            && !gitlab.contains("harbor-guild")
            && !gitlab.contains("lumen-labs"),
        "{}",
        gitlab.text()
    );
    // And a GitHub organisation searched in the GitLab source finds nothing.
    search(&mut s.term, "lumen");
    s.term
        .wait_for("no GitLab match", STEP, |x| {
            x.contains("no repository matches") && !x.contains("lumen-labs")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // A sub-group is its own section, named by its full path.
    search(&mut s.term, "quarry");
    s.term
        .wait_for("the sub-group section", STEP, |x| {
            x.contains("quarry/tools (2)") && x.contains("chisel") && !x.contains("northwind")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.mock.saw("GET", "/api/v3/user/repos"));
    assert!(s.mock.saw("GET", "/api/v4/projects"));
}

/// The search narrows across sections (the empty ones and their headers
/// go), and clearing it brings them back.
#[test]
fn the_search_narrows_the_owner_sections() {
    let mut s = open_clone("pty-clone-hosted-search", "hub-token");
    pick_source(&mut s.term, HUB);
    s.term
        .wait_for("the sections", STEP, |x| {
            x.contains("mara-quinn (2)") && x.contains("harbor-guild (2)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    search(&mut s.term, "lumen");
    s.term
        .wait_for("only the lumen-labs section", STEP, |x| {
            x.contains("lumen-labs (5)") && !x.contains("mara-quinn") && !x.contains("harbor-guild")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("C-u").unwrap();
    s.term
        .wait_for("the first sections again", STEP, |x| {
            x.contains("mara-quinn (2)") && x.contains("harbor-guild (2)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Choosing a repository fills the clone, and the clone really happens
/// (from the mock's `file://` bare repository): the folder is created under
/// the clone root and the new repository opens as a tab.
#[test]
fn choosing_a_hosted_repository_clones_it_for_real() {
    let mut s = open_clone("pty-clone-hosted-clone", "hub-token");
    pick_source(&mut s.term, LAB);
    s.term
        .wait_for("the GitLab sections", STEP, |x| x.contains("northwind (3)"))
        .unwrap_or_else(|e| panic!("{e}"));
    search(&mut s.term, "rockhound");
    s.term
        .wait_for("only rockhound", STEP, |x| {
            x.contains("quarry/tools/rockhound") && !x.contains("chisel")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the choice filled the clone", STEP, |x| {
            x.contains("Full path:") && x.contains("rockhound") && x.contains("\u{2713}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the clone open as a tab", STARTUP, |x| {
            x.contains("Initial commit") && !x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let dest = s.clone_root.path().join("rockhound");
    assert!(
        dest.join(".git").is_dir() && dest.join("README.md").is_file(),
        "no clone at {}",
        dest.display()
    );
}

/// The host answers 401 (a revoked token): the dialog says the list could
/// not be fetched (loosely: the wording belongs to the failure card work),
/// and the other source still lists.
#[test]
fn a_401_from_the_host_is_reported_and_the_other_source_still_lists() {
    let mut s = open_clone("pty-clone-hosted-401", "revoked");
    pick_source(&mut s.term, HUB);
    s.term
        .wait_for("an error instead of the list", STEP, |x| {
            x.contains("could not list repositories") && !x.contains("lumen-labs")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    pick_source(&mut s.term, LAB);
    s.term
        .wait_for("the GitLab list", STEP, |x| x.contains("northwind (3)"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The sources are reachable without the mouse: Alt+S focuses them from
/// anywhere, Down walks them (the list of a source passed over is never
/// asked for), Enter takes the one the highlight rests on to its search,
/// and a clone from it works.
#[test]
fn the_sources_are_reachable_by_keyboard_and_only_the_rested_one_loads() {
    let mut s = open_clone("pty-clone-hosted-keys", "hub-token");
    // The focus starts in the URL field: Alt+S goes to the sources.
    s.term.send_keys("M-s").unwrap();
    // Walk URL -> hub -> lab quickly: hub is passed over.
    s.term.send_keys("Down Down").unwrap();
    s.term
        .wait_for(
            "the GitLab list, loaded by the rested highlight",
            STEP,
            |x| x.contains("northwind (3)"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !s.mock.saw("GET", "/api/v3/user/repos"),
        "the GitHub list was asked for although the highlight only passed it"
    );
    // Enter leaves the sources for the search; type, choose, clone.
    s.term.send_keys("Enter").unwrap();
    s.term.send_text("rockhound").unwrap();
    s.term
        .wait_for("only rockhound", STEP, |x| {
            x.contains("quarry/tools/rockhound") && !x.contains("chisel")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the choice filled the clone", STEP, |x| {
            x.contains("Full path:") && x.contains("\u{2713}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the clone open as a tab", STARTUP, |x| {
            x.contains("Initial commit") && !x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.clone_root.path().join("rockhound").join(".git").is_dir());
}

/// In the sources list `j` and `k` move the highlight exactly like Down
/// and Up (j j k lands on GitHub, whose list loads and GitLab's never
/// does); in the search field the same letters are typed.
#[test]
fn j_and_k_walk_the_sources_like_the_arrows_and_still_type_in_the_search() {
    let mut s = open_clone("pty-clone-hosted-jk", "hub-token");
    s.term.send_keys("M-s").unwrap();
    // URL(0) -> j hub(1) -> j lab(2) -> k hub(1): same as Down Down Up.
    s.term.send_text("jjk").unwrap();
    s.term
        .wait_for(
            "the GitHub list, loaded by the rested highlight",
            STEP,
            |x| x.contains("mara-quinn (2)"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        s.mock.saw("GET", "/api/v3/user/repos"),
        "GitHub never listed"
    );
    assert!(
        !s.mock.saw("GET", "/api/v4/projects"),
        "GitLab listed: j j k did not land on GitHub"
    );
    s.term.send_keys("Enter").unwrap();
    s.term.send_text("jk").unwrap();
    s.term
        .wait_for("jk typed in the search", STEP, |x| x.contains("jk"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Shift+Tab from the first field also reaches the sources: they are the
/// first Tab stop, and the footer names the focus key.
#[test]
fn the_sources_are_the_first_tab_stop_and_the_footer_names_the_key() {
    let mut s = open_clone("pty-clone-hosted-tabstop", "hub-token");
    s.term
        .wait_for("the footer hint", STEP, |x| {
            x.contains("Alt+S") && x.contains("sources")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // URL -> Name -> Shallow -> Sparse -> Clone -> Cancel -> Sources.
    s.term.send_keys("Tab Tab Tab Tab Tab Tab").unwrap();
    s.term.send_keys("Down").unwrap();
    s.term
        .wait_for("the highlight moved to the first profile", STEP, |x| {
            x.contains("northwind") || x.contains("mara-quinn")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}
