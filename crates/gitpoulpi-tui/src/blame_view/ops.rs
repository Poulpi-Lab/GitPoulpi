//! What the Blame view can do: one enum shared by the keys, the detail
//! card and menu, the palette and the mouse, so the four can't drift apart.

#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::palette::vocab::blame::BlameOp;

/// What a hit rect of the Blame view means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlameHit {
    /// The annotation cell of file line `n` (0-based).
    Annot(usize),
    /// The annotation column's border: drag to resize.
    Handle,
    /// The card or menu's body (swallows the pointer).
    Card,
    /// The card's close cross: like Esc, the card is dismissed.
    CardClose,
    /// A row of the card or menu.
    Op(BlameOp),
}
