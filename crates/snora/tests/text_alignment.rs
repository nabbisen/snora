//! RFC-133 R-4: **snora's own text sits at the start edge of its cell**, the left under LTR and the
//! right under RTL.
//!
//! A text that is as narrow as its words and sits in a wider cell is placed by the cell, not by the
//! reading order: under RTL the cell was mirrored and its text stayed at the left (the notice's
//! title and body, the progress label, the footer's content, the toast's title and message: 25 runs
//! in four widgets, RFC-133 Unit 0). The fix makes the text as wide as its cell and aligns it to the
//! start edge, so **the text's box no longer says where the words are**: this test reads the pixels.
//!
//! # What is measured
//!
//! For every text run of every scene in the text sweep (`common/sweep.rs`, so a new scene is
//! measured the day it is added), in both directions: the **cell** is the smallest container or focus
//! target that holds the run; the **ink** is the columns of the run's box whose pixels differ from its
//! background (the modal colour, as `read_run` takes it). The free space left and right of the ink
//! inside the cell is `(left, right)`.
//!
//! A run is **mirrored** when its free space swapped sides between LTR and RTL. A run is **wrong**
//! when it has more than [`ASYMMETRY`] px more room on one side than on the other, and did not
//! swap: it sits at the same physical edge of a mirrored cell.
//!
//! # The blind spot, stated
//!
//! A run that fills its cell, or sits in it with the same room either side (a centred label, a button's
//! text), reads "tight" and cannot be wrong by this measure. A one-word line that fills its cell is
//! therefore never caught here, and a text that is **centred on purpose** is not a defect. The
//! threshold is [`ASYMMETRY`] px because 8-bit anti-aliasing and a glyph's side bearings move the
//! ink by a pixel or two.
//!
//! Application text is not measured (`Kind::App`): snora does not align what it did not draw.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use common::sweep::{Kind, SCALE, build, design_theme, kind_of, leak, runs, scenes};
use iced::Rectangle;
use iced_runtime::core::widget::{Id, Operation};
use snora::LayoutDirection::{Ltr, Rtl};
use snora::design::Tokens;

/// More room than this on one side than on the other is an alignment, not a glyph's bearing.
const ASYMMETRY: f32 = 6.0;
/// The free space may differ by this much between a run and its reflection.
const SWAP_TOLERANCE: f32 = 3.0;

/// Every container and every focus target: the cells a text can sit in.
fn cells(rig: &mut Rig<common::sweep::Msg>) -> Vec<Rectangle> {
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
    let mut v = out.lock().unwrap().clone();
    v.extend(rig.targets().into_iter().map(|t| t.bounds));
    v
}

/// The columns (in logical pixels, relative to nothing) where `bounds` holds ink: the first and the
/// one after the last. `None` if the box holds none.
fn ink_columns(frame: &[u8], b: Rectangle) -> Option<(f32, f32)> {
    let s = SCALE as f32;
    let get = |x: usize, y: usize| {
        let i = (y * 800 * SCALE + x) * 4;
        [frame[i], frame[i + 1], frame[i + 2]]
    };
    let (x0, y0) = ((b.x.max(0.0) * s) as usize, (b.y.max(0.0) * s) as usize);
    let x1 = (((b.x + b.width) * s).ceil() as usize).min(800 * SCALE - 1);
    let y1 = (((b.y + b.height) * s).ceil() as usize).min(600 * SCALE - 1);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let mut counts: std::collections::HashMap<[u8; 3], usize> = std::collections::HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            *counts.entry(get(x, y)).or_default() += 1;
        }
    }
    let bg = counts
        .iter()
        .max_by_key(|(c, n)| (**n, **c))
        .map(|(c, _)| *c)?;
    let differs = |p: [u8; 3]| common::sweep::ratio(p, bg) > 1.05;
    let first = (x0..x1).find(|x| (y0..y1).any(|y| differs(get(*x, y))))?;
    let last = (x0..x1)
        .rev()
        .find(|x| (y0..y1).any(|y| differs(get(*x, y))))?;
    Some((first as f32 / s, (last + 1) as f32 / s))
}

/// One run, in one direction: the text, and the free space `(left, right)` it leaves in its cell.
type Slack = (String, f32, f32);

