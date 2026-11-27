//! RFC-138: **a radio group in a row**, `horizontal(bool)` on `snora::radio_group` and
//! `snora::design::radio_group`.
//!
//! A horizontal group is a row that always wraps: the options keep their logical order, line by line, at the
//! start edge of each line; under RTL each line is its LTR mirror image and the lines break at the same options;
//! the lines stack at the column's pitch; and the space between options is wider than the space between a mark
//! and its own label. The keys are the column's. `horizontal(false)` is today's group, pixel for pixel.
//!
//! **Everything is read from what a click reaches and from the drawn ink**, never from how the group is built:
//! an option's pointer target is found by clicking (no option is selected, so every option publishes), and the
//! gaps are read from pixels at 2x. **The gap test does not read the space the code sets** (`Skin::between`): it
//! measures both gaps from ink and asserts their ratio, so it cannot agree with the code by construction.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); a wgpu run must use `--test-threads=1`.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use common::removal_scenes::{Msg, msg, page, tokens};
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::{Element, Point, Rectangle, Size};
use snora::LayoutDirection::{self, Ltr, Rtl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Path {
    Engine,
    Design,
}

const PATHS: [Path; 2] = [Path::Engine, Path::Design];
const DIRECTIONS: [LayoutDirection; 2] = [Ltr, Rtl];
const LABELS: [&str; 6] = [
    "Small",
    "Medium",
    "Large",
    "Huge",
    "Extra large",
    "Gigantic",
];
const SHORT: [&str; 3] = ["List", "Grid", "Map"];
const SCALE: u32 = 2;

fn labels(count: usize, short: bool) -> &'static [&'static str] {
    if short {
        &SHORT[..count]
    } else {
        &LABELS[..count]
    }
}

/// The group inside the engine's page, in a window `width` wide. `horizontal` is `None` for "no call at all".
fn group(
    path: Path,
    labels: &'static [&'static str],
    dir: LayoutDirection,
    horizontal: Option<bool>,
    selected: Option<usize>,
) -> Element<'static, Msg> {
    let pick = |s: &'static str| msg(s);
    let sel = selected.map(|i| labels[i]);
    let g = match path {
        Path::Engine => snora::radio_group(labels.to_vec(), sel.as_ref(), pick, dir),
        Path::Design => {
            snora::design::radio_group(tokens(), labels.to_vec(), sel.as_ref(), pick, dir)
        }
    };
    let g = match horizontal {
        Some(h) => g.horizontal(h),
        None => g,
    };
    page(container(g).padding(16).into(), dir)
}

fn theme(path: Path) -> iced::Theme {
    match path {
        Path::Engine => iced::Theme::Light,
        Path::Design => snora::design::theme(tokens()),
    }
}

fn rig(view: Element<'static, Msg>, width: f32) -> Rig<Msg> {
    if width >= 800.0 {
        Rig::new(view)
    } else {
        Rig::sized(view, Size::new(width, 400.0))
    }
}

/// Which option a click at `(x, y)` selects (None selected, so every option publishes), by what the
/// click publishes: the pointer's hand covers the whole control, so it cannot tell options apart.
fn hit(rig: &mut Rig<Msg>, x: f32, y: f32) -> Option<String> {
    rig.messages.clear();
    rig.click_at(Point::new(x, y));
    let m = rig.messages.last().map(|m| m.0.clone());
    rig.messages.clear();
    m
}

