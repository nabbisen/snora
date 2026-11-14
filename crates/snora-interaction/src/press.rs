//! What counts as a press of a focused control (RFC-130).
//!
//! > **A focused control presses on Enter or Space when neither Ctrl, Alt nor the logo key is held.**
//! > Shift does not count: `Shift+Enter` and `Shift+Space` still press.
//!
//! With Ctrl, Alt or the logo key held, the key is a chord and not a press. The control neither
//! publishes nor captures it, so it reaches snora's router and an application's
//! `keyboard::listen()` (which passes only what nothing captured), as it does when nothing is
//! focused. Before RFC-130 the control took every chord on Enter and Space: an application's
//! `Ctrl+Enter` ("send") stopped working whenever a snora button had focus, and the button pressed
//! instead.
//!
//! **One predicate, two readers.** [`crate::Control`] asks it to decide whether to press, and
//! [`crate::Inert`] asks it to decide whether a key may have removed a control (RFC-120). Neither
//! keeps its own `matches!`: if they drifted apart, a chord that pressed would go unobserved by the
//! removal rule, or the reverse.

use iced::keyboard::{Key, Modifiers, key::Named};

/// Whether `key`, with `modifiers` held, presses a focused control.
pub(crate) fn is_press(key: &Key, modifiers: Modifiers) -> bool {
    matches!(key.as_ref(), Key::Named(Named::Enter | Named::Space))
        && !modifiers.control()
        && !modifiers.alt()
        && !modifiers.logo()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(n: Named) -> Key {
        Key::Named(n)
    }

    /// Enter and Space press, plain and with Shift; Ctrl, Alt and the logo key (alone, and with
    /// Shift) make a chord, which does not; no other key presses.
    #[test]
    fn enter_and_space_press_unless_ctrl_alt_or_the_logo_key_is_held() {
        for key in [Named::Enter, Named::Space] {
            for (mods, presses) in [
                (Modifiers::empty(), true),
                (Modifiers::SHIFT, true),
                (Modifiers::CTRL, false),
                (Modifiers::ALT, false),
                (Modifiers::LOGO, false),
                (Modifiers::CTRL | Modifiers::SHIFT, false),
                (Modifiers::ALT | Modifiers::SHIFT, false),
                (Modifiers::LOGO | Modifiers::SHIFT, false),
                (Modifiers::CTRL | Modifiers::ALT, false),
            ] {
                assert_eq!(
                    is_press(&named(key), mods),
                    presses,
                    "{key:?} with {mods:?}"
                );
            }
        }
        for other in [
            Named::Tab,
            Named::Escape,
            Named::ArrowDown,
            Named::F6,
            Named::Backspace,
        ] {
            assert!(!is_press(&named(other), Modifiers::empty()), "{other:?}");
        }
        assert!(!is_press(&Key::Character(" ".into()), Modifiers::empty()));
    }

    /// **One predicate, not two copies** (RFC-130 R-2). Neither reader names Enter or Space itself:
    /// a `matches!` of its own in either would be the second copy. (Behaviourally the removal
    /// observer cannot tell: it acts only on a control that published, which only `Control` does,
    /// so a wrong copy there would cost a needless pass and change nothing visible. This pins the
    /// structure instead.)
    #[test]
    fn the_control_and_the_removal_observer_share_the_predicate() {
        for (file, source) in [
            ("control.rs", include_str!("control.rs")),
            ("inert.rs", include_str!("inert.rs")),
        ] {
            assert!(
                source.contains("press::is_press("),
                "{file} does not ask the shared predicate"
            );
            let code: String = source
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !code.contains("Named::Enter") && !code.contains("Named::Space"),
                "{file} keeps its own copy of which keys press"
            );
        }
    }
}
