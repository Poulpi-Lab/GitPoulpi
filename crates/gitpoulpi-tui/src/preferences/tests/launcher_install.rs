//! The "This installation" rows of the Launcher page: what they show, what
//! they write (this installation's entry only), and the buttons around them.
//! A fake installation and temp directories; never the real config.

use super::*;
use crate::preferences::launcher_install::{fake_current, InstallButton};
use gitpoulpi_config::desktop::folder_menu::Env as MenuEnv;
use gitpoulpi_config::desktop::registry::MemoryRegistry;
use gitpoulpi_config::desktop::Place;
use gitpoulpi_config::{InstallationEntry, Kind};

const ME: &str = "aaaa000000000001";

fn install_state(dir: &Path) -> PreferencesState {
    let mut st = state(dir);
    st.set_this(fake_current(false));
    st
}

fn rows(st: &PreferencesState) -> Vec<Field> {
    st.fields_of(PrefSection::Launcher)
}

fn row(st: &PreferencesState, label: &str) -> Field {
    rows(st)
        .into_iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
}

/// Draws the page `width` columns wide.
fn draw_at(st: &mut PreferencesState, width: u16) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(width, 60)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..60)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn other_entry(name: &str, path: &str, submenu: &str, env: &str) -> InstallationEntry {
    InstallationEntry {
        path: path.to_string(),
        kind: Kind::Dev,
        name: name.to_string(),
        folder_menu_submenu: submenu.to_string(),
        env: env.to_string(),
        ..Default::default()
    }
}

fn written(dir: &Path) -> toml::Value {
    toml::from_str(&std::fs::read_to_string(dir.join("config.toml")).unwrap()).unwrap()
}

#[test]
fn installation_rows_render_name_colour_env_and_path() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    go(&mut st, PrefSection::Launcher);
    let screen = draw_at(&mut st, 120);
    for want in [
        "This installation",
        "Installation",
        "/opt/gp/a",
        "Launcher name",
        "GitPoulpi (portable)",
        "Include version in the launcher",
        "Octopus colour",
        "Become the primary",
        "Environment",
        "own (default)",
        "Folder-menu submenu",
    ] {
        assert!(screen.contains(want), "{want} is missing:\n{screen}");
    }
    // The Installation row names kind, path and environment.
    let Control::Info(text) = row(&st, "Installation").control else {
        panic!("an info row");
    };
    assert_eq!(text, format!("portable, /opt/gp/a, environment {ME}"));
}

#[test]
fn a_narrow_page_ellipsises_the_path_never_drops_it() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let mut current = fake_current(false);
    current.identity.root = "/very/long/folder/of/an/install/gitpoulpi-tools".to_string();
    st.set_this(current);
    go(&mut st, PrefSection::Launcher);
    let screen = draw_at(&mut st, 70);
    let line = screen
        .lines()
        .find(|l| l.contains('\u{2026}') && l.contains("portable, /"))
        .unwrap_or_else(|| panic!("the Installation row is drawn:\n{screen}"));
    assert!(line.contains('\u{2026}'), "cut in the middle: {line}");
    assert!(line.contains("/ver"), "the start of the path stays: {line}");
    assert!(line.contains("ols"), "the end of the path stays: {line}");
    // The full text stays one key away: the help line under the row.
    let full = row(&st, "Installation");
    let help = full.help.as_deref().unwrap();
    assert!(help.contains("Folder: /very/long/folder/of/an/install/gitpoulpi-tools"));
    // ... without repeating the value's sentence.
    assert!(!help.contains("portable, /"), "{help}");
    assert!(!help.contains("environment aaaa"), "{help}");
}

