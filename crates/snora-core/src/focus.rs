//! Frame-level keyboard zone navigation — pure, iced-free decision
//! vocabulary (RFC-060).
//!
//! Snora owns the frame (the four skeleton slots [`crate::AppLayout`]
//! composes); applications own what is inside a pane. This module
//! supplies snora's half of keyboard navigation between those slots:
//! given the current zone, a cycle direction, and which optional slots
//! are populated, which zone is next.
//!
//! # What this is not
//!
//! * **Not a key binding.** Snora does not claim Tab or Shift+Tab — Tab
//!   already means "next control" to iced and to every application with
//!   a form. `next_zone` takes a [`Cycle`] direction, not a key; the
//!   recommended binding is **F6 / Shift+F6**, via `snora::keyboard`'s
//!   companion helper.
//! * **Not event capture.** No subscription is installed by this crate
//!   or by `snora`. The application wires `iced::keyboard::listen()`
//!   itself and calls this pure function — the same shape as
//!   `snora::keyboard::dismiss_on_escape`.
//! * **Not modal focus trapping.** `next_zone` reports when cycling is
//!   suspended because a modal is open; it does not enumerate or bound
//!   the modal's contents. Trapping is a separate, staged decision — see
//!   `docs/src/contributing/design-decisions.md`.
//! * **Not state.** The current zone lives in the application's own
//!   state, alongside `toasts` and the overlay flags — this module is
//!   pure *of* that state, not a holder of it.

/// The four skeleton-level navigation zones, in logical cycle order.
///
/// Order is **`Header → SideBar → Body → Footer`**, wrapping — and it is
/// *logical* order, not visual: under RTL the sidebar renders on the
/// opposite physical edge, but it is still the start-edge rail
/// immediately following the header, so this order needs no
/// direction-dependent mirroring (unlike `ToastPosition`'s anchor
/// corner). That is a deliberate ABDD decision, not an omission.
///
/// Tabs and breadcrumbs are **not** zones — [`crate::Tab`] and
/// [`crate::Crumb`] are content an application places *inside* a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusZone {
    /// The header slot (`AppLayout::header`).
    Header,
    /// The side navigation rail (`AppLayout::side_bar`).
    SideBar,
    /// The main content area (`AppLayout::body`). Always present — it is
    /// the one slot `AppLayout` requires.
    Body,
    /// The footer / status bar slot (`AppLayout::footer`).
    Footer,
}

/// Direction to cycle [`FocusZone`]s in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cycle {
    /// Next zone in logical order.
    Forward,
    /// Previous zone in logical order.
    Backward,
}

/// Which optional skeleton slots the current layout has populated.
///
/// `body` is required by [`crate::AppLayout`] and is always present, so
/// it has no field here — it cannot be expressed as absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ZonePresence {
    /// Whether `AppLayout::header` is `Some`.
    pub header: bool,
    /// Whether `AppLayout::side_bar` is `Some`.
    pub side_bar: bool,
    /// Whether `AppLayout::footer` is `Some`.
    pub footer: bool,
}

impl ZonePresence {
    /// All optional slots absent — the body-only degenerate case every
    /// `AppLayout::new` application starts from.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// All optional slots present.
    #[must_use]
    pub fn all() -> Self {
        Self {
            header: true,
            side_bar: true,
            footer: true,
        }
    }

    /// Set whether the header slot is present.
    #[must_use]
    pub fn header(mut self, present: bool) -> Self {
        self.header = present;
        self
    }

    /// Set whether the side-bar slot is present.
    #[must_use]
    pub fn side_bar(mut self, present: bool) -> Self {
        self.side_bar = present;
        self
    }

    /// Set whether the footer slot is present.
    #[must_use]
    pub fn footer(mut self, present: bool) -> Self {
        self.footer = present;
        self
    }

    fn is_present(self, zone: FocusZone) -> bool {
        match zone {
            FocusZone::Header => self.header,
            FocusZone::SideBar => self.side_bar,
            FocusZone::Body => true,
            FocusZone::Footer => self.footer,
        }
    }
}

const ORDER: [FocusZone; 4] = [
    FocusZone::Header,
    FocusZone::SideBar,
    FocusZone::Body,
    FocusZone::Footer,
];

