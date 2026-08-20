//! Text style helpers for Snora Design tokens.
//!
//! These helpers derive `iced::Pixels` sizes and `iced::widget::text::LineHeight`
//! values from a [`Tokens`] typography scale, avoiding magic numbers — and
//! hand-built `LineHeight`s — in application view code.
//!
//! Applying line-height (or not) is still the application's own call for its
//! own view code. For snora's **own** prefab widgets, RFC-068 Q-2 ruled on
//! this per widget, by whether the text can wrap: short single-line labels
//! (tab bar, sidebar, buttons, chips) will **not** adopt line-height — `label`
//! at 1.2 is already tighter than iced's own 1.3 default, and line-height does
//! nothing for the readability of a single line, so applying it would only
//! shrink the label for no legibility gain. Widgets that can render wrapping
//! prose (notice bodies, dialog bodies, card content) remain an open,
//! separate decision — there `body` at 1.4 is looser than the default, so the
//! calculus reverses. What this module adds is the helper for the
//! application-side call: no longer reaching through two struct fields and
//! constructing the iced enum by hand — `body_line_height(&tokens)` sits
//! beside `body_size(&tokens)`, mirrored in name, order, and shape, for all
//! six roles.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_style::text;
//! use iced::widget::text as iced_text;
//!
//! let tokens = Tokens::light();
//! let heading = iced_text("Settings")
//!     .size(text::heading_size(&tokens))
//!     .line_height(text::heading_line_height(&tokens));
//! # let _: iced::widget::Text<'_, iced::Theme, iced::Renderer> = heading;
//! ```

use iced::Pixels;
use iced::widget::text::LineHeight;
use snora_design::Tokens;

/// Returns the `body` text size as [`Pixels`].
#[must_use]
pub fn body_size(tokens: &Tokens) -> Pixels {
    tokens.typography.body.size.into()
}

/// Returns the `body` line-height as a relative [`LineHeight`].
#[must_use]
pub fn body_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.body.line_height)
}

/// Returns the `body_small` text size as [`Pixels`].
#[must_use]
pub fn body_small_size(tokens: &Tokens) -> Pixels {
    tokens.typography.body_small.size.into()
}

/// Returns the `body_small` line-height as a relative [`LineHeight`].
#[must_use]
pub fn body_small_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.body_small.line_height)
}

/// Returns the `label` text size as [`Pixels`].
#[must_use]
pub fn label_size(tokens: &Tokens) -> Pixels {
    tokens.typography.label.size.into()
}

/// Returns the `label` line-height as a relative [`LineHeight`].
#[must_use]
pub fn label_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.label.line_height)
}

/// Returns the `title` text size as [`Pixels`].
#[must_use]
pub fn title_size(tokens: &Tokens) -> Pixels {
    tokens.typography.title.size.into()
}

/// Returns the `title` line-height as a relative [`LineHeight`].
///
/// **`title`'s value (`1.3`) is iced 0.14's own default line-height**
/// (`impl Default for LineHeight` returns `Relative(1.3)`,
/// `iced_core-0.14.0/src/text.rs:215-219`; every `iced::widget::text`
/// that never calls `.line_height()` already renders at this value).
/// Calling this helper is therefore harmless but has **no observable
/// effect on any surface** — it restates what the renderer already
/// does (RFC-070). Kept for symmetry with the other five roles' helpers
/// (the two-axis contract below requires one per role), not because
/// omitting it would change anything.
#[must_use]
pub fn title_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.title.line_height)
}

/// Returns the `heading` text size as [`Pixels`].
#[must_use]
pub fn heading_size(tokens: &Tokens) -> Pixels {
    tokens.typography.heading.size.into()
}

/// Returns the `heading` line-height as a relative [`LineHeight`].
#[must_use]
pub fn heading_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.heading.line_height)
}

/// Returns the `display` text size as [`Pixels`].
#[must_use]
pub fn display_size(tokens: &Tokens) -> Pixels {
    tokens.typography.display.size.into()
}