#[test]
fn legacy_launchers_row_offers_both_buttons() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    // Nothing found: no row.
    assert!(rows(&st).iter().all(|f| f.label != "Legacy launchers"));
    st.set_legacy(vec![Place::Launcher]);
    let legacy = row(&st, "Legacy launchers");
    assert!(matches!(&legacy.control, Control::Info(t) if t.starts_with("found: ")));
    let buttons: Vec<(String, PrefAction)> = rows(&st)
        .into_iter()
        .filter_map(|f| match f.control {
            Control::Buttons(b) => Some(b),
            _ => None,
        })
        .flatten()
        .filter(|b| matches!(b.action, PrefAction::Installation(_)))
        .map(|b| (b.label, b.action))
        .collect();
    assert_eq!(
        buttons,
        [
            (
                "Become the primary".to_string(),
                PrefAction::Installation(InstallButton::BecomePrimary)
            ),
            (
                "Remove legacy launchers".to_string(),
                PrefAction::Installation(InstallButton::RemoveLegacy)
            ),
        ]
    );
    // A primary owns the legacy names: nothing left to take over.
    st.config.installation.primary = Some(ME.to_string());
    assert!(rows(&st).iter().all(|f| f.label != "Legacy launchers"));
}

#[test]
fn name_colour_and_version_write_this_installations_entry_only() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    assert!(st.write_global("installation.name", "GitPoulpi (mine)"));
    assert!(st.write_global("installation.logo_color", "teal"));
    assert!(st.write_global("installation.name_with_version", "true"));
    let file = written(tmp.path());
    let entry = &file["installation"]["settings"][ME];
    assert_eq!(entry["name"].as_str(), Some("GitPoulpi (mine)"));
    assert_eq!(entry["logo_color"].as_str(), Some("teal"));
    assert_eq!(entry["name_with_version"].as_bool(), Some(true));
    assert!(file.get("ui").is_none(), "ui.* is for the primary: {file}");
    assert_eq!(row(&st, "Octopus colour").value_text(), "Teal");
}

#[test]
fn the_primary_also_writes_the_keys_older_builds_read() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    st.set_this(fake_current(true));
    assert!(st.write_global("installation.logo_color", "teal"));
    assert!(st.write_global("installation.command_alias", "gp"));
    let file = written(tmp.path());
    assert_eq!(file["ui"]["logo_color"].as_str(), Some("teal"));
    assert_eq!(file["ui"]["command_alias"].as_str(), Some("gp"));
    // The primary is always called GitPoulpi.
    assert!(!st.write_global("installation.name", "Other"));
    assert!(row(&st, "Launcher name").coming.is_some());
}

#[test]
fn a_name_another_live_installation_uses_is_refused_with_a_note() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "", ""),
    );
    assert!(!st.write_global("installation.name", "gitpoulpi (DEV)"));
    assert_eq!(
        st.take_message().as_deref(),
        Some("gitpoulpi (DEV) is already the name of another installation")
    );
    assert!(!tmp.path().join("config.toml").exists(), "nothing written");
    // A gone installation does not hold its name.
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", "/no/such/folder", "", ""),
    );
    assert!(st.write_global("installation.name", "GitPoulpi (dev)"));
    // A name no launcher file can have is refused.
    assert!(!st.write_global("installation.name", "a/b"));
}

#[test]
fn environment_and_submenu_offer_what_others_use_and_warn_before_restart() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "Tools", "shared"),
    );
    let values = |f: Field| match f.control {
        Control::Combo {
            options, custom, ..
        } => {
            assert!(custom, "a name can be typed");
            options.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
        }
        other => panic!("a combobox: {other:?}"),
    };
    assert_eq!(values(row(&st, "Environment")), ["", "shared"]);
    if cfg!(not(target_os = "macos")) {
        assert_eq!(values(row(&st, "Folder-menu submenu")), ["", "Tools"]);
    }
    assert!(st.write_global("installation.env", "shared"));
    assert_eq!(
        st.take_message().as_deref(),
        Some("Restart this installation's open windows for the new environment to apply.")
    );
    let file = written(tmp.path());
    assert_eq!(
        file["installation"]["settings"][ME]["env"].as_str(),
        Some("shared")
    );
    assert!(st.write_global("installation.folder_menu_submenu", "Tools"));
    assert_eq!(
        st.own_entry().folder_menu_submenu,
        "Tools",
        "the submenu is this installation's"
    );
}