/// The option pointer targets: connected regions of one answer, edges refined at one pixel.
fn targets(rig: &mut Rig<Msg>, g: Rectangle) -> Vec<(usize, Rectangle)> {
    let step = 2usize;
    let x0 = (g.x - 4.0).floor().max(0.0) as usize;
    let y0 = (g.y - 4.0).floor().max(0.0) as usize;
    let x1 = (g.x + g.width + 4.0).ceil() as usize;
    let y1 = (g.y + g.height + 4.0).ceil() as usize;
    let (w, h) = ((x1 - x0) / step + 1, (y1 - y0) / step + 1);
    let mut cell: Vec<Option<String>> = vec![None; w * h];
    for cy in 0..h {
        for cx in 0..w {
            cell[cy * w + cx] = hit(
                rig,
                (x0 + cx * step) as f32 + 0.5,
                (y0 + cy * step) as f32 + 0.5,
            );
        }
    }
    let mut seen = vec![false; w * h];
    let mut found: Vec<(String, Rectangle)> = Vec::new();
    for start in 0..w * h {
        let Some(name) = cell[start].clone() else {
            continue;
        };
        if seen[start] {
            continue;
        }
        let (mut minx, mut maxx, mut miny, mut maxy) = (usize::MAX, 0, usize::MAX, 0);
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(c) = stack.pop() {
            let (cx, cy) = (c % w, c / w);
            minx = minx.min(cx);
            maxx = maxx.max(cx);
            miny = miny.min(cy);
            maxy = maxy.max(cy);
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                    continue;
                }
                let n = ny as usize * w + nx as usize;
                if cell[n].as_deref() == Some(name.as_str()) && !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        let (l, r) = ((x0 + minx * step) as f32, (x0 + maxx * step) as f32);
        let (t, b) = ((y0 + miny * step) as f32, (y0 + maxy * step) as f32);
        let (cxm, cym) = ((l + r) / 2.0, (t + b) / 2.0);
        let is =
            |rig: &mut Rig<Msg>, x: f32, y: f32| hit(rig, x, y).as_deref() == Some(name.as_str());
        let mut left = l;
        for d in (-2..=0).rev() {
            if is(rig, l + d as f32 + 0.5, cym) {
                left = l + d as f32;
            }
        }
        let mut right = r;
        for d in 0..=2 {
            if is(rig, r + d as f32 + 0.5, cym) {
                right = r + d as f32;
            }
        }
        let mut top = t;
        for d in (-2..=0).rev() {
            if is(rig, cxm, t + d as f32 + 0.5) {
                top = t + d as f32;
            }
        }
        let mut bottom = b;
        for d in 0..=2 {
            if is(rig, cxm, b + d as f32 + 0.5) {
                bottom = b + d as f32;
            }
        }
        found.push((
            name,
            Rectangle::new(
                Point::new(left, top),
                Size::new(right - left + 1.0, bottom - top + 1.0),
            ),
        ));
    }
    found.sort_by(|a, b| {
        let (ra, rb) = (a.1, b.1);
        if (ra.y - rb.y).abs() > 4.0 {
            ra.y.partial_cmp(&rb.y).unwrap()
        } else {
            ra.x.partial_cmp(&rb.x).unwrap()
        }
    });
    found
        .into_iter()
        .map(|(name, r)| {
            let i = LABELS
                .iter()
                .position(|l| *l == name)
                .or_else(|| SHORT.iter().position(|l| *l == name))
                .unwrap_or(usize::MAX);
            (i, r)
        })
        .collect()
}

fn lines(opts: &[(usize, Rectangle)]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut last_y = f32::NEG_INFINITY;
    for (i, r) in opts {
        if (r.y - last_y).abs() > 4.0 {
            out.push(Vec::new());
            last_y = r.y;
        }
        out.last_mut().unwrap().push(*i);
    }
    out
}

struct Ink {
    px: Vec<u8>,
    w: usize,
    bg: [u8; 3],
}

