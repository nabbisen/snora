//! Helpers shared by the rendered-layout test binaries.
//!
//! Each file under `tests/` is its own crate, so anything two of them need
//! lives here and is included with `mod common;`. Three things are here,
//! each because getting it wrong produced a test that could not fail:
//!
//! - [`pointer_target`] measures what a pointer can actually hit, by
//!   clicking, rather than by picking a rectangle out of the widget tree
//!   (RFC-100). Several containers hold any given control.
//! - [`simulator`] loads the lucide font, and
//!   [`assert_lucide_font_loaded`] fails if it did not. iced's font system
//!   is process-global, so a "with the font versus without it" comparison
//!   inside one binary measures whichever test ran first (RFC-100 §3).
//! - [`frame_hash`] is the only way to read rendered pixels through
//!   `iced_test`, which exposes them solely as a hash it will write to a
//!   file.
//!
//! Moved here from `dismiss_remove_controls.rs` by RFC-101, which reuses
//! the first two.

// Each test binary uses a subset of this module.
#![allow(dead_code)]

use std::path::Path;

use iced::{Element, Point, Rectangle, Theme};
#[cfg(feature = "lucide-icons")]
use iced::{Size, widget::text};
use iced_test::Simulator;
use iced_test::selector::Candidate;

/// How far inside a candidate rectangle's corners the click probes land.
const CORNER_INSET: f32 = 0.5;

/// WCAG 2.5.8's minimum pointer target, in both dimensions — the floor
/// snora's accessibility checklist mandates.
pub const MIN_TARGET: f32 = 24.0;

/// Allowed difference, in logical pixels, between two positions or sizes
/// that should coincide. Layout is float arithmetic on exact inputs; half
/// a pixel absorbs sub-pixel rounding and is far smaller than any
/// misplacement worth catching (RFC-099's uncentred glyph was ~10px off).
pub const TOLERANCE: f32 = 0.5;

/// lucide's `X` codepoint, as `snora_widgets::icon` and
/// `snora::toast` both obtain it from `char::from(lucide_icons::Icon::X)`.
#[cfg(feature = "lucide-icons")]
pub const LUCIDE_X: &str = "\u{e1b2}";

/// Builds a simulator, loading the lucide font under `lucide-icons`.
///
/// **Every simulator in a test that measures a lucide glyph must come from
/// here.** Without the font, `Font::with_name("lucide")` falls back and the
/// codepoint renders as whatever the fallback font does with it — and a
/// natural-size-then-centre comparison still passes, because the glyph and
/// its reference are then the same fallback, measured consistently.
pub fn simulator<'a, Message>(element: impl Into<Element<'a, Message>>) -> Simulator<'a, Message> {
    #[cfg(feature = "lucide-icons")]
    let settings = iced::Settings {
        fonts: vec![lucide_icons::LUCIDE_FONT_BYTES.into()],
        ..iced::Settings::default()
    };
    #[cfg(not(feature = "lucide-icons"))]
    let settings = iced::Settings::default();

    Simulator::with_settings(settings, element)
}

/// The rendered size of `glyph` on its own, in the lucide font.
#[cfg(feature = "lucide-icons")]
pub fn lucide_glyph_size(glyph: &str, size: f32) -> Size {
    let element: Element<'_, ()> = text(glyph.to_owned())
        .size(size)
        .font(iced::Font::with_name("lucide"))
        .into();
    let mut ui = simulator(element);
    ui.find(glyph)
        .expect("the glyph renders on its own")
        .visible_bounds()
        .expect("the glyph is visible")
        .size()
}

/// Fails if the lucide font was not loaded.
///
/// Lucide glyphs have a square 1em advance: rendered at size `s`, the `X`
/// box is exactly `s` wide. Unloaded, the codepoint falls back to a glyph
/// of a different width — measured 7.2px at size 12, where the loaded
/// glyph measures 12. This checks a property of the glyph itself, so it
/// holds whichever test in the process loaded the font first.
#[cfg(feature = "lucide-icons")]
pub fn assert_lucide_font_loaded(label: &str, size: f32) {
    let width = lucide_glyph_size(LUCIDE_X, size).width;
    assert!(
        (width - size).abs() <= 0.01,
        "{label}: lucide X is {width}px wide at size {size}; a loaded lucide glyph is exactly 1em \
         wide — the lucide font did not load, and every lucide measurement here is of a fallback \
         glyph",
    );
}

/// Every container-like widget's bounds, in tree order. iced reports
/// buttons, containers and stacks alike through `Operation::container`.
pub fn containers<Message>(ui: &mut Simulator<'_, Message>) -> Vec<Rectangle> {
    let mut bounds = Vec::new();
    let _ = ui.find(|candidate: Candidate<'_>| {
        if matches!(candidate, Candidate::Container { .. }) {
            bounds.push(candidate.bounds());
        }
        None::<()>
    });
    bounds
}

/// A control's pointer target: the **largest** container holding `inside`
/// whose four corners all produce a message `hit` accepts.
///
/// Measured by clicking rather than by picking a container out of the
/// tree, because several containers hold any given control — its button,
/// its content row, the row or column around it, the surface it sits on.
/// Only the button and anything sharing its rectangle answer a click at
/// every corner; enclosing containers fail, because their corners are
/// padding or a neighbour. What this returns is what a pointer can hit,
/// which is what WCAG 2.5.8 is about.
///
/// `build` is called once per probe, so each click lands on a fresh tree.
pub fn pointer_target<'a, Message, Build, Hit>(
    label: &str,
    build: Build,
    inside: Point,
    hit: Hit,
) -> Rectangle
where
    Build: Fn() -> Element<'a, Message>,
    Hit: Fn(&Message) -> bool,
{
    let mut ui = simulator(build());
    let mut holding: Vec<Rectangle> = containers(&mut ui)
        .into_iter()
        .filter(|r| r.contains(inside))
        .collect();
    holding.sort_by(|a, b| (b.width * b.height).total_cmp(&(a.width * a.height)));

    let reaches = |point: Point| {
        let mut ui = simulator(build());
        ui.point_at(point);
        let _ = ui.simulate(iced_test::simulator::click());
        ui.into_messages().any(|m| hit(&m))
    };

    holding
        .into_iter()
        .find(|r| {
            [
                Point::new(r.x + CORNER_INSET, r.y + CORNER_INSET),
                Point::new(r.x + r.width - CORNER_INSET, r.y + CORNER_INSET),
                Point::new(r.x + CORNER_INSET, r.y + r.height - CORNER_INSET),
                Point::new(r.x + r.width - CORNER_INSET, r.y + r.height - CORNER_INSET),
            ]
            .into_iter()
            .all(reaches)
        })
        .unwrap_or_else(|| panic!("{label}: no clickable region around {inside:?}"))
}

/// Hash of the frame the simulator renders now.
///
/// `iced_test` exposes rendered pixels only through
/// `Snapshot::matches_hash`, which writes the hash to a file when none
/// exists. Each call gets its own fresh directory under
/// `CARGO_TARGET_TMPDIR`, so the file written is this frame's hash.
pub fn frame_hash<Message>(ui: &mut Simulator<'_, Message>, theme: &Theme, name: &str) -> String {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("frames")
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
