//! Windows executable icon, chosen at build time
//! (`GITPOULPI_LOGO_COLORS`, `GITPOULPI_LOGO_DEFAULT`, `GITPOULPI_ICON_COLOR`;
//! see `gitpoulpi-logo-build`). Does nothing for non-Windows targets.

fn main() {
    gitpoulpi_logo_build::script::run(false, false);
}
