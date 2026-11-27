//! **Form controls the keyboard can reach** (RFC-134): a checkbox, a toggle, a radio group, a
//! select and a slider.
//!
//! iced 0.14's `checkbox`, `toggler`, `radio`, `pick_list` and `slider` register no focus target,
//! so Tab cannot land on them and no key does anything to them (a slider takes ↑ ↓ only while
//! the pointer is over it). Each control here is **one**
//! [`Control`](snora_interaction::Control) around its own marks, so Tab, the focus ring, Enter and
//! Space, "a disabled control is no stop" and the 24 px target are the same ones every other snora
//! control has, and **a click and a key publish the same message** (R-3).
//!
//! | Control | Tab | Keys |
//! |---|---|---|
//! | [`checkbox`], [`toggler`] | one stop | Enter or Space toggles (a chord is not a press) |
//! | [`radio_group`] | **one stop for the group** | ↑ ↓ ← → move **and select**, wrapping; ← → follow the screen |
//! | [`select`] | one stop | Enter, Space or ↓ opens; inside, the menu's keys (↓ ↑ Home End, Enter chooses, Escape closes) |
//! | [`slider`] | one stop | ← → ↑ ↓ one step, Page Up and Page Down ten, Home and End the ends; ← → follow the screen |
//!
//! # Two paths
//!
//! The functions here are the **engine path**: coloured by the host's iced theme, sized in ems of
//! the host's text, ringed as [`snora::control`](https://docs.rs/snora) is. With the `design`
//! feature the same five exist as `snora::design::{checkbox, toggler, radio_group, select,
//! slider}`, which take the tokens. They are the same types.
//!
//! # Direction
//!
//! Every constructor takes the [`LayoutDirection`] last (RFC-133): the box or switch is at the start
//! edge of its label, the toggler's thumb moves toward the end edge when on, the radio group's ← →
//! and the slider's follow the screen, and the slider fills from the start edge.
//!
//! # The select, and the engine
//!
//! Like [`render_menu`](crate::render_menu), **a select gets its ↓ ↑ Home End, its cover over the
//! page and its click-outside only inside `snora::render`**; elsewhere it gets Tab, Enter, Space
//! and Escape. And like a header's menu, **it is open when the application says so** and is closed
//! by the one `on_close_menus` message an application gives the engine: with two selects, map it to
//! "close whichever is open".

use std::rc::Rc;

use iced::keyboard::{Key, key::Named};
use iced::widget::{Id, button, column, container, mouse_area, scrollable, space, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};
use snora_core::{Icon, LayoutDirection};
use snora_interaction::{BarColors, EmBox, MenuAnchor, MirroredRow, OnEscape, SliderBar};

use crate::direction::LogicalRow;
use crate::icon::{icon_element, icon_element_sized};
use crate::menu::{dropdown_card, scrollbar_style};
use crate::style::{BUTTON_DEFAULT_PADDING, ControlGeometry, control, page_background};

use snora_interaction::contrast::readable_text;

// ---------------------------------------------------------------------------
// The two paths: what differs, and nothing else
// ---------------------------------------------------------------------------

/// What a form control is drawn from: the host's theme (engine path) or the tokens (design path).
#[derive(Clone)]
enum Look {
    Unstyled,
    #[cfg(feature = "design")]
    Designed(Rc<snora_design::Tokens>),
}

/// The colours of a control's parts, for one theme.
#[derive(Clone, Copy)]
struct Colors {
    /// The box's, switch's, dot's and track's outline: held at 3:1 against the page **and against a
    /// hovered or pressed row's tint**.
    border: Color,
    /// The checked box, the switch's track when on, the selected dot, the filled part.
    accent: Color,
    /// What is drawn on `accent`: the tick, the thumb of a switch that is on.
    on_accent: Color,
    /// The thumb of a switch that is off.
    off: Color,
    /// The page, for what is drawn as a hole in it (a slider's thumb, a select's card).
    page: Color,
    /// The track's own fill.
    track: Color,
}

impl Colors {
    /// A disabled control is dimmed as every snora control is (an alpha, and exempt from the floors).
    fn dimmed(self) -> Self {
        let dim = |c: Color| Color { a: c.a * 0.45, ..c };
        Self {
            border: dim(self.border),
            accent: dim(self.accent),
            on_accent: dim(self.on_accent),
            off: dim(self.off),
            page: self.page,
            track: dim(self.track),
        }
    }
}

/// A row's style function: what a control is given for its look.
type RowStyle = Box<dyn Fn(&Theme, button::Status) -> button::Style + 'static>;

/// How one path draws the five. Cheap to clone: closures that need it clone it.
#[derive(Clone)]
pub(crate) struct Skin {
    look: Look,
    geometry: ControlGeometry,
    /// The label's size in pixels on the design path (`Typography::label`); `None` follows the
    /// host's text, which only layout can read.
    label_size: Option<f32>,
    /// Between a mark and its label.
    gap: f32,
    /// Between two options of a radio group laid out in a row (RFC-138): **twice [`gap`](Self::gap)**,
    /// on both paths, so that a label is never nearer the next option's mark than its own. Defined
    /// from `gap` and not given a token of its own, so the rule holds for any token set.
    between: f32,
    padding: Padding,
}

