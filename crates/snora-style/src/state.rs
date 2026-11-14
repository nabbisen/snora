//! The one rule for a hovered or pressed control's colours (RFC-129).
//!
//! > **A hovered or pressed control never takes its label under its palette's text level**
//! > ([`Palette::text_contrast`]: 4.5:1 or 7:1), **on any neutral surface it is drawn on**
//! > (`background`, `surface`, `surface_raised`). **It still draws visibly differently from rest**
//! > (ΔE76 of at least 3.0). Disabled is exempt.
//!
//! Every variant's state moves **one** colour, its *knob*: the fill of `primary`, `danger` and the
//! chips; the label (and the border drawn in it) of `secondary` and `ghost`. Each has a shift it
//! has always made, its `today` (hovered lightens `primary`'s fill, pressed darkens it, and so on).
//! [`shifted`] keeps that shift when it holds the rule, and otherwise picks another from a fixed,
//! finite list. So nothing that already held the rule moves.
//!
//! # The candidates, in order
//!
//! 1. `today`;
//! 2. the opposite of `today`;
//! 3. then larger steps of 0.02, up to 0.50, `today`'s direction first, then the opposite.
//!
//! The first that holds the rule is taken. **The search is total.** When none holds it, which only
//! an application's own colours can cause (every preset is swept), the state takes, in this order:
//!
//! 1. the candidate with the **highest contrast** among those still visibly different from rest;
//! 2. the candidate **most different** from rest, if none is visibly different.
//!
//! It is never the resting colour, and it never panics. The same colours always give the same
//! result.
//!
//! # Why a style function can only use the worst surface
//!
//! A style function does not know what is behind its control: `ghost` returns a transparent
//! fill, and the page decides the rest. So one shift has to hold on **all three** neutral
//! surfaces, and the contrast it is held to is the lowest of the three.
//!
//! # Cost
//!
//! A state whose `today` holds the rule (most of them) costs one evaluation, three contrast
//! ratios and three Lab differences. The longest search is 51 evaluations, and only a state that
//! needs it pays it.

use iced::Color;
use snora_design::{Palette, contrast::contrast_ratio};

/// What a state moves: the one colour of a control that a shift changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knob {
    /// The fill (`primary`, `danger` and the chips).
    Fill,
    /// The label (`secondary` and `ghost`).
    Label,
}

/// A control's label and fill.
///
/// The fill may be translucent (a tint over the page); it is composited over each neutral surface
/// before it is measured. The label is measured as opaque, as the contrast suites do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pair {
    /// The label's colour.
    pub label: Color,
    /// The fill behind it.
    pub fill: Color,
}

/// Candidates are `1 / 50 = 0.02` apart. Counted as whole steps and divided, so 0.10 is 0.1 and not
/// 0.099999994.
const STEPS_PER_UNIT: f32 = 50.0;
/// The largest candidate, in steps: 25 of them, 0.50 added to every channel.
const MOST_STEPS: u32 = 25;
/// The smallest ΔE76 between rest and a state that still reads as a change (RFC-129 Q-3): a
/// little over a just-noticeable step (2.3), and under the smallest step any state made before
/// the rule (3.53).
const VISIBLE: f32 = 3.0;

/// Adds `amount` to every channel (positive lightens, negative darkens), clamped to `0..=1`.
pub(crate) fn shade(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).clamp(0.0, 1.0),
        g: (color.g + amount).clamp(0.0, 1.0),
        b: (color.b + amount).clamp(0.0, 1.0),
        a: color.a,
    }
}

fn over(fg: Color, bg: Color) -> Color {
    Color {
        r: fg.a * fg.r + (1.0 - fg.a) * bg.r,
        g: fg.a * fg.g + (1.0 - fg.a) * bg.g,
        b: fg.a * fg.b + (1.0 - fg.a) * bg.b,
        a: 1.0,
    }
}

fn design(c: Color) -> snora_design::Color {
    snora_design::Color::rgb(c.r, c.g, c.b)
}

