//! One Settings row, as data: what it is called, what kind of value it holds,
//! the value now, and the notes around it. A frontend draws these with its own
//! widgets; nothing here knows a terminal or a window.

use gitpoulpi_config::Settings;
use gitpoulpi_i18n::msg::settings_screen as screen;

/// A colour of a swatch strip, as a reference the frontend resolves.
///
/// The ANSI palette is kept apart from `Rgb` so a terminal can draw the
/// colour its own palette defines, while a window picks its own shade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swatch {
    /// The terminal's own default colour.
    Reset,
    /// One of the 16 ANSI colours, 0 = black ... 7 = gray, 8 = dark gray,
    /// 9..=15 = the light variants, 15 = white (the order of the names a
    /// terminal library lists them in).
    Ansi(u8),
    /// A 256-colour palette index.
    Indexed(u8),
    /// A true colour.
    Rgb(u8, u8, u8),
}

/// What a button of a pool row asks for. Only the buttons the shared rows
/// carry are named here; a frontend maps them to its own action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// "Forget all usernames and passwords".
    ForgetCredentials,
    /// "Generate a key pair".
    GenerateSshKey,
    /// A button of a row the frontend builds itself (a launcher shortcut, the
    /// access token...). The text names it, for the descriptor only.
    Host(&'static str),
}

/// One unit of a [`Control::Quantity`] row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantityUnit {
    /// The unit's name for the current number ("week" or "weeks").
    pub label: String,
    /// What the unit key stores for it.
    pub key: &'static str,
    /// How many base units (days, minutes, MiB...) one of it is.
    pub weight: i64,
}

/// One inline button of a [`Control::Buttons`] row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Btn<A = Action> {
    pub label: String,
    pub action: A,
    pub enabled: bool,
    /// A text value: takes all the room the row's other buttons leave.
    pub fill: bool,
}

impl<A> Btn<A> {
    pub fn new(label: &str, action: A) -> Self {
        Self {
            label: label.to_string(),
            action,
            enabled: true,
            fill: false,
        }
    }

    /// This button is a long-text value: it fills the row.
    pub fn filling(mut self) -> Self {
        self.fill = true;
        self
    }

    /// The same button with its action translated.
    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Btn<B> {
        Btn {
            label: self.label,
            action: f(self.action),
            enabled: self.enabled,
            fill: self.fill,
        }
    }
}

/// What a row holds and how it is edited. `A` is the type of a button's action
/// (the shared rows use [`Action`]; a frontend picks its own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control<A = Action> {
    Toggle(bool),
    Number {
        value: i64,
        min: i64,
        max: i64,
    },
    Text(String),
    /// A number and a unit on one line, `[ 2 ] [weeks v]`: a whole number and a
    /// unit picked from a list. The row's key stores `value x unit.weight` (a
    /// base unit such as days); `unit_key` stores the unit's own key.
    Quantity {
        value: i64,
        /// Index into `units` of the unit shown.
        unit: usize,
        units: Vec<QuantityUnit>,
        /// Largest stored total (in base units); the number may not exceed
        /// `max_total / weight` of the chosen unit.
        max_total: i64,
        /// Whether `0` is a valid number (a timer that can be turned off),
        /// else the number is at least 1.
        allow_zero: bool,
        /// The config key the unit's `key` is written to.
        unit_key: &'static str,
    },
    /// One of `options` (their stored words), `index` is the one in force.
    Choice {
        options: Vec<String>,
        index: usize,
    },
    /// A list of known values (label, value) that is also free text when
    /// `custom`: the value is a string, not an index.
    Combo {
        options: Vec<(String, String)>,
        value: String,
        custom: bool,
    },
    /// One or more inline buttons in the value column.
    Buttons(Vec<Btn<A>>),
    /// A sub-heading inside a page. Not selectable.
    Heading,
    /// Read-only text on the right (a live preview, a derived value).
    Info(String),
    /// Read-only colour chips on the right (the theme swatch strip).
    Swatches(Vec<Swatch>),
    /// A row of quick-insert buttons (the AI page's chips).
    Chips(Vec<String>),
    /// A multi-line text, drawn as a box under its label.
    Area(String),
}

impl<A> Control<A> {
    /// The same control with its buttons' actions translated.
    pub fn map_action<B>(self, mut f: impl FnMut(A) -> B) -> Control<B> {
        match self {
            Control::Toggle(v) => Control::Toggle(v),
            Control::Number { value, min, max } => Control::Number { value, min, max },
            Control::Text(v) => Control::Text(v),
            Control::Quantity {
                value,
                unit,
                units,
                max_total,
                allow_zero,
                unit_key,
            } => Control::Quantity {
                value,
                unit,
                units,
                max_total,
                allow_zero,
                unit_key,
            },
            Control::Choice { options, index } => Control::Choice { options, index },
            Control::Combo {
                options,
                value,
                custom,
            } => Control::Combo {
                options,
                value,
                custom,
            },
            Control::Buttons(buttons) => {
                Control::Buttons(buttons.into_iter().map(|b| b.map_action(&mut f)).collect())
            }
            Control::Heading => Control::Heading,
            Control::Info(t) => Control::Info(t),
            Control::Swatches(s) => Control::Swatches(s),
            Control::Chips(c) => Control::Chips(c),
            Control::Area(t) => Control::Area(t),
        }
    }

    /// A read-only line (a heading, an info text, a swatch strip).
    pub fn is_decoration(&self) -> bool {
        matches!(
            self,
            Control::Heading | Control::Info(_) | Control::Swatches(_)
        )
    }
}

/// Where a shared row writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A dotted key of the configuration (`ui.minimap`).
    Global(&'static str),
    /// A button press, no value.
    Action(Action),
    /// Decoration: nothing to commit.
    None,
}