#[test]
fn a_gone_installation_gets_a_forget_button_and_a_live_one_does_not() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "", ""),
    );
    st.config.installation.settings.insert(
        "cccc000000000003".to_string(),
        other_entry("GitPoulpi (old)", "/no/such/folder", "", ""),
    );
    let forgets = |st: &PreferencesState| {
        rows(st)
            .into_iter()
            .filter(|f| matches!(f.target, Target::Action(PrefAction::Installation(_))))
            .count()
    };
    assert_eq!(forgets(&st), 1, "only the gone one");
    assert!(row(&st, "Other: GitPoulpi (old)")
        .value_text()
        .starts_with("gone, "));
    assert!(row(&st, "Other: GitPoulpi (dev)")
        .value_text()
        .starts_with("dev, "));
    // Forget asks first (a destructive button).
    let forget = rows(&st)
        .into_iter()
        .find_map(|f| match f.control {
            Control::Buttons(b) => b.into_iter().find(|b| b.label == "Forget"),
            _ => None,
        })
        .unwrap();
    st.press_button(&forget);
    assert!(st.confirm_action.is_some(), "a question, not a deletion");
}

#[test]
fn an_alias_of_another_installation_reads_used_by_and_create_is_disabled() {
    use gitpoulpi_config::desktop::alias::{State, Status};
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &tmp.path().display().to_string(), "", ""),
    );
    let text = st.alias_owner_text(&Status {
        state: State::Other,
        owner: Some("bbbb000000000002".to_string()),
        shadowed_by: None,
        powershell_builtin: false,
    });
    assert_eq!(text, "Used by GitPoulpi (dev)");
    // The reason is shown as the row's value, not behind "coming".
    let row = Field::plain("x", Control::Info(String::new()), Target::None).coming(&text);
    assert_eq!(row.value_text(), text);
}

/// A launcher machine under `root` (never the real one).
fn temp_machine(root: &Path) -> crate::launcher::Env {
    use gitpoulpi_config::desktop::Dirs;
    std::fs::create_dir_all(root.join("Desktop")).unwrap();
    crate::launcher::Env {
        os: gitpoulpi_config::open_in::Os::Linux,
        dirs: Dirs {
            applications: root.join("data/applications"),
            icon_root: root.join("data/icons/hicolor"),
            desktop: Some(root.join("Desktop")),
            ..Dirs::default()
        },
        exe: PathBuf::from("/opt/gp/gitpoulpi"),
        launch_exe: None,
        newest: false,
        has: std::sync::Arc::new(|p| matches!(p, "kitty" | "foot")),
        which: std::sync::Arc::new(|_| None),
        ctx: gitpoulpi_config::desktop::InstallCtx::legacy(),
        refresh_icon_cache: false,
        submenu: String::new(),
    }
}

/// Polls until the background installation job has been applied.
fn settle_install(st: &mut PreferencesState) {
    for _ in 0..500 {
        st.poll_install();
        if !st.install.busy() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the installation job never finished");
}

#[test]
fn the_primary_rows_follow_the_state_after_the_job() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let root = tmp.path().join("machine");
    st.set_launcher_env(move || temp_machine(&root));
    assert!(matches!(
        row(&st, "Become the primary").control,
        Control::Toggle(false)
    ));
    assert_eq!(
        row(&st, "Launcher name").value_text(),
        "GitPoulpi (portable)"
    );
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    assert!(
        matches!(
            row(&st, "Become the primary").control,
            Control::Toggle(true)
        ),
        "the checkbox shows the new state"
    );
    let name = row(&st, "Launcher name");
    assert_eq!(name.value_text(), "the primary is always called GitPoulpi");
    assert!(name.coming.is_some());
    // And back.
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    assert!(matches!(
        row(&st, "Become the primary").control,
        Control::Toggle(false)
    ));
    assert_eq!(
        row(&st, "Launcher name").value_text(),
        "GitPoulpi (portable)"
    );
}

#[test]
fn resigning_moves_the_launcher_to_its_own_name_and_leaves_one() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    st.set_this(fake_current(true));
    let root = tmp.path().join("machine");
    let r = root.clone();
    st.set_launcher_env(move || temp_machine(&r));
    // The primary's launcher, under the plain name.
    let mut env = temp_machine(&root);
    env.ctx = crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config);
    let logo = st.this_colour();
    let _ = crate::launcher::run(
        &env,
        "",
        &logo,
        crate::launcher::Op::Install(Place::Launcher),
    );
    let apps = root.join("data/applications");
    assert!(apps.join("gitpoulpi.desktop").exists());
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    let names: Vec<String> = std::fs::read_dir(&apps)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        ["gitpoulpi-iaaaa0000.desktop".to_string()],
        "only the launcher of the new role is left"
    );
}

