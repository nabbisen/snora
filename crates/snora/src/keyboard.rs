//! Keyboard navigation helpers, and the shortcuts router.
//!
//! **Shortcuts (RFC-116 R-2).** [`Shortcuts`](crate::keyboard::Shortcuts), passed with
//! [`AppLayout::shortcuts`](snora_core::AppLayout::shortcuts), is the one router for
//! the keys that jump: `Ctrl/Cmd + 1…9`, next and previous tab, the breadcrumb's
//! parent. [`Digits`](crate::keyboard::Digits) says where the digits go, and
//! [`Escape`](crate::keyboard::Escape), passed with
//! [`AppLayout::escape`](snora_core::AppLayout::escape), says what Escape does on each
//! overlay. The rules and the bindings are in [`snora_core::shortcuts`].
//! **The macOS bindings follow Apple's conventions but have not been checked on
//! a Mac.**
//!
//! Beyond those, Snora does not own application shortcut routing, but it provides
//! small helpers for the patterns every multi-region application
//! needs: cycling between frame-level zones, and Tab. Escape on an overlay is
//! `AppLayout::escape`, and an overlay of your own, outside `AppLayout`, writes
//! the three-line `match` itself. See
//! [overlay interaction semantics](https://nabbisen.github.io/snora/reference/overlay-interaction-semantics.html)
//! Laws 7 and 8 for the normative policy.

use iced::keyboard::{Key, Modifiers, key::Named};
use snora_core::focus::Cycle;

pub use snora_core::{Digits, Escape, Platform, ShortcutAction, Shortcuts};

/// Returns the [`Cycle`] direction for `F6` / `Shift+F6`, snora's recommended binding for
/// frame-level zone navigation. Pass it to [`focus_zone`], which moves focus.
///
/// **Not the only legitimate binding.** F6 / Shift+F6 is the desktop convention for region
/// cycling (Tab already means "next control," so snora does not claim it). An application that
/// wants another key maps that key to a [`Cycle`] itself and calls [`focus_zone`]; this helper
/// exists so the common case does not require re-deriving which modifier means "backward."
///
/// Returns `None` for any other key, so it composes with the application's own key handling in
/// the same `if let Some(...) = ... else if let Some(...) = ...` chain without either needing to
/// know about the other.
///
/// # Example
///
/// Two lines in your key handler, with no state of your own:
///
/// ```
/// # use iced::keyboard::{Key, Modifiers};
/// # #[derive(Clone)] enum Message {}
/// fn on_key(key: Key, modifiers: Modifiers) -> iced::Task<Message> {
///     if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
///         return snora::keyboard::focus_zone(cycle);
///     }
///     iced::Task::none()
/// }
/// ```
#[must_use]
pub fn cycle_zones(key: Key, modifiers: Modifiers) -> Option<Cycle> {
    if key != Key::Named(Named::F6) {
        return None;
    }
    if modifiers.shift() {
        Some(Cycle::Backward)
    } else {
        Some(Cycle::Forward)
    }
}

