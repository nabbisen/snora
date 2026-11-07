//! Keyboard dismissal and navigation helpers.
//!
//! Snora does not own application shortcut routing, but it provides
//! small helpers for the two patterns every multi-region application
//! needs: pressing `Escape` to close menus or modals, and cycling
//! between frame-level zones. See
//! [overlay interaction semantics](https://nabbisen.github.io/snora/reference/overlay-interaction-semantics.html)
//! Laws 7 and 8 for the normative policy.

use iced::keyboard::{Key, Modifiers, key::Named};
use snora_core::focus::Cycle;

/// Returns the message to emit when `Escape` is pressed, following the
/// Snora overlay dismissal priority.
///
/// # When the engine already answers Escape (RFC-119)
///
/// **An overlay that closes on a click outside also closes on Escape, with no
/// help from you:** whenever `AppLayout::on_close_menus` or `on_close_modals`
/// is set, the backdrop publishes it on Escape and captures the event, so
/// `iced::keyboard::listen()` never delivers that Escape to you and you do not
/// get a second message. Calling this function as well is harmless: it is never
/// reached for an overlay the engine closed.
///
/// **The one case left to this function** is an overlay that closes on Escape
/// **but not on a click outside**: a modal with no `on_close_modals`. The
/// pointer cannot leave it by clicking outside either (the dim captures the
/// press and does nothing), so Escape is the application's decision.
///
/// Priority: **modal before menu.** If both a modal and a menu are
/// open (a state the overlay semantics docs recommend avoiding), the
/// modal is dismissed first.
///
/// Returns `None` when:
/// - `key` is not `Escape`;
/// - the surface that would be dismissed has no close message (`None`
///   was passed for the relevant sink);
/// - no overlay is open.
///
/// # Example
///
/// `ignore` (RFC-064): a `match` arm fragment plus a `&self` method,
/// neither a standalone compilable item — genuinely partial, not merely
/// unassembled.
///
/// ```rust,ignore
/// // In your subscription:
/// fn subscription(&self) -> Subscription<Message> {
///     let key_sub = iced::keyboard::listen().map(|event| {
///         if let iced::keyboard::Event::KeyPressed { key, .. } = event {
///             Message::KeyPressed(key)
///         } else {
///             Message::NoOp
///         }
///     });
///     Subscription::batch([
///         snora::toast::subscription(&self.toasts, || Message::ToastTick),
///         key_sub,
///     ])
/// }
///
/// // In your update:
/// Message::KeyPressed(key) => {
///     if let Some(msg) = snora::keyboard::dismiss_on_escape(
///         self.show_dialog || self.show_sheet,
///         self.open_menu.is_some(),
///         Some(Message::CloseModals),
///         Some(Message::CloseMenus),
///         key,
///     ) {
///         return self.update(msg);
///     }
/// }
/// ```
pub fn dismiss_on_escape<Message: Clone>(
    has_modal: bool,
    has_menu: bool,
    on_close_modals: Option<Message>,
    on_close_menus: Option<Message>,
    key: Key,
) -> Option<Message> {
    if key != Key::Named(Named::Escape) {
        return None;
    }
    if has_modal {
        return on_close_modals;
    }
    if has_menu {
        return on_close_menus;
    }
    None
}

/// Returns the [`Cycle`] direction for `F6` / `Shift+F6`, snora's
/// recommended binding for frame-level zone navigation
/// ([`crate::focus::next_zone`]).
///
/// **Not the only legitimate binding.** F6 / Shift+F6 is the desktop
/// convention for region cycling (Tab already means "next control," so
/// snora does not claim it — see [`dismiss_on_escape`] for the same
/// non-capture policy applied to `Escape`). An application is free to
/// bind a different key to [`crate::focus::next_zone`] directly;
/// this helper exists so the common case does not require re-deriving
/// which modifier means "backward."
///
/// Returns `None` for any other key, so it composes with
/// [`dismiss_on_escape`] in the same `if let Some(...) = ... else if
/// let Some(...) = ...` chain without either needing to know about the
/// other.
///
/// # Example
///
/// `ignore` (RFC-064): a bare `match` arm fragment referencing
/// application-owned methods (`self.focus_zone`, `self.zone_presence()`,
/// `self.focus_task_for`) — not a standalone item.
///
/// ```rust,ignore
/// // In your update, alongside dismiss_on_escape:
/// Message::KeyPressed(key, modifiers) => {
///     if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
///         let next = snora::focus::next_zone(
///             self.focus_zone,
///             cycle,
///             self.zone_presence(),
///             self.show_dialog || self.show_sheet,
///             self.open_menu.is_some(),
///         );
///         if let Some(zone) = next {
///             self.focus_zone = zone;
///             return self.focus_task_for(zone); // application-owned iced::Task
///         }
///     }
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

#[cfg(test)]
mod tests {
    use super::*;

    const ESC: Key = Key::Named(Named::Escape);
    const ENTER: Key = Key::Named(Named::Enter);

    #[test]
    fn non_escape_key_returns_none() {
        assert_eq!(
            dismiss_on_escape(true, true, Some("close_modal"), Some("close_menu"), ENTER),
            None,
        );
    }

    #[test]
    fn no_overlay_returns_none() {
        assert_eq!(
            dismiss_on_escape(false, false, Some("modal"), Some("menu"), ESC),
            None,
        );
    }

    #[test]
    fn menu_only_returns_close_menus() {
        assert_eq!(
            dismiss_on_escape(false, true, Some("modal"), Some("menu"), ESC),
            Some("menu"),
        );
    }

    #[test]
    fn modal_only_returns_close_modals() {
        assert_eq!(
            dismiss_on_escape(true, false, Some("modal"), Some("menu"), ESC),
            Some("modal"),
        );
    }

    #[test]
    fn both_open_modal_takes_priority() {
        assert_eq!(
            dismiss_on_escape(true, true, Some("modal"), Some("menu"), ESC),
            Some("modal"),
        );
    }

    #[test]
    fn modal_open_but_no_sink_returns_none() {
        assert_eq!(
            dismiss_on_escape(true, false, None::<&str>, Some("menu"), ESC),
            None,
        );
    }

    #[test]
    fn menu_open_but_no_sink_returns_none() {
        assert_eq!(
            dismiss_on_escape(false, true, Some("modal"), None::<&str>, ESC),
            None,
        );
    }

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
/// when an overlay opens is still yours (`widget::Id` and `operate`).
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
    if key != Key::Named(Named::Tab) {
        return None;
    }

    Some(if modifiers.shift() {
        iced::widget::operation::focus_previous()
    } else {
        iced::widget::operation::focus_next()
    })
}

#[cfg(test)]
mod traverse_tests {
    use super::*;

    /// Tab and Shift+Tab are claimed; nothing else is.
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
}
