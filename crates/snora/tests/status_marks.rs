//! RFC-143: **every toast and every status notice carries a mark that says its intent by shape**, and the tests that
//! say *which* shape (RFC-139's rule: "a drawn guard must assert which glyph", not that something is drawn).
//!
//! The expected shapes below are **literal 16 x 16 grids**, written from the ruled description (RFC-143 round 2:
//! error a disc with a bar cut out, warning a rounded square with a "!" cut out, info a ring with an "i", success a
//! rounded-square ring, debug a small dot), not read from `snora-interaction`'s `mark.rs`. A drawn mark is read
//! from the frame at 4x, one sample at the middle of each of the 16 x 16 cells of its slot, classified as ink or
//! surface by luma against the card's own fill, and compared cell by cell.
//!
//! **The threshold is measured, not guessed** (`MATCH`, `APART`): a right mark misses its grid by at most
//! `MATCH` cells, a wrong one by at least `APART`, over both paths, both directions and both renderers; the
//! margin is stated where they are set. Each guard has a breakage row in the review request.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); the rendered jobs run it on both.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use common::sweep::{Msg, TONES, leak};
use iced::widget::{column, container, text};
use iced::{Element, Length, Rectangle, Theme};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::{Tokens, Tone, notice::Notice};
use snora::{AppLayout, LayoutDirection, Toast, ToastIntent};

/// The scale frames are read at: a 16 px slot is 64 px, a cell 4.
const SCALE: usize = 4;

/// The expected shapes, by name: `#` is ink, `.` is the surface (the card's fill, or the notice's).
const EXPECTED: [(&str, [&str; 16]); 5] = [
    (
        "Info",
        [
            ".....######.....",
            "...##########...",
            "..####....####..",
            ".###........###.",
            ".##....##....##.",
            "###....##....###",
            "##............##",
            "##.....##.....##",
            "##.....##.....##",
            "##.....##.....##",
            "###....##....###",
            ".##....##....##.",
            ".###........###.",
            "..####....####..",
            "...##########...",
            ".....######.....",
        ],
    ),
    (
        "Success",
        [
            ".##############.",
            "################",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "##............##",
            "################",
            ".##############.",
        ],
    ),
    (
        "Warning",
        [
            ".##############.",
            "################",
            "################",
            "#######..#######",
            "#######..#######",
            "#######..#######",
            "#######..#######",
            "#######..#######",
            "#######..#######",
            "################",
            "################",
            "#######..#######",
            "#######..#######",
            "################",
            "################",
            ".##############.",
        ],
    ),
    (
        "Error",
        [
            ".....######.....",
            "...##########...",
            "..############..",
            ".##############.",
            ".##############.",
            "################",
            "################",
            "###..........###",
            "###..........###",
            "################",
            "################",
            ".##############.",
            ".##############.",
            "..############..",
            "...##########...",
            ".....######.....",
        ],
    ),
    (
        "Debug",
        [
            "................",
            "................",
            "................",
            "................",
            "................",
            "......####......",
            ".....######.....",
            ".....######.....",
            ".....######.....",
            ".....######.....",
            "......####......",
            "................",
            "................",
            "................",
            "................",
            "................",
        ],
    ),
];

/// Most cells a right mark may miss its grid by: the corners of a circle and a rounded square, where a cell is half covered and
/// the renderer may round it either way. Measured: see `RIGHT_MARKS_MISS`.
const MATCH: usize = 12;

/// Fewest cells a wrong mark may miss another's grid by. The grids differ pairwise by 72 to 220 cells; a mark that is
/// another's draws that grid, not this one.
const APART: usize = 40;

// ---- reading a frame ----------------------------------------------------------------------------------------------------------

fn luma(frame: &[u8], x: usize, y: usize) -> f32 {
    let i = (y * 800 * SCALE + x) * 4;
    0.2126 * f32::from(frame[i])
        + 0.7152 * f32::from(frame[i + 1])
        + 0.0722 * f32::from(frame[i + 2])
}

