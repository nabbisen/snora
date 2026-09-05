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