#[test]
fn resigning_leaves_no_icon_under_the_primary_names() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    st.set_this(fake_current(true));
    let root = tmp.path().join("machine");
    let r = root.clone();
    st.set_launcher_env(move || temp_machine(&r));
    // An icon left under the primary's name with no launcher carrying it.
    let icons = root.join("data/icons/hicolor/256x256/apps");
    std::fs::create_dir_all(&icons).unwrap();
    std::fs::write(icons.join("gitpoulpi-0a1b2c3d.png"), b"x").unwrap();
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    let left: Vec<String> = std::fs::read_dir(&icons)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with("gitpoulpi-i"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn the_loop_keeps_drawing_while_an_installation_job_runs() {
    // The poll that applies a finished job runs in a frame: while a job is in
    // flight the page counts as animating, so no frame waits for a key press.
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let root = tmp.path().join("machine");
    st.set_launcher_env(move || temp_machine(&root));
    assert!(!st.launcher_jobs_running());
    st.install_read();
    assert!(st.launcher_jobs_running());
    assert!(st.animating());
    settle_install(&mut st);
    assert!(!st.launcher_jobs_running());
}

#[test]
fn the_rows_reread_the_file_when_a_job_ends_even_one_that_only_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let root = tmp.path().join("machine");
    st.set_launcher_env(move || temp_machine(&root));
    // Another process (or a handover that failed half way) wrote the primary.
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.install_read();
    settle_install(&mut st);
    assert!(matches!(
        row(&st, "Become the primary").control,
        Control::Toggle(true)
    ));
}

/// The machine of `temp_machine`, but a Windows one: a Start menu and a
/// Desktop folder under `root`. Nothing runs PowerShell here, so only what the
/// handover does by itself (renames) shows on disk.
fn windows_machine(root: &Path) -> crate::launcher::Env {
    let mut env = temp_machine(root);
    let start = root.join("start");
    std::fs::create_dir_all(&start).unwrap();
    env.os = gitpoulpi_config::open_in::Os::Windows;
    env.dirs.start_menu = Some(start);
    env.exe = PathBuf::from("/opt/gp/gitpoulpi.exe");
    env
}

/// A state whose installation is not the primary, on a Windows machine whose
/// folder is `machine`.
fn windows_state(dir: &Path) -> PreferencesState {
    let mut st = install_state(dir);
    let root = dir.join("machine");
    st.set_launcher_env(move || windows_machine(&root));
    st
}

/// A shortcut file as the shell reads it: the owner tag is all that matters.
fn lnk_of(id: &str, marker: bool) -> Vec<u8> {
    let marker = if marker { " X-GitPoulpi=1" } else { "" };
    format!("GitPoulpi{marker} X-GitPoulpi-Install={id} icon.ico").into_bytes()
}

#[test]
fn becoming_the_primary_renames_the_shortcuts_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = windows_state(tmp.path());
    let machine = tmp.path().join("machine");
    let _ = windows_machine(&machine);
    let (start, desktop) = (machine.join("start"), machine.join("Desktop"));
    let own = "GitPoulpi (portable).lnk";
    std::fs::write(start.join(own), lnk_of(ME, false)).unwrap();
    std::fs::write(desktop.join(own), lnk_of(ME, false)).unwrap();
    // Somebody else's shortcut next to ours is not looked at.
    std::fs::write(start.join("GitPoulpi (dev).lnk"), lnk_of("bbbb", false)).unwrap();
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    for dir in [&start, &desktop] {
        assert!(dir.join("GitPoulpi.lnk").exists(), "{dir:?}: plain name");
        assert!(!dir.join(own).exists(), "{dir:?}: old name is gone");
    }
    assert!(start.join("GitPoulpi (dev).lnk").exists());
    // The Launcher page no longer reads "not created".
    let mut env = windows_machine(&machine);
    env.ctx = crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config);
    let snap = crate::launcher::snapshot(&env, "", &st.this_colour());
    let start_row = snap
        .rows
        .iter()
        .find(|r| r.place == Place::StartMenu)
        .unwrap();
    assert_ne!(
        start_row.status.state,
        gitpoulpi_config::desktop::State::NotInstalled
    );
}

