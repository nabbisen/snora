//! Engine toast contrast suite (RFC-086) — the engine's first contrast
//! assertion.
//!
//! Deliberately **not** `snora-design`'s or `snora-widgets`'s contrast
//! math reused: the engine (`snora`) has zero token dependency by design
//! (RFC-021/022), and this RFC's whole boundary is that this stays true
//! — so the WCAG formula below is reimplemented standalone, on
//! `iced::Color`, rather than imported. It is the same published
//! formula (sRGB linearize, `0.2126/0.7152/0.0722` luminance
//! coefficients, `(bright + 0.05) / (dark + 0.05)`) as `snora-design`'s
//! own `contrast::contrast_ratio` — verified independently against the
//! same black/white → 21.0 identity in this module's own tests.
//!
//! # What is derived and what is not
//!
//! [`ALL_INTENTS`] is a hand-listed array — Rust has no reflection over
//! enum variants, the same honest limit RFC-085's suite named for
//! `button::Status`. What *is* exhaustive: [`intent_colors`] (in
//! `../toast.rs`) and [`intent_label`] below both match on
//! [`ToastIntent`] with no wildcard arm, so a sixth variant fails to
//! compile in both places until it is given a pairing and a label —
//! `Palette::usages`' pattern (RFC-063), applied to this enum for the
//! first time.
//!
//! # Scope
//!
//! Both stock `iced::Theme` variants only (`Light`, `Dark`) — no
//! `design`-derived presets. The engine's toasts render on the default
//! path with no features active at all; this is not RFC-085's problem
//! restated; see `toast.rs`'s own module doc and RFC-086 §"Why this is
//! separate from RFC-085".

use iced::widget::button;
use iced::{Color, Theme};

use snora_core::ToastIntent;

use super::{close_button_style, toast_style};

/// WCAG 2.1 SC 1.4.3 normal-text minimum. Sibling constants of the same
/// name/value exist in `snora-design/src/tests.rs` and
/// `snora-widgets/src/contrast_tests.rs`; this one cannot share either
/// (the engine depends on neither crate, by design). Check all three if
/// this value ever changes.
const AA_TEXT: f32 = 4.5;

/// WCAG 2.1 SC 1.4.11 non-text minimum. Same sibling-constant caveat as
/// [`AA_TEXT`].
const NON_TEXT_MIN: f32 = 3.0;

fn linearize_srgb_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(c: Color) -> f32 {
    0.2126 * linearize_srgb_channel(c.r)
        + 0.7152 * linearize_srgb_channel(c.g)
        + 0.0722 * linearize_srgb_channel(c.b)
}

fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let bright = la.max(lb);
    let dark = la.min(lb);
    (bright + 0.05) / (dark + 0.05)
}

/// Alpha-composites `fg` over an opaque `bg`.
///
/// **Added 2026-09-02 (R-2, round 2).** `relative_luminance` reads only
/// `r`/`g`/`b` and never `a` — a `fg` with `a < 1.0` measured directly
/// against `bg` was contrast-checked as if it were fully opaque, which
/// is not what actually renders. A reintroduced alpha fade (the one
/// this suite exists to catch, per `close_button_style`'s own doc
/// comment) passed this suite undetected before this function existed,
/// including at `a = 0.0` — a fully invisible mark. Every contrast
/// check in this module composites through this function first,
/// including [`toast_body_text_meets_aa_all_intents_both_themes`],
/// which sets no alpha today but must not silently stop checking it if
/// that ever changes.
fn composite_over(fg: Color, bg: Color) -> Color {
    let a = fg.a;
    Color {
        r: fg.r * a + bg.r * (1.0 - a),
        g: fg.g * a + bg.g * (1.0 - a),
        b: fg.b * a + bg.b * (1.0 - a),
        a: 1.0,
    }
}

/// Contrast of `fg` against `bg`, compositing `fg`'s own alpha first.
/// The one path every check in this module goes through — see
/// [`composite_over`]'s doc comment for why a direct [`contrast_ratio`]
/// call on a possibly-translucent foreground is the defect this exists
/// to prevent.
fn text_contrast(fg: Color, bg: Color) -> f32 {
    contrast_ratio(composite_over(fg, bg), bg)
}

