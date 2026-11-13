//! Where a sheet is and how big it is (the 0.62 audit: O-035 to O-042, O-049; B-1).
//!
//! The sheet panel's **bounds** are read from the tree, the way `menu_overlay.rs::bounds_of`
//! reads the header's: the container `snora-sheet-panel` is asked where it is laid out. Until
//! 0.62 the only sheet tests asked whether a control in it could be reached, so a sheet that
//! was the wrong size, or on the wrong edge, passed them all. It was: a sheet on the right
//! (`End` under LTR, `Start` under RTL) had the spacer's width from 0.4.0 to 0.61.0, so
//! `OneThird` drew two thirds and `TwoThirds` one third, and `Half` hid it.
//!
//! The expected sizes below are the book's (`overlays.md`: `OneThird` 33 %, `Half` 50 %,
//! `TwoThirds` 67 %, `Ratio(r)` is `r`, `Pixels(p)` is `p`), not the code's `portions()`.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use iced::widget::{space, text};
use iced::{Element, Rectangle, Size, Theme};
use iced_runtime::core::widget::{Id, Operation};
use snora::{AppLayout, Dialog, LayoutDirection, Sheet, SheetEdge, SheetSize};

#[derive(Debug, Clone, PartialEq)]
enum Msg {}

const PANEL: &str = "snora-sheet-panel";

/// The bounds of the container named `wanted`, as the tree reports them.
fn bounds_of(rig: &mut Rig<Msg>, wanted: &'static str) -> Option<Rectangle> {
    struct Find(&'static str, Arc<Mutex<Option<Rectangle>>>);
    impl Operation for Find {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            if id.is_some_and(|i| *i == Id::new(self.0)) {
                *self.1.lock().unwrap() = Some(bounds);
            }
        }
    }
    let out = Arc::new(Mutex::new(None));
    rig.run(Box::new(Find(wanted, out.clone())));
    *out.lock().unwrap()
}

fn page() -> Element<'static, Msg> {
    text("page").into()
}

fn scene(
    sheet: Sheet<Element<'static, Msg>, Msg>,
    direction: LayoutDirection,
) -> Element<'static, Msg> {
    snora::render(AppLayout::new(page()).direction(direction).sheet(sheet))
}

fn sheet(edge: SheetEdge, size: SheetSize) -> Sheet<Element<'static, Msg>, Msg> {
    Sheet::new(text("sheet").into()).at(edge).with_size(size)
}

fn panel(edge: SheetEdge, direction: LayoutDirection, size: SheetSize, window: Size) -> Rectangle {
    let mut rig = Rig::sized(scene(sheet(edge, size), direction), window);
    bounds_of(&mut rig, PANEL).expect("the sheet panel is laid out")
}

/// The share of the window's axis the book names for `size`, or the pixels it asks for.
fn along(size: SheetSize, axis: f32) -> f32 {
    match size {
        SheetSize::OneThird => axis * 0.33,
        SheetSize::Half => axis * 0.50,
        SheetSize::TwoThirds => axis * 0.67,
        SheetSize::Ratio(r) => axis * (r * 100.0).round() / 100.0,
        SheetSize::Pixels(p) => p,
    }
}

const WINDOWS: [Size; 2] = [Size::new(800.0, 600.0), Size::new(1000.0, 500.0)];

fn sizes() -> [SheetSize; 5] {
    [
        SheetSize::OneThird,
        SheetSize::Half,
        SheetSize::TwoThirds,
        SheetSize::Ratio(0.25),
        SheetSize::Pixels(150.0),
    ]
}

/// Which side of the window a logical edge is on: `(left, right, top, bottom)`.
fn side(edge: SheetEdge, direction: LayoutDirection) -> &'static str {
    match (edge, direction) {
        (SheetEdge::Bottom, _) => "bottom",
        (SheetEdge::Top, _) => "top",
        (SheetEdge::Start, LayoutDirection::Ltr) | (SheetEdge::End, LayoutDirection::Rtl) => "left",
        (SheetEdge::End, LayoutDirection::Ltr) | (SheetEdge::Start, LayoutDirection::Rtl) => {
            "right"
        }
    }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1.5
}