/// Moves focus to the first control of the next zone (`Cycle::Forward`) or the previous one
/// (`Cycle::Backward`), counted from **the zone that holds focus when it runs**, read from the
/// tree at that moment (RFC-126). The application routes the key, as it routes Tab
/// ([`traverse`]):
///
/// ```
/// # use iced::keyboard::{Key, Modifiers};
/// # #[derive(Clone)] enum Message {}
/// # fn on_key(key: Key, modifiers: Modifiers) -> iced::Task<Message> {
/// if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
///     return snora::keyboard::focus_zone(cycle);
/// }
/// # iced::Task::none()
/// # }
/// ```
///
/// snora does not answer F6 by itself: an application may already use F6 (a debugger's "step
/// over"), and an upgrade must not take a key away silently. The rule a reader can predict:
/// **snora answers keys inside its own controls and overlays (Enter, Space, the arrows in a menu,
/// Escape). Keys that move between controls (Tab, F6) are routed to it by the application.**
/// Enter and Space, in a control, are the plain keys: a chord with Ctrl, Alt or the logo key is
/// not answered by the control and reaches the application (RFC-130).
///
/// # The rules
///
/// The four zones are the header, the sidebar, the body and the footer of
/// [`AppLayout`](snora_core::AppLayout), in that order, wrapping. The order is **logical**: it does
/// not mirror under RTL.
///
/// 1. **Forward and backward.** F6 goes to the next zone, Shift+F6 to the previous, and lands on
///    the zone's **first control** (a memory of the last control per zone is not kept).
/// 2. **A zone with no control that can take focus is skipped**, whether it is absent or holds
///    none (a footer of plain text).
/// 3. **With focus in no zone** (nothing focused and no click since, a toast, an inline context
///    menu), F6 goes to the first zone that has a control, and Shift+F6 to the last. After a click
///    on a control nothing is focused, but Tab's starting point is that control (RFC-121), so F6
///    counts from **its** zone instead: click in the sidebar, press F6, and focus goes to the body.
///    A press anywhere that is not a control clears the starting point, and so does a move of
///    focus by F6 or Tab. (A dialog's or a sheet's control is rule 5's, not this one's: the page is
///    covered and no zone has a control.)
/// 4. **F6 lands only on a zone's own controls, never inside an open menu.** An open
///    `app_header` dropdown **belongs to its trigger's zone** for finding where focus is: with
///    focus on one of its items, F6 moves on to the next zone. A context menu
///    (`AppLayout::context_menu`) is in **no zone** by comparison: the application opens it at a
///    point, and snora does not know which zone that point means. So F6 from a context menu's
///    control is "focus in no zone" (rule 3). The dropdown differs because it has a trigger, and
///    the trigger's zone is known.
/// 5. **While the page is covered, F6 does nothing:** under a dialog or a sheet, a menu with its
///    backdrop, or an open `app_header` dropdown with a close message. The zones are beneath the
///    cover, and the keyboard does what the pointer can.
/// 6. **F6 never presses anything.** Held, it keeps moving, one zone per repeat, as Tab does.
///
/// The decision is made when the key is pressed, from the tree: nothing is remembered between
/// presses, so a position held by the application cannot go stale, and a focus moved by Tab, a
/// click's starting point or a task is where the next F6 counts from.
///
/// Replaces `snora_core::focus::next_zone` with `FocusZone` and `ZonePresence`, which are
/// deprecated: an application no longer passes a stale answer, and has nothing left to pass.
///
/// [`traverse`]: crate::keyboard::traverse
pub fn focus_zone<Message: 'static>(cycle: Cycle) -> iced::Task<Message> {
    use crate::identifiers::{BODY_REGION, FOOTER_REGION, HEADER_REGION, SIDEBAR_REGION};
    iced_runtime::task::effect(iced_runtime::Action::widget(
        snora_interaction::zone_focus::<()>(
            [HEADER_REGION, SIDEBAR_REGION, BODY_REGION, FOOTER_REGION],
            matches!(cycle, Cycle::Forward),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ESC: Key = Key::Named(Named::Escape);
    const ENTER: Key = Key::Named(Named::Enter);

    const F6: Key = Key::Named(Named::F6);

    #[test]
    fn f6_without_shift_cycles_forward() {
        assert_eq!(cycle_zones(F6, Modifiers::empty()), Some(Cycle::Forward));
    }

    #[test]
    fn shift_f6_cycles_backward() {
        assert_eq!(cycle_zones(F6, Modifiers::SHIFT), Some(Cycle::Backward));
    }

    #[test]
    fn non_f6_key_returns_none_even_with_shift() {
        assert_eq!(cycle_zones(ESC, Modifiers::SHIFT), None);
        assert_eq!(cycle_zones(ENTER, Modifiers::empty()), None);
    }

    #[test]
    fn other_modifiers_alongside_f6_do_not_change_direction() {
        // Ctrl+F6 (no Shift) is still forward; Ctrl+Shift+F6 is still backward.
        assert_eq!(cycle_zones(F6, Modifiers::CTRL), Some(Cycle::Forward));
        assert_eq!(
            cycle_zones(F6, Modifiers::CTRL | Modifiers::SHIFT),
            Some(Cycle::Backward)
        );
    }
}

// ---------------------------------------------------------------------
// Focus traversal (RFC-116 Unit 0)
// ---------------------------------------------------------------------

/// Moves focus with Tab and Shift+Tab.
///
/// # What Tab reaches while an overlay is open (RFC-119)
///
/// **Only what the pointer can press.** Under an open dialog or sheet, and
/// under a menu that has its outside-click backdrop, the page is out of
/// reach: Tab moves between the overlay and the toasts (dialog, then sheet,
/// then toasts), and nothing beneath takes focus or key presses. A menu with no
/// backdrop leaves the page pressable, so Tab reaches it. Where focus starts
/// when an overlay opens is still yours: give the control an id with
/// [`Control::id`](crate::Control::id) and return iced's
/// `iced::widget::operation::focus(id)` from the same `update` that opens it.
///
/// # Why an application has to call this at all
///
/// **iced binds no Tab.** Focus moves only when something runs
/// `iced::widget::operation::focus_next()` or `focus_previous()`
/// (`iced_runtime/src/widget/operation.rs:50,55`), and nothing in iced
/// 0.14 does that for you. So a `control` that registers as a focus
/// target is still unreachable until the application routes the key —
/// which is one line in `update`, and is this function.
///
/// Returns `None` for anything that is not Tab, so an application can
/// hand it every key event and let it decline.
///
/// `ignore` (RFC-064): a bare `match` arm fragment of the application's own
/// `update`, not a standalone item.
///
/// ```rust,ignore
/// // In your update:
/// Message::KeyPressed(key, modifiers) => {
///     if let Some(task) = snora::keyboard::traverse(key, modifiers) {
///         return task;
///     }
///     Task::none()
/// }
/// ```
///
/// # After a click, Tab carries on from the clicked control (RFC-121)
///
/// **A click presses a control and does not focus it**, so no ring appears and
/// no key answers for a control the reader only clicked. But a pointer user who
/// then reaches for the keyboard does not start again from the top: **the next
/// Tab moves to the control after the one clicked, and Shift+Tab to the one
/// before it.** After a press anywhere else, Tab starts from the top. Once
/// anything has focus, Tab moves from that, as it always did. This function runs
/// that operation rather than iced's `focus_next` and `focus_previous`; it needs
/// nothing from you.
///
/// # Tab at the end of a group leaves it
///
/// **Tab on the last focus target does not wrap to the first.** Focus
/// leaves: nothing is focused until the next Tab, which starts again from
/// the first target in the window. That is iced's `focus_next`, and it is
/// right for a window with other controls after the group (a rail, then
/// the page): a wrap would trap a keyboard user inside whichever group
/// they were in. Shift+Tab at the first target behaves the same way, in
/// reverse. Observed in a real window on the `tab` example.
///
/// # What this does not do
///
/// It does not know about snora's zones ([`crate::focus::Cycle`], F6):
/// that cycles between *regions*, this moves between *controls*. The
/// two are different axes and RFC-116 Unit 0 does not join them.
#[must_use]
pub fn traverse<Message: 'static>(key: Key, modifiers: Modifiers) -> Option<iced::Task<Message>> {
    // **Plain Tab and Shift+Tab only** (RFC-116 R-2). `Ctrl+Tab`, `Alt+Tab` and
    // `Cmd+Tab` are not Tab: `Ctrl+Tab` is "next tab" in the shortcuts router, and
    // when the router is silent (under an overlay, or with that route left out)
    // it must not fall through and move focus.
    if key != Key::Named(Named::Tab) || modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }

    // Not iced's `focus_next` / `focus_previous`: the same two passes, except
    // that when nothing has focus they start from the control last clicked
    // (RFC-121 R-2) and not from the top. With something focused they behave
    // exactly as iced's do.
    Some(iced_runtime::task::effect(iced_runtime::Action::widget(
        snora_interaction::sequential_focus::<()>(!modifiers.shift()),
    )))
}

