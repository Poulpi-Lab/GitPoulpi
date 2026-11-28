//! The commit graph's vocabulary: [`TreePart`], [`Sides`], [`NodeKind`], and
//! the neutral colour reference a cell carries.
//!
//! Every cell of the Graph column is a [`TreeCell`]: one [`TreePart`] plus
//! the lane colour it carries. Layout produces parts; each frontend decides
//! how a part is drawn (the terminal UI turns it into a glyph or a bitmap,
//! a desktop UI into vector strokes).
//!
//! Why the indirection: **"does this piece join the piece next to it?" is
//! a property you can enumerate**. Each part declares the [`Sides`] it
//! connects to, and the connectivity tests walk a whole grid asserting the
//! declaration is mutual - if a cell reaches right, its right-hand neighbour
//! must reach left; if it reaches down, the cell below it must reach up.
//!
//! The glyph choice for a terminal, and the evidence behind it, lives with
//! the terminal renderer (`gitpoulpi-tui`'s `graph::part`), not here.

/// Which of its four edges a part draws a stroke out to.
///
/// This is *topology*, not ink: a [`NodeKind::Commit`] circle declares `up`
/// and `down` because the lane runs through it, even though the circle's
/// own outline stops short of the cell edge. What the declaration buys is
/// the mutual-connection invariant - whatever is in the cell above must
/// itself reach down, so the stroke arrives at the node's doorstep rather
/// than stopping a cell short, which is the defect that kept recurring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Sides {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Sides {
    pub const NONE: Sides = Sides {
        up: false,
        down: false,
        left: false,
        right: false,
    };
    pub const VERTICAL: Sides = Sides {
        up: true,
        down: true,
        left: false,
        right: false,
    };
    pub const HORIZONTAL: Sides = Sides {
        up: false,
        down: false,
        left: true,
        right: true,
    };
    pub const ALL: Sides = Sides {
        up: true,
        down: true,
        left: true,
        right: true,
    };

    pub const fn new(up: bool, down: bool, left: bool, right: bool) -> Sides {
        Sides {
            up,
            down,
            left,
            right,
        }
    }
}

/// The commit markers, and nothing else. Adding one means adding a glyph to
/// every terminal node set, which is the point: the sets stay in step
/// because the match is exhaustive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// An ordinary commit: hollow circle.
    Commit,
    /// A commit a branch, tag or HEAD points at: filled circle.
    Ref,
    /// A merge commit: double circle.
    Merge,
    /// The working tree's uncommitted changes - the WIP row. Not a
    /// commit, so it gets the one outline that is not solid.
    Wip,
    /// A stash, on a lane of its own beside the commit it was made on: a
    /// little box. Solid - a stash is recorded, not pending; only its
    /// connector down to the base is dashed.
    Stash,
}

/// One distinct visual piece a Graph cell can hold.
///
/// Exhaustive on purpose. If a layout decision cannot be expressed as one
/// of these, the right move is to add a variant here - with its [`Sides`] -
/// not to special-case it in a renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreePart {
    /// Nothing here.
    Empty,
    /// A lane running straight down through a row that is not its own
    /// commit.
    Vertical,
    /// A horizontal run crossing a column that holds no live lane.
    Horizontal,
    /// A horizontal run crossing a column that *does* hold a live lane.
    /// Both strokes are drawn whole. Deliberately a piece no node kind
    /// uses - a crossing that looked like a node would claim a commit sits
    /// on the crossed lane.
    Crossing,
    /// A commit. `sides` is the topology this particular commit has: which
    /// way its lane continues, and whether it has a transition running out
    /// of it to the left or the right.
    Node { kind: NodeKind, sides: Sides },
    /// A lane branching off *to the left* of the node it leaves. The run
    /// arrives from the right, the new lane continues down.
    BranchOutLeft,
    /// A lane branching off to the right.
    BranchOutRight,
    /// A lane converging *from the left* into the node it joins. The lane
    /// arrived from above, the run leaves to the right.
    MergeInLeft,
    /// A lane converging from the right.
    MergeInRight,
    /// [`TreePart::BranchOutLeft`]/[`TreePart::BranchOutRight`] with a
    /// *third* arm, because another run on the same row passes straight
    /// over this column. An elbow inks only the half of its cell its own
    /// two arms reach, so a run crossing it would come out cut in two.
    BranchOutCrossed,
    /// The mirror: a converging elbow another run passes over.
    MergeInCrossed,
}

