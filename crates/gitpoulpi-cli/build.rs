//! Windows executable icon, chosen at build time
//! (`GITPOULPI_LOGO_COLORS`, `GITPOULPI_LOGO_DEFAULT`, `GITPOULPI_ICON_COLOR`;
//! see `assets/logo/logo_build.rs`). Does nothing for non-Windows targets.

#[path = "../../assets/logo/logo_build.rs"]
mod logo_build;
#[path = "../../assets/logo/logo_build_script.rs"]
mod logo_build_script;

fn main() {
    logo_build_script::run(false, true);
}
