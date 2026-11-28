//! The history-rewrite workbench's rule forms, frontend-neutral.
//!
//! [`form`] lists the fields of one rule kind and applies an edit to the rule
//! (a draft that does not parse is kept beside its reason and the rule is left
//! as it was), together with the text formats of durations, offsets, sizes and
//! working hours and the blank rules the Add menu starts from. A frontend draws
//! the fields its own way; what a field means and what an edit does is decided
//! here.

pub mod form;
