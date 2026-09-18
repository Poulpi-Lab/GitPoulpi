//! Windows executable icon and the About window's colorway table, chosen at build time
//! (`GITPOULPI_LOGO_COLORS`, `GITPOULPI_LOGO_DEFAULT`, `GITPOULPI_ICON_COLOR`;
//! see `assets/logo/logo_build.rs`; the icon only for Windows targets), and the
//! About window's third-party crate list (`build_crates.rs`).

mod build_crates;
mod build_version;
#[path = "../../assets/logo/logo_build.rs"]
mod logo_build;
#[path = "../../assets/logo/logo_build_script.rs"]
mod logo_build_script;

fn main() {
    logo_build_script::run(true, false);
    build_crates::run();
    build_version::run();
}