impl Ink {
    fn new(px: Vec<u8>, window_w: usize, at: (usize, usize)) -> Self {
        let w = window_w * SCALE as usize;
        let i = (at.1 * SCALE as usize * w + at.0 * SCALE as usize) * 4;
        Self {
            bg: [px[i], px[i + 1], px[i + 2]],
            px,
            w,
        }
    }
    fn on(&self, x: usize, y: usize) -> bool {
        let i = (y * self.w + x) * 4;
        let d: i32 = (0..3)
            .map(|c| (i32::from(self.px[i + c]) - i32::from(self.bg[c])).abs())
            .sum();
        d > 60
    }
    /// The columns (in logical px) with ink in `y0..y1` (logical), as runs `(start, end)`.
    fn runs(&self, x0: f32, x1: f32, y0: f32, y1: f32) -> Vec<(f32, f32)> {
        let s = SCALE as f32;
        let (xa, xb) = ((x0 * s) as usize, (x1 * s) as usize);
        let (ya, yb) = ((y0 * s) as usize, (y1 * s) as usize);
        let mut runs: Vec<(usize, usize)> = Vec::new();
        for x in xa..xb {
            if (ya..yb).any(|y| self.on(x, y)) {
                match runs.last_mut() {
                    Some(r) if x == r.1 + 1 => r.1 = x,
                    _ => runs.push((x, x)),
                }
            }
        }
        runs.into_iter()
            .map(|(a, b)| (a as f32 / s, (b + 1) as f32 / s))
            .collect()
    }
}

/// Ink runs merged where closer than `join` px (letters of one label), as `(start, end)`.
fn merged(runs: Vec<(f32, f32)>, join: f32) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = Vec::new();
    for r in runs {
        match out.last_mut() {
            Some(l) if r.0 - l.1 < join => l.1 = r.1,
            _ => out.push(r),
        }
    }
    out
}

fn sorted_lines(opts: &[(usize, Rectangle)]) -> Vec<Vec<usize>> {
    lines(opts)
        .into_iter()
        .map(|mut l| {
            l.sort_unstable();
            l
        })
        .collect()
}

/// The group's focus target (one Tab stop) and every option's pointer target, for one case.
fn measure(
    path: Path,
    ls: &'static [&'static str],
    width: f32,
    dir: LayoutDirection,
    horizontal: Option<bool>,
) -> (Rectangle, Vec<(usize, Rectangle)>) {
    let mut r = rig(group(path, ls, dir, horizontal, None), width);
    let g = r.targets()[0].bounds;
    let opts = targets(&mut r, g);
    (g, opts)
}

// ---- horizontal ----------------------------------------------------------------------------------------------------------------

/// **Horizontal: the options are drawn on one line when there is room**, in logical order: the first at the start edge,
/// each next one further along it, which is to the right under LTR and to the left under RTL. And the setting is what
/// does it: the same group with `horizontal(false)` is three lines.
#[test]
fn the_options_are_drawn_on_one_line_when_there_is_room() {
    for path in PATHS {
        for dir in DIRECTIONS {
            let ls = labels(3, false);
            let (_, row) = measure(path, ls, 800.0, dir, Some(true));
            assert_eq!(row.len(), 3, "{path:?} {dir:?}: three pointer targets");
            assert_eq!(
                sorted_lines(&row),
                vec![vec![0, 1, 2]],
                "{path:?} {dir:?}: three options and room for them are one line"
            );
            let mut by_index = row.clone();
            by_index.sort_by_key(|(i, _)| *i);
            for pair in by_index.windows(2) {
                let (a, b) = (pair[0].1, pair[1].1);
                let forward = match dir {
                    Ltr => b.x > a.x + a.width - 0.5,
                    Rtl => b.x + b.width < a.x + 0.5,
                };
                assert!(
                    forward && (a.y - b.y).abs() < 1.0,
                    "{path:?} {dir:?}: option {} should follow option {} along the start edge on the same line: {a:?} then {b:?}",
                    pair[1].0,
                    pair[0].0
                );
            }
            let (_, column) = measure(path, ls, 800.0, dir, Some(false));
            assert_eq!(
                sorted_lines(&column),
                vec![vec![0], vec![1], vec![2]],
                "{path:?} {dir:?}: `horizontal(false)` is the column: one option to a line"
            );
        }
    }
}

// ---- narrow --------------------------------------------------------------------------------------------------------------------