impl Skin {
    /// The engine path: the host's theme, the host's text size, the ring's literals.
    pub(crate) fn unstyled() -> Self {
        Self {
            look: Look::Unstyled,
            geometry: ControlGeometry::unstyled(),
            label_size: None,
            gap: 8.0,
            between: 2.0 * 8.0,
            padding: BUTTON_DEFAULT_PADDING,
        }
    }

    /// The design path: every value from the tokens.
    #[cfg(feature = "design")]
    pub(crate) fn designed(tokens: &snora_design::Tokens) -> Self {
        Self {
            look: Look::Designed(Rc::new(tokens.clone())),
            geometry: ControlGeometry::from_tokens(tokens),
            label_size: Some(snora_style::text::label_size(tokens).0),
            gap: tokens.spacing.sm,
            between: 2.0 * tokens.spacing.sm,
            padding: Padding::from([tokens.spacing.xs, tokens.spacing.sm]),
        }
    }

    fn colors(&self, theme: &Theme, enabled: bool) -> Colors {
        let colors = match &self.look {
            Look::Unstyled => {
                let page = page_background(theme);
                let ep = theme.extended_palette();
                // What a mark is drawn against: the page, and the tints a hovered and a pressed row
                // put behind it (the theme's own `weak` and `strong` tiers, as `unstyled_row` draws).
                let behind = [page, ep.background.weak.color, ep.background.strong.color];
                // Each held against **all three** by the rule the ring uses: the theme's own colour
                // where it reads, and the stronger of black and white where it does not.
                let border = held_against(&behind, ep.background.base.text);
                let accent = held_against(&behind, ep.primary.strong.color);
                Colors {
                    border,
                    accent,
                    on_accent: readable_text(accent, ep.primary.strong.text),
                    off: border,
                    page,
                    track: ep.background.weak.color,
                }
            }
            #[cfg(feature = "design")]
            Look::Designed(t) => {
                let c = snora_style::color::to_iced_color;
                Colors {
                    // **Not `palette.border`**: that role is held at 3:1 against the page, and a
                    // hovered or pressed row draws a tint behind the mark, which takes 3.12:1 to
                    // 2.5:1 (the sweep read it). `text_secondary` is held at the text level.
                    border: c(t.palette.text_secondary),
                    accent: c(t.palette.accent),
                    on_accent: c(t.palette.accent_text),
                    off: c(t.palette.text_primary),
                    page: c(t.palette.background),
                    track: c(t.palette.surface),
                }
            }
        };
        if enabled { colors } else { colors.dimmed() }
    }

    /// The control's row style: nothing at rest, a tint of the label's own colour hovered and pressed.
    fn row_style(&self) -> RowStyle {
        match &self.look {
            Look::Unstyled => Box::new(unstyled_row),
            #[cfg(feature = "design")]
            Look::Designed(t) => {
                let t = Rc::clone(t);
                Box::new(move |_theme, status| snora_style::form::row(&t, status))
            }
        }
    }

    /// One control around `content`, with this path's padding, ring and minimum target.
    fn control<'a, Message: Clone + 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> snora_interaction::Control<'a, Message> {
        let behind: Box<dyn Fn(&Theme) -> Color> = match &self.look {
            Look::Unstyled => Box::new(page_background),
            #[cfg(feature = "design")]
            Look::Designed(t) => {
                let page = snora_style::color::to_iced_color(t.palette.background);
                Box::new(move |_| page)
            }
        };
        control(
            content,
            self.row_style(),
            self.geometry,
            self.label_size,
            behind,
        )
        .padding(self.padding)
    }

    fn label<'a, Message: 'a>(&self, label: String) -> Element<'a, Message> {
        match self.label_size {
            Some(px) => text(label).size(px).into(),
            None => text(label).into(),
        }
    }

    /// A square of `ems` em, around `content`: the host's em on the engine path, the label's on
    /// the design path.
    fn square<'a, Message: 'a>(
        &self,
        ems: f32,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let inner = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        match self.label_size {
            Some(px) => container(inner)
                .width(Length::Fixed(px * ems))
                .height(Length::Fixed(px * ems))
                .into(),
            None => EmBox::new(inner, ems).into(),
        }
    }

    /// A mark drawn as a text glyph at the label's size, in every build (RFC-139: a Cargo feature never
    /// changes how a mark snora draws itself is drawn).
    fn mark<'a, Message: 'a>(&self, mark: Mark) -> Element<'a, Message> {
        let icon = mark.icon();
        match self.label_size {
            Some(px) => icon_element_sized(&icon, px),
            None => icon_element(&icon),
        }
    }

    /// The em the bar is drawn in on the design path.
    fn unit(&self) -> Option<f32> {
        self.label_size
    }
}

/// The marks drawn as glyphs.
#[derive(Clone, Copy)]
enum Mark {
    Tick,
    Chevron,
}

impl Mark {
    fn icon(self) -> Icon {
        Icon::Text(
            match self {
                Mark::Tick => "\u{2713}",
                Mark::Chevron => "\u{25be}",
            }
            .to_owned(),
        )
    }
}

