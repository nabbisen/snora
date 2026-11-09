//! **The readable-colour rule, once** (RFC-122 Unit 1, review 3.4).
//!
//! Until 0.57 this rule lived twice, byte for byte: in `snora::toast` (RFC-111, for
//! the toast's own text) and in `snora_widgets::style` (RFC-114 Unit 3b, for the
//! sidebar's active name and RFC-116's focus ring). Neither crate could call the
//! other's: `snora` depends on `snora-widgets` only through an optional feature, and
//! the toast is not optional. A third caller, the public `snora::control`, would have
//! been a third copy. Both crates already depend on this one, below them, so the rule
//! lives here and each crate calls it.
//!
//! **Not a public API of snora.** Like the rest of this crate it is published for
//! `snora` and `snora-widgets`; an application that wants a contrast ratio has
//! `snora::design::contrast`.
//!
//! # What the rule is
//!
//! Keep the colour the caller prefers wherever it reads against what is behind it
//! ([`AA_TEXT`], 4.5:1), and otherwise take the stronger of black and white
//! ([`stronger_pole`]), which is at least 4.58:1 against any opaque fill by
//! construction. A theme's deliberate choice survives; only a pairing that fails is
//! replaced.

use iced::Color;

/// WCAG 1.4.3 AA for body text.
///
/// One of the copies `scripts/check-wcag-floors.sh` counts, and the only one the
/// runtime rule in this file reads.
pub const AA_TEXT: f32 = 4.5;

/// Relative luminance, per WCAG 2.x.
#[must_use]
pub fn relative_luminance(c: Color) -> f32 {
    fn channel(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

/// WCAG contrast ratio between two opaque colours.
#[must_use]
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Black or white, whichever contrasts more with `fill`.
///
/// **This always clears AA.** For any opaque colour with luminance `L`, black
/// contrasts at `(L + 0.05) / 0.05` and white at `1.05 / (L + 0.05)`; the two are
/// equal at `L = √(0.05 × 1.05) − 0.05`, where both are `√(1.05 / 0.05) ≈ 4.583:1`.
/// Every other luminance makes one of them larger, so the worse case of the better
/// choice is 4.583, above the floor, for every fill a theme can produce.
#[must_use]
pub fn stronger_pole(fill: Color) -> Color {
    if contrast_ratio(Color::BLACK, fill) >= contrast_ratio(Color::WHITE, fill) {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

/// The caller's `preferred` colour when it clears AA against `fill`, and the
/// stronger pole when it does not.
///
/// iced derives each pair from whatever palette the application set, and six of its
/// 22 built-in themes produce a pair under 4.5:1 (worst 2.42 on Kanagawa Dragon). The
/// preference is kept wherever it holds, because it is the one that looks
/// deliberate; the fallback only replaces a pairing that fails.
#[must_use]
pub fn readable_text(fill: Color, preferred: Color) -> Color {
    if contrast_ratio(preferred, fill) >= AA_TEXT {
        preferred
    } else {
        stronger_pole(fill)
    }
}
