//! RFC-093's channel register for the engine's own toast surface, **as RFC-143 corrected it.**
//!
//! **The channel is no longer colour alone.** RFC-093 recorded that a toast's intent changed
//! colour and nothing else, and ruled that no cue would be added. RFC-143 (2026-11-28, the
//! owner) reversed that for WCAG 1.4.1: an intent now changes its colours **and its mark**, a
//! different shape for each of the five. This register therefore asserts both halves, and each
//! in both directions:
//!
//! - **colour:** [`toast_style`] and [`close_button_style`] still vary by intent only through
//!   colour-bearing fields (the two tests that were here before), so a cue is never carried by
//!   a border width, a shadow or a radius that nobody drew a mark for;
//! - **the mark:** [`mark_kind`] gives each intent a mark of its own
//!   (`every_intent_takes_a_mark_of_its_own`): if the mark stopped depending on the intent, the
//!   channel would be colour alone again, and this fails. What each mark *looks like* is
//!   `tests/status_marks.rs`'s, which reads the drawn pixels.
//!
//! **What this proves and what it does not.** Same scope note as
//! `crates/snora-widgets/src/design/channel_register.rs`, which this
//! module mirrors for `snora::toast`: WCAG 1.4.1 is a property of
//! everything rendered on a surface, not of any colour pair, and no
//! test that only inspects style structs reaches it. What *is*
//! mechanically checkable, and all this module claims: [`ToastIntent`]
//! changes [`toast_style`] and [`close_button_style`]'s output **only**
//! through colour-bearing fields, never through border width, shadow,
//! or any other shape.
//!
//! Kept as a sibling of `contrast_tests.rs` rather than folded into it —
//! contrast and channel-count are different questions (a pair can be
//! high-contrast and still be the only channel), and this project's
//! withdrawal was specifically about the latter.
//!
//! Exhaustive on `intent` via [`intent_label`] (no wildcard arm) — see
//! `contrast_tests.rs`'s own module doc for [`ALL_INTENTS`]'s hand-listed
//! caveat, which applies here identically.
//!
//! # Canonicalization is unconditional, on purpose
//!
//! **Reviewer-required (R-1, 2026-09-02).** The version of this module
//! that shipped in the first review round canonicalized only whichever
//! colour-bearing field happened to vary *today* per function. The
//! reviewer perturbed `toast_style` to vary **border colour** by
//! intent (width, radius, everything else held constant — a
//! colour-only change) and the register reported a false positive: the
//! un-normalized `border.color` field differed and nothing else did,
//! but that function's canonicalizer never touched `border.color`.
//! [`canonicalize_container`] and [`canonicalize_button`] now normalize
//! **every** colour-bearing field on their struct unconditionally —
//! `background`, `text_color`, `border.color`, `shadow.color` for both
//! `container::Style` and `button::Style` — so a colour-only change can
//! no longer be misreported, and every canonicalizer in this module
//! goes through one of these two, so they cannot drift apart again. See
//! the sibling module's identical section for the full incident.

use iced::widget::{button, container};
use iced::{Background, Color, Theme};

use snora_core::ToastIntent;

use super::{close_button_style, mark_kind, toast_style};

const ALL_INTENTS: [ToastIntent; 5] = [
    ToastIntent::Debug,
    ToastIntent::Info,
    ToastIntent::Success,
    ToastIntent::Warning,
    ToastIntent::Error,
];

/// Exhaustive on `intent` — see module doc.
fn intent_label(intent: ToastIntent) -> &'static str {
    match intent {
        ToastIntent::Debug => "Debug",
        ToastIntent::Info => "Info",
        ToastIntent::Success => "Success",
        ToastIntent::Warning => "Warning",
        ToastIntent::Error => "Error",
    }
}