/// CIE L*a*b* (D65), for the size of a step.
fn lab(c: Color) -> [f32; 3] {
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
    let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
    let f = |t: f32| {
        if t > 0.008856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// ΔE76: the distance between two colours in Lab.
fn delta_e(a: Color, b: Color) -> f32 {
    let (p, q) = (lab(a), lab(b));
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
}

/// What one candidate measures: the lowest label contrast over the three neutral surfaces, and the
/// smallest step from rest over them.
#[derive(Clone, Copy)]
struct Measure {
    contrast: f32,
    step: f32,
}

fn measure(surfaces: &[Color; 3], rest: Pair, now: Pair) -> Measure {
    let mut contrast = f32::MAX;
    let mut step = f32::MAX;
    for &surface in surfaces {
        let (rest_fill, fill) = (over(rest.fill, surface), over(now.fill, surface));
        contrast = contrast.min(contrast_ratio(design(now.label), design(fill)));
        step = step.min(delta_e(rest_fill, fill).max(delta_e(rest.label, now.label)));
    }
    Measure { contrast, step }
}

/// The candidate amounts for a `today` shift, in the order they are tried.
fn candidates(today: f32) -> impl Iterator<Item = f32> {
    let first_larger = (today.abs() * STEPS_PER_UNIT + 1e-3).floor() as u32 + 1;
    let sign = if today < 0.0 { -1.0 } else { 1.0 };
    let steps = (first_larger..=MOST_STEPS).flat_map(move |k| {
        let amount = k as f32 / STEPS_PER_UNIT;
        [sign * amount, -sign * amount]
    });
    std::iter::once(today)
        .chain((today != 0.0).then_some(-today))
        .chain(steps)
}

/// A state's colours: `state` with its knob shifted by the first candidate that holds the rule.
///
/// * `palette` supplies the three neutral surfaces and [`Palette::text_contrast`], the level.
/// * `knob` is the colour a state moves, and `today` the amount it has always moved it by
///   (positive lightens).
/// * `rest` is the control at rest, for the step. `state` is the control in this state **before**
///   the shift: its knob is the colour the shift starts from.
///
/// Returns `state` with the knob shifted. `secondary` draws its border in the label's colour, so
/// it reads the returned label.
///
/// ```
/// use iced::Color;
/// use snora_design::Tokens;
/// use snora_style::state::{Knob, Pair, shifted};
///
/// let t = Tokens::light();
/// let c = |c: snora_design::Color| Color::from_rgba(c.r, c.g, c.b, c.a);
/// let rest = Pair { label: c(t.palette.accent_text), fill: c(t.palette.accent) };
/// // The shift primary has always made when hovered: lighter.
/// let hovered = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
/// assert_ne!(hovered.fill, rest.fill, "a hovered control draws differently from rest");
/// assert_eq!(hovered.label, rest.label, "the label is not the knob");
/// ```
#[must_use]
pub fn shifted(palette: &Palette, knob: Knob, today: f32, rest: Pair, state: Pair) -> Pair {
    let surfaces = [palette.background, palette.surface, palette.surface_raised]
        .map(|c| Color::from_rgb(c.r, c.g, c.b));
    let level = palette.text_contrast.ratio();
    let apply = |amount: f32| match knob {
        Knob::Fill => Pair {
            fill: shade(state.fill, amount),
            ..state
        },
        Knob::Label => Pair {
            label: shade(state.label, amount),
            ..state
        },
    };

    // The best candidate that is visibly different, and the one most different, for when none
    // holds the rule.
    let mut best: Option<(f32, Pair)> = None;
    let mut most_different: Option<(f32, Pair)> = None;
    for amount in candidates(today) {
        let now = apply(amount);
        let m = measure(&surfaces, rest, now);
        if m.step >= VISIBLE {
            if m.contrast >= level {
                return now;
            }
            if best.is_none_or(|(c, _)| m.contrast > c) {
                best = Some((m.contrast, now));
            }
        }
        if most_different.is_none_or(|(s, _)| m.step > s) {
            most_different = Some((m.step, now));
        }
    }
    // `candidates` is never empty, so `most_different` is always set.
    best.or(most_different).map_or(state, |(_, pair)| pair)
}

#[cfg(test)]
mod tests;