/// A mark's slot read as 16 x 16 cells: `true` where the cell's middle is nearer the ink than the surface, by luma
/// (so it is the mark **in greyscale**). `fill` is the surface's luma; the ink is the pixel of the slot furthest from it.
fn read(frame: &[u8], x: f32, y: f32, side: f32, fill: f32, dx: i64, dy: i64) -> [[bool; 16]; 16] {
    let s = SCALE as f32;
    let (x0, y0, x1, y1) = (
        (x * s) as i64 + dx,
        (y * s) as i64 + dy,
        ((x + side) * s) as i64 + dx,
        ((y + side) * s) as i64 + dy,
    );
    let mut far = 0.0f32;
    for py in y0..y1 {
        for px in x0..x1 {
            far = far.max((luma(frame, px as usize, py as usize) - fill).abs());
        }
    }
    let mut cells = [[false; 16]; 16];
    for (r, row) in cells.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            let px = ((x + (c as f32 + 0.5) * side / 16.0) * s) as i64 + dx;
            let py = ((y + (r as f32 + 0.5) * side / 16.0) * s) as i64 + dy;
            *cell = (luma(frame, px as usize, py as usize) - fill).abs() > far / 2.0 && far > 1.0;
        }
    }
    cells
}

/// How many of the 256 cells differ from `expected`, at the best of a half-pixel's shifts (a quad snaps to the device grid).
fn misses(frame: &[u8], x: f32, y: f32, side: f32, fill: f32, expected: &[&str; 16]) -> usize {
    let mut best = usize::MAX;
    for dx in -2..=2 {
        for dy in -2..=2 {
            let cells = read(frame, x, y, side, fill, dx, dy);
            let n = (0..16)
                .flat_map(|r| (0..16).map(move |c| (r, c)))
                .filter(|&(r, c)| cells[r][c] != (expected[r].as_bytes()[c] == b'#'))
                .count();
            best = best.min(n);
        }
    }
    best
}

fn between(a: &[[bool; 16]; 16], b: &[[bool; 16]; 16]) -> usize {
    (0..16)
        .flat_map(|r| (0..16).map(move |c| (r, c)))
        .filter(|&(r, c)| a[r][c] != b[r][c])
        .count()
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

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<&'static Tokens> = OnceLock::new();
    T.get_or_init(|| leak(Tokens::light()))
}

// ---- toasts -----------------------------------------------------------------------------------------------------------------

/// Top to bottom on the page: the newest toast is nearest the anchor edge, so the queue is pushed in this order reversed.
const TOASTS: [(&str, ToastIntent); 5] = [
    ("Info", ToastIntent::Info),
    ("Success", ToastIntent::Success),
    ("Warning", ToastIntent::Warning),
    ("Error", ToastIntent::Error),
    ("Debug", ToastIntent::Debug),
];

/// `design`: the token path; otherwise the unstyled engine path.
#[derive(Clone, Copy, Debug)]
struct Cfg {
    design: bool,
    dir: LayoutDirection,
    marks: bool,
}

impl Cfg {
    /// The side of a mark's slot: 1 em of the text it sits with. The unstyled path's text is iced's default, 16.
    fn side(self) -> f32 {
        if self.design {
            tokens().typography.body.size
        } else {
            16.0
        }
    }
}

fn toast_view(cfg: Cfg) -> Element<'static, Msg> {
    let toasts = TOASTS
        .iter()
        .rev()
        .enumerate()
        .map(|(i, (_, intent))| Toast::new(i as u64 + 1, *intent, "Title", "Message", Msg::Go))
        .collect();
    let layout = AppLayout::new(text("page").into())
        .toasts(toasts)
        .direction(cfg.dir)
        .toast_marks(cfg.marks);
    if cfg.design {
        snora::design::render(layout, tokens())
    } else {
        snora::render(layout)
    }
}

/// One drawn toast column: the frame, the cards from the top, and the theme it was drawn under.
struct Drawn {
    frame: Vec<u8>,
    cards: Vec<Rectangle>,
}

fn draw_toasts(cfg: Cfg) -> Drawn {
    let mut rig = Rig::new(toast_view(cfg));
    let width = if cfg.marks { 360.0 } else { 340.0 };
    let mut cards: Vec<Rectangle> = containers(&mut rig)
        .into_iter()
        .filter(|r| (r.width - width).abs() < 0.6 && (50.0..100.0).contains(&r.height))
        .collect();
    cards.sort_by(|a, b| a.y.total_cmp(&b.y));
    assert_eq!(
        cards.len(),
        5,
        "{cfg:?}: five cards {width}px wide (a toast is {width}px with its mark {})",
        if cfg.marks { "on" } else { "off" }
    );
    rig.point_at(None);
    rig.redraw();
    let theme = if cfg.design {
        snora::design::theme(tokens())
    } else {
        Theme::Light
    };
    Drawn {
        frame: rig.frame_scaled(&theme, SCALE as f32),
        cards,
    }
}

