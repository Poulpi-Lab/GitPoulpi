//! The commit column: its states, parts and commands.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// What the one column left of a file's view shows: nothing (a strip brings
/// it back), the commits of that one file (the File History's list), or the
/// Log's commits with its graph. The bar's `File | All` selector, the cross,
/// the strip, `Alt+C` and the palette rows all move between these three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnState {
    Off,
    File,
    Graph,
}

/// A part of a Graph row: the refs, the lanes, the title (the message), the
/// author, the date, the changes and the short hash: the Log's seven
/// columns. Each has a width of its own the user can drag; the title takes
/// what the others leave until it has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    Refs,
    Graph,
    Title,
    Author,
    Date,
    Changes,
    Sha,
}

/// The way a part moves in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

/// The palette's commands on the column. Each is the keyboard twin of a
/// mouse action: the cross and the strip (Toggle), a click on a segment of the
/// bar's selector (Set; on the selected one, off), a click on a row
/// (Focus, then the arrows), a drag of the splitter (Widen / Narrow), the
/// wheel and a drag of a header boundary (the scroll and part commands).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnCmd {
    Toggle,
    Set(ColumnState),
    Focus,
    Widen,
    Narrow,
    ScrollLeft,
    ScrollRight,
    /// The next part the keyboard sizes (among the shown ones).
    NextPart,
    /// Moves the part the keyboard sizes one place left or right among the
    /// shown ones (the twin of dragging its header).
    MovePart(Side),
    PartWider,
    PartNarrower,
    /// Every part back to its automatic width.
    ResetWidths,
    /// Fits the part the keyboard sizes to its content (the twin of
    /// double-clicking its boundary).
    Fit,
    /// Fits every shown part to its content.
    FitAll,
    /// Shows or hides one part of the rows (the header menu's twin).
    ToggleShown(Part),
    /// The header's right-click menu: which parts are shown.
    ColumnsMenu,
    /// The context menu of the selected commit.
    Menu,
    /// One of the menu's rows on the selected commit.
    Show(super::log_menu::ColumnOp),
}

impl ColumnState {
    pub fn label(self) -> &'static str {
        match self {
            ColumnState::Off => gitpoulpi_i18n::msg::commit_column::STATE_OFF.get(),
            ColumnState::File => gitpoulpi_i18n::msg::commit_column::STATE_FILE.get(),
            ColumnState::Graph => gitpoulpi_i18n::msg::commit_column::STATE_GRAPH.get(),
        }
    }
}