/// The colour a mark is drawn in: `preferred` where it holds 3:1 against **every** colour in `behind` (the
/// page and the tints a pointer state puts behind it), and otherwise the stronger of black and white against the
/// weakest of them (at least 4.58:1 against an opaque fill, so a theme snora has never seen cannot fail).
fn held_against(behind: &[Color], preferred: Color) -> Color {
    use snora_interaction::contrast::{contrast_ratio, stronger_pole};
    // A hair above 3:1, so that 8-bit rounding cannot take it under.
    const FLOOR: f32 = 3.05;
    if behind
        .iter()
        .all(|b| contrast_ratio(preferred, *b) >= FLOOR)
    {
        return preferred;
    }
    // The pole that reads against the one that is hardest to read against.
    let hardest = behind
        .iter()
        .copied()
        .min_by(|a, b| contrast_ratio(preferred, *a).total_cmp(&contrast_ratio(preferred, *b)))
        .unwrap_or(preferred);
    stronger_pole(hardest)
}

/// The engine path's row style: nothing at rest, the theme's own `weak` and `strong` tiers (with
/// the text iced pairs with them) hovered and pressed.
fn unstyled_row(theme: &Theme, status: button::Status) -> button::Style {
    let ep = theme.extended_palette();
    let (background, text_color) = match status {
        button::Status::Active => (None, ep.background.base.text),
        button::Status::Hovered => (
            Some(Background::Color(ep.background.weak.color)),
            ep.background.weak.text,
        ),
        button::Status::Pressed => (
            Some(Background::Color(ep.background.strong.color)),
            ep.background.strong.text,
        ),
        button::Status::Disabled => (
            None,
            Color {
                a: ep.background.base.text.a * 0.45,
                ..ep.background.base.text
            },
        ),
    };
    button::Style {
        background,
        text_color,
        border: Border::default().rounded(4.0),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

/// The outline every mark has: 1 px in `border`.
fn outline(colors: Colors, radius: f32) -> Border {
    Border {
        color: colors.border,
        width: 1.0,
        radius: radius.into(),
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// A checkbox the keyboard can reach: Tab lands on it once, and Enter or Space toggles it.
///
/// Made by [`checkbox`] (engine path) or `snora::design::checkbox` (design path). Give it a label
/// and a handler, and turn it into an element with `.into()`.
///
/// ```
/// # use snora_widgets::checkbox;
/// # use snora_core::LayoutDirection;
/// # #[derive(Clone)] enum Message { Remember(bool) }
/// # fn view<'a>(remember: bool) -> iced::Element<'a, Message> {
/// // iced::widget::checkbox(remember).label("Remember me").on_toggle(Message::Remember)
/// checkbox(remember, LayoutDirection::Ltr)
///     .label("Remember me")
///     .on_toggle(Message::Remember)
///     .into()
/// # }
/// ```
///
/// **Disabled when it has no handler to publish with** (`on_toggle`, or `on_toggle_maybe(None)`),
/// as `snora::control` and iced's own are: it is no Tab stop, nothing toggles it, and it draws dimmed.
#[must_use = "a checkbox does nothing until it is added to a view"]
pub struct Checkbox<'a, Message> {
    skin: Skin,
    is_checked: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    direction: LayoutDirection,
    id: Option<Id>,
}

/// A checkbox on the engine path (the host's theme). `is_checked` is the application's state.
///
/// `direction` puts the box at the start edge of its label (RFC-133).
pub fn checkbox<'a, Message>(
    is_checked: bool,
    direction: LayoutDirection,
) -> Checkbox<'a, Message> {
    Checkbox::with_skin(Skin::unstyled(), is_checked, direction)
}

impl<'a, Message> Checkbox<'a, Message> {
    pub(crate) fn with_skin(skin: Skin, is_checked: bool, direction: LayoutDirection) -> Self {
        Self {
            skin,
            is_checked,
            label: None,
            on_toggle: None,
            direction,
            id: None,
        }
    }

    /// The text beside the box. It is part of the target: a click on it toggles.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// What toggling makes: the new state. A click and Enter or Space publish **this** message.
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Like [`on_toggle`](Self::on_toggle), and `None` disables the checkbox.
    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|f| Box::new(f) as Box<dyn Fn(bool) -> Message + 'a>);
        self
    }

    /// The control's `widget::Id`, so that iced's `focus(id)` reaches it (RFC-127).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Checkbox<'a, Message>> for Element<'a, Message> {
    fn from(c: Checkbox<'a, Message>) -> Self {
        let skin = c.skin;
        let enabled = c.on_toggle.is_some();
        let checked = c.is_checked;

        let tick = if checked {
            let inner = skin.mark(Mark::Tick);
            let for_colors = skin.clone();
            container(inner)
                .style(move |theme: &Theme| container::Style {
                    text_color: Some(for_colors.colors(theme, enabled).on_accent),
                    ..container::Style::default()
                })
                .into()
        } else {
            Element::from(space())
        };
        let for_box = skin.clone();
        let mark = skin.square(1.25, tick);
        let mark = container(mark).style(move |theme: &Theme| {
            let colors = for_box.colors(theme, enabled);
            container::Style {
                background: checked.then_some(Background::Color(colors.accent)),
                border: outline(
                    Colors {
                        border: if checked {
                            colors.accent
                        } else {
                            colors.border
                        },
                        ..colors
                    },
                    3.0,
                ),
                ..container::Style::default()
            }
        });

        let mut row = LogicalRow::new(c.direction)
            .spacing(skin.gap)
            .align_y(Alignment::Center)
            .push(mark);
        if let Some(label) = c.label {
            row = row.push(skin.label(label));
        }
        let mut control = skin.control(row);
        if let Some(on_toggle) = c.on_toggle {
            control = control.on_press_with(move || on_toggle(!checked));
        }
        if let Some(id) = c.id {
            control = control.id(id);
        }
        control.into()
    }
}