impl TreePart {
    /// Every variant, with representative `sides` for the node cases. The
    /// width / code-point / connectivity guards iterate this, so a new
    /// variant that is not listed here fails `every_tree_part_is_listed`.
    pub const ALL: [TreePart; 16] = [
        TreePart::Empty,
        TreePart::Vertical,
        TreePart::Horizontal,
        TreePart::Crossing,
        TreePart::Node {
            kind: NodeKind::Commit,
            sides: Sides::VERTICAL,
        },
        TreePart::Node {
            kind: NodeKind::Ref,
            sides: Sides::VERTICAL,
        },
        TreePart::Node {
            kind: NodeKind::Merge,
            sides: Sides::ALL,
        },
        TreePart::Node {
            kind: NodeKind::Commit,
            sides: Sides::NONE,
        },
        TreePart::Node {
            kind: NodeKind::Wip,
            sides: Sides::new(false, true, false, false),
        },
        TreePart::Node {
            kind: NodeKind::Stash,
            sides: Sides::new(false, true, false, false),
        },
        TreePart::BranchOutLeft,
        TreePart::BranchOutRight,
        TreePart::MergeInLeft,
        TreePart::MergeInRight,
        TreePart::BranchOutCrossed,
        TreePart::MergeInCrossed,
    ];

    /// The edges this part draws a stroke out to. The whole connectivity
    /// invariant rests on this function, so it is a plain exhaustive match
    /// with no defaulting.
    pub const fn sides(self) -> Sides {
        match self {
            TreePart::Empty => Sides::NONE,
            TreePart::Vertical => Sides::VERTICAL,
            TreePart::Horizontal => Sides::HORIZONTAL,
            TreePart::Crossing => Sides::ALL,
            TreePart::Node { sides, .. } => sides,
            // down + right: the new lane continues down, the run reaches
            // back across to the node on its right.
            TreePart::BranchOutLeft => Sides::new(false, true, false, true),
            TreePart::BranchOutRight => Sides::new(false, true, true, false),
            TreePart::MergeInLeft => Sides::new(true, false, false, true),
            TreePart::MergeInRight => Sides::new(true, false, true, false),
            TreePart::BranchOutCrossed => Sides::new(false, true, true, true),
            TreePart::MergeInCrossed => Sides::new(true, false, true, true),
        }
    }
}

/// A lane colour, without naming a real one. The layout only decides *which*
/// lane colour a cell takes; the frontend owns the palette and the theme.
///
/// `Lane(n)` is slot `n` of the lane palette (`n` is already reduced modulo
/// the palette's length by the layout); `Wip` and `Stash` stand for the
/// theme's fixed graph colours of those two markers (a stash is not a
/// branch, so it takes no palette slot); `None` is the colour of a blank
/// cell, which carries no lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorRef {
    None,
    Lane(usize),
    Wip,
    Stash,
}

/// A part plus the lane colour it carries. This is what a graph row is
/// made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCell {
    pub part: TreePart,
    pub color: ColorRef,
    /// Bold is used for two things and only two: a transition (a corner or
    /// its horizontal run), so a branch leaving or joining is legible at
    /// 7 px, and a node whose commit carries a ref or is a merge.
    pub bold: bool,
    /// Which of this cell's strokes are **pending** rather than history,
    /// drawn dashed instead of solid. Two connectors set it: the WIP row's
    /// down to HEAD (the WIP node's `down`, every lane cell it passes
    /// through, HEAD's own `up`), and each stash's down to its base (the
    /// stash node's `down`, its lane cells, the elbow and the run into the
    /// base, and the base's arm on that side). Always a subset
    /// of [`TreePart::sides`] - it changes the ink, never the topology,
    /// so the connectivity invariant does not see it.
    pub dashed: Sides,
}

impl TreeCell {
    pub fn new(part: TreePart, color: ColorRef) -> TreeCell {
        TreeCell {
            part,
            color,
            bold: false,
            dashed: Sides::NONE,
        }
    }

    pub fn bold(mut self) -> TreeCell {
        self.bold = true;
        self
    }

    /// The same cell with `sides` drawn dashed. See [`TreeCell::dashed`].
    pub fn with_dashed(mut self, sides: Sides) -> TreeCell {
        self.dashed = sides;
        self
    }

    pub const fn empty() -> TreeCell {
        TreeCell {
            part: TreePart::Empty,
            color: ColorRef::None,
            bold: false,
            dashed: Sides::NONE,
        }
    }

    /// Whether this cell is pending work *as a whole* - the WIP marker,
    /// or a cell that only carries a dashed connector (a lane, a run, a
    /// stash connector's elbow) - and so is drawn dimmed. HEAD's node has
    /// a dashed `up` too, but it is a real commit and keeps its full
    /// colour; a stash's box is recorded work and is not dimmed either.
    pub fn is_pending(self) -> bool {
        match self.part {
            TreePart::Node { kind, .. } => matches!(kind, NodeKind::Wip),
            TreePart::Vertical => self.dashed.up || self.dashed.down,
            TreePart::Horizontal => self.dashed.left || self.dashed.right,
            TreePart::BranchOutLeft
            | TreePart::BranchOutRight
            | TreePart::MergeInLeft
            | TreePart::MergeInRight => self.dashed.up || self.dashed.down,
            _ => false,
        }
    }
}
