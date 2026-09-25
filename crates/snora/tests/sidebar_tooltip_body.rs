//! The sidebar tooltip is drawn on a body of its own (RFC-102 R-4).
//!
//! # What was wrong
//!
//! `tooltip(btn, text(item.tooltip), ..)` drew bare glyphs over whatever
//! the application happened to render beside the rail. orbok's report
//! shows "Settings" landing on top of a page heading. That is a
//! legibility defect rather than a cosmetic one: the tooltip's contrast
//! was a property of the page underneath, so nothing in snora could
//! guarantee it.
//!
//! # Why this test looks the way it does
//!
//! The tooltip overlay takes no part in widget operations in iced 0.14,
//! so `Simulator::find` cannot see the tooltip at all — neither its text
//! nor its body (RFC-100 §4). That leaves rendered frames.
//!
//! **Comparing a hovered tooltip against a hovered empty-string tooltip
//! cannot fail.** With a body, the two frames differ because the body is
//! sized to its text. Without a body, they differ too, because one draws
//! glyphs and the other draws nothing. The comparison is sensitive to
//! the text either way, and says nothing about a body.
//!
//! So this measures the property that actually distinguishes a body: **an
//! opaque body hides what is behind it.** The sidebar is rendered beside
//! a page carrying a small coloured patch, positioned where the tooltip
//! opens, and the same frame is rendered twice with the patch in two
//! different colours. With a body the two frames are identical, because
//! the patch is covered. Without one, the patch shows through and they
//! differ — which is what the tooltip landing on orbok's page heading
//! looks like, measured.

#![cfg(feature = "widgets")]

use std::path::{Path, PathBuf};

use iced::widget::{column, container, row, space};
use iced::{Color, Element, Event, Length, Point, Size, Theme, mouse};
use iced_test::Simulator;

use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

/// Rail geometry, mirrored from `snora_widgets::sidebar` as an
/// expectation (the constants are private; see
/// `crates/snora/tests/side_bar_fit.rs` for the same reasoning).
const RAIL_WIDTH: f32 = 64.0;
const BUTTON_SIZE: f32 = 48.0;
const UNSTYLED_VERTICAL_PADDING: f32 = 16.0;

/// The patch the tooltip body must cover, in page-local coordinates.
///
/// The page starts at the rail's right edge, and the tooltip opens
/// against that edge, vertically centred on the hovered button. The
/// patch is deliberately small and well inside the body's footprint, so
/// that the test measures "the body covers what is behind it" rather
/// than the body's exact extent.
const PATCH_X: f32 = 12.0;
const PATCH_Y: f32 = 34.0;
const PATCH_WIDTH: f32 = 32.0;
const PATCH_HEIGHT: f32 = 12.0;

/// A window just large enough for the rail and the patch.
const WINDOW: Size = Size::new(320.0, 140.0);

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    First,
    Second,
}

fn side_bar() -> SideBar<Msg, u8> {
    SideBar {
        items: vec![
            SideBarItem {
                view_id: 0,
                icon: Icon::Text("S".into()),
                tooltip: "Settings".into(),
                on_press: Msg::First,
            },
            SideBarItem {
                view_id: 1,
                icon: Icon::Text("H".into()),
                tooltip: "Help".into(),
                on_press: Msg::Second,
            },
        ],
        active: 0,
    }
}

/// The page beside the rail, carrying one coloured patch.
fn page<'a>(patch: Color) -> Element<'a, Msg> {
    let patch = container(space())
        .width(PATCH_WIDTH)
        .height(PATCH_HEIGHT)
        .style(move |_: &Theme| container::Style {
            background: Some(patch.into()),
            ..container::Style::default()
        });
    column![
        space().height(PATCH_Y),
        row![space().width(PATCH_X), patch],
        space().height(Length::Fill),
    ]
    .width(Length::Fill)
    .into()
}