// ---------------------------------------------------------------------------
// Toggler
// ---------------------------------------------------------------------------

/// A toggle (a switch) the keyboard can reach: Tab lands on it once, and Enter or Space flips it.
///
/// Made by [`toggler`] or `snora::design::toggler`. The thumb is at the **start edge** when off and
/// the end edge when on: the left and the right under LTR, the reverse under RTL.
///
/// ```
/// # use snora_widgets::toggler;
/// # use snora_core::LayoutDirection;
/// # #[derive(Clone)] enum Message { DarkMode(bool) }
/// # fn view<'a>(dark: bool) -> iced::Element<'a, Message> {
/// toggler(dark, LayoutDirection::Ltr)
///     .label("Dark mode")
///     .on_toggle(Message::DarkMode)
///     .into()
/// # }
/// ```
///
/// **Disabled when it has no handler to publish with** (`on_toggle`, or `on_toggle_maybe(None)`): it
/// is no Tab stop, nothing flips it, and it draws dimmed.
#[must_use = "a toggler does nothing until it is added to a view"]
pub struct Toggler<'a, Message> {
    skin: Skin,
    is_toggled: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    direction: LayoutDirection,
    id: Option<Id>,
}

/// A toggle on the engine path. `is_toggled` is the application's state.
pub fn toggler<'a, Message>(is_toggled: bool, direction: LayoutDirection) -> Toggler<'a, Message> {
    Toggler::with_skin(Skin::unstyled(), is_toggled, direction)
}

impl<'a, Message> Toggler<'a, Message> {
    pub(crate) fn with_skin(skin: Skin, is_toggled: bool, direction: LayoutDirection) -> Self {
        Self {
            skin,
            is_toggled,
            label: None,
            on_toggle: None,
            direction,
            id: None,
        }
    }

    /// The text beside the switch. It is part of the target.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// What flipping makes: the new state. A click and Enter or Space publish **this** message.
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Like [`on_toggle`](Self::on_toggle), and `None` disables the toggle.
    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|f| Box::new(f) as Box<dyn Fn(bool) -> Message + 'a>);
        self
    }

    /// The control's `widget::Id`, so that iced's `focus(id)` reaches it (RFC-127).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Toggler<'a, Message>> for Element<'a, Message> {
    fn from(t: Toggler<'a, Message>) -> Self {
        let skin = t.skin;
        let enabled = t.on_toggle.is_some();
        let on = t.is_toggled;

        // Two square slots side by side; the thumb sits in the first when off and the second when
        // on. A `LogicalRow` puts the first at the start edge, so RTL needs no code of its own.
        let for_thumb = skin.clone();
        let thumb = |skin: &Skin| -> Element<'a, Message> {
            let for_thumb = for_thumb.clone();
            let disc = container(space())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |theme: &Theme| {
                    let colors = for_thumb.colors(theme, enabled);
                    container::Style {
                        background: Some(Background::Color(if on {
                            colors.on_accent
                        } else {
                            colors.off
                        })),
                        border: Border {
                            radius: 1000.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                });
            skin.square(1.25, container(disc).padding(3))
        };
        let empty = |skin: &Skin| -> Element<'a, Message> { skin.square(1.25, space()) };
        let inside = if on {
            LogicalRow::new(t.direction)
                .push(empty(&skin))
                .push(thumb(&skin))
        } else {
            LogicalRow::new(t.direction)
                .push(thumb(&skin))
                .push(empty(&skin))
        };
        let for_track = skin.clone();
        let track = container(inside).style(move |theme: &Theme| {
            let colors = for_track.colors(theme, enabled);
            container::Style {
                background: on.then_some(Background::Color(colors.accent)),
                border: outline(
                    Colors {
                        border: if on { colors.accent } else { colors.border },
                        ..colors
                    },
                    1000.0,
                ),
                ..container::Style::default()
            }
        });

        let mut row = LogicalRow::new(t.direction)
            .spacing(skin.gap)
            .align_y(Alignment::Center)
            .push(track);
        if let Some(label) = t.label {
            row = row.push(skin.label(label));
        }
        let mut control = skin.control(row);
        if let Some(on_toggle) = t.on_toggle {
            control = control.on_press_with(move || on_toggle(!on));
        }
        if let Some(id) = t.id {
            control = control.id(id);
        }
        control.into()
    }
}

// ---------------------------------------------------------------------------
// Radio group
// ---------------------------------------------------------------------------