fn slack(sc: &common::sweep::Scene, t: &'static Tokens, dir: snora::LayoutDirection) -> Vec<Slack> {
    let theme = design_theme(t);
    let surface = if sc.own_page {
        "(own page)"
    } else {
        "background"
    };
    let mut rig = Rig::new(build(sc, t, surface, dir));
    let found = runs(&mut rig);
    let boxes = cells(&mut rig);
    let frame = rig.frame_scaled(&theme, SCALE as f32);
    found
        .into_iter()
        .filter(|(s, _)| kind_of(sc, s) != Some(Kind::App))
        .filter_map(|(s, b)| {
            let (ink_l, ink_r) = ink_columns(&frame, b)?;
            // The tightest box that holds the run.
            let cell = boxes
                .iter()
                .filter(|c| {
                    c.x <= b.x + 0.5
                        && c.y <= b.y + 0.5
                        && c.x + c.width >= b.x + b.width - 0.5
                        && c.y + c.height >= b.y + b.height - 0.5
                })
                .min_by(|a, c| (a.width * a.height).total_cmp(&(c.width * c.height)))?;
            Some((s, ink_l - cell.x, (cell.x + cell.width) - ink_r))
        })
        .collect()
}

/// The runs that are at the same physical edge of a mirrored cell under both directions.
fn same_edge_under_rtl(ltr: &[Slack], rtl: &[Slack]) -> Vec<String> {
    assert_eq!(
        ltr.len(),
        rtl.len(),
        "a different number of runs under the two directions"
    );
    ltr.iter()
        .zip(rtl)
        .filter_map(|((s, ll, lr), (s2, rl, rr))| {
            // The breadcrumb's separator is the one glyph that flips with the direction (RFC-132
            // R-4): `›` under LTR, `‹` under RTL. Every other run is the same text in the same place
            // in the tree, since the order is logical.
            let flipped_separator = matches!((s.as_str(), s2.as_str()), ("›", "‹") | ("‹", "›"));
            assert!(
                s == s2 || flipped_separator,
                "the runs are in a different order under the two directions: {s:?} and {s2:?}"
            );
            let asymmetric = (ll - lr).abs() > ASYMMETRY;
            let swapped = (rl - lr).abs() <= SWAP_TOLERANCE && (rr - ll).abs() <= SWAP_TOLERANCE;
            (asymmetric && !swapped).then(|| {
                format!("{s:?}: free space {ll:.0}|{lr:.0} under LTR and {rl:.0}|{rr:.0} under RTL")
            })
        })
        .collect()
}

#[test]
fn every_snora_text_run_sits_at_the_start_edge_of_its_cell_under_rtl() {
    let t = leak(Tokens::light());
    let mut wrong = Vec::new();
    let mut measured = 0;
    let mut asymmetric = 0;
    for sc in scenes() {
        // A card is the caller's content; its text is the application's.
        if sc.name.starts_with("card,") {
            continue;
        }
        let (ltr, rtl) = (slack(&sc, t, Ltr), slack(&sc, t, Rtl));
        measured += ltr.len();
        asymmetric += ltr
            .iter()
            .filter(|(_, l, r)| (l - r).abs() > ASYMMETRY)
            .count();
        wrong.extend(
            same_edge_under_rtl(&ltr, &rtl)
                .into_iter()
                .map(|w| format!("{} / {w}", sc.name)),
        );
    }
    assert!(
        measured > 50,
        "only {measured} runs were measured: the scenes are not being read"
    );
    assert!(
        asymmetric > 20,
        "only {asymmetric} runs have room to one side: the test has stopped being able to see a wrong edge"
    );
    assert!(
        wrong.is_empty(),
        "{} text runs sit at the same physical edge of a mirrored cell under RTL (RFC-133 R-4):\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

/// The test's own premise: it finds a run that did not move. A text left at the physical left of a
/// cell under both directions is exactly what `same_edge_under_rtl` exists to report.
#[test]
fn a_run_that_keeps_its_physical_edge_is_reported() {
    let ltr = vec![
        ("Notice title".to_owned(), 0.0, 102.0),
        ("Tag".to_owned(), 8.0, 8.0),
    ];
    let kept = vec![
        ("Notice title".to_owned(), 0.0, 102.0),
        ("Tag".to_owned(), 8.0, 8.0),
    ];
    let swapped = vec![
        ("Notice title".to_owned(), 102.0, 0.0),
        ("Tag".to_owned(), 8.0, 8.0),
    ];
    assert_eq!(
        same_edge_under_rtl(&ltr, &kept).len(),
        1,
        "the title kept the left edge"
    );
    assert!(
        same_edge_under_rtl(&ltr, &swapped).is_empty(),
        "swapped, and a centred tag, are fine"
    );
}
