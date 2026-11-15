//! Card and container style functions for Snora Design tokens.
//!
//! Three semantic variants:
//!
//! * `card_surface` — standard card on a `surface` background; the default
//!   card style.
//! * `card_raised` — elevated card on a `surface_raised` background with a
//!   drop shadow, for popovers or prominent panels.
//! * `card_selected` — card with an `accent`-colored border indicating
//!   selection.
//!
//! # iced 0.14 limitation
//!
//! `iced::widget::container` has **no interaction `Status`** in iced 0.14;
//! the style closure receives only `&Theme`. These functions therefore take
//! only `&Tokens`, not a status. Focus rings are not expressible here.

use iced::{Border, Color, Shadow, widget::container};
use snora_design::Tokens;

use super::color::to_iced_color;

/// Standard card on a `surface` background.
///
/// Use for most cards, panels, and content containers that sit on the
/// application background.
#[must_use]
pub fn card_surface(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(to_iced_color(tokens.palette.border))
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// The shadow's alpha, and the share of it that reaches the pixel just outside a raised card's edge.
///
/// A blurred shadow is about half its alpha at the edge of the shape it is cast from, and the 2 px drop adds a
/// little under the bottom edge. **Measured on the drawn card: 0.49 of the alpha, on both renderers** (white
/// reads `#f0f0f0` there); 0.55 keeps margin.
const RAISED_SHADOW_ALPHA: f32 = 0.12;
const SHADOW_AT_THE_EDGE: f32 = 0.55;

/// The border a raised card draws (RFC-135): the `border` role, firmed toward the text just far enough to hold 3:1
/// against **each neutral surface darkened by the card's own shadow**, which is what lies beside the border's
/// bottom edge. Where the role already holds that (four of the six presets), it is the role, to the float.
///
/// The `border` role holds 3:1 against the surfaces as they are (`Palette::usages`); the shadow darkens the page by
/// about 6% right at the edge, and on `light` and `light_aaa` that took it to 2.75:1. The other way to fix it, a
/// weaker shadow (alpha 0.12 to 0.037 on those two presets), would have taken the shadow nearly away; the border
/// moves less.
fn raised_border(tokens: &Tokens) -> Color {
    let p = &tokens.palette;
    let role = to_iced_color(p.border);
    let text = to_iced_color(p.text_primary);
    let under_shadow = 1.0 - RAISED_SHADOW_ALPHA * SHADOW_AT_THE_EDGE;
    let surfaces = [p.background, p.surface, p.surface_raised].map(|c| {
        snora_design::Color::rgb(c.r * under_shadow, c.g * under_shadow, c.b * under_shadow)
    });
    let mix = |share: f32| Color {
        r: role.r + (text.r - role.r) * share,
        g: role.g + (text.g - role.g) * share,
        b: role.b + (text.b - role.b) * share,
        a: 1.0,
    };
    (0..=100)
        .map(|k| mix(k as f32 / 100.0))
        .find(|&c| {
            surfaces.iter().all(|&s| {
                snora_design::contrast::contrast_ratio(snora_design::Color::rgb(c.r, c.g, c.b), s)
                    >= 3.0
            })
        })
        .unwrap_or(text)
}

/// Elevated card on a `surface_raised` background with a soft drop shadow.
///
/// Use for popovers, floating panels, or cards that visually float above the
/// surrounding content.
///
/// **The border holds 3:1 against the page under the card's own shadow** (RFC-135). It is the `border` role where
/// that holds, and that role firmed toward the text where it does not (`light` and `light_aaa`: 7% toward the
/// text). A surface that draws this card without its shadow (the dialog) takes the plain role.
#[must_use]
pub fn card_raised(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface_raised).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(raised_border(tokens))
            .width(1.0),
        shadow: Shadow {
            color: Color {
                a: RAISED_SHADOW_ALPHA,
                ..Color::BLACK
            },
            offset: iced::Vector { x: 0.0, y: 2.0 },
            blur_radius: 8.0,
        },
        snap: true,
    }
}

