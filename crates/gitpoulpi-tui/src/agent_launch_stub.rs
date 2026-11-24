//! `agent_launch` for a build without the `ai` feature: there is no agent to
//! open, so the palette command is not offered and a sequence step fails with
//! the usual "not built" text.
#![allow(dead_code)]

use std::path::Path;

fn not_built() -> String {
    gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string()
}

pub(crate) enum PaletteStep {
    Done(Result<String, String>),
    Choose(Vec<(String, String)>),
}

pub(crate) fn open_here(_id: &str, _path: &Path) -> Result<String, String> {
    Err(not_built())
}

pub(crate) fn palette_step(_path: &Path) -> PaletteStep {
    PaletteStep::Done(Err(not_built()))
}
