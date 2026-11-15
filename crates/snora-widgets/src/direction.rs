//! Direction-aware layout helpers.
//!
//! These helpers let widget authors write layouts in terms of **logical
//! start / end** rather than physical left / right. Pass the application's
//! [`LayoutDirection`] and the helper resolves the placement for you.
//!
//! # The rule: the order is logical, and the positions mirror
//!
//! A [`LogicalRow`] holds its children in **reading order**, under both
//! directions. Under [`LayoutDirection::Ltr`] they are placed from the left
//! edge; under [`LayoutDirection::Rtl`] from the right edge, so the first child
//! is the rightmost one. Nothing moves in the widget tree when the direction
//! changes. So:
//!
//! - **a direction flip keeps every widget's state**, focus and scroll offsets
//!   included (a tree that is reordered per direction loses them, because iced
//!   matches state to a widget by its position in the tree);
//! - **Tab follows reading order**, start to end, under both directions: right
//!   to left on screen under RTL.
//!
//! # Why these exist
//!
//! Without helpers, a widget that wants to be ABDD-compliant has to write
//! a match on [`LayoutDirection`] every time it builds a `row!`. That
//! pattern quickly becomes noisy and error-prone — it is easy to forget
//! one occurrence and ship an RTL bug. These helpers centralise the
//! decision.
//!
//! # Migrating from `Row` (0.63)
//!
//! [`row_dir`] and [`row_dir_three`] used to return an `iced::widget::Row` whose
//! children were **reversed** under RTL. They return a [`LogicalRow`] now. Code
//! that passes the result on as an element, or uses [`push`](LogicalRow::push),
//! [`spacing`](LogicalRow::spacing), [`align_y`](LogicalRow::align_y),
//! [`width`](LogicalRow::width), [`height`](LogicalRow::height) or
//! [`padding`](LogicalRow::padding), compiles unchanged. Code that names `Row`,
//! or calls any other `Row` method, no longer compiles: a method such as `wrap`,
//! `clip` or `extend` has no mirrored meaning, and one that half-works under RTL
//! would be the silent failure this type exists to avoid.

use iced::{Element, Length, Padding, Pixels, alignment, widget::Row};

use snora_core::LayoutDirection;
use snora_interaction::MirroredRow;

/// A horizontal row whose children are in **logical order** and are placed
/// from the start edge: left under [`LayoutDirection::Ltr`], right under
/// [`LayoutDirection::Rtl`].
///
/// The children stay in reading order in the widget tree under both
/// directions, so a direction change keeps every widget's state and Tab goes
/// start to end (see the [module documentation](self)).
///
/// It offers the builders a mirrored row has a meaning for. It is **not** an
/// `iced::widget::Row`: `wrap`, `clip` and the rest of `Row`'s builders are
/// deliberately not here.
///
/// Turn it into an element with `.into()` (or pass it where an
/// `impl Into<Element>` is taken).
#[must_use = "a row does nothing until it is turned into an element"]
pub struct LogicalRow<'a, Message> {
    row: Row<'a, Message>,
    direction: LayoutDirection,
}

impl<'a, Message: 'a> LogicalRow<'a, Message> {
    /// An empty row that places its children for `direction`.
    pub fn new(direction: LayoutDirection) -> Self {
        Self {
            row: Row::new(),
            direction,
        }
    }

    /// Adds a child **after** the ones already there, in logical order: it is
    /// placed to their right under LTR and to their left under RTL.
    pub fn push(mut self, child: impl Into<Element<'a, Message>>) -> Self {
        self.row = self.row.push(child);
        self
    }

    /// The space between children.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.row = self.row.spacing(amount);
        self
    }

    /// The vertical alignment of the children.
    pub fn align_y(mut self, align: impl Into<alignment::Vertical>) -> Self {
        self.row = self.row.align_y(align);
        self
    }

    /// The width of the row. A row that is `Length::Fill` wide places its
    /// children from the start edge of the space it fills.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.row = self.row.width(width);
        self
    }

    /// The height of the row.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.row = self.row.height(height);
        self
    }

    /// The padding of the row. It is logical too: a left padding under LTR is
    /// the same padding on the right under RTL, since the whole row is
    /// reflected.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.row = self.row.padding(padding);
        self
    }
}

impl<'a, Message: 'a> From<LogicalRow<'a, Message>> for Element<'a, Message> {
    fn from(row: LogicalRow<'a, Message>) -> Self {
        MirroredRow::new(row.row, row.direction == LayoutDirection::Rtl).into()
    }
}

/// A two-slot horizontal row in logical order: `start`, then `end`.
///
/// `start` is placed at the start edge (left under [`LayoutDirection::Ltr`],
/// right under [`LayoutDirection::Rtl`]) and `end` after it. **The order in the
/// tree is the same under both directions** (`start`, `end`), and only the
/// positions mirror: Tab visits `start` first under both, and a change of
/// direction keeps the state of both children.
///
/// The returned [`LogicalRow`] can be customised with the builders it offers.
pub fn row_dir<'a, Message>(
    direction: LayoutDirection,
    start: impl Into<Element<'a, Message>>,
    end: impl Into<Element<'a, Message>>,
) -> LogicalRow<'a, Message>
where
    Message: 'a,
{
    LogicalRow::new(direction).push(start).push(end)
}

/// A three-slot horizontal row in logical order: `start`, `center`, `end`.
///
/// The order in the tree is `start`, `center`, `end` under both directions, and
/// only the positions mirror (see [`row_dir`]). `center` sits between the two,
/// as it did; its position follows the others, so under RTL it is left of
/// `start` and right of `end`.
///
/// This is the same as three children pushed in order: it fills nothing by
/// itself, so give `center` a `Length::Fill` width if it should take the space
/// between.
pub fn row_dir_three<'a, Message>(
    direction: LayoutDirection,
    start: impl Into<Element<'a, Message>>,
    center: impl Into<Element<'a, Message>>,
    end: impl Into<Element<'a, Message>>,
) -> LogicalRow<'a, Message>
where
    Message: 'a,
{
    LogicalRow::new(direction)
        .push(start)
        .push(center)
        .push(end)
}
