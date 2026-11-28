//! The diff's frontend-neutral model: logical rows, line numbers, Split
//! pairing, changed-word ranges and neutral styled lines. A frontend does
//! the layout (wrap, scroll, gutter widths in its own unit) and resolves
//! [`DiffRole`]s against its own theme.

pub mod intraline;
pub mod model;
pub mod rows;
pub mod styled;

pub use model::{hunk_at, selected_lines, AnnotSide, AnnotTag, DiffRow, DiffText, RowChange};
pub use rows::{
    effective_hunks, line_numbers, split_change, split_pairs, EffectiveHunk, SplitSide,
};
pub use styled::{DiffRole, Rgb, StyledLine, StyledSpan, SyntaxStyle};

#[cfg(test)]
mod tests;