#[cfg(test)]
mod traverse_tests {
    use super::*;

    /// Tab and Shift+Tab are claimed; nothing else is: not with Ctrl, Alt or
    /// Command either (RFC-116 R-2).
    ///
    /// **What this cannot assert**, and why there is a rendered test
    /// for it instead: a `Task` is opaque, so this cannot tell
    /// `focus_next` from `focus_previous`. It asserts only which keys
    /// are claimed. That the two directions actually differ is shown in
    /// `crates/snora/tests/sidebar_keyboard.rs`, by focusing through a
    /// rail and reading which item answers.
    #[test]
    fn only_tab_is_claimed() {
        assert!(traverse::<()>(Key::Named(Named::Tab), Modifiers::empty()).is_some());
        assert!(traverse::<()>(Key::Named(Named::Tab), Modifiers::SHIFT).is_some());
        assert!(traverse::<()>(Key::Named(Named::Enter), Modifiers::empty()).is_none());
        assert!(traverse::<()>(Key::Named(Named::Escape), Modifiers::empty()).is_none());
        assert!(traverse::<()>(Key::Character("a".into()), Modifiers::empty()).is_none());
    }

    /// **`Ctrl+Tab`, `Alt+Tab` and `Cmd+Tab` do not move focus**, with or without
    /// Shift: they were Tab with a modifier, and `Ctrl+Tab` is "next tab".
    #[test]
    fn tab_with_ctrl_alt_or_command_is_not_claimed() {
        let tab = || Key::Named(Named::Tab);
        for modifiers in [
            Modifiers::CTRL,
            Modifiers::ALT,
            Modifiers::LOGO,
            Modifiers::CTRL | Modifiers::SHIFT,
            Modifiers::ALT | Modifiers::SHIFT,
            Modifiers::LOGO | Modifiers::SHIFT,
        ] {
            assert!(
                traverse::<()>(tab(), modifiers).is_none(),
                "Tab with {modifiers:?} was claimed"
            );
        }
    }
}
