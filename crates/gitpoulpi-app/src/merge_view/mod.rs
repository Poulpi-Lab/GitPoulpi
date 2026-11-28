//! The three-way merge editor's logic, shared by every frontend: the
//! [`model`] holds the picks per conflict, the Output document with an
//! anchor per conflict region, one undo stack and the checks before a
//! save. What is drawn, which keys do what and the IPC stay in the
//! frontend; the palette vocabulary (`MergeCmd`, `MergePane`) is
//! `palette::vocab::merge_view`.

pub mod model;
