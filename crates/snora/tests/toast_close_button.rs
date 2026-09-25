//! Rendered-layout tests for the toast's close button (RFC-101).
//!
//! The close button is **the only interactive control the engine renders
//! itself**, and it was built from literals rather than from tokens:
//! `button(text("×").size(18)).padding([0, 8])`. That padding is zero
//! vertically, so the button's height was the line box — 18 x 1.3 = 23.4 —
//! which is under the 24 x 24 pointer target snora's accessibility
//! checklist mandates (WCAG 2.5.8). snora's height assertions covered
//! token-derived controls, and this one derives from neither.
//!
//! What these tests measure, and why each is shaped the way it is:
//!
//! - **The target** is measured by clicking its corners
//!   ([`common::pointer_target`]), not by reading a rectangle out of the
//!   tree, because what matters is what a pointer can hit.
//! - **Centring** is asserted natural-size-then-centre (RFC-099): a
//!   container that hands its child fixed limits gets a text box that
//!   fills it and a glyph drawn at that box's top-left, so comparing box
//!   centres alone cannot fail.
//! - **The lucide font** must be loaded in this binary or every lucide
//!   measurement here is of a fallback glyph, and the comparison still
//!   passes (RFC-100 §3). `lucide::font_is_loaded` is the control.

#![cfg(feature = "widgets")]

use iced::{Element, Rectangle};
use iced_test::Simulator;

use snora::{AppLayout, Toast, ToastIntent, render};

mod common;
use common::{MIN_TARGET, TOLERANCE, containers, pointer_target, simulator};

/// The width the message column had at 0.50.0: the toast's fixed 340,
/// less its 12px padding on each side, less the 4px row spacing, less the
/// 25px the close button was then. The close button may not take more
/// room than it did, or long messages wrap earlier and the toast grows
/// taller — on a surface whose width is fixed, the button's size and the
/// message's room are the same number seen from two ends.
const MESSAGE_COLUMN_MIN: f32 = 287.0;

/// The close glyph this feature state draws, and the one it must not.
///
/// Test-local, and deliberately not shared with the implementation: an
/// integration test sees only snora's public API, and adding a public item
/// to carry a glyph for a test's convenience is what RFC-099's review
/// declined (RFC-101 Q-3).
#[cfg(feature = "lucide-icons")]
const GLYPH: &str = "\u{e1b2}";
#[cfg(feature = "lucide-icons")]
const NOT_GLYPH: &str = "×";
#[cfg(not(feature = "lucide-icons"))]
const GLYPH: &str = "×";
#[cfg(not(feature = "lucide-icons"))]
const NOT_GLYPH: &str = "\u{e1b2}";

/// The size the engine draws the close glyph at.
const GLYPH_SIZE: f32 = 18.0;

const TOAST_ID: u64 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Body,
    Dismiss(u64),
}

/// Two intents, because `close_button_style` resolves its colour per
/// intent and the button's geometry must not follow it.
fn intents() -> [(&'static str, ToastIntent); 2] {
    [
        ("info", ToastIntent::Info),
        ("warning", ToastIntent::Warning),
    ]
}

fn body() -> Element<'static, Msg> {
    iced::widget::button(iced::widget::text("body"))
        .on_press(Msg::Body)
        .into()
}

/// A toast rendered through the public engine path, as an application
/// would get it.
fn toast(intent: ToastIntent) -> Element<'static, Msg> {
    let toast = Toast::new(
        TOAST_ID,
        intent,
        "Saved",
        "All good.",
        Msg::Dismiss(TOAST_ID),
    );
    render(AppLayout::new(body()).toasts(vec![toast]))
}

fn glyph_bounds(ui: &mut Simulator<'_, Msg>, label: &str) -> Rectangle {
    ui.find(GLYPH)
        .unwrap_or_else(|_| panic!("{label}: the close glyph {GLYPH:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label}: the close glyph {GLYPH:?} is not visible"))
}

/// The close button's pointer target.
fn close_target(intent: ToastIntent, label: &str) -> Rectangle {
    let inside = {
        let mut ui = simulator(toast(intent));
        glyph_bounds(&mut ui, label).center()
    };
    pointer_target(
        label,
        || toast(intent),
        inside,
        |m: &Msg| *m == Msg::Dismiss(TOAST_ID),
    )
}