/// A group of radio options the keyboard can reach: **one Tab stop for the group**, and ↑ ↓ ← →
/// move the selection **and select** (a radio group's arrows are the choice), wrapping at the ends.
/// ← → follow the screen: under RTL ← is forward.
///
/// Made by [`radio_group`] or `snora::design::radio_group`. A click on an option selects it, and
/// publishes the message an arrow does. **A click on the option already selected, and Space or
/// Enter on the group, publish nothing**: it is already the choice, and a message that changes
/// nothing is noise to every handler that logs or saves.
///
/// ```
/// # use snora_widgets::radio_group;
/// # use snora_core::LayoutDirection;
/// # #[derive(Clone)] enum Message { Size(&'static str) }
/// # fn view<'a>(size: &'a &'static str) -> iced::Element<'a, Message> {
/// radio_group(["Small", "Medium", "Large"], Some(size), Message::Size, LayoutDirection::Ltr)
///     .into()
/// # }
/// ```
///
/// With nothing selected, Tab still lands on the group, and → or ↓ selects the first option and ←
/// or ↑ the last.
///
/// **Disabled by [`disabled(true)`](Self::disabled)**, because its handler is a required argument
/// (as iced's own shape has it): it is no Tab stop, takes no key, ignores a click and draws dimmed.
///
/// **In a column, or in a row: [`horizontal(true)`](Self::horizontal).** The column is the default.
#[must_use = "a radio group does nothing until it is added to a view"]
pub struct RadioGroup<'a, T, Message> {
    skin: Skin,
    options: Rc<[T]>,
    selected: Option<usize>,
    on_select: Rc<dyn Fn(T) -> Message + 'a>,
    direction: LayoutDirection,
    disabled: bool,
    horizontal: bool,
    id: Option<Id>,
}

/// A radio group on the engine path: `options`, the selected one (`None` for none), and what
/// choosing an option makes.
///
/// `options` may be an array, a `Vec` or a slice; the label of each is `to_string()`.
pub fn radio_group<'a, T, Message>(
    options: impl Into<Vec<T>>,
    selected: Option<&T>,
    on_select: impl Fn(T) -> Message + 'a,
    direction: LayoutDirection,
) -> RadioGroup<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    RadioGroup::with_skin(Skin::unstyled(), options, selected, on_select, direction)
}

impl<'a, T, Message> RadioGroup<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    pub(crate) fn with_skin(
        skin: Skin,
        options: impl Into<Vec<T>>,
        selected: Option<&T>,
        on_select: impl Fn(T) -> Message + 'a,
        direction: LayoutDirection,
    ) -> Self {
        let options: Rc<[T]> = Rc::from(options.into());
        let selected = selected.and_then(|s| options.iter().position(|o| o == s));
        Self {
            skin,
            options,
            selected,
            on_select: Rc::new(on_select),
            direction,
            disabled: false,
            horizontal: false,
            id: None,
        }
    }

    /// Lays the options out in a row that wraps (RFC-138). **The column is the default**: with
    /// `horizontal(false)` the group is drawn exactly as it is without the call, so a responsive layout
    /// can pass its own condition (`.horizontal(window_is_wide)`).
    ///
    /// ```
    /// # use snora_widgets::radio_group;
    /// # use snora_core::LayoutDirection;
    /// # #[derive(Clone)] enum Message { ViewAs(&'static str) }
    /// # fn view<'a>(view: &'a &'static str) -> iced::Element<'a, Message> {
    /// radio_group(["List", "Grid"], Some(view), Message::ViewAs, LayoutDirection::Ltr)
    ///     .horizontal(true)
    ///     .into()
    /// # }
    /// ```
    ///
    /// - **The row always wraps.** When the options do not fit the room the group has, it continues on
    ///   the next line; there is no setting to turn wrapping off.
    /// - **The options keep their logical order, line by line,** and sit at the start edge of each
    ///   line. Under right-to-left each line is mirrored: the first option is at the right end of the
    ///   first line, and a shorter line starts from the right edge too.
    /// - **The space between options is twice the space between a mark and its label**, so a label
    ///   never reads as belonging to the next option. The lines are stacked at the column's pitch.
    /// - **The keys are the same in either layout**: one Tab stop; ↓ and → move to the next option and
    ///   ↑ and ← to the previous one, wrapping at the ends, with ← → following the screen. **There is
    ///   no two-dimensional movement in a wrapped row: ↓ is "next", not "the option below".**
    /// - **One limit, shared with the column:** a line breaks between options, never inside one. An
    ///   option that alone is wider than the room is laid out as the column lays it out.
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// A disabled group is no Tab stop, takes no key and ignores a click, and draws dimmed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The control's `widget::Id`, so that iced's `focus(id)` reaches it (RFC-127).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'a, T, Message> From<RadioGroup<'a, T, Message>> for Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    fn from(g: RadioGroup<'a, T, Message>) -> Self {
        let skin = g.skin;
        let enabled = !g.disabled;
        let direction = g.direction;

        let rows = g.options.iter().enumerate().map(|(i, option)| {
            let is_selected = g.selected == Some(i);
            let for_dot = skin.clone();
            let ring = container(
                container(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |theme: &Theme| {
                        let colors = for_dot.colors(theme, enabled);
                        container::Style {
                            background: is_selected.then_some(Background::Color(colors.accent)),
                            border: Border {
                                radius: 1000.0.into(),
                                ..Border::default()
                            },
                            ..container::Style::default()
                        }
                    }),
            )
            .padding(4);
            let for_ring = skin.clone();
            let dot = container(skin.square(1.25, ring)).style(move |theme: &Theme| {
                let colors = for_ring.colors(theme, enabled);
                container::Style {
                    border: outline(
                        Colors {
                            border: if is_selected {
                                colors.accent
                            } else {
                                colors.border
                            },
                            ..colors
                        },
                        1000.0,
                    ),
                    ..container::Style::default()
                }
            });
            let content = LogicalRow::new(direction)
                .spacing(skin.gap)
                .align_y(Alignment::Center)
                .padding(Padding::from([6.0, 0.0]))
                .push(dot)
                .push(skin.label(option.to_string()));
            let mut area = mouse_area(content);
            // A click on the option already selected publishes nothing, as Space and Enter do: a
            // message that changes nothing is noise to every handler that logs or saves (RFC-134
            // ruling 3, and R-3: a click and a key are one action with one result).
            if enabled && !is_selected {
                area = area
                    .on_press((g.on_select)(option.clone()))
                    .interaction(iced::mouse::Interaction::Pointer);
            }
            Element::from(area)
        });
        // The options hug the start edge of the group, as a menu's items do.
        let group: Element<'a, Message> = if g.horizontal {
            // A row that wraps, its direct children the options in logical order: `MirroredRow`
            // reflects each child's x within the row's width, which is, line by line, the mirror
            // image under RTL (a shorter line starts from the right edge too). The tree is never
            // reversed. The space between lines is the column's: each option carries its own
            // vertical padding, so there is none here.
            let wrapped = iced::widget::Row::with_children(rows)
                .spacing(skin.between)
                .wrap()
                .vertical_spacing(0.0);
            MirroredRow::new(wrapped, direction == LayoutDirection::Rtl).into()
        } else {
            column(rows)
                .align_x(match direction {
                    LayoutDirection::Ltr => Alignment::Start,
                    LayoutDirection::Rtl => Alignment::End,
                })
                .into()
        };

        let mut control = skin.control(group).width(Length::Shrink);
        if enabled {
            let options = Rc::clone(&g.options);
            let on_select = Rc::clone(&g.on_select);
            let selected = g.selected;
            control = control.focus_only().on_key(move |key, mods| {
                if !mods.is_empty() || options.is_empty() {
                    return None;
                }
                let n = options.len();
                let forward = match key {
                    Key::Named(Named::ArrowDown) => true,
                    Key::Named(Named::ArrowUp) => false,
                    Key::Named(Named::ArrowRight) => direction == LayoutDirection::Ltr,
                    Key::Named(Named::ArrowLeft) => direction == LayoutDirection::Rtl,
                    _ => return None,
                };
                let to = match (selected, forward) {
                    (Some(i), true) => (i + 1) % n,
                    (Some(i), false) => (i + n - 1) % n,
                    (None, true) => 0,
                    (None, false) => n - 1,
                };
                Some(on_select(options[to].clone()))
            });
        } else {
            control = control.on_press_maybe(None);
        }
        if let Some(id) = g.id {
            control = control.id(id);
        }
        control.into()
    }
}

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------

