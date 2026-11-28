//! Text editing shared by every frontend: the caret and selection of a
//! field ([`Caret`], driven by [`Op`]s the frontend decodes from its own key
//! events), the edit history behind undo and redo, and the document model of
//! the built-in editor ([`doc::Doc`]).
//!
//! A frontend with native text widgets only needs [`doc::Doc`] (the merge
//! editor's Output and the file editor use it as the text store); the caret
//! and history are what `Doc` and the terminal UI's fields are built on.

pub mod caret;
pub mod doc;
pub mod history;

pub use caret::{
    char_class, column_of, floor_boundary, line_end, line_start, next_boundary, offset_at_column,
    prev_boundary, word_left, word_right, Caret, KeyEdit, Motion, Op,
};