fn theme_contexts() -> [(&'static str, Theme); 2] {
    [("stock Light", Theme::Light), ("stock Dark", Theme::Dark)]
}

/// A placeholder colour distinct from any real palette colour, swapped
/// in for every colour-bearing field before comparing styles.
const PLACEHOLDER: Color = Color::WHITE;

/// A `Background::Color` becomes the placeholder; any other variant
/// passes through unchanged — see the sibling module's identical
/// helper for why that's deliberate, not an oversight.
fn canonicalize_background(bg: Background) -> Background {
    match bg {
        Background::Color(_) => Background::Color(PLACEHOLDER),
        other => other,
    }
}

/// Normalizes every colour-bearing field of a [`container::Style`]:
/// `background`, `text_color`, `border.color`, `shadow.color`.
fn canonicalize_container(style: container::Style) -> container::Style {
    container::Style {
        background: style.background.map(canonicalize_background),
        text_color: style.text_color.map(|_| PLACEHOLDER),
        border: iced::Border {
            color: PLACEHOLDER,
            ..style.border
        },
        shadow: iced::Shadow {
            color: PLACEHOLDER,
            ..style.shadow
        },
        ..style
    }
}

/// Normalizes every colour-bearing field of a [`button::Style`]:
/// `background`, `text_color`, `border.color`, `shadow.color`.
fn canonicalize_button(style: button::Style) -> button::Style {
    button::Style {
        background: style.background.map(canonicalize_background),
        text_color: PLACEHOLDER,
        border: iced::Border {
            color: PLACEHOLDER,
            ..style.border
        },
        shadow: iced::Shadow {
            color: PLACEHOLDER,
            ..style.shadow
        },
        ..style
    }
}

fn canonical_toast_style(theme: &Theme, intent: ToastIntent) -> container::Style {
    canonicalize_container(toast_style(theme, intent, None))
}

fn canonical_close_button_style(theme: &Theme, intent: ToastIntent) -> button::Style {
    canonicalize_button(close_button_style(
        theme,
        intent,
        button::Status::Active,
        None,
    ))
}

/// Asserts that `canonicalize(theme, intent)` produces the same struct
/// for every [`ToastIntent`], both stock themes, collecting every
/// mismatch before failing — mirrors `contrast_tests.rs`'s
/// failure-collection style.
fn assert_colour_only<T: PartialEq + std::fmt::Debug>(
    surface: &str,
    canonicalize: impl Fn(&Theme, ToastIntent) -> T,
) {
    let mut failures = Vec::new();
    for (theme_name, theme) in theme_contexts() {
        let reference = canonicalize(&theme, ALL_INTENTS[0]);
        for intent in ALL_INTENTS {
            let actual = canonicalize(&theme, intent);
            if actual != reference {
                failures.push(format!(
                    "{surface} / {theme_name} / {}: varies by more than colour\n  {actual:?}\n  != reference {reference:?}",
                    intent_label(intent),
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} channel-register failure(s):\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

#[test]
fn toast_surface_varies_by_colour_only() {
    assert_colour_only("toast surface", canonical_toast_style);
}

#[test]
fn toast_dismiss_mark_varies_by_colour_only() {
    assert_colour_only("toast dismiss mark", canonical_close_button_style);
}

/// **The mark is the second channel** (RFC-143): every intent takes a mark that no other intent takes.
/// Were it one mark for all, a toast would again say its intent by colour alone.
#[test]
fn every_intent_takes_a_mark_of_its_own() {
    let marks: Vec<_> = ALL_INTENTS
        .iter()
        .map(|i| (intent_label(*i), mark_kind(*i)))
        .collect();
    let mut failures = Vec::new();
    for (a, (name_a, mark_a)) in marks.iter().enumerate() {
        for (name_b, mark_b) in marks.iter().skip(a + 1) {
            if mark_a == mark_b {
                failures.push(format!(
                    "{name_a} and {name_b} take the same mark, {mark_a:?}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} channel-register failure(s): the intent is colour alone again\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}
