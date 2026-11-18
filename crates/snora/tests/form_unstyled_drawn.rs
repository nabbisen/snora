//! RFC-134: **the five form controls on the engine path hold 3:1 as drawn, on every stock theme**, at
//! rest and hovered and pressed.
//!
//! The engine path is coloured by whatever iced theme the application has, so its marks cannot be
//! swept over six presets: they are read here over all 22 stock themes (`Theme::ALL`). The marks'
//! colours are derived from the theme (`ep.background.base.text` for an outline, `primary.strong`
//! for an accent) and **held where they do not read** by the rule the focus ring uses (the stronger
//! of black and white); this is what shows the rule does it, from pixels.
//!
//! What is read is what `nontext_sweep.rs` reads on the design path: **a boundary** is a mark's fill
//! or its border, whichever is stronger, against what is beside it, on the weakest side of its
//! outline, at 4x (a 1 px rule is four solid pixels there). The label is the host's own text on the
//! theme's own background, which is iced's to keep readable and not snora's to sweep.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::par;
use common::removal::Rig;
use common::sweep::{Msg, NON_TEXT, State, apply_state, ratio};
use iced::widget::{column, container};
use iced::{Element, Length, Rectangle, Theme};
use iced_runtime::core::widget::{Id, Operation};
use snora::LayoutDirection::{self, Ltr};

const EDGE: usize = 4;
const TOLERANCE: f32 = 0.02;

fn px(frame: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * 800 * EDGE + x) * 4;
    [frame[i], frame[i + 1], frame[i + 2]]
}