#[test]
fn resigning_renames_the_shortcuts_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = windows_state(tmp.path());
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    st.set_this(fake_current(true));
    let machine = tmp.path().join("machine");
    let _ = windows_machine(&machine);
    let (start, desktop) = (machine.join("start"), machine.join("Desktop"));
    std::fs::write(start.join("GitPoulpi.lnk"), lnk_of(ME, true)).unwrap();
    std::fs::write(desktop.join("GitPoulpi.lnk"), lnk_of(ME, true)).unwrap();
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    for dir in [&start, &desktop] {
        assert!(dir.join("GitPoulpi (portable).lnk").exists(), "{dir:?}");
        assert!(!dir.join("GitPoulpi.lnk").exists(), "{dir:?}");
    }
}

fn windows_menu_env(root: &Path, registry: std::sync::Arc<MemoryRegistry>) -> MenuEnv {
    let bin = root.join("bin");
    MenuEnv {
        os: gitpoulpi_config::open_in::Os::Windows,
        data_home: root.join("data"),
        config_home: root.join("config"),
        home: root.join("home"),
        exe: bin.join("gitpoulpi.exe"),
        helper: Some(bin.join("gitpoulpi-launch.exe")),
        has: std::sync::Arc::new(|_| false),
        registry,
        icon_png: None,
        refresh_hooks: false,
        ctx: gitpoulpi_config::desktop::InstallCtx::legacy(),
        ico: None,
        submenu: String::new(),
    }
}

#[test]
fn the_folder_menu_entry_survives_becoming_the_primary_and_resigning() {
    use gitpoulpi_config::desktop::{folder_menu, State};
    let tmp = tempfile::tempdir().unwrap();
    let mut st = windows_state(tmp.path());
    let reg = std::sync::Arc::new(MemoryRegistry::new());
    let root = tmp.path().join("machine");
    let (r, g) = (root.clone(), reg.clone());
    st.set_folder_menu_env(move |_| windows_menu_env(&r, g.clone()));
    let env_now = |st: &PreferencesState| {
        let mut e = windows_menu_env(&root, reg.clone());
        e.ctx = crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config);
        e
    };
    let status_now = |st: &PreferencesState| {
        let e = env_now(st);
        let label = crate::launcher::folder_menu_label(&e.ctx.name);
        folder_menu::status(&e, &label).entries[0].state
    };
    // The entry as the installation has it before: its own name.
    let e = env_now(&st);
    folder_menu::install(&e, &crate::launcher::folder_menu_label(&e.ctx.name)).unwrap();
    assert_eq!(status_now(&st), State::Installed);
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    assert_eq!(
        status_now(&st),
        State::Installed,
        "the entry of the primary is there"
    );
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    assert_eq!(
        status_now(&st),
        State::Installed,
        "and back under its own name"
    );
}

#[test]
fn the_handover_leaves_exactly_one_folder_menu_verb_per_installation() {
    use gitpoulpi_config::desktop::registry::Registry;
    let tmp = tempfile::tempdir().unwrap();
    let mut st = windows_state(tmp.path());
    let reg = std::sync::Arc::new(MemoryRegistry::new());
    let root = tmp.path().join("machine");
    let (r, g) = (root.clone(), reg.clone());
    st.set_folder_menu_env(move |_| windows_menu_env(&r, g.clone()));
    let verbs = || {
        let mut v: Vec<String> = reg
            .subkeys("Software\\Classes\\Directory\\shell")
            .unwrap()
            .into_iter()
            .filter(|n| n.starts_with("gitpoulpi"))
            .collect();
        v.dedup();
        v
    };
    let mut e = windows_menu_env(&root, reg.clone());
    e.ctx = crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config);
    gitpoulpi_config::desktop::folder_menu::install(
        &e,
        &crate::launcher::folder_menu_label(&e.ctx.name),
    )
    .unwrap();
    let own = format!("gitpoulpi.{}", e.ctx.short);
    assert_eq!(verbs(), std::slice::from_ref(&own));
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    assert_eq!(
        verbs(),
        ["gitpoulpi"],
        "the primary has the plain verb only"
    );
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    assert_eq!(verbs(), [own], "resigned: its own verb only, no plain one");
}