/// Returns the `display` line-height as a relative [`LineHeight`].
#[must_use]
pub fn display_line_height(tokens: &Tokens) -> LineHeight {
    LineHeight::Relative(tokens.typography.display.line_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::{TextRole, Tokens, Typography};

    /// The size/line-height helper contract (RFC-068), exhaustive on two
    /// independent axes at once. Neither destructuring pattern below may
    /// gain a `..` — that would silently readmit the exact defect this
    /// test exists to close on the axis it covers.
    ///
    /// **Role axis**: `Typography { body, body_small, label, title,
    /// heading, display }` is exhaustive. A seventh role added to
    /// `Typography` fails to compile here (`E0027`) until it is listed
    /// with both its helpers — a snora developer cannot add a role and
    /// forget to tool it.
    ///
    /// **Field axis**: `TextRole { size, line_height }` is exhaustive
    /// *inside the loop*, re-destructured for every role. A third field
    /// added to `TextRole` fails to compile here until it is either
    /// tooled or explicitly declined — this is the axis that would have
    /// caught RFC-068's own defect class (a token field growing while the
    /// bridge silently does not follow), on the *next* occurrence; it
    /// could not catch the present one, since both fields have existed
    /// since v0.20 and this test is written after the fact.
    ///
    /// Both `assert_eq!`s are load-bearing, not ceremony: six
    /// near-identical one-line helpers is exactly the shape a copy-paste
    /// mistake survives silently — `title_line_height` returning `body`'s
    /// multiplier would pass any "the helper exists and returns *a*
    /// value" check, and is not visible by reading six similar lines.
    /// Naming the role in the failure message is what makes a future
    /// failure locate itself.
    ///
    /// This also replaces the previous `sizes_are_positive_and_monotonic`,
    /// which built its own hand-written six-element array — the RFC-063
    /// hand-maintained-list shape, sitting in the exact file this RFC
    /// edits. A seventh role would not have broken it. Monotonicity is
    /// still asserted (a real ordering property, not coverage), now
    /// driven from the same exhaustive bindings — **not** a second
    /// hand-written array (review R-1): each loop row below carries its
    /// own **visual rank**, which is not `Typography`'s field-declaration
    /// order (`body` is 16.0, `body_small` is 14.0 — declaration order
    /// and visual order disagree on those two), so a seventh role is
    /// forced into both the equality/sanity checks *and* the
    /// monotonicity check by the same loop row, with no second list to
    /// forget to update.
    #[test]
    fn size_and_line_height_helpers_match_their_own_role_exhaustively() {
        let t = Tokens::light();
        let Typography {
            body,
            body_small,
            label,
            title,
            heading,
            display,
        } = t.typography;

        // `rank` is the role's position in ascending visual size order:
        // body_small(0) <= label(1) <= body(2) <= title(3) <= heading(4)
        // <= display(5).
        let mut sizes_by_rank: [Option<f32>; 6] = [None; 6];

        for (name, rank, role, size, line_height) in [
            ("body", 2, body, body_size(&t), body_line_height(&t)),
            (
                "body_small",
                0,
                body_small,
                body_small_size(&t),
                body_small_line_height(&t),
            ),
            ("label", 1, label, label_size(&t), label_line_height(&t)),
            ("title", 3, title, title_size(&t), title_line_height(&t)),
            (
                "heading",
                4,
                heading,
                heading_size(&t),
                heading_line_height(&t),
            ),
            (
                "display",
                5,
                display,
                display_size(&t),
                display_line_height(&t),
            ),
        ] {
            let TextRole {
                size: want_size,
                line_height: want_line_height,
            } = role;

            assert_eq!(size.0, want_size, "{name}_size returns the wrong role");
            assert_eq!(
                line_height,
                LineHeight::Relative(want_line_height),
                "{name}_line_height returns the wrong role"
            );
            assert!(
                size.0.is_finite() && size.0 > 0.0,
                "{name}: non-positive or non-finite size {size:?}"
            );
            // Sanity bound, not an accessibility threshold: below 1.0 the
            // lines of a wrapped paragraph overlap. WCAG 2.1 SC 1.4.12
            // requires content survive a *user* setting line-height to
            // 1.5, not that a design system ship 1.5 — no leading floor
            // is asserted here or anywhere in this crate (RFC-068 §5).
            assert!(
                want_line_height > 1.0,
                "{name}: line_height {want_line_height} <= 1.0 — lines would overlap"
            );

            assert!(
                sizes_by_rank[rank].is_none(),
                "{name}: rank {rank} already used by another role — ranks must be a \
                 permutation of 0..6, one per role, not a coincidence"
            );
            sizes_by_rank[rank] = Some(size.0);
        }

        let sizes_in_visual_order: [f32; 6] = sizes_by_rank
            .map(|s| s.expect("every rank 0..6 must be claimed by exactly one role above"));
        assert!(
            sizes_in_visual_order.windows(2).all(|w| w[0] <= w[1]),
            "sizes are not monotonically non-decreasing in visual order: {sizes_in_visual_order:?}"
        );
    }
}