/// **Narrow: the row wraps, nothing is clipped, and the lines are at the column's pitch.** The column's own pitch is
/// measured on the same path, so the test follows the design's tokens and the host's text size alike.
#[test]
fn a_narrow_room_wraps_nothing_is_clipped_and_the_lines_are_at_the_columns_pitch() {
    for path in PATHS {
        for dir in DIRECTIONS {
            let (_, column) = measure(path, labels(3, false), 800.0, dir, Some(false));
            let ys: Vec<f32> = column.iter().map(|(_, r)| r.y).collect();
            let pitch = ys[1] - ys[0];
            // **Every option must fit the room on its own, with margin, on any default font.** An option wider than the
            // room wraps its own label (the limit the doc comment states), which makes its line taller, and then the
            // lines are not at the column's pitch: CI's wider default font did that to the six-label set at 160 px,
            // where "Extra large" is within a few pixels of the room on this machine's font. So the narrowest room is
            // used only with the three short-enough labels.
            for (count, width) in [(6usize, 300.0f32), (3, 160.0)] {
                let ls = labels(count, false);
                let (g, opts) = measure(path, ls, width, dir, Some(true));
                let found = lines(&opts);
                assert!(
                    found.len() >= 2,
                    "{path:?} {dir:?} at {width} px: {count} options must wrap, got {found:?}"
                );
                assert_eq!(
                    opts.len(),
                    count,
                    "{path:?} {dir:?} at {width} px: every option has a target"
                );
                // Nothing is clipped: every option is inside the window and the group's box.
                for (i, r) in &opts {
                    assert!(
                        r.x >= -0.5 && r.x + r.width <= width + 0.5,
                        "{path:?} {dir:?} at {width} px: option {i} is clipped by the window: {r:?}"
                    );
                    assert!(
                        r.x >= g.x - 0.5 && r.x + r.width <= g.x + g.width + 0.5,
                        "{path:?} {dir:?} at {width} px: option {i} is outside its group {g:?}: {r:?}"
                    );
                }
                // The lines are at the column's pitch.
                let mut line_ys: Vec<f32> = Vec::new();
                for (_, r) in &opts {
                    if !line_ys.iter().any(|y| (y - r.y).abs() < 1.0) {
                        line_ys.push(r.y);
                    }
                }
                line_ys.sort_by(f32::total_cmp);
                for pair in line_ys.windows(2) {
                    assert!(
                        (pair[1] - pair[0] - pitch).abs() <= 1.0,
                        "{path:?} {dir:?} at {width} px: lines are {} px apart, the column's pitch is {pitch} px",
                        pair[1] - pair[0]
                    );
                }
            }
        }
    }
}