#[test]
fn a_script_folder_entry_is_replaced_never_deleted_by_the_handover() {
    use gitpoulpi_config::desktop::folder_menu;
    // GNOME Files finds its scripts by the owner tag, not by name: removing
    // "the old one" after writing the new one would delete the new one.
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let root = tmp.path().join("machine");
    let env_for = {
        let root = root.clone();
        move || {
            let bin = root.join("bin");
            std::fs::create_dir_all(&bin).unwrap();
            MenuEnv {
                os: gitpoulpi_config::open_in::Os::Linux,
                data_home: root.join("data"),
                config_home: root.join("config"),
                home: root.join("home"),
                exe: bin.join("gitpoulpi"),
                helper: Some(bin.join("gitpoulpi")),
                has: std::sync::Arc::new(|p| p == "nautilus"),
                registry: std::sync::Arc::new(MemoryRegistry::new()),
                icon_png: Some(vec![1, 2, 3]),
                refresh_hooks: false,
                ctx: gitpoulpi_config::desktop::InstallCtx::legacy(),
                ico: None,
                submenu: String::new(),
            }
        }
    };
    let r = root.clone();
    st.set_launcher_env(move || temp_machine(&r));
    let e = env_for.clone();
    st.set_folder_menu_env(move |_| e());
    let mut e = env_for();
    e.ctx = crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config);
    folder_menu::install(&e, &crate::launcher::folder_menu_label(&e.ctx.name)).unwrap();
    let scripts = root.join("data/nautilus/scripts");
    let names = || -> Vec<String> {
        std::fs::read_dir(&scripts)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect()
    };
    assert_eq!(names().len(), 1);
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    assert_eq!(names(), ["Open in GitPoulpi".to_string()]);
}

/// An alias machine under `root`, as `ctx` writes (never the real home).
fn alias_machine(root: &Path) -> crate::preferences::launcher_alias::Machine {
    use gitpoulpi_config::desktop::path as user_path;
    struct Nothing;
    impl user_path::UserPath for Nothing {
        fn read(&self) -> Result<user_path::UserPathValues, String> {
            Ok(user_path::UserPathValues::default())
        }
        fn write(&self, _: &str, _: Option<&str>) -> Result<(), String> {
            Ok(())
        }
    }
    let alias = gitpoulpi_config::desktop::alias::Env {
        os: gitpoulpi_config::open_in::Os::Linux,
        bin_dir: root.join("bin"),
        exe: PathBuf::from("/opt/gp/gitpoulpi"),
        newest: false,
        path_var: Some(std::ffi::OsString::new()),
        pathext: None,
        ctx: gitpoulpi_config::desktop::InstallCtx::legacy(),
        cargo_home: None,
    };
    let path = user_path::Env {
        os: gitpoulpi_config::open_in::Os::Linux,
        home: root.join("home"),
        config_home: root.join("home/.config"),
        wanted: Vec::new(),
        path_var: Some(std::ffi::OsString::new()),
        keep: Vec::new(),
    };
    crate::preferences::launcher_alias::Machine {
        alias,
        path,
        user: std::sync::Arc::new(Nothing),
        registry: std::sync::Arc::new(MemoryRegistry::new()),
    }
}