/// Card with an `accent`-colored border indicating the selected/active state.
///
/// Use when multiple cards can be selected and one needs to be visually
/// distinguished as the active choice.
#[must_use]
pub fn card_selected(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(to_iced_color(tokens.palette.accent))
            .width(2.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::Tokens;

    // RFC-118: every preset, from the one public source. A preset that
    // passed `snora-design`'s own suite and then failed a widget-level
    // style assertion is the obvious way for a new preset to go wrong,
    // and a hand-kept list is how one gets missed.
    fn all_presets() -> Vec<Tokens> {
        Tokens::presets().into_iter().map(|(_, t)| t).collect()
    }

    #[test]
    fn all_card_styles_produce_background_for_all_presets() {
        for t in all_presets() {
            assert!(
                card_surface(&t).background.is_some(),
                "card_surface no background"
            );
            assert!(
                card_raised(&t).background.is_some(),
                "card_raised no background"
            );
            assert!(
                card_selected(&t).background.is_some(),
                "card_selected no background"
            );
        }
    }

    #[test]
    fn card_selected_has_wider_border_than_surface() {
        let t = Tokens::light();
        assert!(
            card_selected(&t).border.width > card_surface(&t).border.width,
            "card_selected border should be wider than card_surface"
        );
    }

    #[test]
    fn card_raised_has_shadow_offset() {
        let t = Tokens::dark();
        let s = card_raised(&t);
        assert!(
            s.shadow.blur_radius > 0.0,
            "card_raised should have a shadow"
        );
        // The offset the test is named for (0.63 audit): the shadow is cast down, not sideways, and by
        // 2 px; it was asserting only the blur.
        assert_eq!(
            (s.shadow.offset.x, s.shadow.offset.y),
            (0.0, 2.0),
            "card_raised casts its shadow 2 px down"
        );
        // And `card_surface` casts none, so the raised card is the one that rises.
        assert_eq!(card_surface(&t).shadow.blur_radius, 0.0);
    }

    fn lum(c: Color) -> f64 {
        let f = |v: f32| {
            let v = f64::from(v);
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
    }

    /// WCAG's ratio, written out here.
    fn ratio(a: Color, b: Color) -> f32 {
        let (la, lb) = (lum(a), lum(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        ((hi + 0.05) / (lo + 0.05)) as f32
    }

    /// **A raised card's border holds 3:1 against the page under the card's own shadow** (RFC-135), on every neutral
    /// surface, in every preset. The shadow is taken at 0.55 of its alpha at the edge (measured 0.49 on the drawn card);
    /// the figures are computed here, not read from the function.
    #[test]
    fn a_raised_cards_border_holds_3_to_1_against_the_page_under_its_shadow() {
        for (name, t) in Tokens::presets() {
            let border = card_raised(&t).border.color;
            let dark = 1.0 - 0.12 * 0.55;
            for (surface_name, s) in [
                ("background", t.palette.background),
                ("surface", t.palette.surface),
                ("surface_raised", t.palette.surface_raised),
            ] {
                let under = Color::from_rgb(s.r * dark, s.g * dark, s.b * dark);
                let r = ratio(border, under);
                assert!(
                    r >= 3.0,
                    "{name}: border {r:.2}:1 against {surface_name} under the shadow"
                );
            }
        }
    }

    /// **Nothing that passes moves** (RFC-135): four of the six presets draw the `border` role exactly; only `light` and
    /// `light_aaa`, where the shadow took it to 2.75:1, firm it toward the text.
    #[test]
    fn only_light_and_light_aaa_firm_the_raised_cards_border() {
        for (name, t) in Tokens::presets() {
            let role = to_iced_color(t.palette.border);
            let drawn = card_raised(&t).border.color;
            if name == "light" || name == "light_aaa" {
                assert!(
                    lum(drawn) < lum(role),
                    "{name}: firmed toward the (dark) text"
                );
                assert!(
                    ratio(drawn, role) < 1.2,
                    "{name}: by a small step ({:.2})",
                    ratio(drawn, role)
                );
            } else {
                assert_eq!(drawn, role, "{name}: the plain role, to the float");
            }
        }
    }
}
