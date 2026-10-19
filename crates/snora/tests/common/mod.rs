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
//! - [`frame_hash`] reads a frame as a SHA-256 digest, which answers
//!   "did this frame change?" and nothing else.
//! - [`frame_pixels`] reads the frame itself (RFC-108 R-1), by taking the
//!   lossless PNG `iced_test` writes and decoding it. It also reports
//!   **which renderer drew**, and fails if that is not the one asked for
//!   — a silent fallback to tiny-skia would otherwise make a wgpu run
//!   look like a pass.
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
/// `Snapshot::matches_hash` writes the hash to a file when none exists.
/// Each call gets its own fresh directory under `CARGO_TARGET_TMPDIR`, so
/// the file written is this frame's hash.
///
/// A hash answers only "did this frame change?". To read the pixels
/// themselves — a drawn colour, a rule's coverage — use [`frame_pixels`].
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

/// One rendered frame: the renderer that drew it, its physical size, and
/// its pixels as RGBA.
#[derive(Debug, Clone)]
pub struct Frame {
    /// The renderer that actually drew, read from the file name
    /// `iced_test` chose — not from what was requested.
    pub renderer: String,
    /// Physical width. `Simulator::snapshot` fixes the scale factor at
    /// 2.0, so this is twice the logical width.
    pub width: u32,
    /// Physical height, likewise twice the logical height.
    pub height: u32,
    /// Row-major RGBA, four bytes per pixel.
    pub rgba: Vec<u8>,
}

impl Frame {
    /// The scale factor `Simulator::snapshot` fixes (`simulator.rs:221`).
    /// Logical coordinates from `visible_bounds` are multiplied by this
    /// to index into [`Frame::rgba`].
    pub const SCALE: f32 = 2.0;

    /// The RGBA at a physical pixel, or `None` outside the frame.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y * self.width + x) * 4) as usize;
        Some([
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ])
    }

    /// Every pixel of `region`, which is in **logical** coordinates and is
    /// scaled by [`Frame::SCALE`] and clipped to the frame.
    pub fn pixels_in(&self, region: Rectangle) -> Vec<[u8; 4]> {
        let x0 = (region.x * Self::SCALE).round().max(0.0) as u32;
        let y0 = (region.y * Self::SCALE).round().max(0.0) as u32;
        let x1 = ((region.x + region.width) * Self::SCALE).round().max(0.0) as u32;
        let y1 = ((region.y + region.height) * Self::SCALE).round().max(0.0) as u32;

        let mut out = Vec::new();
        for y in y0..y1.min(self.height) {
            for x in x0..x1.min(self.width) {
                if let Some(p) = self.pixel(x, y) {
                    out.push(p);
                }
            }
        }
        out
    }

    /// The most common colour in `region` — what the element reads as.
    ///
    /// The mode, not the mean: averaging an element against its
    /// background invents a colour that was never drawn, and an
    /// antialiased edge would drag it. The mode is a colour actually on
    /// screen.
    pub fn modal_colour(&self, region: Rectangle) -> [u8; 4] {
        let pixels = self.pixels_in(region);
        assert!(
            !pixels.is_empty(),
            "no pixels in {region:?}; the frame is {}x{} physical",
            self.width,
            self.height
        );

        let mut counts: std::collections::HashMap<[u8; 4], usize> =
            std::collections::HashMap::new();
        for p in pixels {
            *counts.entry(p).or_default() += 1;
        }
        // Ties broken by the colour's own bytes, so the answer does not
        // depend on hash iteration order.
        counts
            .into_iter()
            .max_by_key(|&(colour, count)| (count, colour))
            .map(|(colour, _)| colour)
            .expect("a non-empty region has a modal colour")
    }

    /// The modal colour of each physical row of `region`, top to bottom.
    ///
    /// For an element under 2 logical pixels, the interesting number is
    /// the strongest row, not the region's mode: at scale 2 a 1px rule is
    /// two physical rows, and a fractional offset smears it across three
    /// at partial coverage (RFC-108's tab-rule measurement).
    pub fn row_modes(&self, region: Rectangle) -> Vec<(u32, [u8; 4])> {
        let y0 = (region.y * Self::SCALE).round().max(0.0) as u32;
        let y1 = ((region.y + region.height) * Self::SCALE).round().max(0.0) as u32;

        (y0..y1.min(self.height))
            .map(|y| {
                let row = Rectangle {
                    y: y as f32 / Self::SCALE,
                    height: 1.0 / Self::SCALE,
                    ..region
                };
                (y, self.modal_colour(row))
            })
            .collect()
    }
}

