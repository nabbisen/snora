//! Direction-aware layout helpers.
//!
//! These helpers let widget authors write layouts in terms of **logical
//! start / end** rather than physical left / right. Pass the application's
//! [`LayoutDirection`] and the helper resolves the order for you.
//!
//! # Why these exist
//!
//! Without helpers, a widget that wants to be ABDD-compliant has to write
//! a match on [`LayoutDirection`] every time it builds a `row!`. That
//! pattern quickly becomes noisy and error-prone — it is easy to forget
//! one occurrence and ship an RTL bug. These helpers centralise the
//! decision.

use iced::{
    Element,
    widget::{Row, row},
};

use snora_core::LayoutDirection;

/// A two-slot horizontal row in logical order.
///
/// Under [`LayoutDirection::Ltr`], `start` is placed first (left) and
/// `end` last (right). Under [`LayoutDirection::Rtl`] the order is
/// reversed. The returned [`Row`] can be customised further with standard
/// iced builder methods.
pub fn row_dir<'a, Message>(
    direction: LayoutDirection,
    start: impl Into<Element<'a, Message>>,
    end: impl Into<Element<'a, Message>>,
) -> Row<'a, Message>
where
    Message: 'a,
{
    match direction {
        LayoutDirection::Ltr => row![start.into(), end.into()],
        LayoutDirection::Rtl => row![end.into(), start.into()],
    }
}

/// A three-slot horizontal row in logical order: `start`, `center`, `end`.
///
/// `center` is position-stable — it does not flip with direction. Only
/// `start` and `end` swap. This matches how a typical app header is
/// organised (title at start, status in the middle, controls at end).
pub fn row_dir_three<'a, Message>(
    direction: LayoutDirection,
    start: impl Into<Element<'a, Message>>,
    center: impl Into<Element<'a, Message>>,
    end: impl Into<Element<'a, Message>>,
) -> Row<'a, Message>
where
    Message: 'a,
{
    match direction {
        LayoutDirection::Ltr => row![start.into(), center.into(), end.into()],
        LayoutDirection::Rtl => row![end.into(), center.into(), start.into()],
    }
}