/// A slider the keyboard can reach: Tab lands on it once; ← → ↑ ↓ move it one step, Page Up and
/// Page Down ten steps, Home and End to the ends. ← → follow the screen (under RTL → lowers), and
/// the fill runs from the start edge.
///
/// Made by [`slider`] or `snora::design::slider`. A click or a drag publishes the value under the
/// pointer, snapped to the step, and **a key publishes through the same handler**.
///
/// ```
/// # use snora_widgets::slider;
/// # use snora_core::LayoutDirection;
/// # #[derive(Clone)] enum Message { Volume(f32) }
/// # fn view<'a>(volume: f32) -> iced::Element<'a, Message> {
/// // iced::widget::slider(0.0..=100.0, volume, Message::Volume)
/// slider(0.0..=100.0, volume, Message::Volume, LayoutDirection::Ltr)
///     .step(5.0)
///     .into()
/// # }
/// ```
///
/// **The step** is the amount a key moves and the grid a pointer snaps to. Unless said it is **one
/// hundredth of the range**, so a 0 to 1 slider moves by 0.01 and a 0 to 100 slider by 1. A move
/// that would not change the value (an arrow at the end) publishes nothing.
///
/// Values are `f32`. (iced's slider is generic over the number type; this one is not, and an
/// integer value is `value as f32`.)
///
/// **Disabled by [`disabled(true)`](Self::disabled)**, because its handler is a required argument (as
/// iced's own shape has it): it is no Tab stop, takes no key, ignores the pointer and draws dimmed.
#[must_use = "a slider does nothing until it is added to a view"]
pub struct Slider<'a, Message> {
    skin: Skin,
    range: (f32, f32),
    value: f32,
    on_change: Rc<dyn Fn(f32) -> Message + 'a>,
    step: Option<f32>,
    direction: LayoutDirection,
    disabled: bool,
    width: Length,
    id: Option<Id>,
}

/// A slider on the engine path: `range`, the current `value`, and what a new value makes.
pub fn slider<'a, Message>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
    direction: LayoutDirection,
) -> Slider<'a, Message> {
    Slider::with_skin(Skin::unstyled(), range, value, on_change, direction)
}

impl<'a, Message> Slider<'a, Message> {
    pub(crate) fn with_skin(
        skin: Skin,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        on_change: impl Fn(f32) -> Message + 'a,
        direction: LayoutDirection,
    ) -> Self {
        let (start, end) = (*range.start(), *range.end());
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        Self {
            skin,
            range: (start, end),
            value: value.clamp(start, end),
            on_change: Rc::new(on_change),
            step: None,
            direction,
            disabled: false,
            width: Length::Fill,
            id: None,
        }
    }

