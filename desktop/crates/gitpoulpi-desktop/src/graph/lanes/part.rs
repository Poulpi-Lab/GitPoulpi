//! Cell vocabulary of the lane layout (the terminal UI's `part.rs`).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sides {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Sides {
    pub const NONE: Sides = Sides::new(false, false, false, false);
    pub const VERTICAL: Sides = Sides::new(true, true, false, false);
    pub const HORIZONTAL: Sides = Sides::new(false, false, true, true);
    pub const fn new(up: bool, down: bool, left: bool, right: bool) -> Sides {
        Sides {
            up,
            down,
            left,
            right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Commit,
    Ref,
    Merge,
    Wip,
    Stash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreePart {
    Empty,
    Vertical,
    Horizontal,
    Crossing,
    Node { kind: NodeKind, sides: Sides },
    BranchOutLeft,
    BranchOutRight,
    MergeInLeft,
    MergeInRight,
    BranchOutCrossed,
    MergeInCrossed,
}

impl TreePart {
    pub const fn sides(self) -> Sides {
        match self {
            TreePart::Empty => Sides::NONE,
            TreePart::Vertical => Sides::VERTICAL,
            TreePart::Horizontal => Sides::HORIZONTAL,
            TreePart::Crossing => Sides::new(true, true, true, true),
            TreePart::Node { sides, .. } => sides,
            TreePart::BranchOutLeft => Sides::new(false, true, false, true),
            TreePart::BranchOutRight => Sides::new(false, true, true, false),
            TreePart::MergeInLeft => Sides::new(true, false, false, true),
            TreePart::MergeInRight => Sides::new(true, false, true, false),
            TreePart::BranchOutCrossed => Sides::new(false, true, true, true),
            TreePart::MergeInCrossed => Sides::new(true, false, true, true),
        }
    }
}

/// Neutral colour: a palette slot (taken modulo the palette length at paint time) or one of the two
/// fixed graph colours of the theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorRef {
    Lane(usize),
    Wip,
    Stash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCell {
    pub part: TreePart,
    pub color: ColorRef,
    pub dashed: Sides,
}

impl TreeCell {
    pub fn new(part: TreePart, color: ColorRef) -> TreeCell {
        TreeCell {
            part,
            color,
            dashed: Sides::NONE,
        }
    }
    pub fn empty() -> TreeCell {
        TreeCell::new(TreePart::Empty, ColorRef::Lane(0))
    }
    pub fn with_dashed(mut self, dashed: Sides) -> TreeCell {
        self.dashed = dashed;
        self
    }
}
