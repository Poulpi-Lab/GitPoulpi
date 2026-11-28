//! `gitpoulpi-desktop`: the desktop frontend. See `desktop/README.md`.

use gitpoulpi_desktop::{app, cli, daemon::Backend, prefs::PrefsStore};

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let args = match cli::parse(std::env::args().skip(1), cwd) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("gitpoulpi-desktop: {e}");
            std::process::exit(2);
        }
    };
    let backend = match Backend::start(&args.repo) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("gitpoulpi-desktop: cannot start: {e}");
            std::process::exit(1);
        }
    };
    let overrides = args.overrides;
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            if let Err(e) = app::init(cx, PrefsStore::default_dir(), &overrides) {
                eprintln!("gitpoulpi-desktop: cannot initialise: {e}");
                cx.quit();
                return;
            }
            let root = backend.repo_root().to_string_lossy().into_owned();
            PrefsStore::update(cx, |p| p.remember_recent(&root));
            if let Err(e) = app::open_main_window(backend, cx) {
                eprintln!("gitpoulpi-desktop: cannot open the window: {e}");
                cx.quit();
            }
        });
}