    /// The amount a key moves, and the grid a click or drag snaps to. Unset: a hundredth of the range.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// A disabled slider is no Tab stop, takes no key and ignores the pointer, and draws dimmed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The slider's width. `Fill` unless said.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// The control's `widget::Id`, so that iced's `focus(id)` reaches it (RFC-127).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

/// `value` snapped to the grid `start + k * step`, inside `start..=end`. The end itself is a stop
/// even when the range is not a whole number of steps.
fn snapped(value: f32, start: f32, end: f32, step: f32) -> f32 {
    if step <= 0.0 || !step.is_finite() {
        return value.clamp(start, end);
    }
    let k = ((value - start) / step).round();
    let on_grid = (start + k * step).clamp(start, end);
    // Within half a step of the end means the end.
    if (end - value).abs() <= step / 2.0 && (end - on_grid).abs() <= step {
        end
    } else {
        on_grid
    }
}

impl<'a, Message: Clone + 'a> From<Slider<'a, Message>> for Element<'a, Message> {
    fn from(s: Slider<'a, Message>) -> Self {
        let skin = s.skin;
        let enabled = !s.disabled;
        let (start, end) = s.range;
        let step = s.step.unwrap_or((end - start) / 100.0);
        let value = s.value;
        let direction = s.direction;
        let rtl = direction == LayoutDirection::Rtl;
        let span = end - start;
        let fraction = if span > 0.0 {
            (value - start) / span
        } else {
            0.0
        };

        // What the pointer means: the value under it, snapped, and nothing if that is the value
        // the slider already has.
        let on_change = Rc::clone(&s.on_change);
        let from_pointer = move |f: f32| -> Option<Message> {
            let v = snapped(start + f * span, start, end, step);
            ((v - value).abs() > f32::EPSILON * span.abs().max(1.0)).then(|| on_change(v))
        };
        let for_colors = skin.clone();
        let mut bar = SliderBar::new(
            fraction,
            rtl,
            move |theme: &Theme| {
                let colors = for_colors.colors(theme, enabled);
                BarColors {
                    outline: colors.border,
                    track: colors.track,
                    fill: colors.accent,
                    thumb: colors.page,
                    thumb_outline: colors.border,
                }
            },
            from_pointer,
        )
        .enabled(enabled)
        .width(s.width);
        if let Some(unit) = skin.unit() {
            bar = bar.unit(unit);
        }

        let mut control = skin.control(bar).width(s.width);
        if enabled {
            let on_change = Rc::clone(&s.on_change);
            let page = step * 10.0;
            control = control.focus_only().on_key(move |key, mods| {
                if !mods.is_empty() {
                    return None;
                }
                // ← → by the screen: under RTL the right arrow lowers.
                let right = if rtl { -1.0 } else { 1.0 };
                let to = match key {
                    Key::Named(Named::ArrowRight) => value + right * step,
                    Key::Named(Named::ArrowLeft) => value - right * step,
                    Key::Named(Named::ArrowUp) => value + step,
                    Key::Named(Named::ArrowDown) => value - step,
                    Key::Named(Named::PageUp) => value + page,
                    Key::Named(Named::PageDown) => value - page,
                    Key::Named(Named::Home) => start,
                    Key::Named(Named::End) => end,
                    _ => return None,
                };
                let to = to.clamp(start, end);
                ((to - value).abs() > f32::EPSILON * span.abs().max(1.0)).then(|| on_change(to))
            });
        } else {
            control = control.on_press_maybe(None);
        }
        if let Some(id) = s.id {
            control = control.id(id);
        }
        control.into()
    }
}

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// A select (a pick list) the keyboard can reach: Tab lands on its trigger once; Enter, Space or ↓
/// opens it; inside, ↓ ↑ Home End move, Enter chooses and Escape closes, and focus returns to the
/// trigger. (↓ ↑ Home End and the click-outside need `snora::render`; see the
/// module documentation.)
///
/// Made by [`select`] or `snora::design::select`. **It is open when the application says so**
/// ([`opened`](Self::opened)), and says so back through [`on_open_change`](Self::on_open_change):
/// the trigger, Enter, Space, ↓ and Escape publish `on_open_change(true or false)`, and choosing an
/// item publishes `on_select(item)` and **nothing else**: close it there. One message per action.
///
/// ```
/// # use snora_widgets::select;
/// # use snora_core::LayoutDirection;
/// # #[derive(Clone)] enum Message { Fruit(&'static str), Open(bool) }
/// # fn view<'a>(fruit: Option<&'a &'static str>, open: bool) -> iced::Element<'a, Message> {
/// select(["Apple", "Banana", "Cherry"], fruit, Message::Fruit, LayoutDirection::Ltr)
///     .opened(open)
///     .on_open_change(Message::Open)
///     .into()
/// # }
/// ```
///
/// **Disabled when it has no handler to publish with** (`on_open_change`), as `snora::control` is: it
/// is no Tab stop, nothing opens it and it draws dimmed.
#[must_use = "a select does nothing until it is added to a view"]
pub struct Select<'a, T, Message> {
    skin: Skin,
    options: Vec<T>,
    selected: Option<usize>,
    on_select: Rc<dyn Fn(T) -> Message + 'a>,
    on_open_change: Option<Rc<dyn Fn(bool) -> Message + 'a>>,
    open: bool,
    placeholder: Option<String>,
    direction: LayoutDirection,
    id: Option<Id>,
}