/// Hash of one rendered frame, with the first rail button hovered.
///
/// `iced_test` exposes rendered pixels only through
/// `Snapshot::matches_hash`, which writes the hash when the file does
/// not exist yet; each call gets a fresh directory, so the file it
/// writes is this frame's hash.
fn hovered_frame_hash(name: &str, rail: Element<'_, Msg>, theme: &Theme, patch: Color) -> String {
    let element: Element<'_, Msg> = row![rail, page(patch)].into();
    let mut ui = Simulator::with_size(iced::Settings::default(), WINDOW, element);

    let position = Point::new(
        RAIL_WIDTH / 2.0,
        UNSTYLED_VERTICAL_PADDING + BUTTON_SIZE / 2.0,
    );
    ui.point_at(position);
    let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position })]);

    let directory: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("sidebar_tooltip_body")
        .join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("create frame hash directory");

    let snapshot = ui.snapshot(theme).expect("render a frame");
    assert!(
        snapshot
            .matches_hash(directory.join("frame"))
            .expect("write frame hash")
    );

    let written = std::fs::read_dir(&directory)
        .expect("read frame hash directory")
        .next()
        .expect("a frame hash was written")
        .expect("directory entry");
    std::fs::read_to_string(written.path()).expect("read frame hash")
}

/// Two patch colours far enough apart that no rendering of one could be
/// mistaken for the other.
const PATCH_A: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const PATCH_B: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

fn assert_tooltip_body_covers_the_page(
    label: &str,
    theme: &Theme,
    rail: impl Fn() -> Element<'static, Msg>,
) {
    let with_a = hovered_frame_hash(&format!("{label}-a"), rail(), theme, PATCH_A);
    let with_b = hovered_frame_hash(&format!("{label}-b"), rail(), theme, PATCH_B);

    assert_eq!(
        with_a, with_b,
        "{label}: the page shows through where the tooltip is drawn — two frames differing only \
         in a patch of page colour behind the tooltip rendered differently, so the tooltip has no \
         body of its own and its legibility is whatever the page happens to provide (RFC-102 R-4)",
    );
}

/// A control for the test above: with the tooltip **not** hovered, the
/// two patch colours must produce different frames. Without this, a
/// comparison that found the frames equal for some unrelated reason —
/// the patch mispositioned, the page not rendered, hashing the wrong
/// thing — would look like a passing body test.
#[test]
fn control_the_patch_is_visible_when_no_tooltip_is_open() {
    let idle = |patch: Color| -> String {
        let element: Element<'_, Msg> = row![
            snora::widget::app_side_bar(side_bar(), LayoutDirection::Ltr),
            page(patch)
        ]
        .into();
        let mut ui = Simulator::with_size(iced::Settings::default(), WINDOW, element);
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("sidebar_tooltip_body")
            .join(format!("idle-{:?}", patch.r));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("create frame hash directory");
        let snapshot = ui.snapshot(&Theme::Light).expect("render a frame");
        assert!(
            snapshot
                .matches_hash(directory.join("frame"))
                .expect("write frame hash")
        );
        let written = std::fs::read_dir(&directory)
            .expect("read directory")
            .next()
            .expect("a frame hash was written")
            .expect("entry");
        std::fs::read_to_string(written.path()).expect("read frame hash")
    };

    assert_ne!(
        idle(PATCH_A),
        idle(PATCH_B),
        "the two patch colours render identically with no tooltip open — the patch is not being \
         drawn, so the body test above would pass for the wrong reason",
    );
}

#[test]
fn unstyled_tooltip_has_a_body() {
    assert_tooltip_body_covers_the_page("unstyled", &Theme::Light, || {
        snora::widget::app_side_bar(side_bar(), LayoutDirection::Ltr)
    });
}

#[cfg(feature = "design")]
mod styled {
    use super::*;
    use snora::design::Tokens;

    fn presets() -> [(&'static str, Tokens); 4] {
        [
            ("styled light", Tokens::light()),
            ("styled dark", Tokens::dark()),
            ("styled high_contrast_light", Tokens::high_contrast_light()),
            ("styled high_contrast_dark", Tokens::high_contrast_dark()),
        ]
    }

    #[test]
    fn styled_tooltip_has_a_body() {
        for (label, tokens) in presets() {
            let theme = snora::design::theme(&tokens);
            let with_a = hovered_frame_hash(
                &format!("{label}-a"),
                snora::design::widget::app_side_bar(&tokens, side_bar(), LayoutDirection::Ltr),
                &theme,
                PATCH_A,
            );
            let with_b = hovered_frame_hash(
                &format!("{label}-b"),
                snora::design::widget::app_side_bar(&tokens, side_bar(), LayoutDirection::Ltr),
                &theme,
                PATCH_B,
            );
            assert_eq!(
                with_a, with_b,
                "{label}: the page shows through where the tooltip is drawn — the tooltip has no \
                 body of its own (RFC-102 R-4)",
            );
        }
    }
}
