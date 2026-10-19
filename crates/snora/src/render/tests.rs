//! The unstyled modal dim (RFC-110).
//!
//! The dim's job is to say "the thing behind this is not what you are
//! interacting with". It can only say that if the dimmed page is
//! distinguishable from the undimmed one, so the pairing that matters is
//! **the page against itself, with and without the dim on top** — not
//! the dim against anything else.
//!
//! Until RFC-110 the unstyled dim was black at 40% alpha, a literal, and
//! nothing asserted it. On a dark theme, black over a nearly-black page
//! changes almost nothing: measured 1.02–1.33:1 across iced's 16 dark
//! themes, against a 3.0 floor. The `design` path had solved this
//! already, by picking the pole opposite the background
//! (`snora_design::surfaces::modal_dim`, RFC-065); this module asserts
//! the unstyled path now follows the same rule.

use iced::{Color, Theme};

use super::{ChromeStyle, UNSTYLED_DIM_ALPHA};

/// WCAG 1.4.11's non-text floor. A fourth in-crate copy of the same
/// number; `scripts/check-wcag-floors.sh` pins how many exist and fails
/// if two disagree.
const NON_TEXT_MIN: f32 = 3.0;

fn linearize(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(c: Color) -> f32 {
    0.2126 * linearize(c.r) + 0.7152 * linearize(c.g) + 0.0722 * linearize(c.b)
}

fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Alpha-composites `fg` over an opaque `bg`. The dim is translucent by
/// definition, so every measurement here goes through this first:
/// reading its contrast directly would measure a colour that is never
/// on screen.
fn composite_over(fg: Color, bg: Color) -> Color {
    Color {
        r: fg.r * fg.a + bg.r * (1.0 - fg.a),
        g: fg.g * fg.a + bg.g * (1.0 - fg.a),
        b: fg.b * fg.a + bg.b * (1.0 - fg.a),
        a: 1.0,
    }
}

#[test]
fn contrast_ratio_matches_wcag_reference_identity() {
    assert!((contrast_ratio(Color::BLACK, Color::WHITE) - 21.0).abs() < 0.01);
}

/// The dim the unstyled path paints on a given theme.
fn unstyled_dim(theme: &Theme) -> Color {
    ChromeStyle::unstyled().dim.resolve(theme)
}

/// **The defect RFC-110 exists for.** Every theme iced ships, dimmed
/// page against undimmed page, at the non-text floor.
#[test]
fn unstyled_dim_separates_the_page_from_itself_on_every_theme() {
    let mut failures = Vec::new();
    for theme in Theme::ALL {
        let page = theme.extended_palette().background.base.color;
        let dimmed = composite_over(unstyled_dim(theme), page);
        let r = contrast_ratio(dimmed, page);
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{theme}: dimmed vs undimmed page {r:.2} < {NON_TEXT_MIN}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} themes do not dim visibly:\n  {}",
        failures.len(),
        Theme::ALL.len(),
        failures.join("\n  "),
    );
}

/// The rule itself, mirroring `design/render/tests.rs`'s
/// `dim_color_picks_the_opposite_pole`: white over a dark page, black
/// over a light one, at the alpha the design path uses.
#[test]
fn unstyled_dim_picks_the_opposite_pole() {
    for theme in Theme::ALL {
        let dim = unstyled_dim(theme);
        let expected = if theme.extended_palette().is_dark {
            Color::WHITE
        } else {
            Color::BLACK
        };
        assert_eq!(
            (dim.r, dim.g, dim.b),
            (expected.r, expected.g, expected.b),
            "{theme}: the dim must be the pole opposite the page",
        );
        assert!(
            (dim.a - UNSTYLED_DIM_ALPHA).abs() < f32::EPSILON,
            "{theme}: the dim's alpha is {}, not {UNSTYLED_DIM_ALPHA}",
            dim.a,
        );
    }
}

/// The measurement the CHANGELOG's before/after ranges come from, and
/// the record of what the literal did. Prints rather than asserts.
#[test]
fn report_dim_ranges() {
    let mut dark: Vec<f32> = Vec::new();
    let mut light: Vec<f32> = Vec::new();
    for theme in Theme::ALL {
        let ep = theme.extended_palette();
        let page = ep.background.base.color;
        let now = contrast_ratio(composite_over(unstyled_dim(theme), page), page);
        let before = contrast_ratio(
            composite_over(Color::from_rgba(0.0, 0.0, 0.0, 0.4), page),
            page,
        );
        println!(
            "dim | {:<26} | dark={:<5} | before {before:.2} | after {now:.2}",
            theme.to_string(),
            ep.is_dark
        );
        if ep.is_dark {
            dark.push(now)
        } else {
            light.push(now)
        }
    }
    let range = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        format!("{:.2}-{:.2} over {} themes", v[0], v[v.len() - 1], v.len())
    };
    println!("dim | dark after:  {}", range(&mut dark));
    println!("dim | light after: {}", range(&mut light));
}
