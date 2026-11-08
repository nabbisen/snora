//! `design::button::link`: underlined per line, at the ruled thickness and gap
//! (RFC-117 Unit 1).
//!
//! # The rule being asserted
//!
//! > An underline is 1/16 em thick, and its top edge is 1/16 em below its
//! > line's box. Each of the two is rounded to the nearest whole pixel, halves
//! > up, and is never under 1 px. Every wrapped line has its own underline.
//! > Snapping to the pixel grid never moves an underline up toward its text.
//!
//! # Why this renders by itself instead of through `iced_test`'s Simulator
//!
//! `Simulator::snapshot` fixes the scale factor at 2, and most desktops run at
//! 1, 1.25 or 1.5. Unit 0b measured at 2 only. This draws through the
//! renderer's own `Headless::screenshot`, which takes the scale, so the rule is
//! measured at **1 and 2** (asserted) and at **1.5** (reported).
//!
//! # How an underline is found
//!
//! By **difference from a ghost button** with the same label, size and width.
//! The two draw the same text at the same place — which is itself the C-2
//! assertion — so the rows where the frames differ are the underlines, one
//! group per line, with no density heuristic to mistake a glyph for a rule.
//! The glyphs' own ink comes from the ghost frame.
//!
//! # Renderers
//!
//! Follows `ICED_TEST_BACKEND` (default tiny-skia), asserting from the
//! renderer's own name that the backend asked for is the one that drew. **Any
//! wgpu run must use `--test-threads=1`** (parallel wgpu contexts segfault).

#![cfg(all(feature = "widgets", feature = "design"))]

use iced::widget::{column, container, row};
use iced::{Alignment, Color, Element, Font, Length, Pixels, Rectangle, Size, Theme};
use iced_runtime::core::clipboard;
use iced_runtime::core::renderer::Headless;
use iced_runtime::core::widget::Operation;
use iced_runtime::core::widget::operation::Focusable;
use iced_runtime::core::{Event, mouse, renderer, window};
use iced_runtime::user_interface::{self, UserInterface};

use snora::design::{Tokens, button};

mod common;
use common::{AA_TEXT, NON_TEXT_MIN, contrast_ratio};

const WINDOW: Size = Size::new(420.0, 360.0);
const PAD: f32 = 20.0;
const COLUMN: f32 = 260.0;

/// Descenders in every wrapped line of a natural wrap at these sizes.
const SENTENCE: &str = "Choose another folder for your projects, journals and query logs together";
/// Lines that start with the tallest ink there is (an accented capital, a
/// ring-bearing one, an ascender) and each carry descenders, so each
/// underline has real ink above its box and real ink below the previous one.
const TALL_STARTS: &str = "gjpqy\n\u{c9}gjpqy\n\u{c5}gjpqy\nlgjpqy";

#[derive(Debug, Clone, PartialEq)]
struct Msg(&'static str);

fn tokens_at(size: f32) -> Tokens {
    let mut tokens = Tokens::light();
    tokens.typography.label.size = size;
    tokens
}

fn bg_of(tokens: &Tokens) -> Color {
    let c = tokens.palette.background;
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

/// The window around `inner`: the preset's background, padded.
fn window_of(tokens: &Tokens, inner: Element<'static, Msg>) -> Element<'static, Msg> {
    let bg = bg_of(tokens);
    container(column![inner].padding(PAD))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(bg.into()),
            ..container::Style::default()
        })
        .into()
}

fn link_of(tokens: &Tokens, label: &str) -> Element<'static, Msg> {
    container(button::link(tokens, label.to_owned(), Msg("link")))
        .width(Length::Fixed(COLUMN))
        .into()
}

fn ghost_of(tokens: &Tokens, label: &str) -> Element<'static, Msg> {
    container(button::ghost(tokens, label.to_owned(), Msg("ghost")))
        .width(Length::Fixed(COLUMN))
        .into()
}

// ---- rendering at a scale -----------------------------------------------------