/// Where a card's mark slot is, and the luma of the card's fill beside it.
fn toast_slot(d: &Drawn, i: usize, cfg: Cfg) -> (f32, f32, f32, f32) {
    let card = d.cards[i];
    let side = cfg.side();
    let x = match cfg.dir {
        LayoutDirection::Ltr => card.x + 12.0,
        LayoutDirection::Rtl => card.x + card.width - 12.0 - side,
    };
    let y = card.y + card.height / 2.0 - side / 2.0;
    let beside = match cfg.dir {
        LayoutDirection::Ltr => card.x + 5.0,
        LayoutDirection::Rtl => card.x + card.width - 5.0,
    };
    let fill = luma(
        &d.frame,
        (beside * SCALE as f32) as usize,
        ((card.y + card.height / 2.0) * SCALE as f32) as usize,
    );
    (x, y, side, fill)
}

fn configs() -> Vec<Cfg> {
    let mut v = Vec::new();
    for design in [false, true] {
        for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
            v.push(Cfg {
                design,
                dir,
                marks: true,
            });
        }
    }
    v
}

/// What every toast's mark is, against every grid: `misses[i][k]` is toast `i` against grid `k`.
fn toast_misses(cfg: Cfg) -> Vec<Vec<usize>> {
    let d = draw_toasts(cfg);
    (0..5)
        .map(|i| {
            let (x, y, side, fill) = toast_slot(&d, i, cfg);
            EXPECTED
                .iter()
                .map(|(_, g)| misses(&d.frame, x, y, side, fill, g))
                .collect()
        })
        .collect()
}

/// **Every intent has a mark.** The slot holds *one of the five marks*: some grid is matched to within `MATCH` cells. It is not
/// enough that the slot has ink: **with no mark the title's own text starts in the slot and supplies the ink** (the first
/// version of this test counted ink and could not fail on a toast with its mark missing; the breakage row showed it).
#[test]
fn every_toast_intent_has_a_mark() {
    for cfg in configs() {
        let m = toast_misses(cfg);
        for (i, (name, _)) in TOASTS.iter().enumerate() {
            let best = m[i].iter().min().unwrap();
            assert!(
                *best <= MATCH,
                "{cfg:?} / {name}: its slot holds none of the five marks (nearest is {best} cells off)"
            );
        }
    }
}