/// The next zone in logical cycle order, or `None` if cycling is
/// suspended.
///
/// Absent optional zones are skipped. In the body-only degenerate case
/// (`ZonePresence::none()`), the only present zone is `Body`, so the
/// result is always `Some(FocusZone::Body)` — cycling has nowhere else
/// to go, which is the correct behaviour rather than a special case.
///
/// **Overlay containment**, mirroring
/// `snora::keyboard::dismiss_on_escape`'s modal-before-menu precedence
/// exactly — same two flags, same priority:
/// * `has_modal == true` — cycling is **suspended** (`None`), regardless
///   of `has_menu`. Focus belongs inside the modal, whose contents are
///   an application-supplied `Node` this function cannot enumerate; it
///   reports the suspension rather than silently returning `Body`.
/// * `has_modal == false && has_menu == true` — cycling proceeds
///   **unaffected**. Menus are light-weight, dismissible on outside
///   click, and do not own focus, so `has_menu` never changes the
///   result — it is accepted only to mirror `dismiss_on_escape`'s shape
///   and to make that non-effect directly testable.
///
/// # Example
///
/// ```
/// use snora_core::focus::{Cycle, FocusZone, ZonePresence, next_zone};
///
/// // Body-only layout: cycling always lands back on Body.
/// let result = next_zone(FocusZone::Body, Cycle::Forward, ZonePresence::none(), false, false);
/// assert_eq!(result, Some(FocusZone::Body));
///
/// // A modal open suspends cycling, even with a full layout.
/// let result = next_zone(FocusZone::Header, Cycle::Forward, ZonePresence::all(), true, false);
/// assert_eq!(result, None);
///
/// // A menu alone does not affect cycling.
/// let result = next_zone(FocusZone::Header, Cycle::Forward, ZonePresence::all(), false, true);
/// assert_eq!(result, Some(FocusZone::SideBar));
/// ```
#[must_use]
pub fn next_zone(
    current: FocusZone,
    cycle: Cycle,
    present: ZonePresence,
    has_modal: bool,
    has_menu: bool,
) -> Option<FocusZone> {
    // Deliberately does not affect the result — see the doc comment's
    // containment section. Accepted only to mirror `dismiss_on_escape`'s
    // shape and to make "menu alone is unaffected" directly testable.
    let _ = has_menu;

    if has_modal {
        return None;
    }

    // Exhaustive rather than a search over ORDER: if a fifth FocusZone is
    // ever added and not added to ORDER, this fails to compile instead of
    // panicking at runtime inside a pure function.
    let start = match current {
        FocusZone::Header => 0,
        FocusZone::SideBar => 1,
        FocusZone::Body => 2,
        FocusZone::Footer => 3,
    };
    let len = ORDER.len();

    (1..=len)
        .map(|step| match cycle {
            Cycle::Forward => ORDER[(start + step) % len],
            Cycle::Backward => ORDER[(start + len - step) % len],
        })
        .find(|&zone| present.is_present(zone))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `next_zone`'s exhaustive index match must agree with `ORDER`. The
    /// match is exhaustive so a new variant fails to compile, but nothing
    /// otherwise stops the two from drifting apart.
    #[test]
    fn index_match_agrees_with_order() {
        for (i, &zone) in ORDER.iter().enumerate() {
            let via_match = match zone {
                FocusZone::Header => 0,
                FocusZone::SideBar => 1,
                FocusZone::Body => 2,
                FocusZone::Footer => 3,
            };
            assert_eq!(via_match, i, "{zone:?} index disagrees with ORDER");
        }
    }

    #[test]
    fn forward_cycles_through_all_four_zones_in_logical_order() {
        let present = ZonePresence::all();
        assert_eq!(
            next_zone(FocusZone::Header, Cycle::Forward, present, false, false),
            Some(FocusZone::SideBar)
        );
        assert_eq!(
            next_zone(FocusZone::SideBar, Cycle::Forward, present, false, false),
            Some(FocusZone::Body)
        );
        assert_eq!(
            next_zone(FocusZone::Body, Cycle::Forward, present, false, false),
            Some(FocusZone::Footer)
        );
        assert_eq!(
            next_zone(FocusZone::Footer, Cycle::Forward, present, false, false),
            Some(FocusZone::Header)
        );
    }

    #[test]
    fn backward_cycles_through_all_four_zones_in_reverse_logical_order() {
        let present = ZonePresence::all();
        assert_eq!(
            next_zone(FocusZone::Header, Cycle::Backward, present, false, false),
            Some(FocusZone::Footer)
        );
        assert_eq!(
            next_zone(FocusZone::Footer, Cycle::Backward, present, false, false),
            Some(FocusZone::Body)
        );
        assert_eq!(
            next_zone(FocusZone::Body, Cycle::Backward, present, false, false),
            Some(FocusZone::SideBar)
        );
        assert_eq!(
            next_zone(FocusZone::SideBar, Cycle::Backward, present, false, false),
            Some(FocusZone::Header)
        );
    }

    #[test]
    fn forward_wraps_around_from_footer_to_header() {
        assert_eq!(
            next_zone(
                FocusZone::Footer,
                Cycle::Forward,
                ZonePresence::all(),
                false,
                false
            ),
            Some(FocusZone::Header)
        );
    }

    #[test]
    fn backward_wraps_around_from_header_to_footer() {
        assert_eq!(
            next_zone(
                FocusZone::Header,
                Cycle::Backward,
                ZonePresence::all(),
                false,
                false
            ),
            Some(FocusZone::Footer)
        );
    }

    #[test]
    fn body_only_layout_always_lands_on_body() {
        let present = ZonePresence::none();
        for (start, cycle) in [
            (FocusZone::Body, Cycle::Forward),
            (FocusZone::Body, Cycle::Backward),
            (FocusZone::Header, Cycle::Forward),
            (FocusZone::Footer, Cycle::Backward),
        ] {
            assert_eq!(
                next_zone(start, cycle, present, false, false),
                Some(FocusZone::Body),
                "start={start:?} cycle={cycle:?}"
            );
        }
    }

    #[test]
    fn absent_header_is_skipped_going_forward() {
        // header absent, side_bar + footer present.
        let present = ZonePresence::none().side_bar(true).footer(true);
        // From Footer, forward wraps past absent Header straight to SideBar.
        assert_eq!(
            next_zone(FocusZone::Footer, Cycle::Forward, present, false, false),
            Some(FocusZone::SideBar)
        );
    }

    #[test]
    fn absent_footer_is_skipped_going_backward() {
        // footer absent, header + side_bar present.
        let present = ZonePresence::none().header(true).side_bar(true);
        // From Body, backward would land on Footer if present; skips to SideBar.
        assert_eq!(
            next_zone(FocusZone::Body, Cycle::Backward, present, false, false),
            Some(FocusZone::SideBar)
        );
    }

    #[test]
    fn every_combination_of_absent_optional_slots_stays_on_body_or_a_present_zone() {
        for header in [false, true] {
            for side_bar in [false, true] {
                for footer in [false, true] {
                    let present = ZonePresence::none()
                        .header(header)
                        .side_bar(side_bar)
                        .footer(footer);
                    for start in [
                        FocusZone::Header,
                        FocusZone::SideBar,
                        FocusZone::Body,
                        FocusZone::Footer,
                    ] {
                        for cycle in [Cycle::Forward, Cycle::Backward] {
                            let next = next_zone(start, cycle, present, false, false)
                                .expect("Body is always present, so a next zone always exists");
                            assert!(
                                present.is_present(next),
                                "present={present:?} start={start:?} cycle={cycle:?} \
                                 produced absent zone {next:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn modal_open_suspends_cycling() {
        assert_eq!(
            next_zone(
                FocusZone::Header,
                Cycle::Forward,
                ZonePresence::all(),
                true,
                false
            ),
            None
        );
    }

    #[test]
    fn menu_alone_does_not_affect_cycling() {
        assert_eq!(
            next_zone(
                FocusZone::Header,
                Cycle::Forward,
                ZonePresence::all(),
                false,
                true
            ),
            Some(FocusZone::SideBar)
        );
    }

    #[test]
    fn both_modal_and_menu_open_modal_wins() {
        // Mirrors dismiss_on_escape's both_open_modal_takes_priority.
        assert_eq!(
            next_zone(
                FocusZone::Header,
                Cycle::Forward,
                ZonePresence::all(),
                true,
                true
            ),
            None
        );
    }
}
