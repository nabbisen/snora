//! Per-preset visibility tests for token-derived engine surface styling
//! (RFC-039): the modal dim and the dialog card.
//!
//! - **Dim visibility**: the derived dim, composited over `background`,
//!   must be distinguishable from `background` itself, for all four
//!   presets — including the two clamping-adjacent cases that broke
//!   RFC-038's first attempt (`light`'s pure-white background,
//!   `high_contrast_dark`'s pure-black one).
//! - **Card border distinguishable**: `surface_raised` (the card's fill)
//!   is bitwise identical to `background` in two of the four built-in
//!   presets (`light`, `high_contrast_light`) — a card in those presets
//!   is visible *only* because of its border, exactly the "border-defined,
//!   not shadow-defined" design RFC-039 specifies. So this suite tests
//!   the border's contrast against `background`, not the fill's — testing
//!   fill-vs-background would assert something false by construction in
//!   half the presets.
//! - **Card fidelity**: the card's fill/border/radius match the token
//!   roles RFC-039 specifies exactly (`surface_raised`, `border`,
//!   `radius.lg`) — proving [`dialog_card_style`] is a direct token
//!   mapping, not an invented derivation.
//! - **Card text contrast**: `text_primary` on the card fill meets WCAG
//!   AA, independently re-verified here even though the underlying
//!   values come from [`snora_widgets::design::style::container::card_raised`],
//!   which has its own tests — this is the guarantee for *this* code
//!   path specifically.

use super::*;
use iced::Size;
use iced::widget::Id;
use iced_test::Simulator;
use snora_core::{AppLayout, Dialog};
use snora_design::{Color as SnColor, Tokens, contrast::contrast_ratio};

const AA: f32 = 4.5;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Noop,
}

/// Modest, clearly-nonzero visibility floor for the dim composited over
/// `background`, and for the card border against `background`. Chosen the
/// same way RFC-038's `1.5:1` border floor was: well below WCAG SC
/// 1.4.11's full `3.0` non-text-contrast threshold, but a real,
/// non-trivial bar. Both floors reuse the same constant because both
/// measure the same kind of thing — "is this element visually distinct
/// from the page behind it" — not because the two elements are otherwise
/// related.
const VISIBILITY_FLOOR: f32 = 1.3;

fn named_presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

fn to_sn(c: iced::Color) -> SnColor {
    SnColor::rgba(c.r, c.g, c.b, c.a)
}