/// The frame the simulator renders now, as pixels.
///
/// `iced_test` exposes the rendered buffer only by writing it somewhere:
/// `matches_hash` writes a digest, `matches_image` writes a lossless RGBA
/// PNG. This takes the second route and decodes the file, which is the
/// only way to read a drawn colour (RFC-108 R-1).
///
/// The file `iced_test` writes is named `<stem>-<renderer>.png`, where
/// `<renderer>` is what actually drew, delegated through the fallback
/// renderer (`iced_renderer-0.14.0/src/fallback.rs:701`). So the name is
/// evidence, and this asserts it matches `ICED_TEST_BACKEND` (default
/// `tiny-skia`, pinned in `.cargo/config.toml`). **Without that check a
/// wgpu run that silently fell back would report tiny-skia's pixels as
/// wgpu's** — which is the whole question RFC-108 Unit 0 asks.
pub fn frame_pixels<Message>(ui: &mut Simulator<'_, Message>, theme: &Theme, name: &str) -> Frame {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("frame-pixels")
        .join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("create frame pixel directory");

    let snapshot = ui.snapshot(theme).expect("render a frame");
    assert!(
        snapshot
            .matches_image(directory.join("frame"))
            .expect("write frame image")
    );

    let written = std::fs::read_dir(&directory)
        .expect("read frame pixel directory")
        .next()
        .expect("a frame image was written")
        .expect("directory entry")
        .path();

    let renderer = written
        .file_stem()
        .expect("the written file has a stem")
        .to_string_lossy()
        .strip_prefix("frame-")
        .expect("iced_test names the file <stem>-<renderer>.png")
        .to_owned();

    let requested = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".to_owned());
    assert_eq!(
        renderer, requested,
        "{name}: asked for the {requested} renderer and {renderer} drew. iced_test falls back \
         silently when a backend cannot start, so these pixels are not the ones under test",
    );

    let file = std::fs::File::open(&written).expect("open the written frame");
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("read PNG header");
    let mut rgba = vec![0; reader.output_buffer_size().expect("frame fits in memory")];
    let info = reader.next_frame(&mut rgba).expect("decode the frame");
    rgba.truncate(info.buffer_size());

    assert_eq!(
        info.color_type,
        png::ColorType::Rgba,
        "{name}: iced_test writes RGBA; got {:?}",
        info.color_type
    );

    Frame {
        renderer,
        width: info.width,
        height: info.height,
        rgba,
    }
}

// ---------------------------------------------------------------------
// Contrast, on drawn pixels (RFC-108 R-2)
// ---------------------------------------------------------------------

/// WCAG 1.4.11's non-text floor, and 1.4.3's AA floor for body text.
///
/// One copy for every test binary that includes this module, rather
/// than one per file; `scripts/check-wcag-floors.sh` pins how many
/// copies exist under `crates/` and fails if two disagree.
pub const NON_TEXT_MIN: f32 = 3.0;
pub const AA_TEXT: f32 = 4.5;

/// A drawn pixel as a colour. The frame is opaque, so there is no alpha
/// left to composite: whatever translucency an element had is already
/// in these bytes, which is the point of reading them.
pub fn colour_of(pixel: [u8; 4]) -> iced::Color {
    iced::Color::from_rgba8(pixel[0], pixel[1], pixel[2], 1.0)
}