/// **The ring encloses every option and hugs the drawn lines**, not the room the parent gave: read as the frame
/// difference of the group focused against not.
#[test]
fn the_ring_encloses_every_option_and_hugs_the_widest_line() {
    for path in PATHS {
        let ls = labels(6, false);
        let width = 300.0;
        let (_, opts) = measure(path, ls, width, Ltr, Some(true));
        let th = theme(path);
        let mut clean = rig(group(path, ls, Ltr, Some(true), None), width);
        clean.point_at(None);
        clean.redraw();
        let a = clean.frame_scaled(&th, SCALE as f32);
        let mut focused = rig(group(path, ls, Ltr, Some(true), None), width);
        focused.tab(false);
        focused.point_at(None);
        focused.redraw();
        let b = focused.frame_scaled(&th, SCALE as f32);
        let fw = width as usize * SCALE as usize;
        let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0usize, usize::MAX, 0usize);
        for (i, (p, q)) in a.chunks(4).zip(b.chunks(4)).enumerate() {
            if p != q {
                let (x, y) = (i % fw, i / fw);
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
        assert!(x0 != usize::MAX, "{path:?}: no ring was drawn");
        let s = SCALE as f32;
        let ring = Rectangle::new(
            Point::new(x0 as f32 / s, y0 as f32 / s),
            Size::new((x1 + 1 - x0) as f32 / s, (y1 + 1 - y0) as f32 / s),
        );
        for (i, r) in &opts {
            assert!(
                r.x >= ring.x - 0.5
                    && r.y >= ring.y - 0.5
                    && r.x + r.width <= ring.x + ring.width + 0.5
                    && r.y + r.height <= ring.y + ring.height + 0.5,
                "{path:?}: the ring {ring:?} does not enclose option {i} {r:?}"
            );
        }
        let widest = opts.iter().map(|(_, r)| r.x + r.width).fold(0.0, f32::max);
        assert!(
            ring.x + ring.width <= widest + 16.0,
            "{path:?}: the ring {ring:?} reaches well past the widest line (right edge {widest}): it fills the room, not the lines"
        );
    }
}

// ---- mirrored ------------------------------------------------------------------------------------------------------------------

/// **Mirrored: under RTL each line is the mirror image of its LTR line, and the lines break at the same options.** The
/// mirror law is the one `mirrored_layout.rs` uses, read relative to the group's own box: an option at `x`, `w` wide, is
/// at `box width - x - w` under RTL. A shorter line then starts from the right edge, which is R-2. The cases are chosen
/// so that some line is shorter than the first.
#[test]
fn under_rtl_each_line_is_the_mirror_image_and_breaks_at_the_same_options() {
    for path in PATHS {
        let mut shorter_line_seen = false;
        for (count, width) in [(3usize, 300.0f32), (6, 300.0), (3, 160.0)] {
            let ls = labels(count, false);
            let (gl, l) = measure(path, ls, width, Ltr, Some(true));
            let found = lines(&l);
            let first = found.first().map_or(0, Vec::len);
            shorter_line_seen |= found.iter().any(|line| line.len() < first);
            let (gr, r) = measure(path, ls, width, Rtl, Some(true));
            assert_eq!(
                sorted_lines(&l),
                sorted_lines(&r),
                "{path:?} n={count} at {width} px: the lines break at different options under RTL"
            );
            for (i, lr) in &l {
                let (_, rr) = r.iter().find(|(j, _)| j == i).expect("the same options");
                let want = gl.width - (lr.x - gl.x) - lr.width;
                let got = rr.x - gr.x;
                assert!(
                    (want - got).abs() <= 1.5
                        && (lr.y - rr.y).abs() <= 1.0
                        && (lr.width - rr.width).abs() <= 1.0,
                    "{path:?} n={count} at {width} px: option {i} is not the mirror image of its LTR place: \
                     wanted x {want:.1} in the group, got {got:.1} (y {:.1} vs {:.1})",
                    lr.y,
                    rr.y
                );
            }
        }
        assert!(
            shorter_line_seen,
            "{path:?}: premise: some case should have a line shorter than the first, so that a shorter line starting at the right edge is exercised"
        );
    }
}

// ---- gap -----------------------------------------------------------------------------------------------------------------------

/// **Gap: the space between options is wider than the space between a mark and its own label**, measured from the
/// pixels at 2x. The ratio of the narrowest option-to-option gap to the widest mark-to-label gap must be at least 1.5
/// (the design draws about 1.8; an eight-pixel gap draws 1.0, where a label is as near the next option as its own).
///
/// **This does not read the value the code sets.** A test that asserted `between == 2 * gap` would agree with the code
/// by construction; this one cannot.
#[test]
fn the_space_between_options_is_wider_than_a_mark_from_its_own_label() {
    for path in PATHS {
        for dir in DIRECTIONS {
            let ls = labels(3, false);
            let width = 800.0;
            let mut r = rig(group(path, ls, dir, Some(true), None), width);
            let g = r.targets()[0].bounds;
            let opts = targets(&mut r, g);
            r.point_at(None);
            r.redraw();
            let frame = r.frame_scaled(&theme(path), SCALE as f32);
            let ink = Ink::new(frame, width as usize, (2, 2));
            let rects: Vec<Rectangle> = opts.iter().map(|(_, r)| *r).collect();
            let (y0, y1) = (
                rects.iter().map(|r| r.y).fold(f32::MAX, f32::min),
                rects.iter().map(|r| r.y + r.height).fold(0.0, f32::max),
            );
            let (x0, x1) = (
                rects.iter().map(|r| r.x).fold(f32::MAX, f32::min) - 6.0,
                rects.iter().map(|r| r.x + r.width).fold(0.0, f32::max) + 6.0,
            );
            // Letters closer than 3.5 px are one run; a mark and its label are two.
            let runs = merged(ink.runs(x0, x1, y0, y1), 3.5);
            assert_eq!(
                runs.len(),
                6,
                "{path:?} {dir:?}: premise: three options are three marks and three labels, got runs {runs:?}"
            );
            let gaps: Vec<f32> = runs.windows(2).map(|w| w[1].0 - w[0].1).collect();
            let within: Vec<f32> = gaps.iter().copied().step_by(2).collect();
            let between: Vec<f32> = gaps.iter().copied().skip(1).step_by(2).collect();
            let widest_within = within.iter().copied().fold(0.0, f32::max);
            let narrowest_between = between.iter().copied().fold(f32::MAX, f32::min);
            assert!(
                narrowest_between >= 1.5 * widest_within,
                "{path:?} {dir:?}: a label is too near the next option: the space between options is {narrowest_between} px at its \
                 narrowest, a mark is {widest_within} px from its own label at its widest (ratio {:.2}, wanted at least 1.5)",
                narrowest_between / widest_within
            );
        }
    }
}

// ---- keys ----------------------------------------------------------------------------------------------------------------------

/// **Keys: the same messages in either layout, for every arrow, with and without a selection**, in both directions on
/// both paths. In a wrapped row ↓ is "next", not "the option below": the keys never read the layout.
#[test]
fn the_keys_publish_the_same_messages_in_a_row_as_in_a_column() {
    let arrows = [
        Named::ArrowDown,
        Named::ArrowUp,
        Named::ArrowRight,
        Named::ArrowLeft,
    ];
    for path in PATHS {
        for dir in DIRECTIONS {
            for sel in [None, Some(0usize), Some(1), Some(2)] {
                for key in arrows {
                    let mut got = Vec::new();
                    for horizontal in [false, true] {
                        let mut r = rig(
                            group(path, labels(3, true), dir, Some(horizontal), sel),
                            800.0,
                        );
                        r.tab(false);
                        r.messages.clear();
                        r.named(key, false);
                        got.push(r.messages.last().map(|m| m.0.clone()));
                    }
                    assert!(
                        got[0].is_some(),
                        "{path:?} {dir:?} selected {sel:?} {key:?}: the column published nothing (premise)"
                    );
                    assert_eq!(
                        got[0], got[1],
                        "{path:?} {dir:?} selected {sel:?} {key:?}: the row must publish what the column does"
                    );
                }
            }
        }
    }
}

// ---- the column unchanged ------------------------------------------------------------------------------------------------------

/// **The column is unchanged:** `horizontal(false)` draws, pixel for pixel, as the group does without the call.
#[test]
fn horizontal_false_draws_as_the_group_without_the_call() {
    for path in PATHS {
        for dir in DIRECTIONS {
            for sel in [None, Some(1usize)] {
                let ls = labels(3, false);
                let th = theme(path);
                let with = rig(group(path, ls, dir, Some(false), sel), 800.0)
                    .frame_scaled(&th, SCALE as f32);
                let without =
                    rig(group(path, ls, dir, None, sel), 800.0).frame_scaled(&th, SCALE as f32);
                assert!(
                    with == without,
                    "{path:?} {dir:?} selected {sel:?}: `horizontal(false)` changed what the group draws"
                );
            }
        }
    }
}