fn containers(rig: &mut Rig<Msg>) -> Vec<Rectangle> {
    struct Collect(Arc<Mutex<Vec<Rectangle>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
            self.0.lock().unwrap().push(bounds);
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    out.lock().unwrap().clone()
}

/// The weakest side of `r`: its border or its fill, whichever is stronger, against the pixel just outside.
fn weakest_side(frame: &[u8], r: Rectangle, vertical_only: bool) -> f32 {
    let s = EDGE as f32;
    let (x0, y0, x1, y1) = (
        (r.x * s) as i64,
        (r.y * s) as i64,
        ((r.x + r.width) * s) as i64,
        ((r.y + r.height) * s) as i64,
    );
    let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
    let fill_inset = (3.0 * s) as i64;
    let mut worst = f32::MAX;
    for side in 0..4 {
        if vertical_only && side >= 2 {
            continue;
        }
        let (border, fill, outside) = match side {
            0 => ((mx, y0 + 1), (mx, y0 + fill_inset), (mx, y0 - 2)),
            1 => ((mx, y1 - 2), (mx, y1 - fill_inset), (mx, y1 + 1)),
            2 => ((x0 + 1, my), (x0 + fill_inset, my), (x0 - 2, my)),
            _ => ((x1 - 2, my), (x1 - fill_inset, my), (x1 + 1, my)),
        };
        let at = |p: (i64, i64)| px(frame, p.0 as usize, p.1 as usize);
        // A rule too thin to have a fill reads its border; a fill that is as thin as the mark is not read.
        let thick = match side {
            0 | 1 => r.height * s > 2.0 * fill_inset as f32,
            _ => r.width * s > 2.0 * fill_inset as f32,
        };
        let mut q = ratio(at(border), at(outside));
        if thick {
            q = q.max(ratio(at(fill), at(outside)));
        }
        worst = worst.min(q);
    }
    worst
}

// ---- finders: the marks at the host's text size (an em is 16) ----------------------------------------------------------------

fn nth_square(els: &[Rectangle], n: usize) -> Option<Rectangle> {
    let mut v: Vec<Rectangle> = els
        .iter()
        .filter(|r| (r.width - r.height).abs() < 0.6 && (19.0..21.0).contains(&r.width))
        .copied()
        .collect();
    v.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    v.dedup_by(|a, b| (a.y - b.y).abs() < 1.0 && (a.x - b.x).abs() < 1.0);
    v.get(n).copied()
}

fn track(els: &[Rectangle]) -> Option<Rectangle> {
    els.iter()
        .find(|r| (19.0..21.0).contains(&r.height) && (r.width - 2.0 * r.height).abs() < 1.0)
        .copied()
}

fn thumb(els: &[Rectangle]) -> Option<Rectangle> {
    els.iter()
        .find(|r| (r.width - r.height).abs() < 0.6 && (13.0..15.0).contains(&r.width))
        .copied()
}

fn disc(els: &[Rectangle], n: usize) -> Option<Rectangle> {
    let mut v: Vec<Rectangle> = els
        .iter()
        .filter(|r| (r.width - r.height).abs() < 0.6 && (11.0..13.0).contains(&r.width))
        .copied()
        .collect();
    v.sort_by(|a, b| a.y.total_cmp(&b.y));
    v.dedup_by(|a, b| (a.y - b.y).abs() < 1.0);
    v.get(n).copied()
}

fn bar(els: &[Rectangle], keep: impl Fn(&Rectangle) -> bool) -> Option<Rectangle> {
    els.iter()
        .find(|r| r.height > 4.0 && r.height < 8.0 && keep(r))
        .copied()
}

struct Subject {
    name: &'static str,
    make: fn(LayoutDirection) -> Element<'static, Msg>,
    /// Which rectangle, and whether only its top and bottom count (a track whose ends are under the thumb).
    pick: fn(&[Rectangle]) -> Option<(Rectangle, bool)>,
}

fn subjects() -> Vec<Subject> {
    vec![
        Subject {
            name: "checkbox box (checked)",
            make: |d| {
                snora::checkbox(true, d)
                    .label("Remember me")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| nth_square(e, 0).map(|r| (r, false)),
        },
        Subject {
            name: "checkbox box (unchecked)",
            make: |d| {
                snora::checkbox(false, d)
                    .label("Subscribe")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| nth_square(e, 0).map(|r| (r, false)),
        },
        Subject {
            name: "toggler track (on)",
            make: |d| {
                snora::toggler(true, d)
                    .label("Dark mode")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| track(e).map(|r| (r, false)),
        },
        Subject {
            name: "toggler track (off)",
            make: |d| {
                snora::toggler(false, d)
                    .label("Dark mode")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| track(e).map(|r| (r, false)),
        },
        Subject {
            name: "toggler thumb (on)",
            make: |d| {
                snora::toggler(true, d)
                    .label("Dark mode")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| thumb(e).map(|r| (r, false)),
        },
        Subject {
            name: "toggler thumb (off)",
            make: |d| {
                snora::toggler(false, d)
                    .label("Dark mode")
                    .on_toggle(|_| Msg::Go)
                    .into()
            },
            pick: |e| thumb(e).map(|r| (r, false)),
        },
        Subject {
            name: "radio ring (unselected)",
            make: |d| {
                snora::radio_group(["Small", "Medium"], Some(&"Medium"), |_| Msg::Go, d).into()
            },
            pick: |e| nth_square(e, 0).map(|r| (r, false)),
        },
        Subject {
            name: "radio ring (selected)",
            make: |d| {
                snora::radio_group(["Small", "Medium"], Some(&"Medium"), |_| Msg::Go, d).into()
            },
            pick: |e| nth_square(e, 1).map(|r| (r, false)),
        },
        Subject {
            name: "radio dot (selected)",
            make: |d| {
                snora::radio_group(["Small", "Medium"], Some(&"Medium"), |_| Msg::Go, d).into()
            },
            pick: |e| disc(e, 1).map(|r| (r, false)),
        },
        Subject {
            name: "slider track",
            make: |d| {
                snora::slider(0.0..=100.0, 35.0, |_| Msg::Go, d)
                    .width(Length::Fixed(300.0))
                    .into()
            },
            pick: |e| bar(e, |r| r.width > 100.0).map(|r| (r, true)),
        },
        Subject {
            name: "slider thumb",
            make: |d| {
                snora::slider(0.0..=100.0, 35.0, |_| Msg::Go, d)
                    .width(Length::Fixed(300.0))
                    .into()
            },
            pick: |e| nth_square(e, 0).map(|r| (r, false)),
        },
        Subject {
            name: "select trigger",
            make: |d| {
                snora::select(["Apple", "Banana"], Some(&"Apple"), |_| Msg::Go, d)
                    .on_open_change(|_| Msg::Go)
                    .into()
            },
            pick: |e| {
                // The control reports its own box (twice: its own and the one it is laid out in); the outline is
                // the widest container inside it, then the face's own content. So: the widest that is narrower
                // than the widest.
                let of_size: Vec<&Rectangle> = e
                    .iter()
                    .filter(|r| {
                        r.width > 60.0 && r.width < 300.0 && r.height > 20.0 && r.height < 40.0
                    })
                    .collect();
                let widest = of_size.iter().map(|r| r.width).fold(0.0, f32::max);
                of_size
                    .into_iter()
                    .filter(|r| r.width < widest - 1.0)
                    .max_by(|a, b| a.width.total_cmp(&b.width))
                    .map(|r| (*r, false))
            },
        },
    ]
}

fn page(el: Element<'static, Msg>) -> Element<'static, Msg> {
    container(column![el])
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[test]
fn every_mark_of_the_five_holds_3_to_1_on_every_stock_theme_in_every_state() {
    // One theme to a thread (`common/par.rs`), put back in theme order: the lines come out as one thread drew them.
    let themes: Vec<&Theme> = Theme::ALL.iter().collect();
    let parts = par::map(&themes, |theme| {
        let theme: &Theme = theme;
        let mut lines = Vec::new();
        for sub in subjects() {
            // One direction: a colour does not depend on it (the text sweep pairs them), and what a
            // direction moves is read by `form_controls.rs`' mirror test.
            for dir in [Ltr] {
                for state in [State::Rest, State::Hovered, State::Pressed] {
                    let mut rig = Rig::new(page((sub.make)(dir)));
                    let els = containers(&mut rig);
                    let (r, vertical) = (sub.pick)(&els).unwrap_or_else(|| {
                        panic!("{theme} / {} / {dir:?}: the mark was not found", sub.name)
                    });
                    if state != State::Rest {
                        let target = rig.targets()[0].bounds;
                        apply_state(&mut rig, target, state);
                    }
                    let frame = rig.frame_scaled(theme, EDGE as f32);
                    let q = weakest_side(&frame, r, vertical);
                    lines.push((format!("{theme} / {} / {dir:?} / {state:?}", sub.name), q));
                }
            }
        }
        lines
    });
    let mut failures = Vec::new();
    let mut cells = 0;
    let mut lowest = (f32::MAX, String::new());
    let mut dump = Vec::new();
    for (what, q) in parts.into_iter().flatten() {
        cells += 1;
        dump.push(format!("{what}\t{q:?}"));
        let what = format!("{what}: {q:.2}:1");
        if q < lowest.0 {
            lowest = (q, what.clone());
        }
        if q + TOLERANCE < NON_TEXT {
            failures.push(what);
        }
    }
    par::dump_cells("form_unstyled_drawn", dump);
    println!("{cells} cells; the lowest is {}", lowest.1);
    assert!(cells > 750, "premise: {cells} cells read");
    assert!(
        failures.is_empty(),
        "{} cells under 3:1:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}