/// One rendered setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub label: String,
    pub control: Control,
    pub help: Option<String>,
    pub warning: Option<String>,
    /// The warning is drawn in the danger colour (a setting that exposes
    /// something when it is left on or off the wrong way).
    pub danger: bool,
    pub target: Target,
    /// `false` renders the "not wired yet" note. Derived from the config's
    /// implemented keys for global keys; stated explicitly otherwise.
    pub live: bool,
    /// `Some(reason)` makes the row **not editable at all**, and the reason is
    /// shown in place of a value.
    pub coming: Option<String>,
    /// What an empty `Area` shows grayed instead of `(empty)`.
    pub placeholder: Option<String>,
    /// A read-only value's piece that gives way first when the row is narrow:
    /// cut in the middle with an ellipsis, never dropped (the install path).
    pub ellipsize: Option<String>,
}

impl Row {
    /// A row writing the config key `key`.
    pub fn global(label: &str, key: &'static str, control: Control) -> Self {
        Self {
            label: label.to_string(),
            control,
            help: None,
            warning: None,
            danger: false,
            target: Target::Global(key),
            live: Settings::key_is_implemented(key),
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    pub fn plain(label: &str, control: Control, target: Target) -> Self {
        Self {
            label: label.to_string(),
            control,
            help: None,
            warning: None,
            danger: false,
            target,
            live: true,
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    /// A row that is one button, labelled by its own text.
    pub fn button(label: &str, action: Action) -> Self {
        Self::plain(
            label,
            Control::Buttons(vec![Btn::new(label, action)]),
            Target::Action(action),
        )
    }

    pub fn heading(label: &str) -> Self {
        Self::plain(label, Control::Heading, Target::None)
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    /// Marks `piece` of the value as the part that is cut in the middle when
    /// the row is too narrow for the whole text.
    pub fn with_ellipsis_of(mut self, piece: &str) -> Self {
        self.ellipsize = Some(piece.to_string());
        self
    }

    pub fn with_placeholder(mut self, text: &str) -> Self {
        self.placeholder = Some(text.to_string());
        self
    }

    pub fn with_warning(mut self, warning: &str) -> Self {
        self.warning = Some(warning.to_string());
        self
    }

    /// A warning drawn in the danger colour.
    pub fn with_danger(mut self, warning: &str) -> Self {
        self.warning = Some(warning.to_string());
        self.danger = true;
        self
    }

    /// Marks the row as read by a component of the program.
    pub fn live(mut self) -> Self {
        self.live = true;
        self
    }

    /// Marks the row **coming**: not editable, with `reason` shown in place
    /// of the value.
    pub fn coming(mut self, reason: &str) -> Self {
        self.coming = Some(reason.to_string());
        self
    }

    /// The config key the row writes, when it writes one.
    pub fn key(&self) -> Option<&'static str> {
        match self.target {
            Target::Global(key) => Some(key),
            _ => None,
        }
    }

    pub fn selectable(&self) -> bool {
        self.coming.is_none()
            && !matches!(
                self.control,
                Control::Heading | Control::Info(_) | Control::Swatches(_)
            )
    }

    /// The text shown for the raw choice `raw` of this row: the translated
    /// word for a schema value (`ask`), the raw text otherwise. The stored
    /// value stays `raw`.
    pub fn option_label(&self, raw: &str) -> String {
        match self.target {
            Target::Global(key) => super::choice_labels::label(key, raw),
            _ => raw.to_string(),
        }
    }

    /// The right-hand side as a one-line text. `first_line` shortens an
    /// `Area`'s text to what fits one line.
    pub fn value_text(&self, first_line: impl Fn(&str) -> String) -> String {
        value_text(
            self.coming.as_deref(),
            &self.control,
            |raw| self.option_label(raw),
            first_line,
        )
    }
}

/// The text a row shows as its value (a frontend with a row type of its own
/// calls this with its parts). `coming` is the row's reason, `option_label`
/// the display text of a raw choice.
pub fn value_text<A>(
    coming: Option<&str>,
    control: &Control<A>,
    option_label: impl Fn(&str) -> String,
    first_line: impl Fn(&str) -> String,
) -> String {
    if let Some(reason) = coming {
        if super::notes::is_note(reason) {
            return reason.to_string();
        }
        return screen::coming_value(reason);
    }
    match control {
        Control::Toggle(on) => (if *on { "[x]" } else { "[ ]" }).to_string(),
        Control::Number { value, .. } => value.to_string(),
        Control::Text(value) if value.is_empty() => screen::NOT_SET.get().to_string(),
        Control::Text(value) => value.clone(),
        Control::Quantity {
            value, unit, units, ..
        } => match units.get(*unit) {
            Some(u) => format!("{value} {}", u.label),
            None => value.to_string(),
        },
        Control::Choice { options, index } => options
            .get(*index)
            .map(|o| option_label(o))
            .unwrap_or_else(|| screen::NONE.get().to_string()),
        Control::Combo { options, value, .. } => options
            .iter()
            .find(|(_, v)| v == value)
            .map(|(l, _)| l.clone())
            .filter(|_| !value.is_empty() || options.iter().any(|(_, v)| v.is_empty()))
            .unwrap_or_else(|| {
                if value.is_empty() {
                    screen::NOT_SET.get().to_string()
                } else {
                    value.clone()
                }
            }),
        Control::Buttons(buttons) => buttons
            .iter()
            .map(|b| b.label.as_str())
            .collect::<Vec<_>>()
            .join(" | "),
        Control::Heading => String::new(),
        Control::Info(text) => text.clone(),
        Control::Swatches(_) => String::new(),
        Control::Chips(labels) => labels.join(" | "),
        Control::Area(text) => first_line(text),
    }
}
