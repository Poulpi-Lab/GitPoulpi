//! Windows executable icon and the About window's colorway table, chosen at build time
//! (`GITPOULPI_LOGO_COLORS`, `GITPOULPI_LOGO_DEFAULT`, `GITPOULPI_ICON_COLOR`;
//! see `gitpoulpi-logo-build`; the icon only for Windows targets), and the
//! About window's third-party crate list (`build_crates.rs`).

mod build_crates;
mod build_version;
fn main() {
    gitpoulpi_logo_build::script::run(true, false);
    build_crates::run();
    build_version::run();
}
