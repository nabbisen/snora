//! RFC-139 R-3: **the five marks snora draws itself, in the state an adopter starts in.**
//!
//! A Cargo feature never changes how a mark snora draws is drawn (RFC-139 R-1), so with `lucide-icons` on and **no
//! Lucide font loaded** (the state of an application that turned the feature on, or whose dependency did, and has not
//! found `LUCIDE_FONT_BYTES`) each mark must still be the glyph snora means: "×" on the toast, the notice and the
//! chip, "✓" in a checked checkbox and "▾" on the select. Until RFC-139 every rendered test ran with the font loaded
//! (`common::simulator` loads it), so no test saw the other state.
//!
//! **Read from pixels.** Each mark's ink is isolated inside its control and compared with the same glyph drawn by
//! `text` in the text's font at the size the mark is drawn at: the tight boxes agree to a pixel, and the masks overlap.
//! A Lucide codepoint with no font is a missing-glyph box or a fragment of another font, and neither agrees.
//!
//! **This binary never loads the font, and says so.** iced's font system is process-global, so a single call to
//! `common::simulator` (which loads it) would put every later test here in the other state. This file uses `Rig`
//! only, and the first thing each test does is assert that the font system has no family named `lucide`.
//!
//! The renderer follows `ICED_TEST_BACKEND`; a wgpu run must use `--test-threads=1`.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use common::removal_scenes::tokens;
use common::sweep::Msg;
use iced::widget::text;
use iced::{Element, Rectangle};
use snora::LayoutDirection::{self, Ltr};
use snora::design::style::text::{body_size, label_size};
use snora::{AppLayout, Toast, ToastIntent};

/// The host's default text size in these rigs (`Headless::new(.., Pixels(16.0), ..)`).
const HOST: f32 = 16.0;
/// RFC-115's toast close mark: 1.125 em.
const CLOSE_EM: f32 = 1.125;
const SCALE: usize = 2;
const WINDOW: usize = 800;

fn assert_the_lucide_font_is_not_loaded() {
    let fs = iced_test::renderer::graphics::text::font_system();
    let mut fs = fs.write().expect("the font system");
    let loaded = fs.raw().db().faces().any(|f| {
        f.families
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("lucide"))
    });
    assert!(
        !loaded,
        "premise: the Lucide font is loaded in this process, so this is not the adopter's starting state. \
         Something in this binary called `common::simulator` or loaded `LUCIDE_FONT_BYTES`."
    );
}

fn body() -> Element<'static, Msg> {
    text("body").into()
}

fn toast_layout(d: LayoutDirection) -> AppLayout<Element<'static, Msg>, Msg> {
    AppLayout::new(body())
        .toasts(vec![Toast::new(
            7,
            ToastIntent::Info,
            "Title",
            "Message",
            Msg::Go,
        )])
        .direction(d)
}

/// A scene, the focus target that is the mark's control, how much of the control's edge to leave out so only the
/// mark is inside, the glyph it must be, and the size it is drawn at (px).
struct Mark {
    name: &'static str,
    design: bool,
    target: usize,
    /// Logical px left out on each side of the control (its ring, its box, its label).
    region: Region,
    glyph: &'static str,
    px: f32,
    make: fn(LayoutDirection) -> Element<'static, Msg>,
}

/// Where inside the control the mark is read.
#[derive(Clone, Copy)]
enum Region {
    /// The control's box, `px` logical pixels in from every side (a close mark has nothing else in its control).
    Inset(f32),
    /// Inside the outline the control draws (a checkbox's box, a select's trigger): its tight ink box in from every
    /// side by `inner`, and, if `right` is `Some(w)`, only the last `w` px of it (a select's chevron, not its label).
    InsideOutline { inner: f32, right: Option<f32> },
}