/// **Each intent draws its own shape**, on both paths and in both directions: the mask it draws is the grid ruled for
/// it, and not any other's.
#[test]
fn each_toast_intent_draws_the_shape_ruled_for_it() {
    let mut failures = Vec::new();
    for cfg in configs() {
        let m = toast_misses(cfg);
        for (i, (name, _)) in TOASTS.iter().enumerate() {
            for (k, (kname, _)) in EXPECTED.iter().enumerate() {
                if *kname == *name {
                    if m[i][k] > MATCH {
                        failures.push(format!(
                            "{cfg:?} / {name}: misses its own grid by {} cells (at most {MATCH})",
                            m[i][k]
                        ));
                    }
                } else if m[i][k] < APART {
                    failures.push(format!(
                        "{cfg:?} / {name}: only {} cells from the {kname} grid (at least {APART})",
                        m[i][k]
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failure(s):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// **The five differ from one another in greyscale**: read as masks from luma alone, every pair of the five drawn
/// marks differs in at least `APART` of the 256 cells. (Read from what is drawn, not from the grids.)
#[test]
fn the_five_toast_marks_differ_in_greyscale() {
    let mut failures = Vec::new();
    for cfg in configs() {
        let d = draw_toasts(cfg);
        let drawn: Vec<_> = (0..5)
            .map(|i| {
                let (x, y, side, fill) = toast_slot(&d, i, cfg);
                read(&d.frame, x, y, side, fill, 0, 0)
            })
            .collect();
        for a in 0..5 {
            for b in a + 1..5 {
                let n = between(&drawn[a], &drawn[b]);
                if n < APART {
                    failures.push(format!(
                        "{cfg:?}: {} and {} differ in {n} cells (at least {APART})",
                        TOASTS[a].0, TOASTS[b].0
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failure(s):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// **Under RTL the mark is at the right, the start edge**, with the text and the close mark mirrored around it: the
/// right-hand slot holds the mark, and the left-hand one (where LTR puts it) holds the close mark, not the mark.
#[test]
fn a_toast_mark_is_at_the_start_edge_whichever_way_the_window_reads() {
    for design in [false, true] {
        let rtl = Cfg {
            design,
            dir: LayoutDirection::Rtl,
            marks: true,
        };
        let d = draw_toasts(rtl);
        for (i, (name, _)) in TOASTS.iter().enumerate() {
            let own = EXPECTED.iter().find(|(n, _)| n == name).unwrap();
            let (x, y, side, fill) = toast_slot(&d, i, rtl);
            assert!(
                misses(&d.frame, x, y, side, fill, &own.1) <= MATCH,
                "{rtl:?} / {name}: the mark is not at the right-hand (start) edge"
            );
            // Where an LTR card has it: the left end of an RTL card, which is the end edge, holds the close mark.
            let card = d.cards[i];
            let left = misses(&d.frame, card.x + 12.0, y, side, fill, &own.1);
            assert!(
                left > MATCH,
                "{rtl:?} / {name}: a mark-shaped thing also sits at the end edge ({left} cells off)"
            );
        }
    }
}

/// **The width follows the switch**, and **the switch removes the mark and only the mark**: with the mark off a toast is the
/// 340 px of 0.66 and everything right of where the mark was is the same pixels (the text and the close mark, whichever
/// way the window reads). Both are on the unstyled path, where the mark is the 16 px that its 20 px of room is made for.
#[test]
fn the_switch_removes_the_mark_and_only_the_mark() {
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let on = draw_toasts(Cfg {
            design: false,
            dir,
            marks: true,
        });
        let off = draw_toasts(Cfg {
            design: false,
            dir,
            marks: false,
        });
        for i in 0..5 {
            let (a, b) = (on.cards[i], off.cards[i]);
            assert!(
                (a.width - 360.0).abs() < 0.6 && (b.width - 340.0).abs() < 0.6,
                "{dir:?}: 360 with the mark, 340 without"
            );
            // What is compared: the card's rows, from the text's start edge to the card's far edge.
            let (y0, y1) = (
                (a.y * SCALE as f32) as usize,
                ((a.y + a.height) * SCALE as f32) as usize,
            );
            let span = ((a.width - 12.0 - 16.0 - 4.0) * SCALE as f32) as usize;
            let (xa, xb) = match dir {
                LayoutDirection::Ltr => (
                    ((a.x + 12.0 + 16.0 + 4.0) * SCALE as f32) as usize,
                    ((b.x + 12.0) * SCALE as f32) as usize,
                ),
                LayoutDirection::Rtl => {
                    ((a.x * SCALE as f32) as usize, (b.x * SCALE as f32) as usize)
                }
            };
            let mut differing = 0;
            for y in y0..y1 {
                for x in 0..span {
                    let (ia, ib) = (
                        (y * 800 * SCALE + xa + x) * 4,
                        (y * 800 * SCALE + xb + x) * 4,
                    );
                    if on.frame[ia..ia + 3] != off.frame[ib..ib + 3] {
                        differing += 1;
                    }
                }
            }
            assert_eq!(
                differing, 0,
                "{dir:?} / {}: the switch changed {differing} pixels beyond the mark",
                TOASTS[i].0
            );
        }
    }
    // And the design path: 360 and 340.
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        for (marks, w) in [(true, 360.0), (false, 340.0)] {
            let d = draw_toasts(Cfg {
                design: true,
                dir,
                marks,
            });
            assert!(
                (d.cards[0].width - w).abs() < 0.6,
                "design / {dir:?} / marks {marks}: {w} px"
            );
        }
    }
}

// ---- notices ------------------------------------------------------------------------------------------------------------------

/// The notices' tones, top to bottom, and the mark each takes (`None`: no status, no mark; `Danger` takes `Error`'s).
fn notice_marks() -> [(&'static str, Tone, Option<&'static str>); 6] {
    [
        ("info", TONES[0].1, Some("Info")),
        ("success", TONES[1].1, Some("Success")),
        ("warning", TONES[2].1, Some("Warning")),
        ("danger", TONES[3].1, Some("Error")),
        ("accent", TONES[4].1, None),
        ("neutral", TONES[5].1, None),
    ]
}

fn notice_view(dir: LayoutDirection, mark: bool) -> Element<'static, Msg> {
    let t = tokens();
    let rows: Vec<Element<'static, Msg>> = notice_marks()
        .iter()
        .map(|(_, tone, _)| {
            Notice::new(t, *tone, "Body of the notice.", dir)
                .title("Title")
                .mark(mark)
                .render()
        })
        .collect();
    container(column(rows).spacing(8))
        .width(Length::Fixed(340.0))
        .into()
}

fn draw_notices(dir: LayoutDirection, mark: bool) -> Drawn {
    let mut rig = Rig::new(notice_view(dir, mark));
    let mut cards: Vec<Rectangle> = containers(&mut rig)
        .into_iter()
        .filter(|r| (r.width - 340.0).abs() < 0.6 && (30.0..100.0).contains(&r.height))
        .collect();
    cards.sort_by(|a, b| a.y.total_cmp(&b.y));
    // A notice reports its outer container and the box inside its border: one card is two rectangles a pixel apart.
    cards.dedup_by(|a, b| (a.y - b.y).abs() < 2.5);
    assert_eq!(cards.len(), 6, "six notices 340px wide");
    rig.point_at(None);
    rig.redraw();
    Drawn {
        frame: rig.frame_scaled(&snora::design::theme(tokens()), SCALE as f32),
        cards,
    }
}

/// A notice's slot: after the 4 px bar and the container's `spacing.md`, 1 body line square, centred.
fn notice_slot(d: &Drawn, i: usize, dir: LayoutDirection) -> (f32, f32, f32, f32) {
    let t = tokens();
    let card = d.cards[i];
    let side = t.typography.body.size;
    let x = match dir {
        LayoutDirection::Ltr => card.x + 4.0 + t.spacing.md,
        LayoutDirection::Rtl => card.x + card.width - 4.0 - t.spacing.md - side,
    };
    let y = card.y + card.height / 2.0 - side / 2.0;
    let beside = match dir {
        LayoutDirection::Ltr => card.x + 4.0 + 2.0,
        LayoutDirection::Rtl => card.x + card.width - 4.0 - 2.0,
    };
    let fill = luma(
        &d.frame,
        (beside * SCALE as f32) as usize,
        ((card.y + card.height / 2.0) * SCALE as f32) as usize,
    );
    (x, y, side, fill)
}

/// **Each status tone draws its own mark, and `Danger` draws `Error`'s**, in both directions; `Neutral` and `Accent`
/// draw none (their notices are the same pixels with the switch on or off).
#[test]
fn each_status_tone_draws_the_mark_ruled_for_it() {
    let mut failures = Vec::new();
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let d = draw_notices(dir, true);
        for (i, (tone, _, mark)) in notice_marks().iter().enumerate() {
            let Some(mark) = mark else { continue };
            let (x, y, side, fill) = notice_slot(&d, i, dir);
            for (kname, g) in &EXPECTED {
                let n = misses(&d.frame, x, y, side, fill, g);
                if kname == mark {
                    if n > MATCH {
                        failures.push(format!("{dir:?} / {tone}: misses the {mark} grid by {n} cells (at most {MATCH})"));
                    }
                } else if n < APART {
                    failures.push(format!(
                        "{dir:?} / {tone}: only {n} cells from the {kname} grid (at least {APART})"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failure(s):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// **`Neutral` and `Accent` say no status, so they take no mark**, and the switch changes nothing for them; it changes
/// every status tone, and by the mark and the room it takes only: the notice's text moves by the mark's side and the 8 px gap
/// (`spacing.sm`), and nothing right of it differs.
#[test]
fn the_notice_switch_removes_the_mark_and_only_the_mark() {
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let on = draw_notices(dir, true);
        let off = draw_notices(dir, false);
        let t = tokens();
        let shift = t.typography.body.size + t.spacing.sm;
        for (i, (tone, _, mark)) in notice_marks().iter().enumerate() {
            let card = on.cards[i];
            let (y0, y1) = (
                (card.y * SCALE as f32) as usize + 4,
                ((card.y + card.height) * SCALE as f32) as usize - 4,
            );
            let same = |xa: f32, xb: f32, width: f32| -> usize {
                let (mut n, mut first) = (0, None);
                for y in y0..y1 {
                    for x in 0..(width * SCALE as f32) as usize {
                        let (ia, ib) = (
                            (y * 800 * SCALE + (xa * SCALE as f32) as usize + x) * 4,
                            (y * 800 * SCALE + (xb * SCALE as f32) as usize + x) * 4,
                        );
                        if on.frame[ia..ia + 3] != off.frame[ib..ib + 3] {
                            n += 1;
                            first.get_or_insert((x, y));
                        }
                    }
                }
                if let Some((x, y)) = first {
                    println!(
                        "{dir:?} / {tone}: first differing pixel at +{x}, y {y} (device px), {n} in all"
                    );
                }
                n
            };
            if mark.is_none() {
                let n = same(card.x, card.x, card.width);
                assert_eq!(
                    n, 0,
                    "{dir:?} / {tone}: a tone with no mark changed {n} pixels with the switch"
                );
                continue;
            }
            // The text's box: from where the text starts with the mark on, to the notice's far padding (not the border
            // and the padding, which do not move with the text), against the same stretch, `shift` earlier, with it off.
            let md = t.spacing.md;
            let inner = card.width - 4.0 - 2.0 * md - shift - 2.0;
            let (xa, xb) = match dir {
                LayoutDirection::Ltr => (card.x + 4.0 + md + shift, card.x + 4.0 + md),
                LayoutDirection::Rtl => (card.x + md + 2.0, card.x + md + 2.0 + shift),
            };
            let n = same(xa, xb, inner);
            assert_eq!(
                n, 0,
                "{dir:?} / {tone}: the switch changed {n} pixels beyond the mark"
            );
        }
    }
}

/// **The mark is at the start edge in both directions** is covered by `each_status_tone_draws_the_mark_ruled_for_it`,
/// which reads the slot where the direction puts it; this adds that the end edge of a notice holds no second mark.
#[test]
fn a_notice_mark_is_not_at_the_end_edge() {
    let t = tokens();
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let d = draw_notices(dir, true);
        for (i, (tone, _, mark)) in notice_marks().iter().enumerate() {
            let Some(mark) = mark else { continue };
            let own = EXPECTED.iter().find(|(n, _)| n == mark).unwrap();
            let (_, y, side, fill) = notice_slot(&d, i, dir);
            let card = d.cards[i];
            let end_x = match dir {
                LayoutDirection::Ltr => card.x + card.width - 4.0 - t.spacing.md - side,
                LayoutDirection::Rtl => card.x + 4.0 + t.spacing.md,
            };
            let n = misses(&d.frame, end_x, y, side, fill, &own.1);
            assert!(
                n > MATCH,
                "{dir:?} / {tone}: a mark-shaped thing sits at the end edge ({n} cells off)"
            );
        }
    }
}

/// Prints the margin `MATCH` and `APART` are set from, over everything these tests read: run by hand, on each renderer
/// (`ICED_TEST_BACKEND=wgpu cargo test -p snora --all-features --test status_marks the_margins -- --ignored --nocapture`).
#[test]
#[ignore = "prints the margins MATCH and APART are set from; run by hand, on purpose"]
fn the_margins() {
    let (mut own, mut cross) = (0usize, usize::MAX);
    let mut seen = 0;
    for cfg in configs() {
        let m = toast_misses(cfg);
        for (i, (name, _)) in TOASTS.iter().enumerate() {
            for (k, (kname, _)) in EXPECTED.iter().enumerate() {
                seen += 1;
                if kname == name {
                    own = own.max(m[i][k]);
                } else {
                    cross = cross.min(m[i][k]);
                }
            }
        }
    }
    for dir in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let d = draw_notices(dir, true);
        for (i, (_, _, mark)) in notice_marks().iter().enumerate() {
            let Some(mark) = mark else { continue };
            let (x, y, side, fill) = notice_slot(&d, i, dir);
            for (kname, g) in &EXPECTED {
                seen += 1;
                let n = misses(&d.frame, x, y, side, fill, g);
                if kname == mark {
                    own = own.max(n)
                } else {
                    cross = cross.min(n)
                }
            }
        }
    }
    println!(
        "MARGINS over {seen} comparisons: a right mark misses its grid by at most {own} cells; a wrong one by at least {cross}"
    );
}
