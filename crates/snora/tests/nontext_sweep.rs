//! RFC-135: **every non-text element a reader needs in order to see a control or its state holds 3:1
//! against what is beside it, on the drawn pixels**: a glyph that is not a word, a boundary, a track, a
//! mark and a scrollbar. The non-text half of RFC-131's sweep, on the same scenes
//! (`common/sweep.rs`), in every preset, on every neutral surface, in each state, in both directions.
//!
//! * **A glyph** holds 3:1 if the pixel furthest in contrast from the background beside it, drawn at
//!   twice the pixel density, is at least 3:1 (RFC-131 M-2). The glyph rule lives here;
//!   `text_sweep.rs` keeps the classification (a glyph must be listed as one, so a label cannot dodge
//!   its level).
//! * **A boundary** is the component's visible extent: its **fill or its border, whichever is stronger**,
//!   against what is beside it, on the weakest side of its outline. It is read from a frame drawn at
//!   **4x**, because a 1 px rule is four solid pixels there and still two at 2x, and sampling a thin edge
//!   at a fractional offset is the one place anti-aliasing reaches a boundary.
//!
//! # What is not a gap, and why (RFC-135)
//!
//! Listed in [`EXCEPTIONS`], **each with its reason, and a check that the reason still holds**, so
//! nobody "fixes" a dialog border or a hover tint, and an exception that has stopped being needed fails
//! the test and is removed:
//!
//! * the dialog's and the sheet's **border** against the dimmed page: their **fill** against it is the
//!   boundary a reader sees, and holds 3:1 (the rule above takes the stronger of the two);
//! * a **hover or press tint**: WCAG 1.4.11 does not ask a hover tint for 3:1; RFC-129's cue (the label's
//!   colour and a ΔE76 of 3, `text_floor.rs`) carries the state;
//! * the **drop shadow**: decorative;
//! * a **tooltip's border over coloured content**: the tooltip floats over whatever is there and is
//!   content, not a control; its text holds its level (`text_sweep.rs`). Over the three neutral surfaces
//!   it is held to 3:1;
//! * **disabled**: exempt, as everywhere.
//!
//! # The tripwires
//!
//! The text sweep's three (`text_sweep.rs`) and a fourth, here: **a file that calls `fill_quad(` itself
//! draws something no `container` reports**, so it needs a finder in this file. Each has a plant.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); the rendered jobs run it on both.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use common::par;
use common::removal::Rig;
use common::sweep::{
    Kind, Msg, NON_TEXT, ON_MENU, SURFACES, Scene, State, apply_state, build, design_theme, hex,
    leak, menu, page, ratio, scenes, surface_colour, sweep_of,
};
use iced::widget::text;
use iced::{Element, Length, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::{Tokens, Tone, button, chip, notice::Notice, widget as dw};
use snora::{AppLayout, LayoutDirection};

/// The slack a 3:1 floor gets for 8-bit rounding of two colours (a ratio moves by about 0.02). Smaller than the
/// text sweep's 0.05: a boundary at 2.97 is under.
const TOLERANCE: f32 = 0.02;

/// The scale boundaries are read at.
const EDGE: usize = 4;

fn px(frame: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * 800 * EDGE + x) * 4;
    [frame[i], frame[i + 1], frame[i + 2]]
}

// ---- elements --------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum El {
    Container,
    Target,
}

/// Every container and every focus target in the tree, with its bounds.
fn elements(rig: &mut Rig<Msg>) -> Vec<(El, Rectangle)> {
    struct Collect(Arc<Mutex<Vec<(El, Rectangle)>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
            self.0.lock().unwrap().push((El::Container, bounds));
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    let mut v = out.lock().unwrap().clone();
    for t in rig.targets() {
        v.push((El::Target, t.bounds));
    }
    v
}

fn rect_of(els: &[(El, Rectangle)], kind: El, w: f32, h: f32) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| *k == kind && (r.width - w).abs() < 0.6 && (r.height - h).abs() < 0.6)
        .map(|x| x.1)
}

/// A container of this height at least `min_width` wide (a rule).
fn rule(els: &[(El, Rectangle)], h: f32, min_width: f32, max_width: f32) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| {
            *k == El::Container
                && (r.height - h).abs() < 0.6
                && r.width >= min_width
                && r.width <= max_width
        })
        .map(|x| x.1)
}

/// The first container whose size is in the given ranges.
fn sized(els: &[(El, Rectangle)], w: (f32, f32), h: (f32, f32)) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| {
            *k == El::Container
                && r.width >= w.0
                && r.width <= w.1
                && r.height >= h.0
                && r.height <= h.1
        })
        .map(|x| x.1)
}

// ---- reading a boundary ------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Sides {
    All,
    Top,
    Bottom,
    Right,
    Left,
    /// Top and bottom only (a track whose left end is under its bar).
    Vertical,
}

impl Sides {
    fn list(self) -> Vec<u8> {
        match self {
            Sides::All => vec![0, 1, 2, 3],
            Sides::Top => vec![0],
            Sides::Bottom => vec![1],
            Sides::Left => vec![2],
            Sides::Vertical => vec![0, 1],
            Sides::Right => vec![3],
        }
    }
}

/// One side of a rectangle: the **stronger** of its border (the pixel just inside the edge) and its
/// fill (a pixel 3 logical px inside, when the element is thick enough to have one) against the pixel
/// just outside. `None` where the outside is off the window.
fn side(frame: &[u8], r: Rectangle, side: u8) -> Option<(f32, [u8; 3], [u8; 3])> {
    let s = EDGE as f32;
    let (x0, y0, x1, y1) = (
        (r.x * s) as i64,
        (r.y * s) as i64,
        ((r.x + r.width) * s) as i64,
        ((r.y + r.height) * s) as i64,
    );
    let (w, h) = (800 * EDGE as i64, 600 * EDGE as i64);
    let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
    let depth = (3.0 * s) as i64;
    // (border inside, fill inside, outside), and the thickness of the element across this side.
    let (border, fill, out, thick) = match side {
        0 => ((mx, y0 + 1), (mx, y0 + depth), (mx, y0 - 2), r.height),
        1 => ((mx, y1 - 2), (mx, y1 - depth), (mx, y1 + 1), r.height),
        2 => ((x0 + 1, my), (x0 + depth, my), (x0 - 2, my), r.width),
        _ => ((x1 - 2, my), (x1 - depth, my), (x1 + 1, my), r.width),
    };
    let on = |p: (i64, i64)| p.0 >= 0 && p.1 >= 0 && p.0 < w && p.1 < h;
    if !on(out) || !on(border) {
        return None;
    }
    let outside = px(frame, out.0 as usize, out.1 as usize);
    let b = px(frame, border.0 as usize, border.1 as usize);
    let mut best = (ratio(b, outside), b);
    if thick >= 8.0 && on(fill) {
        let f = px(frame, fill.0 as usize, fill.1 as usize);
        let rf = ratio(f, outside);
        if rf > best.0 {
            best = (rf, f);
        }
    }
    Some((best.0, best.1, outside))
}

/// The weakest of the chosen sides: the whole outline must be seen.
fn boundary(frame: &[u8], r: Rectangle, sides: Sides) -> Option<(f32, [u8; 3], [u8; 3])> {
    let mut v: Vec<_> = sides
        .list()
        .into_iter()
        .filter_map(|s| side(frame, r, s))
        .collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    v.first().copied()
}