/// A select on the engine path: `options`, the selected one, and what choosing makes.
///
/// The label of each is `to_string()`.
pub fn select<'a, T, Message>(
    options: impl Into<Vec<T>>,
    selected: Option<&T>,
    on_select: impl Fn(T) -> Message + 'a,
    direction: LayoutDirection,
) -> Select<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    Select::with_skin(Skin::unstyled(), options, selected, on_select, direction)
}

impl<'a, T, Message> Select<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    pub(crate) fn with_skin(
        skin: Skin,
        options: impl Into<Vec<T>>,
        selected: Option<&T>,
        on_select: impl Fn(T) -> Message + 'a,
        direction: LayoutDirection,
    ) -> Self {
        let options = options.into();
        let selected = selected.and_then(|s| options.iter().position(|o| o == s));
        Self {
            skin,
            options,
            selected,
            on_select: Rc::new(on_select),
            on_open_change: None,
            open: false,
            placeholder: None,
            direction,
            id: None,
        }
    }

    /// Whether the list is open. The application's state, closed by `on_open_change(false)` and by
    /// the engine's `on_close_menus`.
    pub fn opened(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// What opening and closing make: `true` from the trigger, Enter, Space and ↓, `false` from
    /// Escape and from the trigger when open. Without it the select is disabled.
    pub fn on_open_change(mut self, on_open_change: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_open_change = Some(Rc::new(on_open_change));
        self
    }

    /// What the trigger says while nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// The trigger's `widget::Id`, so that iced's `focus(id)` reaches it (RFC-127).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl<'a, T, Message> From<Select<'a, T, Message>> for Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    fn from(s: Select<'a, T, Message>) -> Self {
        let skin = s.skin;
        let direction = s.direction;
        let enabled = s.on_open_change.is_some();
        let open = s.open;

        // ---- the trigger: the chosen label and a chevron, in an outline ----
        let label = s
            .selected
            .map(|i| s.options[i].to_string())
            .or(s.placeholder)
            .unwrap_or_default();
        let for_outline = skin.clone();
        let face = container(
            LogicalRow::new(direction)
                .spacing(skin.gap)
                .align_y(Alignment::Center)
                .push(skin.label(label))
                .push(space().width(Length::Fixed(12.0)))
                .push(skin.mark(Mark::Chevron)),
        )
        .padding(skin.padding)
        .style(move |theme: &Theme| container::Style {
            border: outline(for_outline.colors(theme, enabled), 4.0),
            ..container::Style::default()
        });
        // The control's own padding is the sides' only, and the outline sits inside it: the outline's
        // start edge is where every other control's mark starts, in a column of the five.
        let mut trigger = skin.control(face).padding(Padding {
            top: 0.0,
            bottom: 0.0,
            ..skin.padding
        });
        if let Some(on_open_change) = &s.on_open_change {
            let publish = Rc::clone(on_open_change);
            trigger = trigger.on_press_with(move || publish(!open));
            if !open {
                let publish = Rc::clone(on_open_change);
                trigger = trigger.on_key(move |key, mods| {
                    (mods.is_empty() && matches!(key, Key::Named(Named::ArrowDown)))
                        .then(|| publish(true))
                });
            }
        }
        if let Some(id) = s.id {
            trigger = trigger.id(id);
        }

        // ---- the dropdown, when open ----
        let dropdown = (open && enabled).then(|| {
            let reach = skin.geometry_ring_offset();
            let items = s.options.iter().enumerate().map(|(i, option)| {
                let chosen = s.selected == Some(i);
                let for_tick = skin.clone();
                let tick: Element<'a, Message> = if chosen {
                    container(skin.mark(Mark::Tick))
                        .style(move |theme: &Theme| container::Style {
                            text_color: Some(for_tick.colors(theme, true).accent),
                            ..container::Style::default()
                        })
                        .into()
                } else {
                    space().into()
                };
                let content = LogicalRow::new(direction)
                    .spacing(skin.gap)
                    .align_y(Alignment::Center)
                    .push(skin.square(1.0, tick))
                    .push(skin.label(option.to_string()));
                skin.control(content)
                    .on_press((s.on_select)(option.clone()))
                    .into()
            });
            let items: Vec<Element<'a, Message>> = items.collect();
            // The items hug the start edge, as a menu's do.
            let start = match direction {
                LayoutDirection::Ltr => Alignment::Start,
                LayoutDirection::Rtl => Alignment::End,
            };
            let card: Element<'a, Message> = container(
                scrollable(container(column(items).align_x(start)).padding(reach))
                    .style(scrollbar_style),
            )
            .style(dropdown_card)
            .into();
            card
        });

        let anchored = MenuAnchor::new(
            trigger,
            dropdown,
            direction == LayoutDirection::Rtl,
            skin.geometry_ring_offset(),
        );
        let close = s.on_open_change.as_ref().map(|f| f(false));
        OnEscape::maybe(anchored, close.filter(|_| open)).into()
    }
}

impl Skin {
    /// How far outside a control its ring is drawn.
    fn geometry_ring_offset(&self) -> f32 {
        self.geometry.ring_offset
    }
}