/// All five [`ToastIntent`] variants. Hand-listed — see module doc.
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

fn toast_background(theme: &Theme, intent: ToastIntent) -> Color {
    match toast_style(theme, intent).background {
        Some(iced::Background::Color(c)) => c,
        other => panic!("toast_style background changed shape: {other:?}"),
    }
}

#[test]
fn contrast_ratio_matches_wcag_reference_identity() {
    let r = contrast_ratio(Color::BLACK, Color::WHITE);
    assert!(
        (r - 21.0).abs() < 0.01,
        "black/white should measure 21.0:1, got {r:.3}"
    );
}

/// Regression for R-2 (round 2, 2026-09-02): a fully transparent
/// foreground must measure as indistinguishable from the background
/// (ratio 1.0), not as if it were fully opaque. Before `composite_over`
/// existed, `contrast_ratio(black, white)` reported 21.0:1 even at
/// `black.a = 0.0` — a fully invisible mark passed every check in this
/// module. This is the exact probe the reviewer ran by hand; kept here
/// so nobody has to re-derive it.
#[test]
fn text_contrast_sees_alpha() {
    let invisible_black = Color {
        a: 0.0,
        ..Color::BLACK
    };
    let r = text_contrast(invisible_black, Color::WHITE);
    assert!(
        (r - 1.0).abs() < 0.01,
        "a fully transparent foreground over white should measure 1.0:1 \
         (indistinguishable from the background), got {r:.3}"
    );
}

/// Every intent's body text against its own actual background, both
/// stock themes. Failures are collected across the full sweep rather
/// than stopping at the first, so a regression states every affected
/// combination at once.
#[test]
fn toast_body_text_meets_aa_all_intents_both_themes() {
    let mut failures = Vec::new();
    for (theme_name, theme) in theme_contexts() {
        for intent in ALL_INTENTS {
            let style = toast_style(&theme, intent);
            let bg = toast_background(&theme, intent);
            let fg = style
                .text_color
                .expect("toast_style always sets text_color");
            let r = text_contrast(fg, bg);
            if r < AA_TEXT {
                failures.push(format!(
                    "{theme_name} / {}: body text {r:.3}:1 < {AA_TEXT} (fg {fg:?} bg {bg:?})",
                    intent_label(intent),
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failing combination(s):\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

/// The dismiss `×`'s own text colour against the toast's actual
/// background — the assertion F-06 exists for. Checked against
/// [`NON_TEXT_MIN`] rather than [`AA_TEXT`] per the Handoff's own
/// acceptance criterion ("`Debug`'s `×` clears the non-text floor").
/// [`close_button_style`] shares [`super::intent_colors`] with the
/// body, so in practice this floor is always cleared with the body
/// text's own margin — kept as its own assertion, not folded into the
/// body-text one, so a future change to the mark's own colour is still
/// checked independently against the floor named for it.
///
/// **Reviewer-required (R-1, 2026-09-02): sweeps all four
/// [`button::Status`] variants, not just [`button::Status::Active`].**
/// The mark's own colour no longer varies by status (the hover/rest
/// alpha fade was removed for the reason `close_button_style`'s own doc
/// comment gives), and today that makes every status equivalent — but
/// that equivalence is exactly the property a future change could
/// silently break, the same defect class RFC-090 exists to close for
/// the release process. Checking only `Active` would not have caught a
/// reintroduced hover-only fade; checking all four does.
#[test]
fn toast_dismiss_mark_meets_non_text_floor_all_intents_both_themes() {
    const ALL_STATUSES: [button::Status; 4] = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];

    let mut failures = Vec::new();
    for (theme_name, theme) in theme_contexts() {
        for intent in ALL_INTENTS {
            let bg = toast_background(&theme, intent);
            for status in ALL_STATUSES {
                let btn = close_button_style(&theme, intent, status);
                let r = text_contrast(btn.text_color, bg);
                if r < NON_TEXT_MIN {
                    failures.push(format!(
                        "{theme_name} / {} / {status:?}: dismiss mark {r:.3}:1 < {NON_TEXT_MIN} \
                         (fg {:?} bg {bg:?})",
                        intent_label(intent),
                        btn.text_color,
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failing combination(s):\n{}",
        failures.len(),
        failures.join("\n"),
    );
}