/// Border alone, ignoring the fill (the exceptions are about this reading).
fn border_only(frame: &[u8], r: Rectangle, sides: Sides) -> Option<f32> {
    let s = EDGE as f32;
    let mut worst: Option<f32> = None;
    for sd in sides.list() {
        let (x0, y0, x1, y1) = (
            (r.x * s) as i64,
            (r.y * s) as i64,
            ((r.x + r.width) * s) as i64,
            ((r.y + r.height) * s) as i64,
        );
        let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
        let (b, o) = match sd {
            0 => ((mx, y0 + 1), (mx, y0 - 2)),
            1 => ((mx, y1 - 2), (mx, y1 + 1)),
            2 => ((x0 + 1, my), (x0 - 2, my)),
            _ => ((x1 - 2, my), (x1 + 1, my)),
        };
        if o.0 < 0 || o.1 < 0 || o.0 >= 800 * EDGE as i64 || o.1 >= 600 * EDGE as i64 {
            continue;
        }
        let q = ratio(
            px(frame, b.0 as usize, b.1 as usize),
            px(frame, o.0 as usize, o.1 as usize),
        );
        worst = Some(worst.map_or(q, |w: f32| w.min(q)));
    }
    worst
}

// ---- the subjects ------------------------------------------------------------------------------------------------------------------

/// What a subject picks out of a drawn scene: its rectangle, which sides count, and (for a control) the
/// focus target that the pointer states apply to.
struct Pick {
    rect: Rectangle,
    sides: Sides,
    target: Option<usize>,
}

type Picker = fn(&[(El, Rectangle)], &[Rectangle]) -> Option<Pick>;

struct Subject {
    name: &'static str,
    /// The inventory row (`audit-0-63-a/evidence/non-text-inventory.md`).
    row: &'static str,
    scene: &'static str,
    pick: Picker,
}

fn all(rect: Rectangle, target: Option<usize>) -> Option<Pick> {
    Some(Pick {
        rect,
        sides: Sides::All,
        target,
    })
}

// ---- finders for RFC-134's marks (what the form controls report as containers) ------------------------------------------------

/// The `n`-th square mark of about an em and a quarter (a checkbox's box, a radio's ring), top to bottom: a mark is
/// several nested containers of one size, so rectangles within a pixel of each other are one.
fn square(els: &[(El, Rectangle)], n: usize) -> Option<Rectangle> {
    let mut v: Vec<Rectangle> = els
        .iter()
        .filter(|(k, r)| {
            *k == El::Container
                && (r.width - r.height).abs() < 0.6
                && (16.5..19.0).contains(&r.width)
        })
        .map(|x| x.1)
        .collect();
    v.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    v.dedup_by(|a, b| (a.y - b.y).abs() < 1.0 && (a.x - b.x).abs() < 1.0);
    v.get(n).copied()
}

/// A toggler's track: twice as wide as it is tall.
fn track_of_toggler(els: &[(El, Rectangle)]) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| {
            *k == El::Container
                && (16.5..19.0).contains(&r.height)
                && (r.width - 2.0 * r.height).abs() < 1.0
        })
        .map(|x| x.1)
}

/// A toggler's thumb: the disc inside a slot, a little under an em.
fn thumb_of_toggler(els: &[(El, Rectangle)]) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| {
            *k == El::Container
                && (r.width - r.height).abs() < 0.6
                && (10.5..12.5).contains(&r.width)
        })
        .map(|x| x.1)
}

/// A part of a slider's bar the bar reports: a container as thin as a track, that `keep` holds.
fn bar_part(els: &[(El, Rectangle)], keep: impl Fn(&Rectangle) -> bool) -> Option<Rectangle> {
    els.iter()
        .find(|(k, r)| *k == El::Container && r.height < 10.0 && r.height > 3.0 && keep(r))
        .map(|x| x.1)
}

