//! Tab after a click carries on from the clicked control (RFC-121 R-2).
//!
//! # The rule
//!
//! > After a click on a control, **Tab moves to the control after it**, and
//! > Shift+Tab to the one before it. After a press anywhere else, Tab starts
//! > from the top.
//!
//! A click does not focus a control (RFC-121 R-1), but the pointer user who
//! then reaches for the keyboard should not start again from the top. Browsers
//! meet this without focusing: a click sets a **starting point for Tab**, and
//! the next Tab moves from it (HTML's "sequential focus navigation starting
//! point"). This is that.
//!
//! # How
//!
//! A click marks the control in its **own state** (and a press anywhere else
//! clears the mark, as it clears focus). The runtime's own `focus_next` knows
//! nothing of marks, so `snora::keyboard::traverse` runs [`sequential_focus`]
//! instead: the same two passes (count, then act), except that **when nothing
//! has focus** it starts from the marked control rather than from the top. When
//! something has focus, it behaves exactly as `focus_next` and
//! `focus_previous` do: Tab past the last control leaves the group, with no wrap.
//!
//! The mark is reported to an operation as a [`custom`] value keyed by the
//! control's `Id`, so a control whose focus target is hidden (a covered layer,
//! RFC-119) does not take part: its mark names no target.
//!
//! [`custom`]: iced_core::widget::Operation::custom

use iced_core::Rectangle;
use iced_core::widget::Id;
use iced_core::widget::operation::{self, Focusable, Operation, Outcome};
use std::any::Any;

/// What the first pass learned.
struct Survey {
    forward: bool,
    total: usize,
    focused: Option<usize>,
    marked: Option<usize>,
}

/// First pass: count the focus targets, and find the focused one and the
/// marked one.
struct Look {
    forward: bool,
    ids: Vec<Option<Id>>,
    focused: Option<usize>,
    marked_ids: Vec<Id>,
}

impl Operation<Survey> for Look {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Survey>)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() && self.focused.is_none() {
            self.focused = Some(self.ids.len());
        }
        self.ids.push(id.cloned());
    }

    fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let (Some(id), Some(true)) = (id, state.downcast_ref::<bool>().copied()) {
            self.marked_ids.push(id.clone());
        }
    }

    fn finish(&self) -> Outcome<Survey> {
        let marked = self
            .ids
            .iter()
            .position(|id| id.as_ref().is_some_and(|id| self.marked_ids.contains(id)));
        Outcome::Some(Survey {
            forward: self.forward,
            total: self.ids.len(),
            focused: self.focused,
            marked,
        })
    }
}

/// Second pass: take focus from the focused control and give it to the target,
/// and clear every mark (the keyboard has taken over).
struct Act {
    focused: Option<usize>,
    target: Option<usize>,
    current: usize,
}

impl<T> Operation<T> for Act {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if Some(self.current) == self.focused {
            state.unfocus();
        }
        if Some(self.current) == self.target {
            state.focus();
        }
        self.current += 1;
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let Some(mark) = state.downcast_mut::<bool>() {
            *mark = false;
        }
    }
}

/// Where focus goes, from what the first pass found.
///
/// A focused control wins (this is `focus_next` / `focus_previous`, which do not
/// wrap); otherwise a marked one is the starting point; otherwise the top (or,
/// backwards, the bottom).
fn target(survey: &Survey) -> Option<usize> {
    let Survey {
        forward,
        total,
        focused,
        marked,
    } = *survey;
    match (focused.or(marked), forward) {
        (Some(from), true) => (from + 1 < total).then_some(from + 1),
        (Some(from), false) => from.checked_sub(1),
        (None, true) => (total > 0).then_some(0),
        (None, false) => total.checked_sub(1),
    }
}

/// Moves focus one control forward or back, **starting from the control last
/// clicked** when nothing has focus (RFC-121 R-2). Run it where `focus_next`
/// and `focus_previous` would be: `snora::keyboard::traverse` does.
pub fn sequential_focus<T>(forward: bool) -> impl Operation<T>
where
    T: Send + 'static,
{
    operation::then(
        Look {
            forward,
            ids: Vec::new(),
            focused: None,
            marked_ids: Vec::new(),
        },
        |survey| Act {
            focused: survey.focused,
            target: target(&survey),
            current: 0,
        },
    )
}
