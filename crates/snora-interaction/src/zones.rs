//! F6 moves focus between the four zones of the frame (RFC-126).
//!
//! # The rules
//!
//! 1. **F6 and Shift+F6 move focus to the first control of the next and the previous zone**, counted
//!    from the zone that holds focus **when the key is pressed**. The order is Header, SideBar, Body,
//!    Footer, wrapping. It is logical, and does not mirror under RTL.
//! 2. **A zone with no control that can take focus is skipped**, whether it is absent or holds none.
//! 3. **With focus in no zone** (nothing focused, a toast, a dialog's control, an inline context
//!    menu): F6 goes to the first zone that has a control, and Shift+F6 to the last.
//! 4. **An open menu's dropdown belongs to its trigger's zone**, for finding where focus is. **F6 lands
//!    only on a zone's own controls, never inside an open menu.**
//! 5. **While the page is covered, F6 does nothing**: under a dialog or a sheet, a menu with its
//!    backdrop, or an open `app_header` dropdown with a close message.
//! 6. **F6 never presses anything.** Held, it keeps moving, one zone per repeat, as Tab does.
//!
//! # How
//!
//! [`zone_focus`] is one window-wide operation in two passes, like `sequential_focus`:
//!
//! 1. **Look.** Walk everything an operation reaches (the tree, then any overlay) and note, for each
//!    focus target, **which zone it is in** (the innermost of the four region containers it sits in)
//!    and whether it holds focus. An open menu's dropdown is an overlay, outside the header's
//!    container, so the anchor marks it ([`DropdownMark`]) and it is counted in **its trigger's
//!    zone** (the zone the anchor's [`OpenMenu`] mark was seen in), but only for finding where focus
//!    is, never as somewhere to land.
//! 2. **Act.** Focus the first control of the next (or previous) zone that has one, counted from the
//!    zone that holds focus now; or do nothing.
//!
//! Nothing is remembered between key presses, so nothing can go stale.
//!
//! **What a covered page contributes is `Inert`'s** (rule 5): it hides a covered layer's focus
//! targets from every operation, so under a dialog, a sheet or a menu with its backdrop no zone has
//! a control, and there is nowhere to go. The one thing `Inert` cannot do is hide the dropdown's own
//! items (they are the cover's), which is why they are never a place to land (rule 4).
//!
//! Public because `snora` builds it (`snora::keyboard::focus_zone`); not for applications.

use iced_core::Rectangle;
use iced_core::widget::Id;
use iced_core::widget::operation::{self, Focusable, Operation, Outcome};
use std::any::Any;

use crate::menu_anchor::{DropdownMark, OpenMenu};

/// One focus target, as the first pass sees it.
struct Seen {
    /// Its zone (an index into the four regions), if it is in one.
    zone: Option<usize>,
    focused: bool,
    /// Somewhere F6 may land. A dropdown's items are not.
    landing: bool,
}

/// What the first pass decided: the index (among all focus targets, in the order they are visited)
/// to focus, or `None` for "nothing moves".
struct Plan {
    target: Option<usize>,
}

struct Look {
    regions: [Id; 4],
    forward: bool,
    /// The region the container just announced is, if it is one, until its `traverse`.
    pending: Option<usize>,
    stack: Vec<Option<usize>>,
    seen: Vec<Seen>,
    /// The zone an open menu's trigger is in.
    menu_zone: Option<usize>,
    in_dropdown: usize,
}

impl Look {
    fn zone_now(&self) -> Option<usize> {
        self.stack.iter().rev().find_map(|z| *z)
    }
}

impl Operation<Plan> for Look {
    fn container(&mut self, id: Option<&Id>, _bounds: Rectangle) {
        self.pending = id.and_then(|id| self.regions.iter().position(|r| r == id));
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Plan>)) {
        self.stack.push(self.pending.take());
        operate(self);
        self.stack.pop();
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        let in_dropdown = self.in_dropdown > 0;
        self.seen.push(Seen {
            zone: if in_dropdown {
                self.menu_zone
            } else {
                self.zone_now()
            },
            focused: state.is_focused(),
            landing: !in_dropdown,
        });
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if state.is::<OpenMenu>() {
            self.menu_zone = self.zone_now();
        } else if let Some(mark) = state.downcast_ref::<DropdownMark>() {
            if mark.enter {
                self.in_dropdown += 1;
            } else {
                self.in_dropdown = self.in_dropdown.saturating_sub(1);
            }
        }
    }

    fn finish(&self) -> Outcome<Plan> {
        // The first control that can be landed on, in each zone.
        let mut first: [Option<usize>; 4] = [None; 4];
        for (i, s) in self.seen.iter().enumerate() {
            if let (true, Some(z)) = (s.landing, s.zone) {
                first[z].get_or_insert(i);
            }
        }
        let held = self.seen.iter().find(|s| s.focused).and_then(|s| s.zone);
        let target = match held {
            // From the zone that holds focus: the next one with a control, never this one.
            Some(from) => (1..4)
                .map(|step| {
                    if self.forward {
                        (from + step) % 4
                    } else {
                        (from + 4 - step) % 4
                    }
                })
                .find_map(|z| first[z]),
            // In no zone (nothing focused, a toast, a dialog's control).
            None if self.forward => first.iter().find_map(|f| *f),
            None => first.iter().rev().find_map(|f| *f),
        };
        Outcome::Some(Plan { target })
    }
}

/// Second pass: focus the target and take focus from every other control. With no target, nothing.
struct Act {
    target: Option<usize>,
    current: usize,
}

impl<T> Operation<T> for Act {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if let Some(target) = self.target {
            if self.current == target {
                state.focus();
            } else {
                state.unfocus();
            }
        }
        self.current += 1;
    }
}

/// Moves focus to the first control of the next zone (`forward`) or the previous one, from the zone
/// that holds focus when it runs. `regions` are the container ids of the header, the sidebar, the body
/// and the footer, in that (logical) order. Run it as a task: `snora::keyboard::focus_zone` does.
pub fn zone_focus<T>(regions: [&'static str; 4], forward: bool) -> impl Operation<T>
where
    T: Send + 'static,
{
    operation::then(
        Look {
            regions: regions.map(Id::new),
            forward,
            pending: None,
            stack: Vec::new(),
            seen: Vec::new(),
            menu_zone: None,
            in_dropdown: 0,
        },
        |plan: Plan| Act {
            target: plan.target,
            current: 0,
        },
    )
}
