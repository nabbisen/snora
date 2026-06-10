//! Keyboard dismissal helpers.
//!
//! Snora does not own application shortcut routing, but it provides a
//! small helper for the most common overlay-dismissal pattern: pressing
//! `Escape` to close menus or modals. See
//! [overlay interaction semantics](https://docs.snora.dev/reference/overlay-interaction-semantics.html)
//! Laws 7 and 8 for the normative policy.

use iced::keyboard::{Key, key::Named};

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
/// ```rust,ignore
/// // In your subscription:
/// fn subscription(&self) -> Subscription<Message> {
///     iced::keyboard::on_key_press(|key, _mods| {
///         Some(Message::KeyPressed(key))
///     })
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
}