/// Alpha-composites `fg` (with its own alpha) over opaque `bg`, matching
/// what the renderer actually draws when the dim layer is painted over
/// the page background.
fn composite(fg: iced::Color, bg: iced::Color) -> iced::Color {
    let a = fg.a;
    iced::Color {
        r: fg.r * a + bg.r * (1.0 - a),
        g: fg.g * a + bg.g * (1.0 - a),
        b: fg.b * a + bg.b * (1.0 - a),
        a: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Dim visibility.
// ---------------------------------------------------------------------------

#[test]
fn dim_visible_against_background_all_presets() {
    for (name, t) in named_presets() {
        let background = to_iced_color(t.palette.background);
        let dim = dim_color(&t);
        let composited = composite(dim, background);
        let r = contrast_ratio(to_sn(composited), to_sn(background));
        assert!(
            r >= VISIBILITY_FLOOR,
            "{name}: dim composited over background contrast {r:.3} < {VISIBILITY_FLOOR} \
             — the modal dim is not visibly distinguishable from the page"
        );
    }
}

#[test]
fn dim_alpha_is_always_dim_alpha_constant() {
    for (name, t) in named_presets() {
        let dim = dim_color(&t);
        assert_eq!(
            dim.a, DIM_ALPHA,
            "{name}: dim_color's alpha must stay fixed at DIM_ALPHA regardless of preset"
        );
    }
}

/// Direct unit test of the pole choice, isolated from any specific
/// preset. `dim_color` must pick the pole *opposite* the background's
/// own darkness — this is the property that makes the derivation safe at
/// both luminance extremes without needing `shift_away_from`'s
/// clamp-fallback: the two poles can never both describe the same input.
#[test]
fn dim_color_picks_the_opposite_pole() {
    let mut white_bg_tokens = Tokens::light();
    white_bg_tokens.palette.background = SnColor::rgb(1.0, 1.0, 1.0);
    let dim = dim_color(&white_bg_tokens);
    assert_eq!(
        (dim.r, dim.g, dim.b),
        (0.0, 0.0, 0.0),
        "a light (white) background must pick the black pole"
    );

    let mut black_bg_tokens = Tokens::high_contrast_dark();
    black_bg_tokens.palette.background = SnColor::rgb(0.0, 0.0, 0.0);
    let dim = dim_color(&black_bg_tokens);
    assert_eq!(
        (dim.r, dim.g, dim.b),
        (1.0, 1.0, 1.0),
        "a dark (black) background must pick the white pole"
    );
}

// ---------------------------------------------------------------------------
// Card border distinguishable from background.
// ---------------------------------------------------------------------------

#[test]
fn card_border_distinguishable_from_background_all_presets() {
    for (name, t) in named_presets() {
        let background = to_iced_color(t.palette.background);
        let card = dialog_card_style(&t);
        let r = contrast_ratio(to_sn(card.style.border.color), to_sn(background));
        assert!(
            r >= VISIBILITY_FLOOR,
            "{name}: card border vs background contrast {r:.3} < {VISIBILITY_FLOOR} \
             — a border this close to the page would make the card's edge invisible"
        );
    }
}

/// `light` and `high_contrast_light` have `surface_raised == background`
/// in the token data itself (both pure white) — pinning that fact
/// directly, since it is *why* the border-distinguishable test above
/// exists instead of a fill-distinguishable one.
#[test]
fn fill_equals_background_in_light_presets_by_token_design() {
    for name in ["light", "high_contrast_light"] {
        let t = if name == "light" {
            Tokens::light()
        } else {
            Tokens::high_contrast_light()
        };
        assert_eq!(
            t.palette.surface_raised, t.palette.background,
            "{name}: this test's premise (fill == background in these two presets) no longer \
             holds — re-evaluate whether card_border_distinguishable_from_background_all_presets \
             is still the right test, or whether a fill-distinguishable test is now possible too"
        );
    }
}

// ---------------------------------------------------------------------------
// Card fidelity: fill/border/radius are the exact token roles RFC-039
// specifies, not an invented derivation.
// ---------------------------------------------------------------------------

#[test]
fn card_style_matches_token_roles_exactly() {
    for (name, t) in named_presets() {
        let card = dialog_card_style(&t);

        assert_eq!(
            card.style.background,
            Some(to_iced_color(t.palette.surface_raised).into()),
            "{name}: card fill must equal surface_raised exactly"
        );
        assert_eq!(
            card.style.border.color,
            to_iced_color(t.palette.border),
            "{name}: card border color must equal the border role exactly"
        );
        assert_eq!(
            card.style.border.radius,
            t.radius.lg.into(),
            "{name}: card border radius must equal radius.lg exactly"
        );
        assert_eq!(
            card.padding, t.spacing.lg,
            "{name}: card padding must equal spacing.lg exactly"
        );
        assert_eq!(
            card.style.shadow,
            iced::Shadow::default(),
            "{name}: card must be border-defined, not shadow-defined (RFC-039) — \
             card_raised's shadow must be zeroed, not inherited"
        );
    }
}

// ---------------------------------------------------------------------------
// Card text contrast.
// ---------------------------------------------------------------------------

#[test]
fn card_text_meets_aa_all_presets() {
    for (name, t) in named_presets() {
        let card = dialog_card_style(&t);
        let text = card
            .style
            .text_color
            .expect("card style must set a text color");
        let fill = to_iced_color(t.palette.surface_raised);
        let r = contrast_ratio(to_sn(text), to_sn(fill));
        assert!(
            r >= AA,
            "{name}: card text contrast {r:.2} < {AA} (WCAG AA)"
        );
    }
}

// ---------------------------------------------------------------------------
// The dialog identifier resolves to the card, not its centring wrapper
// (RFC-049).
// ---------------------------------------------------------------------------

/// The defect RFC-049 exists to catch: before this release,
/// `snora-dialog-card` was attached to the dialog's full-window centring
/// container, not the styled card — "present" in every render, on both
/// paths, but never resolving to the actual card. A presence-only check
/// (`sim.find(id).is_ok()`) cannot distinguish this from a correct fix,
/// because the old, wrong identifier was present too. This test instead
/// asserts a property only the real card has: its bounds are strictly
/// smaller than the window, since it is padded content behind a border,
/// not a container that fills the screen — see `dialog_card_style`
/// above, whose `padding` and `style.border` this element actually
/// applies.
///
/// If this regresses to asserting presence alone, it stops catching the
/// RFC-049 defect class even though it would still pass.
#[test]
fn dialog_card_identifier_resolves_to_the_card_not_the_window() {
    let tokens = Tokens::light();
    let window_size = Size::new(1024.0, 768.0);

    let dialog: Dialog<Element<'_, Msg>, Msg> =
        Dialog::new(iced::widget::text("dialog content").into());
    let layout = AppLayout::new(iced::widget::text("body").into())
        .dialog(dialog)
        .on_close_modals(Msg::Noop);

    let element = render(layout, &tokens);
    let mut sim = Simulator::with_size(iced_test::core::Settings::default(), window_size, element);

    let wrapper_bounds = sim
        .find(Id::new(crate::identifiers::DIALOG))
        .expect("snora-dialog must resolve on the design path")
        .bounds();
    let card_bounds = sim
        .find(Id::new(crate::identifiers::DIALOG_CARD))
        .expect("snora-dialog-card must resolve on the design path")
        .bounds();

    assert!(
        (wrapper_bounds.width - window_size.width).abs() < 1.0
            && (wrapper_bounds.height - window_size.height).abs() < 1.0,
        "sanity check: snora-dialog's centring wrapper is expected to fill the window \
         ({wrapper_bounds:?} vs window {window_size:?}) — if this no longer holds, this \
         test needs a different baseline to compare the card's bounds against"
    );

    assert!(
        card_bounds.width < wrapper_bounds.width && card_bounds.height < wrapper_bounds.height,
        "snora-dialog-card must resolve to bounds strictly smaller than snora-dialog's \
         ({card_bounds:?} vs wrapper {wrapper_bounds:?}) — equal bounds would mean it is \
         still resolving to the full-window wrapper, the RFC-049 defect this test exists \
         to catch"
    );
}
