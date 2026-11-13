//! The overlay laws, as drawn and as pressed (the 0.62 audit). Laws 1–8 are in
//! `docs/src/overlay-interaction-semantics.md`; each test says which row of the audit's tables it
//! answers (`rfcs/handoffs/audit-0-62-keyboard-and-overlay-claims/claims-a.md`).
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use common::removal::Rig;
use iced::widget::{column, text};
use iced::{Element, Point};
use snora::{AppLayout, Dialog};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    CloseModals,
}

fn page() -> Element<'static, Msg> {
    column![text("page")].into()
}

/// An empty page pixel, far from the centred dialog and from the page's own text.
const EMPTY: Point = Point::new(700.0, 500.0);

fn px(frame: &[u8]) -> [u8; 3] {
    let i = (EMPTY.y as usize * 800 + EMPTY.x as usize) * 4;
    [frame[i], frame[i + 1], frame[i + 2]]
}

/// The page is white (255) under the stock Light theme the rig draws with; black at 44 % over it
/// is 255 × 0.56 = 142.8, which is 143. 40 % would be 153.
const DIMMED: u8 = 143;

fn dialog() -> Dialog<Element<'static, Msg>, Msg> {
    Dialog::new(text("dialog").into())
}

/// A-1 / L-002 / L-009: the dim is 44 % black over a light page, **with** a close message and
/// **without** one (a dim with no sink still paints), on `snora::render`.
#[test]
fn the_dim_is_44_percent_with_and_without_a_close_message_on_the_engine_path() {
    let mut bare = Rig::new(snora::render(AppLayout::new(page())));
    assert_eq!(
        px(&bare.frame()),
        [255, 255, 255],
        "premise: the empty page pixel is white"
    );

    let mut sinkless = Rig::new(snora::render(AppLayout::new(page()).dialog(dialog())));
    let mut sink = Rig::new(snora::render(
        AppLayout::new(page())
            .dialog(dialog())
            .on_close_modals(Msg::CloseModals),
    ));
    for (what, rig) in [
        ("no close message", &mut sinkless),
        ("a close message", &mut sink),
    ] {
        let p = px(&rig.frame());
        assert!(
            p.iter().all(|c| c.abs_diff(DIMMED) <= 1),
            "{what}: the dim over white is {p:?}, 44 % is {DIMMED}"
        );
    }
}

/// The same, on `snora::design::render`, which derives the dim from the tokens.
#[cfg(feature = "design")]
#[test]
fn the_dim_is_44_percent_with_and_without_a_close_message_on_the_design_path() {
    let t: &'static snora::design::Tokens = Box::leak(Box::new(snora::design::Tokens::light()));
    let mut bare = Rig::new(snora::design::render(AppLayout::new(page()), t));
    assert_eq!(
        px(&bare.frame()),
        [255, 255, 255],
        "premise: the empty page pixel is white"
    );

    let mut sinkless = Rig::new(snora::design::render(
        AppLayout::new(page()).dialog(dialog()),
        t,
    ));
    let mut sink = Rig::new(snora::design::render(
        AppLayout::new(page())
            .dialog(dialog())
            .on_close_modals(Msg::CloseModals),
        t,
    ));
    for (what, rig) in [
        ("no close message", &mut sinkless),
        ("a close message", &mut sink),
    ] {
        let p = px(&rig.frame());
        assert!(
            p.iter().all(|c| c.abs_diff(DIMMED) <= 1),
            "{what}: the dim over white is {p:?}, 44 % is {DIMMED}"
        );
    }
}