fn subjects() -> Vec<Subject> {
    vec![
        Subject {
            name: "primary button vs its surface",
            row: "1",
            scene: "button, primary",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "danger button vs its surface",
            row: "2",
            scene: "button, danger",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "secondary button border vs its surface",
            row: "3",
            scene: "button, secondary",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "selected chip vs its surface",
            row: "7",
            scene: "filter chip, selected",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "unselected chip outline vs its surface",
            row: "8",
            scene: "filter chip, unselected",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "notice tone bar vs the notice (info)",
            row: "12",
            scene: "notice, info",
            pick: |e, _| {
                rect_of(e, El::Container, 4.0, 59.0).map(|rect| Pick {
                    rect,
                    sides: Sides::Right,
                    target: None,
                })
            },
        },
        Subject {
            name: "notice tone bar vs the notice (neutral)",
            row: "12",
            scene: "notice, neutral",
            pick: |e, _| {
                rect_of(e, El::Container, 4.0, 59.0).map(|rect| Pick {
                    rect,
                    sides: Sides::Right,
                    target: None,
                })
            },
        },
        Subject {
            name: "notice outer border vs the page (info)",
            row: "13",
            scene: "notice, info",
            pick: |e, _| sized(e, (700.0, 790.0), (50.0, 70.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "notice outer border vs the page (neutral)",
            row: "13",
            scene: "notice, neutral",
            pick: |e, _| sized(e, (700.0, 790.0), (50.0, 70.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "toast fill vs the page (info)",
            row: "16",
            scene: "toast, info",
            pick: |e, _| rect_of(e, El::Container, 340.0, 72.0).and_then(|r| all(r, None)),
        },
        Subject {
            name: "toast fill vs the page (error)",
            row: "16",
            scene: "toast, error",
            pick: |e, _| rect_of(e, El::Container, 340.0, 72.0).and_then(|r| all(r, None)),
        },
        Subject {
            name: "toast fill vs the page (success)",
            row: "16",
            scene: "toast, success",
            pick: |e, _| rect_of(e, El::Container, 340.0, 72.0).and_then(|r| all(r, None)),
        },
        Subject {
            name: "toast fill vs the page (warning)",
            row: "16",
            scene: "toast, warning",
            pick: |e, _| rect_of(e, El::Container, 340.0, 72.0).and_then(|r| all(r, None)),
        },
        Subject {
            name: "toast fill vs the page (Debug)",
            row: "16",
            scene: "toast, debug",
            pick: |e, _| rect_of(e, El::Container, 340.0, 72.0).and_then(|r| all(r, None)),
        },
        Subject {
            name: "tab bar rule vs the page",
            row: "18",
            scene: "tab bar",
            pick: |e, _| {
                rule(e, 1.0, 700.0, 800.0).map(|rect| Pick {
                    rect,
                    sides: Sides::Bottom,
                    target: None,
                })
            },
        },
        Subject {
            name: "active tab underline vs the bar",
            row: "19",
            scene: "tab bar",
            pick: |e, _| {
                rule(e, 2.0, 20.0, 200.0).map(|rect| Pick {
                    rect,
                    sides: Sides::Top,
                    target: None,
                })
            },
        },
        Subject {
            name: "breadcrumb ancestor rule vs the page",
            row: "21",
            scene: "breadcrumb",
            pick: |e, _| {
                rule(e, 1.0, 10.0, 200.0).map(|rect| Pick {
                    rect,
                    sides: Sides::Bottom,
                    target: None,
                })
            },
        },
        Subject {
            name: "active rail item vs the rail (names beneath)",
            row: "24",
            scene: "sidebar, names beneath",
            pick: |_, t| all(t[1], Some(1)),
        },
        Subject {
            name: "active rail item vs the rail (names beside)",
            row: "24",
            scene: "sidebar, names beside",
            pick: |_, t| all(t[1], Some(1)),
        },
        Subject {
            name: "header chrome border vs the page",
            row: "29",
            scene: "header",
            pick: |e, _| sized(e, (700.0, 790.0), (40.0, 50.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "footer chrome border vs the page",
            row: "29",
            scene: "footer",
            pick: |e, _| sized(e, (700.0, 790.0), (30.0, 45.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "menu dropdown card vs the page",
            row: "30",
            scene: "header with an open menu",
            pick: |e, _| {
                // The outermost container under the header that is not the window: the card.
                e.iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && r.width < 400.0
                            && r.height >= 40.0
                            && r.height < 300.0
                            && r.y > 30.0
                    })
                    .max_by(|a, b| (a.1.width * a.1.height).total_cmp(&(b.1.width * b.1.height)))
                    .and_then(|x| all(x.1, None))
            },
        },
        Subject {
            name: "card::surface border vs the page",
            row: "37",
            scene: "card, surface",
            pick: |e, _| sized(e, (30.0, 200.0), (30.0, 80.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "card::raised border vs the page, shadow beside it",
            row: "38",
            scene: "card, raised",
            pick: |e, _| sized(e, (30.0, 200.0), (30.0, 80.0)).and_then(|r| all(r, None)),
        },
        Subject {
            name: "card::selected 2 px border vs the page",
            row: "39",
            scene: "card, selected",
            pick: |e, _| sized(e, (30.0, 200.0), (30.0, 80.0)).and_then(|r| all(r, None)),
        },
        // ---- RFC-134: the five form controls ----------------------------------------------------------------------------------
        Subject {
            name: "checkbox box (checked) vs its surface",
            row: "134",
            scene: "checkbox, checked",
            pick: |e, _| square(e, 0).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "checkbox box (unchecked) outline vs its surface",
            row: "134",
            scene: "checkbox, unchecked",
            pick: |e, _| square(e, 0).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "toggler track (on) vs its surface",
            row: "134",
            scene: "toggler, on",
            pick: |e, _| track_of_toggler(e).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "toggler track (off) outline vs its surface",
            row: "134",
            scene: "toggler, off",
            pick: |e, _| track_of_toggler(e).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "toggler thumb (on) vs its track",
            row: "134",
            scene: "toggler, on",
            pick: |e, _| thumb_of_toggler(e).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "toggler thumb (off) vs its track",
            row: "134",
            scene: "toggler, off",
            pick: |e, _| thumb_of_toggler(e).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "radio dot ring (unselected) vs its surface",
            row: "134",
            scene: "radio group",
            pick: |e, _| square(e, 0).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "radio dot ring (selected) vs its surface",
            row: "134",
            scene: "radio group",
            pick: |e, _| square(e, 1).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "radio dot (selected) vs its ring",
            row: "134",
            scene: "radio group",
            pick: |e, _| {
                // The three options each hold a disc; the selected one is the second.
                let mut discs: Vec<Rectangle> = e
                    .iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && (9.0..10.5).contains(&r.width)
                            && (9.0..10.5).contains(&r.height)
                    })
                    .map(|x| x.1)
                    .collect();
                discs.sort_by(|a, b| a.y.total_cmp(&b.y));
                discs.dedup_by(|a, b| (a.y - b.y).abs() < 1.0);
                discs.get(1).copied().and_then(|r| all(r, Some(0)))
            },
        },
        // RFC-138: the same marks of the group laid out in a row (the three options share a line, the selected one the middle).
        Subject {
            name: "radio dot ring (unselected), horizontal, vs its surface",
            row: "138",
            scene: "radio group, horizontal",
            pick: |e, _| square(e, 0).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "radio dot ring (selected), horizontal, vs its surface",
            row: "138",
            scene: "radio group, horizontal",
            pick: |e, _| square(e, 1).and_then(|r| all(r, Some(0))),
        },
        Subject {
            name: "radio dot (selected), horizontal, vs its ring",
            row: "138",
            scene: "radio group, horizontal",
            pick: |e, _| {
                // The three options share a line and the selected one is the middle one, so by x it is the second disc
                // whichever way the line is mirrored.
                let mut discs: Vec<Rectangle> = e
                    .iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && (9.0..10.5).contains(&r.width)
                            && (9.0..10.5).contains(&r.height)
                    })
                    .map(|x| x.1)
                    .collect();
                discs.sort_by(|a, b| a.x.total_cmp(&b.x));
                discs.dedup_by(|a, b| (a.y - b.y).abs() < 1.0 && (a.x - b.x).abs() < 1.0);
                discs.get(1).copied().and_then(|r| all(r, Some(0)))
            },
        },
        Subject {
            name: "slider track vs its surface",
            row: "134",
            scene: "slider",
            pick: |e, _| {
                bar_part(e, |r| r.width > 100.0).map(|r| Pick {
                    rect: r,
                    sides: Sides::Vertical,
                    target: Some(0),
                })
            },
        },
        Subject {
            name: "slider fill vs its surface",
            row: "134",
            scene: "slider",
            pick: |e, _| {
                // The filled part is the thin container that is not the track: shorter than it.
                let track = bar_part(e, |r| r.width > 100.0)?;
                bar_part(e, |r| r.width < track.width - 1.0 && r.width > 10.0).map(|r| Pick {
                    rect: r,
                    sides: Sides::Vertical,
                    target: Some(0),
                })
            },
        },
        Subject {
            name: "slider thumb vs what is beside it",
            row: "134",
            scene: "slider",
            pick: |e, _| {
                e.iter()
                    .find(|(k, r)| {
                        *k == El::Container
                            && (r.width - r.height).abs() < 0.6
                            && (16.0..19.0).contains(&r.width)
                    })
                    .and_then(|x| all(x.1, Some(0)))
            },
        },
        Subject {
            name: "select trigger outline vs its surface",
            row: "134",
            scene: "select, closed",
            pick: |e, _| {
                // The control reports its own box; the outline is the widest container inside it (the control's
                // padding is the sides'), then comes the face's own content. So: the widest that is narrower than
                // the widest.
                let of_size: Vec<Rectangle> = e
                    .iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && (60.0..300.0).contains(&r.width)
                            && (20.0..40.0).contains(&r.height)
                    })
                    .map(|x| x.1)
                    .collect();
                let widest = of_size.iter().map(|r| r.width).fold(0.0, f32::max);
                of_size
                    .into_iter()
                    .filter(|r| r.width < widest - 1.0)
                    .max_by(|a, b| a.width.total_cmp(&b.width))
                    .and_then(|r| all(r, Some(0)))
            },
        },
        Subject {
            name: "select dropdown card vs the page",
            row: "134",
            scene: "select, open",
            pick: |e, _| {
                e.iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && r.width < 400.0
                            && r.height >= 40.0
                            && r.height < 300.0
                            && r.y >= 27.0
                            && r.y < 40.0
                    })
                    .max_by(|a, b| (a.1.width * a.1.height).total_cmp(&(b.1.width * b.1.height)))
                    .and_then(|x| all(x.1, None))
            },
        },
        Subject {
            name: "progress::card border vs the page",
            row: "41",
            scene: "progress card",
            pick: |e, _| sized(e, (700.0, 790.0), (50.0, 80.0)).and_then(|r| all(r, None)),
        },
    ]
}