fn marks() -> Vec<Mark> {
    let t = tokens();
    vec![
        Mark {
            name: "toast close, engine path",
            design: false,
            target: 0,
            region: Region::Inset(2.0),
            glyph: "×",
            px: CLOSE_EM * HOST,
            make: |d| snora::render(toast_layout(d)),
        },
        Mark {
            name: "toast close, design path",
            design: true,
            target: 0,
            region: Region::Inset(2.0),
            glyph: "×",
            px: CLOSE_EM * body_size(t).0,
            make: |d| snora::design::render(toast_layout(d), tokens()),
        },
        Mark {
            name: "notice dismiss, design path",
            design: true,
            target: 0,
            region: Region::Inset(4.0),
            glyph: "×",
            px: label_size(t).0,
            make: |d| {
                snora::design::notice::Notice::new(
                    tokens(),
                    snora::design::Tone::Info,
                    "Body text of the notice.",
                    d,
                )
                .dismiss(Msg::Go)
                .render()
            },
        },
        Mark {
            name: "chip remove, design path",
            design: true,
            target: 1,
            region: Region::Inset(5.0),
            glyph: "×",
            px: label_size(t).0,
            make: |d| snora::design::chip::removable(tokens(), "Rust", false, Msg::Go, Msg::Go, d),
        },
        Mark {
            name: "checkbox tick, engine path",
            design: false,
            target: 0,
            region: Region::InsideOutline {
                inner: 4.0,
                right: None,
            },
            glyph: "✓",
            px: HOST,
            make: |d| snora::checkbox(true, d).on_toggle(|_| Msg::Go).into(),
        },
        Mark {
            name: "checkbox tick, design path",
            design: true,
            target: 0,
            region: Region::InsideOutline {
                inner: 4.0,
                right: None,
            },
            glyph: "✓",
            px: label_size(t).0,
            make: |d| {
                snora::design::checkbox(tokens(), true, d)
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
        },
        Mark {
            name: "select chevron, engine path",
            design: false,
            target: 0,
            region: Region::InsideOutline {
                inner: 4.0,
                right: Some(24.0),
            },
            glyph: "▾",
            px: HOST,
            make: |d| {
                snora::select(["Apple", "Banana"], Some(&"Apple"), |_| Msg::Go, d)
                    .on_open_change(|_| Msg::Go)
                    .into()
            },
        },
        Mark {
            name: "select chevron, design path",
            design: true,
            target: 0,
            region: Region::InsideOutline {
                inner: 4.0,
                right: Some(24.0),
            },
            glyph: "▾",
            px: label_size(t).0,
            make: |d| {
                snora::design::select(
                    tokens(),
                    ["Apple", "Banana"],
                    Some(&"Apple"),
                    |_| Msg::Go,
                    d,
                )
                .on_open_change(|_| Msg::Go)
                .into()
            },
        },
    ]
}

/// A binary ink mask over a rectangle of a 2x frame: `true` where a pixel differs from the rectangle's own corner.
struct Mask {
    w: usize,
    h: usize,
    on: Vec<bool>,
}

impl Mask {
    fn of(frame: &[u8], x0: usize, y0: usize, w: usize, h: usize) -> Mask {
        let at = |x: usize, y: usize| {
            let i = (y * WINDOW * SCALE + x) * 4;
            [frame[i], frame[i + 1], frame[i + 2]]
        };
        let bg = at(x0, y0);
        let mut on = vec![false; w * h];
        for y in 0..h {
            for x in 0..w {
                let p = at(x0 + x, y0 + y);
                on[y * w + x] = (0..3)
                    .map(|c| (i32::from(p[c]) - i32::from(bg[c])).abs())
                    .sum::<i32>()
                    > 90;
            }
        }
        Mask { w, h, on }
    }

    /// The tight box of the ink, as (x0, y0, x1, y1) exclusive, or None.
    fn tight(&self) -> Option<(usize, usize, usize, usize)> {
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        for y in 0..self.h {
            for x in 0..self.w {
                if self.on[y * self.w + x] {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        (x0 != usize::MAX).then_some((x0, y0, x1, y1))
    }

    /// The ink cropped to its tight box.
    fn cropped(&self) -> Option<(usize, usize, Vec<bool>)> {
        let (x0, y0, x1, y1) = self.tight()?;
        let (w, h) = (x1 - x0, y1 - y0);
        let mut v = vec![false; w * h];
        for y in 0..h {
            for x in 0..w {
                v[y * w + x] = self.on[(y0 + y) * self.w + x0 + x];
            }
        }
        Some((w, h, v))
    }
}

/// The ink of the mark inside its control, at 2x.
fn mark_ink(m: &Mark) -> Mask {
    let mut rig = Rig::new((m.make)(Ltr));
    let targets = rig.targets();
    let b: Rectangle = targets
        .get(m.target)
        .unwrap_or_else(|| {
            panic!(
                "{}: {} focus targets, wanted target {}",
                m.name,
                targets.len(),
                m.target
            )
        })
        .bounds;
    rig.point_at(None);
    rig.redraw();
    let theme = if m.design {
        snora::design::theme(tokens())
    } else {
        iced::Theme::Light
    };
    let frame = rig.frame_scaled(&theme, SCALE as f32);
    let sc = SCALE as f32;
    let (x0, y0, w, h) = match m.region {
        Region::Inset(px) => (
            ((b.x + px) * sc) as usize,
            ((b.y + px) * sc) as usize,
            ((b.width - 2.0 * px) * sc) as usize,
            ((b.height - 2.0 * px) * sc) as usize,
        ),
        Region::InsideOutline { inner, right } => {
            let whole = Mask::of(
                &frame,
                (b.x * sc) as usize,
                (b.y * sc) as usize,
                (b.width * sc) as usize,
                (b.height * sc) as usize,
            );
            let (ox0, oy0, ox1, oy1) = whole
                .tight()
                .unwrap_or_else(|| panic!("{}: the control draws no outline", m.name));
            let (bx, by) = ((b.x * sc) as usize, (b.y * sc) as usize);
            let inner = (inner * sc) as usize;
            let x_end = bx + ox1 - inner;
            let x_start = match right {
                Some(width) => (x_end).saturating_sub((width * sc) as usize),
                None => bx + ox0 + inner,
            };
            (
                x_start,
                by + oy0 + inner,
                x_end - x_start,
                (oy1 - oy0).saturating_sub(2 * inner),
            )
        }
    };
    Mask::of(&frame, x0, y0, w, h)
}

/// The glyph drawn alone by `text` in the text's font at `px`.
fn reference_ink(glyph: &'static str, px: f32) -> Mask {
    let view: Element<'static, Msg> = iced::widget::container(text(glyph).size(px))
        .padding(20)
        .into();
    let mut rig = Rig::new(view);
    rig.redraw();
    let frame = rig.frame_scaled(&iced::Theme::Light, SCALE as f32);
    Mask::of(&frame, 0, 0, 120 * SCALE, 90 * SCALE)
}

/// How two inks compare: whether their tight boxes agree to a logical pixel (2 samples at 2x), and how much of the two
/// masks overlap, aligned at the boxes' top-left.
struct Likeness {
    a: (usize, usize),
    b: (usize, usize),
    boxes: bool,
    overlap: f32,
}

impl Likeness {
    fn of(a: &Mask, b: &Mask) -> Option<Likeness> {
        let (aw, ah, av) = a.cropped()?;
        let (bw, bh, bv) = b.cropped()?;
        let (w, h) = (aw.min(bw), ah.min(bh));
        let (mut both, mut either) = (0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                let (p, q) = (av[y * aw + x], bv[y * bw + x]);
                both += usize::from(p && q);
                either += usize::from(p || q);
            }
        }
        Some(Likeness {
            a: (aw, ah),
            b: (bw, bh),
            boxes: aw.abs_diff(bw) <= 2 && ah.abs_diff(bh) <= 2,
            overlap: if either == 0 {
                0.0
            } else {
                both as f32 / either as f32
            },
        })
    }

    /// The same glyph: the boxes agree and the masks overlap by at least half.
    fn same(&self) -> bool {
        self.boxes && self.overlap >= 0.5
    }
}

fn px(samples: usize) -> f32 {
    samples as f32 / SCALE as f32
}

/// A codepoint no installed font covers: a plane-16 private-use codepoint. What it draws is a missing-glyph box, or
/// nothing; either way it is what a mark looks like when its glyph is not there.
const UNCOVERED: &str = "\u{10FFFD}";

/// **Premise (RFC-139 Q-1 and Q-3): each reference glyph is a glyph and not a missing-glyph box.** The box is defined by
/// drawing it (an uncovered codepoint, at the same size), not by assuming its shape. If a machine has no font for "✓" or
/// "▾", the reference would be a box, a mark that is a box would match it, and the test would pass for the wrong reason:
/// so it fails here instead, and says which glyph.
fn assert_the_references_are_glyphs() {
    for (glyph, size) in [
        ("×", CLOSE_EM * HOST),
        ("×", 14.0),
        ("✓", HOST),
        ("✓", 14.0),
        ("▾", HOST),
        ("▾", 14.0),
    ] {
        let tofu = reference_ink(UNCOVERED, size);
        let real = reference_ink(glyph, size);
        let Some(real_cropped) = real.cropped() else {
            panic!("premise: {glyph:?} at {size} px drew nothing: this machine has no font for it");
        };
        if let Some(l) = Likeness::of(&real, &tofu) {
            assert!(
                !l.same(),
                "premise: {glyph:?} at {size} px draws as the missing-glyph box ({:.1}x{:.1} px, the same as an uncovered codepoint): \
                 this machine has no font for it, so the test cannot tell a mark from a box here (RFC-139 Q-3)",
                px(real_cropped.0),
                px(real_cropped.1),
            );
        }
    }
    // And the missing-glyph box is not "×" itself, or the check above could never fire.
    let (tofu, cross) = (reference_ink(UNCOVERED, HOST), reference_ink("×", HOST));
    if let Some(l) = Likeness::of(&cross, &tofu) {
        assert!(
            !l.same(),
            "premise: an uncovered codepoint draws the same as \"×\" ({l_boxes:?}): choose another",
            l_boxes = (l.a, l.b)
        );
    }
}

#[test]
fn each_mark_snora_draws_is_the_glyph_it_means_with_no_lucide_font_loaded() {
    assert_the_lucide_font_is_not_loaded();
    assert_the_references_are_glyphs();
    let mut off = Vec::new();
    for m in marks() {
        let ink = mark_ink(&m);
        let reference = reference_ink(m.glyph, m.px);
        match Likeness::of(&ink, &reference) {
            None => off.push(format!("{}: no ink inside the control", m.name)),
            Some(l) if !l.same() => off.push(format!(
                "{}: the mark's ink is {:.1}x{:.1} px and overlaps the intended {:?} drawn by `text` ({:.1}x{:.1} px) by {:.0}%: \
                 it is not that glyph",
                m.name,
                px(l.a.0),
                px(l.a.1),
                m.glyph,
                px(l.b.0),
                px(l.b.1),
                l.overlap * 100.0
            )),
            Some(_) => {}
        }
    }
    // The premise again, **after** the rigs ran: a font loaded while the test drew (by a rig, or by anything it called) would
    // put the marks above in the other state, and a check made only before the first rig cannot see that.
    assert_the_lucide_font_is_not_loaded();
    assert!(
        off.is_empty(),
        "{} of {} marks are not the glyph snora means, with no Lucide font loaded:\n  {}",
        off.len(),
        marks().len(),
        off.join("\n  ")
    );
}

/// The premise on its own, so a machine without a font for a glyph fails with that one message and nothing else.
#[test]
fn the_reference_glyphs_are_not_missing_glyph_boxes() {
    assert_the_lucide_font_is_not_loaded();
    assert_the_references_are_glyphs();
}