fn linearize(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance.
pub fn relative_luminance(c: iced::Color) -> f32 {
    0.2126 * linearize(c.r) + 0.7152 * linearize(c.g) + 0.0722 * linearize(c.b)
}

/// WCAG contrast ratio. Checked against the reference identity by
/// every suite that uses it.
pub fn contrast_ratio(a: iced::Color, b: iced::Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The colour in `region` furthest in luminance from `against` — the
/// peak-coverage pixel. For a thin or antialiased mark this is a pixel
/// the renderer covered fully, which carries the style's own colour;
/// every blended neighbour is weaker.
pub fn peak_colour(frame: &Frame, region: Rectangle, against: iced::Color) -> iced::Color {
    frame
        .pixels_in(region)
        .into_iter()
        .map(colour_of)
        .max_by(|a, b| contrast_ratio(*a, against).total_cmp(&contrast_ratio(*b, against)))
        .expect("the region has pixels")
}

/// The strongest physical row of `region` against `against`, as
/// `(physical row, colour, ratio)`.
pub fn peak_row(frame: &Frame, region: Rectangle, against: iced::Color) -> (u32, iced::Color, f32) {
    frame
        .row_modes(region)
        .into_iter()
        .map(|(y, pixel)| {
            let colour = colour_of(pixel);
            (y, colour, contrast_ratio(colour, against))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .expect("the region has rows")
}

// ---------------------------------------------------------------------
// Cross-renderer parity output (RFC-108 R-4)
// ---------------------------------------------------------------------

/// Records one element's core colour and bounds, when
/// `SNORA_PARITY_OUT` names a file.
///
/// **The two renderers cannot be compared inside one process.**
/// `iced_test` fixes the backend from `ICED_TEST_BACKEND` when the
/// first simulator is built, and reaching for `std::env::set_var` to
/// get around that is both unsound in a threaded test binary and a lie
/// about what was measured. So each rendered job writes its own file
/// and `scripts/check-renderer-parity.py` compares them across jobs.
///
/// The **core** colour is [`peak_colour`] against `reference` — the
/// pixel the renderer covered fully — and not the region's mode. The
/// mode is what differs between renderers by design: tiny-skia spreads
/// a thin element over partial rows and wgpu does not, so a region
/// average encodes the antialiasing rather than the colour. The fully
/// covered pixel carries the style's own colour on both, which is the
/// thing parity is actually about (RFC-108 Unit 0 measured identical
/// cores and different edges).
///
/// Bounds are rounded to whole physical pixels, which is the
/// resolution a 1 px difference is allowed at.
/// Which statistic is an element's core colour.
///
/// **A mark and a fill need different statistics, and using one for
/// the other is how RFC-108's Row B first failed to reproduce.** A
/// thin mark is a few pixels on a background, so its colour is the
/// pixel the renderer covered fully; the mode over its region would be
/// the background. A filled box is mostly its fill with a label on
/// top, so its colour is the mode — the *most-contrasting* pixel of a
/// tab is its label, which is what made a tab's fill look unchanged
/// while the two renderers were in fact painting it differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Core {
    /// A thin element: the fully covered pixel.
    Mark,
    /// A surface: the mode over its interior.
    Fill,
}

impl Core {
    fn name(self) -> &'static str {
        match self {
            Core::Mark => "mark",
            Core::Fill => "fill",
        }
    }
}

/// A thin element's core: the fully covered pixel, against `reference`.
pub fn record_mark(element: &str, frame: &Frame, region: Rectangle, reference: iced::Color) {
    record_parity(element, Core::Mark, frame, region, Some(reference));
}

/// A surface's core: the mode over its interior.
pub fn record_fill(element: &str, frame: &Frame, region: Rectangle) {
    record_parity(element, Core::Fill, frame, region, None);
}

fn record_parity(
    element: &str,
    kind: Core,
    frame: &Frame,
    region: Rectangle,
    reference: Option<iced::Color>,
) {
    let Ok(path) = std::env::var("SNORA_PARITY_OUT") else {
        return;
    };

    // **Excluding the edge pixels**, which is the whole reason this can
    // compare two renderers at all. An element's outermost physical row
    // is partly covered, and what a renderer does with partial coverage
    // is exactly where the two differ: measured, the tab underline's
    // bottom row is the pure colour on wgpu and a blend with the black
    // rule beneath it on tiny-skia. Reading the region whole reports
    // that blend as the element's colour and fails parity on a
    // difference that is antialiasing, not colour — which is what
    // happened on the first CI run of this check.
    let inset = Rectangle {
        x: region.x + 1.0 / Frame::SCALE,
        y: region.y + 1.0 / Frame::SCALE,
        width: (region.width - 2.0 / Frame::SCALE).max(0.0),
        height: (region.height - 2.0 / Frame::SCALE).max(0.0),
    };
    // A 1-logical-pixel element is two physical rows, and insetting
    // leaves nothing; fall back to the region itself, where both
    // renderers put a fully covered row anyway.
    let core_region = if inset.width > 0.0 && inset.height > 0.0 {
        inset
    } else {
        region
    };

    let core = match kind {
        Core::Mark => peak_colour(
            frame,
            core_region,
            reference.expect("a mark row needs the colour it is read against"),
        ),
        Core::Fill => colour_of(frame.modal_colour(core_region)),
    };

    let line = format!(
        "{element}\t{}\t{}\t#{:02x}{:02x}{:02x}\t{} {} {} {}\n",
        kind.name(),
        frame.renderer,
        (core.r * 255.0).round() as u8,
        (core.g * 255.0).round() as u8,
        (core.b * 255.0).round() as u8,
        (region.x * Frame::SCALE).round() as i64,
        (region.y * Frame::SCALE).round() as i64,
        (region.width * Frame::SCALE).round() as i64,
        (region.height * Frame::SCALE).round() as i64,
    );

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("open {path} for parity output: {e}"));
    file.write_all(line.as_bytes())
        .unwrap_or_else(|e| panic!("write parity output to {path}: {e}"));
}