/// Hover and press tints (rows 4, 20, 23, 25): an exception (RFC-135), measured so the exception is live.
fn tints() -> Vec<Subject> {
    vec![
        Subject {
            name: "hover tint: a ghost button",
            row: "4",
            scene: "button, ghost",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "hover tint: an inactive tab",
            row: "20",
            scene: "tab bar",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "hover tint: an ancestor crumb",
            row: "23",
            scene: "breadcrumb",
            pick: |_, t| all(t[0], Some(0)),
        },
        Subject {
            name: "hover tint: an inactive rail item",
            row: "25",
            scene: "sidebar, names beneath",
            pick: |_, t| all(t[0], Some(0)),
        },
    ]
}

/// The scenes this file adds to `common/sweep.rs`' (they hold no text of their own to name).
fn scene_named(name: &str) -> Scene {
    scenes()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no scene {name:?}"))
}

#[derive(Debug, Clone)]
struct Cell {
    preset: &'static str,
    subject: String,
    row: &'static str,
    surface: &'static str,
    direction: LayoutDirection,
    state: &'static str,
    ratio: f32,
    inside: [u8; 3],
    outside: [u8; 3],
}

/// Every subject of `list`, in every preset, on every surface, in each state and each direction.
///
/// **Each frame is drawn once** (0.64 housekeeping, B-1 (i)): subjects are grouped by scene, and for a
/// scene, a preset, a surface and a direction the rest frame is drawn once, and the hovered and the
/// pressed frame of each control once, whatever number of subjects read it. The rig that found the
/// subjects' elements is the one the rest frame is drawn on, so no rig is built only to look.
fn sweep_list(list: Vec<Subject>) -> Vec<Cell> {
    // Scenes in the order they first appear, each with the subjects that read it.
    let mut by_scene: Vec<(&'static str, Vec<&Subject>)> = Vec::new();
    for sub in &list {
        match by_scene.iter_mut().find(|(name, _)| *name == sub.scene) {
            Some((_, subs)) => subs.push(sub),
            None => by_scene.push((sub.scene, vec![sub])),
        }
    }
    // The presets are drawn on threads of their own and put back in preset order (`common/par.rs`): the cells come out
    // exactly as they did when one thread drew them all.
    let parts = par::by_preset(|preset, tokens| {
        let mut cells = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for (scene_name, subs) in &by_scene {
            let sc = scene_named(scene_name);
            let dirs: &[LayoutDirection] = if sc.directional {
                &[LayoutDirection::Ltr, LayoutDirection::Rtl]
            } else {
                &[LayoutDirection::Ltr]
            };
            let surfaces: &[&'static str] = if sc.own_page {
                &["(own page)"]
            } else {
                &SURFACES
            };
            for &dir in dirs {
                for &surface in surfaces {
                    let mut rest = Rig::new(build(&sc, t, surface, dir));
                    let els = elements(&mut rest);
                    let targets: Vec<Rectangle> =
                        rest.targets().into_iter().map(|x| x.bounds).collect();
                    let firsts: Vec<Pick> = subs
                        .iter()
                        .map(|sub| {
                            (sub.pick)(&els, &targets).unwrap_or_else(|| {
                                panic!(
                                    "{preset} / {} / {surface} / {dir:?}: the subject's element was not found in the scene",
                                    sub.name
                                )
                            })
                        })
                        .collect();
                    // The frames to draw: the rest frame, and the hovered and the pressed frame of
                    // each control some subject's pointer states apply to.
                    let mut frames: Vec<(&'static str, Option<usize>, State)> =
                        vec![("rest", None, State::Rest)];
                    if sc.states {
                        for first in &firsts {
                            let Some(i) = first.target else { continue };
                            for (label, state) in
                                [("hovered", State::Hovered), ("pressed", State::Pressed)]
                            {
                                if !frames.iter().any(|f| f.1 == Some(i) && f.2 == state) {
                                    frames.push((label, Some(i), state));
                                }
                            }
                        }
                    }
                    let mut hot: Option<Rig<Msg>> = None;
                    for (label, target, state) in frames {
                        let rig: &mut Rig<Msg> = if state == State::Rest {
                            &mut rest
                        } else {
                            let mut r = Rig::new(build(&sc, t, surface, dir));
                            let tb =
                                r.targets()[target.expect("a pointer state has a control")].bounds;
                            apply_state(&mut r, tb, state);
                            hot.insert(r)
                        };
                        let els = elements(rig);
                        let targets: Vec<Rectangle> =
                            rig.targets().into_iter().map(|x| x.bounds).collect();
                        let frame = rig.frame_scaled(&theme, EDGE as f32);
                        for (sub, first) in subs.iter().zip(&firsts) {
                            // A subject reads the pointer states of its own control, and the rest frame.
                            if state != State::Rest && first.target != target {
                                continue;
                            }
                            let Some(p) = (sub.pick)(&els, &targets) else {
                                continue;
                            };
                            if let Some((r, a, b)) = boundary(&frame, p.rect, p.sides) {
                                cells.push(Cell {
                                    preset,
                                    subject: sub.name.to_owned(),
                                    row: sub.row,
                                    surface,
                                    direction: dir,
                                    state: label,
                                    ratio: r,
                                    inside: a,
                                    outside: b,
                                });
                            }
                        }
                    }
                }
            }
        }
        cells
    });
    parts.into_iter().flatten().collect()
}
fn under(cells: &[Cell]) -> Vec<String> {
    cells
        .iter()
        .filter(|c| c.ratio + TOLERANCE < NON_TEXT)
        .map(|c| {
            format!(
                "row {} | {} / {} / {} / {:?} / {}: {:.2}:1 ({} against {})",
                c.row,
                c.preset,
                c.subject,
                c.surface,
                c.direction,
                c.state,
                c.ratio,
                hex(c.inside),
                hex(c.outside)
            )
        })
        .collect()
}

// ---- the subjects that are not one container's edge -------------------------------------------------------------------------------

fn on(t: &'static Tokens, surface: &str, el: Element<'static, Msg>) -> Element<'static, Msg> {
    let c = surface_colour(t, surface);
    let fill = iced::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    };
    iced::widget::container(el)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(fill.into()),
            ..Default::default()
        })
        .into()
}

#[allow(clippy::too_many_arguments)] // a row of the table, built where it is measured
fn cell(
    preset: &'static str,
    subject: &str,
    row: &'static str,
    surface: &'static str,
    state: &'static str,
    r: f32,
    a: [u8; 3],
    b: [u8; 3],
) -> Cell {
    Cell {
        preset,
        subject: subject.to_owned(),
        row,
        surface,
        direction: LayoutDirection::Ltr,
        state,
        ratio: r,
        inside: a,
        outside: b,
    }
}

fn top_and_bottom() -> Sides {
    Sides::Vertical
}