#[test]
fn a_sheet_is_on_its_edge_and_its_size_on_all_four_edges_both_directions_two_windows() {
    let mut failures = Vec::new();
    for window in WINDOWS {
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
            for edge in [
                SheetEdge::Bottom,
                SheetEdge::Top,
                SheetEdge::Start,
                SheetEdge::End,
            ] {
                for size in sizes() {
                    let b = panel(edge, direction, size, window);
                    let s = side(edge, direction);
                    let (axis, length, flush) = match s {
                        "bottom" | "top" => (window.height, b.height, b.width),
                        _ => (window.width, b.width, b.height),
                    };
                    let full = match s {
                        "bottom" | "top" => window.width,
                        _ => window.height,
                    };
                    let at_edge = match s {
                        "left" => near(b.x, 0.0),
                        "right" => near(b.x + b.width, window.width),
                        "top" => near(b.y, 0.0),
                        _ => near(b.y + b.height, window.height),
                    };
                    let row = format!(
                        "{edge:?} {direction:?} {size:?} in {}x{}: {b:?} (wanted the {s} edge, {} long, full across)",
                        window.width,
                        window.height,
                        along(size, axis),
                    );
                    if !at_edge || !near(length, along(size, axis)) || !near(flush, full) {
                        failures.push(row);
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of 80 sheets are not where or as big as the book says:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn a_sheet_on_the_right_has_the_width_that_was_asked_for_not_the_spacers() {
    // The 0.4.0 to 0.61.0 defect, by name: `OneThird` is a third, `TwoThirds` two thirds, on the right too.
    let window = Size::new(800.0, 600.0);
    for (edge, direction) in [
        (SheetEdge::End, LayoutDirection::Ltr),
        (SheetEdge::Start, LayoutDirection::Rtl),
    ] {
        let third = panel(edge, direction, SheetSize::OneThird, window);
        let two_thirds = panel(edge, direction, SheetSize::TwoThirds, window);
        assert!(
            near(third.width, 264.0),
            "{edge:?} {direction:?}: OneThird is a third of 800 px (264), not {}",
            third.width
        );
        assert!(
            near(two_thirds.width, 536.0),
            "{edge:?} {direction:?}: TwoThirds is two thirds of 800 px (536), not {}",
            two_thirds.width
        );
        assert!(
            near(third.x + third.width, 800.0),
            "flush with the right edge"
        );
    }
}

#[test]
fn a_sheet_made_with_new_is_a_third_of_the_window_along_the_bottom() {
    // O-040, O-035: the defaults, drawn.
    let mut rig = Rig::new(snora::render(
        AppLayout::new(page()).sheet(Sheet::new(text("sheet").into())),
    ));
    let b = bounds_of(&mut rig, PANEL).expect("the sheet panel is laid out");
    assert!(near(b.width, 800.0), "full width: {b:?}");
    assert!(near(b.y + b.height, 600.0), "along the bottom: {b:?}");
    assert!(near(b.height, 198.0), "a third (33 %) of 600 px: {b:?}");
}

#[test]
fn pixels_ignore_the_window() {
    // O-042: 150 px in a small and a large window, on a horizontal and a vertical edge.
    for window in [Size::new(500.0, 400.0), Size::new(1200.0, 900.0)] {
        for edge in [SheetEdge::Bottom, SheetEdge::End] {
            let b = panel(edge, LayoutDirection::Ltr, SheetSize::Pixels(150.0), window);
            let length = if edge == SheetEdge::Bottom {
                b.height
            } else {
                b.width
            };
            assert!(near(length, 150.0), "{edge:?} in {window:?}: {b:?}");
        }
    }
}

// ---- O-049: the engine path's sheet has a card of its own; its dialog has none ---------------

fn px(frame: &[u8], width: u32, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * width + x) * 4) as usize;
    [frame[i], frame[i + 1], frame[i + 2]]
}

fn rgb(c: iced::Color) -> [u8; 3] {
    let c = c.into_rgba8();
    [c[0], c[1], c[2]]
}

fn close(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 2)
}

#[test]
fn the_engine_paths_sheet_has_a_fill_and_a_border_and_its_dialog_has_neither() {
    let palette = *Theme::Light.extended_palette();
    let fill = rgb(palette.background.base.color);
    let border = rgb(palette.background.weak.color);
    let width = 800;

    // The sheet: along the bottom, 198 px. Its fill is the theme's background; its top edge is the border.
    let mut rig = Rig::new(scene(
        sheet(SheetEdge::Bottom, SheetSize::OneThird),
        LayoutDirection::Ltr,
    ));
    let b = bounds_of(&mut rig, PANEL).expect("panel");
    let frame = rig.frame();
    let dim = px(&frame, width, 400, 100);
    let inside = px(&frame, width, 700, 580);
    let top_edge = px(&frame, width, 400, b.y.round() as u32);
    assert_ne!(
        dim, fill,
        "premise: the dim is not the sheet's fill, so the two can be told apart"
    );
    assert_ne!(dim, border, "premise: nor the border's colour");
    assert!(
        close(inside, fill),
        "the sheet is filled with the theme's background: {inside:?} vs {fill:?}"
    );
    assert!(
        close(top_edge, border),
        "the sheet's first row is its 1 px border: {top_edge:?} vs {border:?}"
    );

    // The dialog: nothing behind its content. The content is a transparent space, so the pixel at
    // its centre is the dim's.
    let mut rig = Rig::new(snora::render(
        AppLayout::new(page()).dialog(Dialog::new(space().width(200).height(100).into())),
    ));
    let frame = rig.frame();
    let beside = px(&frame, width, 100, 300);
    let centre = px(&frame, width, 400, 300);
    assert_eq!(
        centre, beside,
        "the engine path's dialog draws no card behind its content"
    );
}