#[test]
fn a_role_change_rewrites_this_installations_own_aliases_and_nobody_elses() {
    use gitpoulpi_config::desktop::alias::{self, Kind, State};
    const OTHER: &str = "bbbb000000000002";
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let other_dir = tmp.path().join("other");
    std::fs::create_dir_all(&other_dir).unwrap();
    st.config.installation.settings.insert(
        OTHER.to_string(),
        other_entry("GitPoulpi (dev)", &other_dir.display().to_string(), "", ""),
    );
    std::fs::write(
        tmp.path().join("config.toml"),
        format!(
            "[installation.settings.{OTHER}]\npath = {:?}\nkind = \"dev\"\nname = \"GitPoulpi (dev)\"\n",
            other_dir.display().to_string()
        ),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    let root = tmp.path().join("machine");
    let r = root.clone();
    st.set_launcher_env(move || temp_machine(&r));
    let r = root.clone();
    st.set_tools_machine(move || alias_machine(&r));
    // The aliases as the running installation made them before the change
    // (no legacy marker), and one of the other installation's.
    let ctx_of = |st: &PreferencesState| {
        crate::launcher::ctx_in(&st.config, &st.this().identity, &st.this().config)
    };
    let mine = alias_machine(&root).alias.with_ctx(ctx_of(&st));
    alias::create_kind(&mine, "gpa", Kind::Add).unwrap();
    let mut theirs_ctx = ctx_of(&st);
    theirs_ctx.id = OTHER.to_string();
    theirs_ctx.short = "ibbbb0000".to_string();
    let theirs = alias_machine(&root).alias.with_ctx(theirs_ctx);
    alias::create_kind(&theirs, "gpd", Kind::Add).unwrap();
    let their_text = std::fs::read_to_string(root.join("bin/gpd")).unwrap();
    assert!(!std::fs::read_to_string(root.join("bin/gpa"))
        .unwrap()
        .contains("X-GitPoulpi=1"));

    // Becoming the primary: the alias takes the role's text.
    assert!(st.write_global("installation.primary", "true"));
    settle_install(&mut st);
    let now = alias_machine(&root).alias.with_ctx(ctx_of(&st));
    assert!(now.ctx.is_primary());
    assert_eq!(
        alias::status_kind(&now, "gpa", Kind::Add).state,
        State::Installed,
        "{}",
        std::fs::read_to_string(root.join("bin/gpa")).unwrap()
    );
    assert!(std::fs::read_to_string(root.join("bin/gpa"))
        .unwrap()
        .contains("X-GitPoulpi=1"));
    assert_eq!(
        std::fs::read_to_string(root.join("bin/gpd")).unwrap(),
        their_text,
        "another installation's alias is left as it is"
    );

    // And back: the legacy marker goes again.
    assert!(st.write_global("installation.primary", "false"));
    settle_install(&mut st);
    let now = alias_machine(&root).alias.with_ctx(ctx_of(&st));
    assert!(!now.ctx.is_primary());
    assert_eq!(
        alias::status_kind(&now, "gpa", Kind::Add).state,
        State::Installed
    );
    assert_eq!(
        std::fs::read_to_string(root.join("bin/gpd")).unwrap(),
        their_text
    );
}

#[test]
fn changing_the_launcher_name_or_the_version_renames_the_shortcuts_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = windows_state(tmp.path());
    let machine = tmp.path().join("machine");
    let _ = windows_machine(&machine);
    let (start, desktop) = (machine.join("start"), machine.join("Desktop"));
    let own = "GitPoulpi (portable).lnk";
    std::fs::write(start.join(own), lnk_of(ME, false)).unwrap();
    std::fs::write(desktop.join(own), lnk_of(ME, false)).unwrap();
    // Somebody else's shortcut next to ours is not looked at.
    std::fs::write(start.join("GitPoulpi (dev).lnk"), lnk_of("bbbb", false)).unwrap();

    assert!(st.write_global("installation.name", "Poulpi Test"));
    settle_install(&mut st);
    for dir in [&start, &desktop] {
        assert!(
            dir.join("Poulpi Test.lnk").exists(),
            "{dir:?}: the new name"
        );
        assert!(!dir.join(own).exists(), "{dir:?}: the old name is gone");
    }
    assert!(start.join("GitPoulpi (dev).lnk").exists());

    // The version in the name changes the effective name: same treatment.
    assert!(st.write_global("installation.name_with_version", "true"));
    settle_install(&mut st);
    let version = {
        let mut parts = env!("CARGO_PKG_VERSION").split('.');
        format!("{}.{}", parts.next().unwrap(), parts.next().unwrap())
    };
    let versioned = format!("Poulpi Test {version}.lnk");
    for dir in [&start, &desktop] {
        assert!(dir.join(&versioned).exists(), "{dir:?}: {versioned}");
        assert!(!dir.join("Poulpi Test.lnk").exists(), "{dir:?}");
    }
    assert!(start.join("GitPoulpi (dev).lnk").exists());
}