/// The glyph rendered alone at the engine's size, in the font this
/// feature state uses — built here rather than taken from the engine, so
/// a wrong glyph or font there cannot also be the reference.
fn reference_glyph_size() -> iced::Size {
    let glyph = iced::widget::text(GLYPH).size(GLYPH_SIZE);
    #[cfg(feature = "lucide-icons")]
    let glyph = glyph.font(iced::Font::with_name("lucide"));
    let element: Element<'static, Msg> = glyph.into();

    let mut ui = simulator(element);
    ui.find(GLYPH)
        .expect("the reference glyph renders")
        .visible_bounds()
        .expect("the reference glyph is visible")
        .size()
}

// ---------------------------------------------------------------------------
// (a) The pointer target
// ---------------------------------------------------------------------------

/// The target is **exactly** the floor, not merely above it.
///
/// `>=` would pass just as well with the `[0, 8]` padding this RFC
/// removed, which made the button 40 x 24: over the floor, and 15px of it
/// taken from the message column. On a fixed-width toast those are the
/// same 15px. Asserting equality is what stops padding creeping back.
#[test]
fn close_button_target_is_exactly_the_floor() {
    for (label, intent) in intents() {
        let target = close_target(intent, label);
        assert!(
            (target.width - MIN_TARGET).abs() <= TOLERANCE
                && (target.height - MIN_TARGET).abs() <= TOLERANCE,
            "{label}: the toast close button's pointer target is {}x{}, not \
             {MIN_TARGET}x{MIN_TARGET} — below the floor is a WCAG 2.5.8 defect, and above it \
             is room taken from the message column of a fixed-width toast",
            target.width,
            target.height,
        );
    }
}

