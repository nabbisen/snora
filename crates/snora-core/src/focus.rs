//! Frame-level keyboard zone navigation: the cycle direction (RFC-060, RFC-126).
//!
//! Snora owns the frame (the four skeleton slots [`crate::AppLayout`] composes); applications own
//! what is inside a pane. Moving between those slots is `snora::keyboard::focus_zone(cycle)`, which
//! reads from the tree when the key is pressed which zone holds focus and which have a control that
//! can take it. This module is the one piece of vocabulary that decision shares with the binding:
//! the [`Cycle`] direction.
//!
//! `FocusZone`, `ZonePresence` and `next_zone` were removed in 0.64.0, as announced in 0.61.0
//! (RFC-126): `next_zone` only returned a zone, from a zone the application held, and could not move
//! focus. See the 0.60 to 0.61 migration guide for the two-line replacement.
//!
//! # What this is not
//!
//! * **Not a key binding.** Snora does not claim Tab or Shift+Tab: Tab already means "next control"
//!   to iced and to every application with a form. The recommended binding is **F6 / Shift+F6**,
//!   through `snora::keyboard`.
//! * **Not event capture.** No subscription is installed by this crate or by `snora`.
//! * **Not modal focus trapping.** A modal's contents are an application-supplied `Node`.

/// Direction to cycle the frame's zones in.
///
/// The order is **header, side bar, body, footer**, wrapping. It is *logical* order, not visual:
/// under RTL the side bar renders on the opposite physical edge, but it is still the start-edge rail
/// that follows the header, so the order needs no direction-dependent mirroring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cycle {
    /// Next zone in logical order.
    Forward,
    /// Previous zone in logical order.
    Backward,
}
