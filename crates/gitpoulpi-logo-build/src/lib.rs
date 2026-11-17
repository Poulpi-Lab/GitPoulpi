//! Build-time logo support of GitPoulpi: the colorway table, the selection a
//! custom build makes (`GITPOULPI_LOGO_*`), and the build-script body of the
//! binary crates (the table of the About window, the Windows executable
//! icon). A build-dependency, not meant to be used at run time.

pub mod logo_build;
pub mod script;