/// The message column keeps the room it had at 0.50.0.
///
/// Found by structure rather than by arithmetic on the toast's literals:
/// the message column is the largest container that holds the message and
/// **not** the close glyph. The toast surface and the body row hold both;
/// the text column inside it is smaller.
#[test]
fn message_column_keeps_its_width() {
    for (label, intent) in intents() {
        let mut ui = simulator(toast(intent));
        let message = ui
            .find("All good.")
            .expect("the toast renders its message")
            .visible_bounds()
            .expect("the message is visible")
            .center();
        let glyph = glyph_bounds(&mut ui, label).center();

        let column = containers(&mut ui)
            .into_iter()
            .filter(|r| r.contains(message) && !r.contains(glyph))
            .max_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)))
            .unwrap_or_else(|| panic!("{label}: no container holds the message alone"));

        assert!(
            column.width >= MESSAGE_COLUMN_MIN,
            "{label}: the message column is {}px wide, under 0.50.0's {MESSAGE_COLUMN_MIN}px — \
             the close button has taken room from it, so longer messages wrap earlier and the \
             toast grows taller",
            column.width,
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Centring
// ---------------------------------------------------------------------------

#[test]
fn close_glyph_is_natural_size_and_centred_in_its_target() {
    let natural = reference_glyph_size();
    for (label, intent) in intents() {
        let target = close_target(intent, label);
        let glyph = {
            let mut ui = simulator(toast(intent));
            glyph_bounds(&mut ui, label)
        };

        assert!(
            (glyph.width - natural.width).abs() <= TOLERANCE
                && (glyph.height - natural.height).abs() <= TOLERANCE,
            "{label}: the close glyph's box is {}x{} but the glyph's natural size is {}x{} — \
             it fills its container instead of being centred in it",
            glyph.width,
            glyph.height,
            natural.width,
            natural.height,
        );

        let (g, t) = (glyph.center(), target.center());
        assert!(
            (g.x - t.x).abs() <= TOLERANCE && (g.y - t.y).abs() <= TOLERANCE,
            "{label}: the close glyph's centre {g:?} is not within {TOLERANCE}px of its \
             button's centre {t:?} (button {target:?}, glyph {glyph:?})",
        );
    }
}

// ---------------------------------------------------------------------------
// (c) The glyph
// ---------------------------------------------------------------------------

#[test]
fn close_button_draws_the_feature_state_glyph() {
    for (label, intent) in intents() {
        let mut ui = simulator(toast(intent));
        assert!(
            ui.find(GLYPH).is_ok(),
            "{label}: the close button does not draw {GLYPH:?}"
        );
        assert!(
            ui.find(NOT_GLYPH).is_err(),
            "{label}: the close button draws {NOT_GLYPH:?}, but this feature state must draw \
             {GLYPH:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Lucide: the font control, and the engine/widgets drift test
// ---------------------------------------------------------------------------

#[cfg(feature = "lucide-icons")]
mod lucide {
    use super::*;
    use common::assert_lucide_font_loaded;

    /// Fails if the lucide font was not loaded in this binary. See
    /// [`common::assert_lucide_font_loaded`] for why it measures a 1em
    /// advance rather than comparing a loaded glyph with an unloaded one.
    #[test]
    fn font_is_loaded() {
        assert_lucide_font_loaded("toast close glyph", GLYPH_SIZE);
    }

    /// The engine and `snora-widgets` draw the same close glyph, in the
    /// same font (RFC-101 Q-1 (a)).
    ///
    /// They are two independent sites — `snora::toast` builds its own
    /// `text(..).font(Font::with_name("lucide"))`, because the engine does
    /// not depend on `snora-widgets` — so nothing but a test stops one
    /// from changing codepoint or losing the font while the other keeps
    /// it. Both are reached through public API only.
    ///
    /// **What the width check catches, measured rather than assumed.**
    /// Finding `'\u{e1b2}'` in both trees proves only that both asked for
    /// that codepoint. A loaded lucide glyph is exactly 1em wide at its
    /// own size, so asserting that of each also catches the codepoint
    /// rendering as *some other font's* glyph — which is what happens when
    /// the lucide font is absent from the process (measured: 10.8px at
    /// size 18, against 18.0 loaded).
    ///
    /// It does **not** catch a site dropping its
    /// `.font(Font::with_name("lucide"))`. Measured: with the font loaded,
    /// `text("\u{e1b2}")` renders 18.000 x 23.400 with or without that
    /// call, because cosmic-text resolves a codepoint by coverage against
    /// every font loaded in the process. So that edit has no rendered
    /// consequence to catch — both spellings draw the same glyph — and no
    /// assertion over rendered output can distinguish them.
    #[cfg(all(feature = "widgets", feature = "design"))]
    #[test]
    fn engine_and_widgets_draw_the_same_lucide_glyph() {
        use snora::design::Tokens;
        use snora::design::notice::Notice;
        use snora::design::style::text::label_size;

        let tokens = Tokens::light();

        // The engine's toast close button, at its own glyph size.
        let engine = {
            let mut ui = simulator(toast(ToastIntent::Info));
            glyph_bounds(&mut ui, "engine toast")
        };

        // snora-widgets' notice dismiss control, at the label size it
        // uses. A different size on purpose: the shared property is
        // "1em in the lucide font", not a shared pixel size.
        let widgets_size = label_size(&tokens).0;
        let widgets = {
            let element: Element<'_, Msg> = Notice::new(&tokens, snora::design::Tone::Info, "body")
                .dismiss(Msg::Dismiss(TOAST_ID))
                .render();
            let mut ui = simulator(element);
            ui.find(GLYPH)
                .expect("the notice dismiss control draws the lucide glyph")
                .visible_bounds()
                .expect("the notice dismiss glyph is visible")
        };

        assert!(
            (engine.width - GLYPH_SIZE).abs() <= 0.01,
            "the engine's close glyph is {}px wide at size {GLYPH_SIZE}; a loaded lucide glyph \
             is exactly 1em wide, so this one is a fallback — the engine has lost its \
             `Font::with_name(\"lucide\")`",
            engine.width,
        );
        assert!(
            (widgets.width - widgets_size).abs() <= 0.01,
            "snora-widgets' dismiss glyph is {}px wide at size {widgets_size}; a loaded lucide \
             glyph is exactly 1em wide, so this one is a fallback",
            widgets.width,
        );
    }
}