/// A frame at a chosen scale factor, and what the tree reported about itself.
struct Shot {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    scale: f32,
    /// Focus targets' boxes, logical, in tree order.
    controls: Vec<Rectangle>,
    /// Text boxes announced to operations, logical, in tree order.
    texts: Vec<Rectangle>,
}

#[derive(Default)]
struct Report {
    controls: Vec<Rectangle>,
    texts: Vec<Rectangle>,
}

impl Operation for Report {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&iced_runtime::core::widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn Focusable,
    ) {
        self.controls.push(bounds);
    }

    fn text(&mut self, _id: Option<&iced_runtime::core::widget::Id>, bounds: Rectangle, _t: &str) {
        self.texts.push(bounds);
    }
}

/// What a control looks like: nothing happening, the pointer over it, the
/// pointer held down on it, or keyboard focus on the first control with the
/// pointer away (a click no longer focuses, RFC-121, so `Down` draws no ring).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Pointer {
    Away,
    Over,
    Down,
    Focused,
}

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned())
}

fn shoot(
    element: Element<'static, Msg>,
    theme: &Theme,
    scale: f32,
    font: Font,
    pointer: Pointer,
) -> Shot {
    use iced::theme::Base;
    let wanted = backend();
    let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        font,
        Pixels(16.0),
        Some(&wanted),
    ))
    .expect("the requested renderer starts");
    assert_eq!(
        Headless::name(&renderer),
        wanted,
        "asked for the {wanted} renderer and another drew: these pixels are not the ones under test",
    );

    let mut ui = UserInterface::build(
        element,
        WINDOW,
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();

    // Where the pointer goes: the middle of the first control.
    let mut report = Report::default();
    ui.operate(&renderer, &mut report);
    let mut cursor = mouse::Cursor::Unavailable;
    if pointer != Pointer::Away {
        let at = report.controls.first().expect("a control").center();
        cursor = mouse::Cursor::Available(at);
        let _ = ui.update(
            &[Event::Mouse(mouse::Event::CursorMoved { position: at })],
            cursor,
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
    }
    if pointer == Pointer::Down {
        let _ = ui.update(
            &[Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
            cursor,
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
    }
    if pointer == Pointer::Focused {
        // `focus_next`, run to completion as the window loop does.
        let mut current: Box<dyn Operation<()>> =
            Box::new(iced_runtime::core::widget::operation::focusable::focus_next::<()>());
        loop {
            ui.operate(
                &renderer,
                &mut iced_runtime::core::widget::operation::black_box(current.as_mut()),
            );
            match current.finish() {
                iced_runtime::core::widget::operation::Outcome::Chain(next) => current = next,
                _ => break,
            }
        }
    }
    // A control records what it is about to look like on the redraw request,
    // so a frame drawn without one is drawn "disabled".
    let _ = ui.update(
        &[Event::Window(window::Event::RedrawRequested(
            iced_runtime::core::time::Instant::now(),
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );

    let base = theme.base();
    ui.draw(
        &mut renderer,
        theme,
        &renderer::Style {
            text_color: base.text_color,
        },
        cursor,
    );

    let physical = Size::new(
        (WINDOW.width * scale).round() as u32,
        (WINDOW.height * scale).round() as u32,
    );
    let rgba = Headless::screenshot(&mut renderer, physical, scale, base.background_color);
    Shot {
        width: physical.width,
        height: physical.height,
        rgba,
        scale,
        controls: report.controls,
        texts: report.texts,
    }
}

impl Shot {
    fn pixel(&self, x: u32, y: u32) -> Color {
        let i = ((y * self.width + x) * 4) as usize;
        Color::from_rgba8(self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], 1.0)
    }

    fn differs(&self, other: &Shot, x: u32, y: u32) -> bool {
        let (a, b) = (self.pixel(x, y), other.pixel(x, y));
        (a.r - b.r).abs() > 1e-6 || (a.g - b.g).abs() > 1e-6 || (a.b - b.b).abs() > 1e-6
    }

    /// Physical rows, within the padded column, that carry any ink.
    fn ink_rows(&self, bg: Color) -> Vec<u32> {
        let x0 = (PAD * self.scale) as u32;
        let x1 = (((PAD + COLUMN + 20.0) * self.scale) as u32).min(self.width);
        (0..self.height)
            .filter(|&y| (x0..x1).any(|x| contrast_ratio(self.pixel(x, y), bg) > 1.02))
            .collect()
    }
}

/// One underline: physical rows, with the colour of its strongest pixel.
#[derive(Debug, Clone, Copy)]
struct Underline {
    top: u32,
    bottom: u32,
    peak: Color,
}

impl Underline {
    fn rows(&self) -> u32 {
        self.bottom - self.top + 1
    }
}

/// Fewest differing pixels for a row to count as part of an underline. An
/// underline spans a word at least; a stray anti-aliasing difference does not.
const MIN_DIFF: u32 = 6;

fn underlines(link: &Shot, ghost: &Shot, bg: Color) -> Vec<Underline> {
    assert_eq!((link.width, link.height), (ghost.width, ghost.height));
    let x0 = (PAD * link.scale) as u32;
    let x1 = (((PAD + COLUMN + 20.0) * link.scale) as u32).min(link.width);
    let mut rows: Vec<(u32, Color)> = Vec::new();
    for y in 0..link.height {
        let mut n = 0;
        let mut peak = bg;
        for x in x0..x1 {
            if link.differs(ghost, x, y) {
                n += 1;
                let c = link.pixel(x, y);
                if contrast_ratio(c, bg) > contrast_ratio(peak, bg) {
                    peak = c;
                }
            }
        }
        if n >= MIN_DIFF {
            rows.push((y, peak));
        }
    }
    let mut out: Vec<Underline> = Vec::new();
    for (y, peak) in rows {
        match out.last_mut() {
            Some(u) if y == u.bottom + 1 => {
                u.bottom = y;
                if contrast_ratio(peak, bg) > contrast_ratio(u.peak, bg) {
                    u.peak = peak;
                }
            }
            _ => out.push(Underline {
                top: y,
                bottom: y,
                peak,
            }),
        }
    }
    out
}

/// What one underline clears, in physical rows, **on both sides**.
#[derive(Debug, Clone, Copy)]
struct Clearance {
    /// Rows between its own line's lowest ink and its top. `None`: that line
    /// has no ink above it to measure from.
    below_text: Option<i64>,
    /// Rows between its bottom and the next line's highest ink. `None` for
    /// the last line.
    above_next: Option<i64>,
}

fn clearances(ghost: &Shot, found: &[Underline], bg: Color) -> Vec<Clearance> {
    let ink = ghost.ink_rows(bg);
    found
        .iter()
        .enumerate()
        .map(|(k, u)| {
            let lo = if k == 0 { 0 } else { found[k - 1].bottom };
            let lowest = ink.iter().copied().filter(|&y| y > lo && y < u.top).max();
            let next_top = found.get(k + 1).map_or(u32::MAX, |n| n.top);
            let next = ink.iter().copied().find(|&y| y > u.bottom && y < next_top);
            Clearance {
                below_text: lowest.map(|l| i64::from(u.top) - i64::from(l) - 1),
                above_next: found
                    .get(k + 1)
                    .and(next)
                    .map(|n| i64::from(n) - i64::from(u.bottom) - 1),
            }
        })
        .collect()
}

// ---- fonts, sizes, scales ------------------------------------------------------

fn fonts() -> [(&'static str, Font); 3] {
    [
        ("default sans", Font::DEFAULT),
        ("monospace", Font::MONOSPACE),
        (
            "bold",
            Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            },
        ),
    ]
}

const SIZES: [f32; 4] = [12.0, 16.0, 24.0, 32.0];
const SCALES: [f32; 2] = [1.0, 2.0];

/// The ruled thickness and offset, in logical pixels: 1/16 em, rounded half
/// up, never under 1. Written here from the sentence of the rule, not taken
/// from the widget.
fn ruled(size: f32) -> f32 {
    (size / 16.0 + 0.5).floor().max(1.0)
}

/// The extra height of each line of a link, written from the rule: four units
/// (thickness, offset, and two more of clearance).
fn leading(size: f32) -> f32 {
    4.0 * ruled(size)
}

struct Measured {
    link: Shot,
    found: Vec<Underline>,
    clear: Vec<Clearance>,
}

/// What the link's text must look like **without** its underline, built from
/// the rule and not from the widget: plain text with each line four units
/// (thickness, offset, two more) taller (the text centred in it), wrapping at the same width, drawn
/// where the link's own label was found.
fn reference_of(tokens: &Tokens, text: &str, size: f32, label: Rectangle) -> Element<'static, Msg> {
    let line = 1.3 * size + leading(size);
    let plain = iced::widget::text(text.to_owned())
        .size(size)
        .line_height(iced::widget::text::LineHeight::Absolute(Pixels(line)))
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        // The widget shapes with `Advanced`; `Basic` would kern differently and
        // differ from it in every glyph, not only under the underline.
        .shaping(iced::widget::text::Shaping::Advanced)
        .color(
            snora::design::style::button::ghost(tokens, iced::widget::button::Status::Active)
                .text_color,
        )
        // The widest line of the link's own layout, which no other word fits
        // beside: so it wraps where the link wrapped.
        .width(Length::Fixed(label.width.ceil() + 1.0));
    window_of(
        tokens,
        container(plain)
            .padding(iced::Padding {
                top: label.y - PAD,
                left: label.x - PAD,
                ..iced::Padding::ZERO
            })
            .into(),
    )
}

fn measure(tokens: &Tokens, text: &str, scale: f32, font: Font, pointer: Pointer) -> Measured {
    let theme = snora::design::theme(tokens);
    let bg = bg_of(tokens);
    let size = tokens.typography.label.size;
    let link = shoot(
        window_of(tokens, link_of(tokens, text)),
        &theme,
        scale,
        font,
        pointer,
    );
    let reference = shoot(
        reference_of(tokens, text, size, link.texts[0]),
        &theme,
        scale,
        font,
        Pointer::Away,
    );
    let found = underlines(&link, &reference, bg);
    let clear = clearances(&reference, &found, bg);
    Measured { link, found, clear }
}

fn assert_clean(what: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} {what} failure(s) (backend {}):\n  {}",
        failures.len(),
        backend(),
        failures.join("\n  "),
    );
}

// ---- the assertions ------------------------------------------------------------

/// **Every wrapped line has its own underline**, and the label wraps.
#[test]
fn every_wrapped_line_of_a_link_is_underlined() {
    let mut failures = Vec::new();
    for size in SIZES {
        let tokens = tokens_at(size);
        let m = measure(&tokens, SENTENCE, 2.0, Font::DEFAULT, Pointer::Away);
        // Lines, counted from the **layout**: the label's box is that many
        // line boxes tall. Counting runs of ink rows failed on CI's fonts,
        // where a dotted `i` leaves a blank row inside one line and reads as
        // two; the layout does not depend on what the glyphs look like.
        let line_box = 1.3 * size + leading(size);
        let runs = (m.link.texts[0].height / line_box).round() as usize;
        if runs < 2 {
            failures.push(format!(
                "{size}px: the sample did not wrap ({runs} line), so this proves nothing"
            ));
        }
        if m.found.len() != runs {
            failures.push(format!(
                "{size}px: {runs} lines of text but {} underlines",
                m.found.len()
            ));
        }
    }
    assert_clean("wrapped-line", failures);
}

/// **Thickness**: 1/16 em, rounded, never under 1 — in physical rows,
/// `ruled(size) x scale`, on every line, at 1 and 2.
#[test]
fn an_underlines_thickness_is_the_ruled_one() {
    let mut failures = Vec::new();
    // 7 px is where 1/16 em rounds to 0: the "never under 1" floor.
    for size in [7.0, 12.0, 16.0, 24.0, 32.0, 40.0] {
        for scale in SCALES {
            let m = measure(
                &tokens_at(size),
                "Open the notes",
                scale,
                Font::DEFAULT,
                Pointer::Away,
            );
            let want = (ruled(size) * scale) as u32;
            if m.found.is_empty() {
                failures.push(format!("{size}px @ {scale}: no underline at all"));
            }
            for (k, u) in m.found.iter().enumerate() {
                if u.rows() != want {
                    failures.push(format!(
                        "{size}px @ {scale}: line {k} has {} rows, the rule says {want}",
                        u.rows()
                    ));
                }
            }
        }
    }
    assert_clean("thickness", failures);
}

/// **The gap below the descenders**, at 1 and 2, in three fonts: at least the
/// ruled offset, in physical rows (the descenders end at or above the line's
/// box, so the offset is the least the gap can be), and so **it grows with the
/// text**: a constant 1 px offset would leave 3 rows at 32 px, where the rule
/// asks for 4.
#[test]
fn an_underline_clears_its_own_lines_descenders() {
    let mut failures = Vec::new();
    for (name, font) in fonts() {
        for scale in SCALES {
            for size in SIZES {
                let m = measure(&tokens_at(size), SENTENCE, scale, font, Pointer::Away);
                let need = (ruled(size) * scale) as i64;
                match m.clear.iter().filter_map(|c| c.below_text).min() {
                    None => failures.push(format!("{name} {size}px @ {scale}: no line has ink")),
                    Some(g) if g < need => failures.push(format!(
                        "{name} {size}px @ {scale}: the underline is {g} row(s) from the \
                         descenders; the ruled offset is {need}"
                    )),
                    Some(_) => {}
                }
            }
        }
    }
    assert_clean("gap-below", failures);
}

/// **C-1: the underline sits inside the next line's box**, so it must clear
/// that line's highest ink too. Lines that start with `É`, `Å` and `l` are the
/// worst case for that: the tallest ink there is, directly after an underline.
#[test]
fn an_underline_clears_the_next_lines_ink() {
    let mut failures = Vec::new();
    let mut report = Vec::new();
    for (name, font) in fonts() {
        for scale in SCALES {
            for size in SIZES {
                let m = measure(&tokens_at(size), TALL_STARTS, scale, font, Pointer::Away);
                if m.found.len() != 4 {
                    failures.push(format!(
                        "{name} {size}px @ {scale}: {} underlines for 4 lines",
                        m.found.len()
                    ));
                    continue;
                }
                let tightest = m.clear.iter().filter_map(|c| c.above_next).min();
                report.push(format!("{name} {size}px @ {scale}: {tightest:?}"));
                match tightest {
                    Some(g) if g >= 1 => {}
                    other => failures.push(format!(
                        "{name} {size}px @ {scale}: an underline comes within {other:?} \
                         row(s) of the next line's ink (needs at least 1)"
                    )),
                }
            }
        }
    }
    println!(
        "gap above the next line's ink, tightest per case:\n  {}",
        report.join("\n  ")
    );
    assert_clean("gap-above", failures);
}

/// Scale 1.5, once, as a figure: nothing asserted, because the logical pixel
/// is not the physical grid there and the renderer decides the edge.
#[test]
fn report_scale_one_and_a_half() {
    for size in SIZES {
        let m = measure(
            &tokens_at(size),
            TALL_STARTS,
            1.5,
            Font::DEFAULT,
            Pointer::Away,
        );
        println!(
            "scale 1.5, {size}px: rows {:?}, gap below text {:?}, gap above next {:?}",
            m.found.iter().map(Underline::rows).collect::<Vec<_>>(),
            m.clear.iter().map(|c| c.below_text).collect::<Vec<_>>(),
            m.clear.iter().map(|c| c.above_next).collect::<Vec<_>>(),
        );
    }
}

/// **The underline is the label's colour, in every state** — and the label's
/// colour is the ghost style's, in that state.
#[test]
fn the_underline_is_the_labels_colour_at_rest_hover_and_pressed() {
    let mut failures = Vec::new();
    for (preset, mut tokens) in Tokens::presets() {
        tokens.typography.label.size = 24.0;
        for (state, pointer, status) in [
            ("rest", Pointer::Away, iced::widget::button::Status::Active),
            (
                "hover",
                Pointer::Over,
                iced::widget::button::Status::Hovered,
            ),
            (
                "pressed",
                Pointer::Down,
                iced::widget::button::Status::Pressed,
            ),
        ] {
            // The reference is a **ghost button in the same state**: one line
            // of the same label draws identically (C-2), and a pressed or
            // hovered control has a fill the plain-text reference of the other
            // tests does not, which would be mistaken for an underline.
            let theme = snora::design::theme(&tokens);
            let bg = bg_of(&tokens);
            let link = shoot(
                window_of(&tokens, link_of(&tokens, "Open")),
                &theme,
                2.0,
                Font::DEFAULT,
                pointer,
            );
            let ghost = shoot(
                window_of(&tokens, ghost_of(&tokens, "Open")),
                &theme,
                2.0,
                Font::DEFAULT,
                pointer,
            );
            // Only the rows inside the label's own box: a link in the
            // high-contrast presets is a pixel taller than the ghost (room
            // for the ring), and its fill's extra row differs from the ghost's
            // too.
            let label = link.texts[0];
            let (lo, hi) = (
                (label.y * 2.0) as u32,
                ((label.y + label.height) * 2.0).ceil() as u32,
            );
            let found: Vec<Underline> = underlines(&link, &ghost, bg)
                .into_iter()
                .filter(|u| u.top >= lo && u.top <= hi)
                .map(|u| Underline {
                    bottom: u.bottom.min(hi),
                    ..u
                })
                .collect();
            let want = snora::design::style::button::ghost(&tokens, status).text_color;
            let Some(u) = found.first() else {
                failures.push(format!("{preset} {state}: no underline"));
                continue;
            };
            // The underline's own first row, mid-label: a fully covered pixel
            // of the underline and of nothing else (the strongest pixel of the
            // differing rows could be the focus ring's, which a taller link
            // draws lower than the ghost's).
            let got = link.pixel(((label.x + label.width / 2.0) * 2.0) as u32, u.top);
            let close = |a: f32, b: f32| (a - b).abs() <= 1.5 / 255.0;
            if !(close(got.r, want.r) && close(got.g, want.g) && close(got.b, want.b)) {
                failures.push(format!(
                    "{preset} {state}: the underline is {got:?}, the label's colour is {want:?}"
                ));
            }
        }
    }
    assert_clean("state-colour", failures);
}

/// **Contrast on every preset**: the underline at 3:1 (non-text) and the label
/// at 4.5:1 (text), from drawn pixels.
#[test]
fn the_underline_and_the_label_clear_their_floors_on_every_preset() {
    let mut failures = Vec::new();
    for (preset, mut tokens) in Tokens::presets() {
        tokens.typography.label.size = 24.0;
        let bg = bg_of(&tokens);
        let m = measure(
            &tokens,
            "Choose another folder",
            2.0,
            Font::DEFAULT,
            Pointer::Away,
        );
        let Some(u) = m.found.first() else {
            failures.push(format!("{preset}: no underline"));
            continue;
        };
        let ratio = contrast_ratio(u.peak, bg);
        if ratio < NON_TEXT_MIN {
            failures.push(format!(
                "{preset}: the underline is {ratio:.2}:1, under {NON_TEXT_MIN}"
            ));
        }
        // The label: the strongest ink in rows above the underline.
        let x1 = (((PAD + COLUMN) * 2.0) as u32).min(m.link.width);
        let label_peak = (0..u.top)
            .flat_map(|y| ((PAD * 2.0) as u32..x1).map(move |x| (x, y)))
            .map(|(x, y)| m.link.pixel(x, y))
            .max_by(|a, b| contrast_ratio(*a, bg).total_cmp(&contrast_ratio(*b, bg)))
            .expect("pixels");
        let label = contrast_ratio(label_peak, bg);
        if label < AA_TEXT {
            failures.push(format!(
                "{preset}: the label is {label:.2}:1, under {AA_TEXT}"
            ));
        }
    }
    assert_clean("contrast", failures);
}

/// **C-2: a link's label lines up with a ghost label beside it.** In a row
/// aligned to the centre, the lowest ink of `HEEL` (no descenders, so its
/// baseline) is at the same row, within one, at 1 and 2.
#[test]
fn a_link_and_a_ghost_beside_it_sit_at_the_same_height() {
    let mut failures = Vec::new();
    for size in [14.0_f32, 16.0, 24.0, 32.0] {
        for scale in SCALES {
            let tokens = tokens_at(size);
            let theme = snora::design::theme(&tokens);
            let bg = bg_of(&tokens);
            let pair: Element<'static, Msg> = row![
                button::ghost(&tokens, "HEEL", Msg("ghost")),
                button::link(&tokens, "HEEL", Msg("link")),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .into();
            let shot = shoot(
                window_of(&tokens, pair),
                &theme,
                scale,
                Font::DEFAULT,
                Pointer::Away,
            );
            assert_eq!(shot.texts.len(), 2, "two labels expected");

            // The lowest ink inside each label's own box that is not the
            // underline. The underline is inside the box now, and is told from
            // the text by being a row inked across nearly the label's width:
            // `HEEL` has no row like that.
            let lowest = |b: Rectangle| -> u32 {
                let (x0, x1) = (
                    (b.x * scale) as u32,
                    ((b.x + b.width) * scale).ceil() as u32,
                );
                let (y0, y1) = (
                    (b.y * scale) as u32,
                    ((b.y + b.height) * scale).ceil() as u32,
                );
                let width = (x1 - x0) as f32;
                (y0..y1.min(shot.height))
                    .filter(|&y| {
                        let inked = (x0..x1.min(shot.width))
                            .filter(|&x| contrast_ratio(shot.pixel(x, y), bg) > 1.02)
                            .count() as f32;
                        inked > 0.0 && inked < 0.9 * width
                    })
                    .max()
                    .expect("the label has ink")
            };
            let (ghost, link) = (lowest(shot.texts[0]), lowest(shot.texts[1]));
            if ghost.abs_diff(link) > 1 {
                failures.push(format!(
                    "{size}px @ {scale}: the ghost's baseline is row {ghost}, the link's {link}"
                ));
            }
            // And the boxes themselves: same height, so neither is taller.
            let (a, b) = (shot.controls[0], shot.controls[1]);
            if (a.height - b.height).abs() > 0.01 {
                failures.push(format!(
                    "{size}px @ {scale}: the ghost is {:.2} tall, the link {:.2}",
                    a.height, b.height
                ));
            }
        }
    }
    assert_clean("alignment", failures);
}

/// **`ghost` is unchanged (R-4)**: it draws nothing below its label.
#[test]
fn a_ghost_button_has_no_underline() {
    for size in SIZES {
        let tokens = tokens_at(size);
        let theme = snora::design::theme(&tokens);
        let bg = bg_of(&tokens);
        let shot = shoot(
            window_of(&tokens, ghost_of(&tokens, "Cancel")),
            &theme,
            2.0,
            Font::DEFAULT,
            Pointer::Away,
        );
        let label = shot.texts[0];
        let below = ((label.y + label.height) * 2.0).ceil() as u32;
        let inked_below = shot
            .ink_rows(bg)
            .into_iter()
            .filter(|&y| y >= below)
            .count();
        assert_eq!(
            inked_below, 0,
            "{size}px: the ghost button has ink {inked_below} row(s) below its label: that is an underline"
        );
    }
}

/// A link and a ghost are the same size and shape in the default text: the
/// underline lives in the padding, not in extra height.
#[test]
fn a_link_is_the_size_of_the_ghost_it_replaces() {
    for size in [14.0_f32, 24.0, 32.0] {
        let tokens = tokens_at(size);
        let theme = snora::design::theme(&tokens);
        let g = shoot(
            window_of(&tokens, ghost_of(&tokens, "Cancel")),
            &theme,
            2.0,
            Font::DEFAULT,
            Pointer::Away,
        );
        let l = shoot(
            window_of(&tokens, link_of(&tokens, "Cancel")),
            &theme,
            2.0,
            Font::DEFAULT,
            Pointer::Away,
        );
        let (g, l) = (g.controls[0], l.controls[0]);
        assert!(
            (g.height - l.height).abs() <= 0.01 && (g.width - l.width).abs() <= 1.0,
            "{size}px: the ghost is {:.2} x {:.2}, the link {:.2} x {:.2}",
            g.width,
            g.height,
            l.width,
            l.height,
        );
    }
}

/// **The focus ring never cuts the underline.** A focused link's underline is
/// whole: every row it has unfocused is still its colour with the ring drawn.
///
/// The high-contrast presets' ring is 3 px wide at a 2 px offset, so its inner
/// edge lies 1 px **inside** the control, where the underline's bottom row is
/// at 24 px and above (there is no padding left below it).
#[test]
fn a_focused_links_ring_does_not_cover_its_underline() {
    let mut failures = Vec::new();
    for (preset, mut tokens) in Tokens::presets() {
        for size in [14.0_f32, 16.0, 24.0, 32.0] {
            tokens.typography.label.size = size;
            let theme = snora::design::theme(&tokens);
            let bg = bg_of(&tokens);
            let rest_link = shoot(
                window_of(&tokens, link_of(&tokens, "Open")),
                &theme,
                2.0,
                Font::DEFAULT,
                Pointer::Away,
            );
            let rest_ghost = shoot(
                window_of(&tokens, ghost_of(&tokens, "Open")),
                &theme,
                2.0,
                Font::DEFAULT,
                Pointer::Away,
            );
            let found = underlines(&rest_link, &rest_ghost, bg);
            let Some(u) = found.first().copied() else {
                failures.push(format!("{preset} {size}px: no underline"));
                continue;
            };
            // Focused by the keyboard, so the ring is drawn and the label (and
            // so the underline) is the resting colour. (Pressing used to focus
            // the control, and this was drawn from a press; it no longer does.)
            let down = shoot(
                window_of(&tokens, link_of(&tokens, "Open")),
                &theme,
                2.0,
                Font::DEFAULT,
                Pointer::Focused,
            );
            // The premise, asserted: a ring is drawn, i.e. the frame differs from
            // the resting one somewhere. Without it this test passes just as well
            // when nothing is focused.
            // The ring lies outside the control (its inner edge may reach 1px
            // in), so a differing pixel *outside* the control's box is the ring.
            let own = down.controls[0];
            let ring_drawn = (0..down.height).any(|y| {
                (0..down.width).any(|x| {
                    let at = iced::Point::new(x as f32 / down.scale, y as f32 / down.scale);
                    !own.contains(at) && down.differs(&rest_link, x, y)
                })
            });
            if !ring_drawn {
                failures.push(format!(
                    "{preset} {size}px: focusing the link drew no ring, so the underline check below proves nothing"
                ));
            }
            let want =
                snora::design::style::button::ghost(&tokens, iced::widget::button::Status::Active)
                    .text_color;
            let x = ((PAD + 20.0) * 2.0) as u32;
            for y in u.top..=u.bottom {
                let got = down.pixel(x, y);
                let close = |a: f32, b: f32| (a - b).abs() <= 1.5 / 255.0;
                if !(close(got.r, want.r) && close(got.g, want.g) && close(got.b, want.b)) {
                    failures.push(format!(
                        "{preset} {size}px: row {y} of the underline is {got:?} once focused, not \
                         {want:?}: the ring has covered it"
                    ));
                }
            }
        }
    }
    assert_clean("ring-over-underline", failures);
}