/// **The progress bar** (rows 41, 42): the track against what it sits on (its outline, or its fill, whichever is
/// stronger, on its top and bottom edges: the left one is under the bar), the bar against the track, and the
/// bar against what it sits on. Six tones, on the three surfaces (the row) and inside the card (the card).
fn progress_cells(out: &mut Vec<Cell>) {
    const TONES: [(&str, Tone); 6] = [
        ("accent", Tone::Accent),
        ("success", Tone::Success),
        ("warning", Tone::Warning),
        ("danger", Tone::Danger),
        ("info", Tone::Info),
        ("neutral", Tone::Neutral),
    ];
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for (scene, surfaces) in [
            ("progress row", SURFACES.to_vec()),
            ("progress card", vec!["background"]),
        ] {
            for surface in surfaces {
                for (tn, tone) in TONES {
                    let el = if scene == "progress row" {
                        snora::design::progress::row(
                            t,
                            "Progress",
                            Some(0.5),
                            tone,
                            snora::LayoutDirection::Ltr,
                        )
                    } else {
                        snora::design::progress::card(
                            t,
                            "Progress",
                            Some(0.5),
                            tone,
                            snora::LayoutDirection::Ltr,
                        )
                    };
                    let mut rig = Rig::new(on(t, surface, el));
                    let els = elements(&mut rig);
                    let frame = rig.frame_scaled(&theme, EDGE as f32);
                    let rr = sized(&els, (700.0, 790.0), (30.0, 36.0)).expect("the progress row");
                    let s = EDGE as f32;
                    // the bar's rows: the bottom 8 logical px of the row
                    let bar = Rectangle {
                        x: rr.x,
                        y: rr.y + rr.height - 8.0,
                        width: rr.width,
                        height: 8.0,
                    };
                    let unfilled = Rectangle {
                        x: rr.x + rr.width * 0.5,
                        width: rr.width * 0.5,
                        ..bar
                    };
                    let mid = ((bar.y + 4.0) * s) as usize;
                    let above = ((bar.y - 3.0) * s) as usize;
                    let fill_x = ((rr.x + rr.width * 0.25) * s) as usize;
                    let track_x = ((rr.x + rr.width * 0.75) * s) as usize;
                    let (barc, trackc, pagec) = (
                        px(&frame, fill_x, mid),
                        px(&frame, track_x, mid),
                        px(&frame, track_x, above),
                    );
                    let name = |what: &str| format!("progress {what} ({tn})");
                    if let Some((r, a, b)) = boundary(&frame, unfilled, top_and_bottom()) {
                        part.push(cell(
                            preset,
                            &name("track vs what it sits on"),
                            "41",
                            scene_surface(scene, surface),
                            "rest",
                            r,
                            a,
                            b,
                        ));
                    }
                    part.push(cell(
                        preset,
                        &name("bar vs the track"),
                        "42",
                        scene_surface(scene, surface),
                        "rest",
                        ratio(barc, trackc),
                        barc,
                        trackc,
                    ));
                    part.push(cell(
                        preset,
                        &name("bar vs what it sits on"),
                        "42",
                        scene_surface(scene, surface),
                        "rest",
                        ratio(barc, pagec),
                        barc,
                        pagec,
                    ));
                }
            }
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

fn scene_surface(scene: &str, surface: &'static str) -> &'static str {
    if scene == "progress card" {
        "card on background"
    } else {
        surface
    }
}

/// **The link's underline** (row 6): the label's colour, a rule under it, against the page; read as a run (ink against the
/// modal colour) in the band under the label, at rest, hovered and pressed.
fn link_cells(out: &mut Vec<Cell>) {
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for surface in SURFACES {
            for (label, state) in [
                ("rest", State::Rest),
                ("hovered", State::Hovered),
                ("pressed", State::Pressed),
            ] {
                let mut rig = Rig::new(on(t, surface, button::link(t, "Open", Msg::Go)));
                let tb = rig.targets()[0].bounds;
                apply_state(&mut rig, tb, state);
                let frame = rig.frame_scaled(&theme, common::sweep::SCALE as f32);
                let band = Rectangle {
                    x: tb.x,
                    y: tb.y + tb.height - 6.0,
                    width: tb.width,
                    height: 6.0,
                };
                if let Some((ink, bg, r)) = common::sweep::read_run(&frame, band) {
                    part.push(cell(
                        preset,
                        "link underline vs the page",
                        "6",
                        surface,
                        label,
                        r,
                        ink,
                        bg,
                    ));
                }
            }
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// The pixels that differ between two frames, as a box, outside `skip` (grown by 2 logical px).
fn changed_box(
    a: &[u8],
    b: &[u8],
    skip: Option<Rectangle>,
) -> Option<(usize, usize, usize, usize)> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
    let ex = |v: f32| (v * EDGE as f32) as usize;
    for y in 0..600 * EDGE {
        for x in 0..800 * EDGE {
            if skip.is_some_and(|s| {
                x + 8 >= ex(s.x)
                    && x <= ex(s.x + s.width) + 8
                    && y + 8 >= ex(s.y)
                    && y <= ex(s.y + s.height) + 8
            }) {
                continue;
            }
            let i = (y * 800 * EDGE + x) * 4;
            if a[i..i + 3] != b[i..i + 3] {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x1 >= x0 && y1 >= y0).then_some((x0, y0, x1, y1))
}

/// **The focus ring** (rows 10, 11): twelve controls, three surfaces: the ring is what changes when the control takes
/// focus, and it covers what the unfocused frame showed there.
fn ring_cells(out: &mut Vec<Cell>) {
    let rings: [(&str, usize); 12] = [
        ("button, primary", 0),
        ("button, secondary", 0),
        ("button, ghost", 0),
        ("button, link", 0),
        ("filter chip, unselected", 0),
        ("tab bar", 0),
        ("breadcrumb", 0),
        ("sidebar, names beneath", 0),
        ("header", 0),
        ("notice, info", 0),
        ("toast, info", 0),
        ("removable chip, selected", 1),
    ];
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for (name, idx) in rings {
            let sc = scene_named(name);
            let surfaces: &[&'static str] = if sc.own_page {
                &["(own page)"]
            } else {
                &SURFACES
            };
            for &surface in surfaces {
                let mut rig = Rig::new(build(&sc, t, surface, LayoutDirection::Ltr));
                let n = rig.targets().len();
                let idx = idx.min(n.saturating_sub(1));
                let rest = rig.frame_scaled(&theme, EDGE as f32);
                rig.focus_at(idx);
                rig.redraw();
                let hot = rig.frame_scaled(&theme, EDGE as f32);
                let Some((x0, y0, x1, y1)) = changed_box(&rest, &hot, None) else {
                    panic!("{preset} / {name} / {surface}: focusing the control drew no ring");
                };
                let mut counts: std::collections::HashMap<([u8; 3], [u8; 3]), usize> =
                    std::collections::HashMap::new();
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let i = (y * 800 * EDGE + x) * 4;
                        if rest[i..i + 3] != hot[i..i + 3] {
                            *counts
                                .entry((
                                    [hot[i], hot[i + 1], hot[i + 2]],
                                    [rest[i], rest[i + 1], rest[i + 2]],
                                ))
                                .or_default() += 1;
                        }
                    }
                }
                let (ringc, under_) = counts
                    .iter()
                    .max_by_key(|(k, n)| (**n, k.0))
                    .map(|(k, _)| *k)
                    .unwrap();
                part.push(cell(
                    preset,
                    &format!("focus ring vs what it covers ({name})"),
                    "10/11",
                    surface,
                    "focused",
                    ratio(ringc, under_),
                    ringc,
                    under_,
                ));
            }
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// **A tooltip's body border** (rows 26 to 28) against what it covers: found by what changes on hover. The sidebar's
/// tooltip floats over the page (a neutral surface), and is held to 3:1 there; the chip's and the notice's float over
/// their own, coloured, content, and are an exception (RFC-135: the tooltip is content, not a control).
fn tooltip_cells(out: &mut Vec<Cell>) {
    type MakeT = Box<dyn Fn(&'static Tokens) -> Element<'static, Msg> + Send + Sync>;
    let tips: Vec<(&str, &str, usize, MakeT)> = vec![
        (
            "tooltip border vs the page (the sidebar's)",
            "26/27",
            0,
            Box::new(|t| {
                dw::app_side_bar_with(
                    t,
                    common::sweep::rail(3),
                    LayoutDirection::Ltr,
                    dw::SideBarLabels::Tooltip,
                )
            }),
        ),
        (
            "tooltip border over content (the chip's)",
            "28",
            1,
            Box::new(|t| {
                chip::removable_with_tooltip(
                    t,
                    "Tag",
                    false,
                    Msg::Go,
                    Msg::Go,
                    "Remove tag",
                    snora::LayoutDirection::Ltr,
                )
            }),
        ),
        (
            "tooltip border over content (the notice's)",
            "28",
            0,
            Box::new(|t| {
                Notice::new(t, Tone::Info, "Body", snora::LayoutDirection::Ltr)
                    .dismiss(Msg::Go)
                    .dismiss_tooltip("Dismiss")
                    .render()
            }),
        ),
    ];
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for (name, row, idx, make) in &tips {
            for surface in SURFACES {
                // The sidebar's rail is drawn in a window, where it has room for its tooltip (see `common/sweep.rs`).
                let el = if name.contains("sidebar") {
                    let rail = make(t);
                    snora::design::render(AppLayout::new(page()).side_bar(rail), t)
                } else {
                    on(t, surface, make(t))
                };
                let mut rig = Rig::new(el);
                let tb = rig.targets()[*idx].bounds;
                rig.redraw();
                let rest = rig.frame_scaled(&theme, EDGE as f32);
                rig.point_at(Some(tb.center()));
                rig.redraw();
                let hot = rig.frame_scaled(&theme, EDGE as f32);
                let Some((x0, y0, x1, y1)) = changed_box(&rest, &hot, Some(tb)) else {
                    panic!("{preset} / {name} / {surface}: hovering opened no tooltip");
                };
                let mut counts: std::collections::HashMap<[u8; 3], usize> =
                    std::collections::HashMap::new();
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        *counts.entry(px(&hot, x, y)).or_default() += 1;
                    }
                }
                let fill = counts
                    .iter()
                    .max_by_key(|(c, n)| (**n, **c))
                    .map(|(c, _)| *c)
                    .unwrap();
                let my = (y0 + y1) / 2;
                let fx = (x0..=x1)
                    .find(|x| px(&hot, *x, my) == fill)
                    .expect("the tooltip body's fill");
                // The border pixel is the one just left of the fill; it covers what the unhovered frame showed there.
                let bx = fx.saturating_sub(2);
                let (border, covered) = (px(&hot, bx, my), px(&rest, bx, my));
                let surface: &'static str = if name.contains("sidebar") {
                    "(own page)"
                } else {
                    surface
                };
                part.push(cell(
                    preset,
                    name,
                    row,
                    surface,
                    "hovered",
                    ratio(border, covered),
                    border,
                    covered,
                ));
            }
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// **A scrollbar** (row 32), in a menu taller than the window: its thumb against the page it sits on (the dropdown
/// card) and against its track. The thumb is read where it is (the top of the rail), the track where it is not.
fn scrollbar_cells(out: &mut Vec<Cell>) {
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        let h = dw::app_header(
            t,
            "App",
            vec![menu(0, "File", 60)],
            &ON_MENU,
            Some(&0u8),
            None,
            LayoutDirection::Ltr,
        );
        let mut rig = Rig::new(snora::design::render(AppLayout::new(page()).header(h), t));
        rig.redraw();
        let els = elements(&mut rig);
        let frame = rig.frame_scaled(&theme, EDGE as f32);
        let card = els
            .iter()
            .filter(|(k, r)| {
                *k == El::Container && r.width < 400.0 && r.height > 300.0 && r.y > 30.0
            })
            .max_by(|a, b| a.1.height.total_cmp(&b.1.height))
            .map(|x| x.1)
            .expect("the dropdown card of a menu taller than the window");
        let s = EDGE as f32;
        let page_colour = px(
            &frame,
            ((card.x + 6.0) * s) as usize,
            ((card.y + 100.0) * s) as usize,
        );
        let strip = |y: f32| -> [u8; 3] {
            let mut best = (1.0f32, page_colour);
            for x in (((card.x + card.width - 9.5) * s) as usize)
                ..(((card.x + card.width - 0.5) * s) as usize)
            {
                let c = px(&frame, x, (y * s) as usize);
                let r = ratio(c, page_colour);
                if r > best.0 {
                    best = (r, c);
                }
            }
            best.1
        };
        let thumb = strip(card.y + 15.0);
        let track = strip(card.y + 450.0);
        part.push(cell(
            preset,
            "scrollbar thumb vs the page it sits on",
            "32",
            "(own page)",
            "rest",
            ratio(thumb, page_colour),
            thumb,
            page_colour,
        ));
        let against_track = if track == page_colour {
            page_colour
        } else {
            track
        };
        part.push(cell(
            preset,
            "scrollbar thumb vs its track",
            "32",
            "(own page)",
            "rest",
            ratio(thumb, against_track),
            thumb,
            against_track,
        ));
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// A scene with a modal over a page: the page's content, and the modal when `modal`.
fn modal_scene(kind: &'static str, modal: bool, t: &'static Tokens) -> Element<'static, Msg> {
    let body: Element<'static, Msg> =
        iced::widget::column![text("Page text"), button::ghost(t, "Under", Msg::Go)].into();
    let mut l = AppLayout::new(body);
    if modal {
        let content: Element<'static, Msg> =
            iced::widget::column![text("Modal body"), button::primary(t, "OK", Msg::Go)]
                .spacing(8)
                .into();
        l = match kind {
            "dialog" => l.dialog(snora::Dialog::new(content)),
            _ => l.sheet(
                snora::Sheet::new(content)
                    .at(snora::SheetEdge::End)
                    .with_size(snora::SheetSize::OneThird),
            ),
        };
    }
    snora::design::render(l, t)
}

/// **The dim, the dialog and the sheet** (rows 33 to 36): the dimmed page against the page; and the dialog's and the
/// sheet's **boundary** (the stronger of fill and border) against the dimmed page, with the border alone beside it,
/// because the border alone is an exception (RFC-135): the fill is what a reader sees.
fn modal_cells(out: &mut Vec<Cell>) {
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        for kind in ["dialog", "sheet"] {
            let mut plain = Rig::new(modal_scene(kind, false, t));
            let f0 = plain.frame_scaled(&theme, EDGE as f32);
            let mut rig = Rig::new(modal_scene(kind, true, t));
            let els = elements(&mut rig);
            let f1 = rig.frame_scaled(&theme, EDGE as f32);
            let at = (100 * EDGE, 400 * EDGE);
            let (a, b) = (px(&f1, at.0, at.1), px(&f0, at.0, at.1));
            part.push(cell(
                preset,
                &format!("modal dim vs the page ({kind})"),
                "33/34",
                "(own page)",
                "rest",
                ratio(a, b),
                a,
                b,
            ));
            let card = if kind == "dialog" {
                els.iter()
                    .filter(|(k, r)| {
                        *k == El::Container
                            && r.width > 80.0
                            && r.width < 400.0
                            && r.height > 60.0
                            && r.height < 300.0
                            && r.x < 400.0
                            && r.x + r.width > 400.0
                            && r.y < 300.0
                            && r.y + r.height > 300.0
                    })
                    .max_by(|a, b| (a.1.width * a.1.height).total_cmp(&(b.1.width * b.1.height)))
                    .map(|x| x.1)
            } else {
                rect_of(&els, El::Container, 264.0, 600.0)
            };
            let card = card.unwrap_or_else(|| panic!("{preset}: the {kind}'s card was not found"));
            let sides = if kind == "dialog" {
                Sides::All
            } else {
                Sides::Left
            };
            let (row, what) = if kind == "dialog" {
                ("35", "dialog card")
            } else {
                ("36", "sheet")
            };
            if let Some((r, a, b)) = boundary(&f1, card, sides) {
                part.push(cell(
                    preset,
                    &format!("{what} vs the dimmed page"),
                    row,
                    "(own page)",
                    "rest",
                    r,
                    a,
                    b,
                ));
            }
            if let Some(r) = border_only(&f1, card, sides) {
                part.push(cell(
                    preset,
                    &format!("{what} border alone vs the dimmed page"),
                    row,
                    "(own page)",
                    "rest",
                    r,
                    [0; 3],
                    [0; 3],
                ));
            }
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// **The drop shadow** (row 40), against the page it falls on: decorative, an exception, measured so the exception is live.
fn shadow_cells(out: &mut Vec<Cell>) {
    let parts = par::by_preset(|preset, tokens| {
        let mut part = Vec::new();
        let t = leak(tokens);
        let theme = design_theme(t);
        let sc = scene_named("card, raised");
        for surface in SURFACES {
            let mut rig = Rig::new(build(&sc, t, surface, LayoutDirection::Ltr));
            let els = elements(&mut rig);
            let f = rig.frame_scaled(&theme, EDGE as f32);
            let card = sized(&els, (30.0, 200.0), (30.0, 80.0)).expect("the raised card");
            let x = ((card.x + card.width / 2.0) * EDGE as f32) as usize;
            let below = ((card.y + card.height) * EDGE as f32) as usize + 6;
            let far = below + 40 * EDGE;
            let (a, b) = (px(&f, x, below), px(&f, x, far));
            part.push(cell(
                preset,
                "drop shadow vs the page",
                "40",
                surface,
                "rest",
                ratio(a, b),
                a,
                b,
            ));
        }
        part
    });
    out.extend(parts.into_iter().flatten());
}

/// A glyph's name in the table: the close mark is **one** subject whatever it is drawn with (lucide's `X` under the
/// `lucide-icons` feature, a plain `×` without it), so the table holds in every feature combination CI builds.
fn glyph_name(why: &str, text: &str) -> String {
    if why == "close mark" {
        "close mark".to_owned()
    } else {
        format!("{text:?}")
    }
}

/// Glyphs that are not words (M-1): every one `text_sweep.rs`'s tables classify as a glyph, read at 2x (M-2).
fn glyph_cells(out: &mut Vec<Cell>) {
    let with_glyphs: Vec<Scene> = scenes()
        .into_iter()
        .filter(|s| s.runs.iter().any(|(_, k)| matches!(k, Kind::Glyph(_))))
        .collect();
    let s = sweep_of(with_glyphs, None);
    for c in s.cells {
        if let Some(Kind::Glyph(why)) = c.kind {
            out.push(Cell {
                preset: c.preset,
                subject: format!("glyph ({why}) {} in {}", glyph_name(why, &c.text), c.scene),
                row: "9/14/17/22/43",
                surface: c.surface,
                direction: c.direction,
                state: c.state,
                ratio: c.ratio,
                inside: c.ink,
                outside: c.bg,
            });
        }
    }
}

// ---- everything, the exceptions, and the checks ---------------------------------------------------------------------------------

/// Every non-text cell: the boundaries, the tints, the progress bar, the link's underline, the focus ring, the tooltips,
/// the scrollbar, the modals, the shadow and the glyphs. Drawn once and shared.
fn all_cells() -> &'static Vec<Cell> {
    static CELLS: std::sync::OnceLock<Vec<Cell>> = std::sync::OnceLock::new();
    CELLS.get_or_init(|| {
        let mut v = sweep_list(subjects());
        v.extend(sweep_list(tints()));
        progress_cells(&mut v);
        link_cells(&mut v);
        ring_cells(&mut v);
        tooltip_cells(&mut v);
        scrollbar_cells(&mut v);
        modal_cells(&mut v);
        shadow_cells(&mut v);
        glyph_cells(&mut v);
        par::dump_cells(
            "nontext_sweep",
            v.iter().map(|c| {
                format!(
                    "{}\t{}\t{}\t{}\t{:?}\t{}\t{:?}\t{}\t{}",
                    c.preset,
                    c.subject,
                    c.row,
                    c.surface,
                    c.direction,
                    c.state,
                    c.ratio,
                    hex(c.inside),
                    hex(c.outside)
                )
            }),
        );
        v
    })
}

/// **What is not a gap, and why** (RFC-135). A cell whose subject starts with one of these is not held to 3:1, and the
/// exception must be **live**: at least one of its cells is under 3:1, or the exception is stale and is removed.
const EXCEPTIONS: [(&str, &str); 6] = [
    (
        "dialog card border alone",
        "the dialog's fill against the dimmed page is the boundary a reader sees, and holds 3:1 (the rule takes the stronger of fill and border)",
    ),
    (
        "sheet border alone",
        "the sheet's fill against the dimmed page is the boundary a reader sees, and holds 3:1 (the rule takes the stronger of fill and border)",
    ),
    (
        "hover tint",
        "WCAG 1.4.11 does not ask a hover tint for 3:1; RFC-129's cue (the label's colour, a ΔE76 of 3) carries the state (text_floor.rs)",
    ),
    (
        "drop shadow",
        "decorative: no rule depends on it; the card's boundary is its border and its fill",
    ),
    (
        "tooltip border over content",
        "the tooltip floats over whatever is there and is content, not a control; its text holds its level (text_sweep.rs)",
    ),
    (
        "disabled",
        "exempt from WCAG 1.4.3 and 1.4.11 everywhere (not drawn by this sweep)",
    ),
];

fn exception_of(subject: &str) -> Option<usize> {
    EXCEPTIONS
        .iter()
        .position(|(prefix, _)| subject.starts_with(prefix))
}

/// **The sweep.** Every non-text subject, in every preset, on every surface, in each state and each direction, at 3:1:
/// before RFC-135 the progress track, a `Debug` toast, the menu's scrollbar and `card::raised`'s border were not.
#[test]
fn every_non_text_element_holds_3_to_1_as_drawn_in_both_directions() {
    let cells = all_cells();
    assert!(cells.len() > 3000, "premise: {} cells", cells.len());
    let held: Vec<&Cell> = cells
        .iter()
        .filter(|c| exception_of(&c.subject).is_none())
        .collect();
    let failures: Vec<String> = under(&held.iter().map(|c| (*c).clone()).collect::<Vec<_>>());
    let mut lowest: BTreeMap<String, (f32, String)> = BTreeMap::new();
    for c in &held {
        let e = lowest
            .entry(c.subject.clone())
            .or_insert((f32::MAX, String::new()));
        if c.ratio < e.0 {
            *e = (
                c.ratio,
                format!("{:.2}:1 on {} {} {}", c.ratio, c.preset, c.surface, c.state),
            );
        }
    }
    for (subject, (_, what)) in &lowest {
        println!("lowest | {subject:<70} | {what}");
    }
    println!(
        "{} non-text cells ({} held to 3:1), {} under",
        cells.len(),
        held.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {} cells under 3:1:\n  {}",
        failures.len(),
        held.len(),
        failures.join("\n  ")
    );
}

/// Each exception has its reason in writing, and is **live**: a cell of it is under 3:1. An exception that stopped being
/// needed fails here and is removed, so the list stays the list of what is not a gap.
#[test]
fn the_exceptions_are_written_down_and_each_is_still_needed() {
    let cells = all_cells();
    for (i, (prefix, reason)) in EXCEPTIONS.iter().enumerate() {
        assert!(reason.len() > 20, "{prefix}: a reason in words");
        if *prefix == "disabled" {
            continue; // not drawn by this sweep: nothing to measure
        }
        let mine: Vec<&Cell> = cells
            .iter()
            .filter(|c| exception_of(&c.subject) == Some(i))
            .collect();
        assert!(
            !mine.is_empty(),
            "{prefix}: no cell measured, so the exception names nothing"
        );
        let lowest = mine.iter().map(|c| c.ratio).fold(f32::MAX, f32::min);
        assert!(
            lowest + TOLERANCE < NON_TEXT,
            "{prefix}: its lowest cell is {lowest:.2}:1, so it is no longer an exception: remove it"
        );
    }
    // The cue the tints' exception leans on is still tested.
    let tf = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/text_floor.rs"),
    )
    .unwrap();
    assert!(
        tf.contains("fn hovered_and_pressed_each_differ_from_rest_by_a_visible_step"),
        "the hover tint exception leans on text_floor.rs' visible-step test"
    );
    // The dialog's and the sheet's reason: their boundary (fill or border) holds.
    for c in cells.iter().filter(|c| {
        c.subject.ends_with("vs the dimmed page") && !c.subject.contains("border alone")
    }) {
        assert!(
            c.ratio + TOLERANCE >= NON_TEXT,
            "{} / {}: {:.2}:1",
            c.preset,
            c.subject,
            c.ratio
        );
    }
}

// ---- tripwire 4 -----------------------------------------------------------------------------------------------------------------

/// The custom drawers of non-text elements: a file that calls `fill_quad(` itself draws something no `container` reports.
/// Today: the focus ring (`control.rs`), the link's underline (`underlined.rs`) and the slider's bar
/// (`slider_bar.rs`, RFC-134), each with its finder above (`ring_cells`, `link_cells`, and the slider's three
/// subjects, which read the track, the fill and the thumb the bar reports as containers).
const QUAD_DRAWERS: [&str; 3] = [
    "snora-interaction/src/control.rs",
    "snora-interaction/src/slider_bar.rs",
    "snora-interaction/src/underlined.rs",
];

fn quad_drawers(files: &[(String, String)]) -> Vec<String> {
    files
        .iter()
        .filter(|(_, src)| src.contains("fill_quad("))
        .map(|(p, _)| {
            p.rsplit("crates/")
                .next()
                .unwrap_or(p)
                .trim_start_matches("snora/../")
                .to_owned()
        })
        .filter(|p| !QUAD_DRAWERS.contains(&p.as_str()))
        .collect()
}

fn source_files() -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let name = p.to_string_lossy().to_string();
                if name.contains("/tests") || name.ends_with("tests.rs") {
                    continue;
                }
                out.push((name, std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for krate in [
        "snora",
        "snora-widgets",
        "snora-interaction",
        "snora-style",
        "snora-design",
        "snora-core",
    ] {
        walk(&root.join(krate).join("src"), &mut out);
    }
    out
}

/// **Tripwire 4.** A file that draws a quad itself has no finder in this sweep until one is written.
#[test]
fn tripwire_4_a_file_that_draws_a_quad_the_sweep_has_no_finder_for_fails() {
    let files = source_files();
    assert!(
        files.len() > 50,
        "premise: the sources were read ({} files)",
        files.len()
    );
    assert!(
        quad_drawers(&files).is_empty(),
        "these files draw a quad and have no finder: {:?}",
        quad_drawers(&files)
    );
    // THE PLANT: a widget drawing a rule with the renderer's primitive.
    let mut planted = files.clone();
    planted.push((
        "crates/snora-widgets/src/scratch_rule.rs".into(),
        "fn draw(r: &mut R) { r.fill_quad(q, bg); }".into(),
    ));
    assert_eq!(
        quad_drawers(&planted),
        vec!["snora-widgets/src/scratch_rule.rs".to_owned()]
    );
}

// ---- nothing that passes moves ---------------------------------------------------------------------------------------------------

/// The subjects RFC-135 changes **on purpose**, which the table of today's colours therefore leaves out: the four fixes
/// (the progress track, a `Debug` toast's border, the menu's scrollbar, `card::raised`'s border) and the tooltip bodies
/// that are `card_raised` too (the same defect: their border sits against the same shadow). Every other subject must draw
/// the colours it drew.
const CHANGED_ON_PURPOSE: [&str; 5] = [
    "toast fill vs the page (Debug)",
    "progress track",
    "scrollbar",
    "card::raised",
    "tooltip border vs the page",
];

fn in_the_table(c: &Cell) -> bool {
    c.direction == LayoutDirection::Ltr
        && exception_of(&c.subject).is_none()
        && !CHANGED_ON_PURPOSE.iter().any(|p| c.subject.starts_with(p))
}

fn key(c: &Cell) -> String {
    format!("{}\t{}\t{}\t{}", c.preset, c.subject, c.surface, c.state)
}

/// Writes the table (run once, on the code before the change):
/// `SNORA_NONTEXT_TABLE=path cargo test -p snora --all-features --test nontext_sweep record_the_table -- --ignored`.
#[test]
#[ignore = "records the table of today's colours; run by hand, on purpose"]
fn record_the_table() {
    let path = std::env::var("SNORA_NONTEXT_TABLE").expect("SNORA_NONTEXT_TABLE=path");
    let mut out = String::new();
    for c in all_cells().iter().filter(|c| in_the_table(c)) {
        let _ = writeln!(out, "{}\t{}\t{}", key(c), hex(c.inside), hex(c.outside));
    }
    std::fs::write(&path, out).unwrap();
}

fn unhex(s: &str) -> [u8; 3] {
    let v = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap();
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// **Nothing that passes moves** (RFC-135 R-2): the colours every passing subject drew **before** this RFC (recorded by
/// `record_the_table`, in `data/nontext_before_rfc135.tsv`), asserted again. The renderers round a blended edge pixel by
/// up to a unit, so a channel may differ by 3 of 255 and no more.
#[test]
fn nothing_that_passes_moves() {
    const TABLE: &str = include_str!("data/nontext_before_rfc135.tsv");
    let mut before: BTreeMap<String, ([u8; 3], [u8; 3])> = BTreeMap::new();
    for line in TABLE.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 6, "a row of the table: {line:?}");
        before.insert(
            format!("{}\t{}\t{}\t{}", f[0], f[1], f[2], f[3]),
            (unhex(f[4]), unhex(f[5])),
        );
    }
    assert!(
        before.len() > 1500,
        "premise: the table has {} rows",
        before.len()
    );
    let now: BTreeMap<String, &Cell> = all_cells()
        .iter()
        .filter(|c| in_the_table(c))
        .map(|c| (key(c), c))
        .collect();
    let near = |a: [u8; 3], b: [u8; 3]| {
        a.iter()
            .zip(b)
            .all(|(x, y)| (i32::from(*x) - i32::from(y)).abs() <= 3)
    };
    let mut moved = Vec::new();
    for (k, (i0, o0)) in &before {
        match now.get(k) {
            None => moved.push(format!("{k}: no longer drawn")),
            Some(c) if !(near(c.inside, *i0) && near(c.outside, *o0)) => moved.push(format!(
                "{k}: was {} against {}, is {} against {}",
                hex(*i0),
                hex(*o0),
                hex(c.inside),
                hex(c.outside)
            )),
            _ => {}
        }
    }
    assert!(
        moved.is_empty(),
        "{} cells moved:\n  {}",
        moved.len(),
        moved.join("\n  ")
    );
}
