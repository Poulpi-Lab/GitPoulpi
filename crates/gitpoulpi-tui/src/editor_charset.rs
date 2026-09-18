//! The editor's text-format vocabulary (encodings, line breaks, indentation
//! conversion). The pure functions live in `gitpoulpi_core::textconv`, so the
//! daemon can convert files on disk with exactly the code the editor uses.

pub(crate) use gitpoulpi_core::textconv::*;
