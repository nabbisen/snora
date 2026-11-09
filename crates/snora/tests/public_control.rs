//! RFC-122: what `snora::control` and `snora::design::control` do to the layout, on
//! aaai's three cases (`common/aaai_cases.rs`).
//!
//! The cases are also subjects of `every_control.rs` (the keyboard probe, the ring in
//! the 22 stock themes and the 6 presets, the target floor, a control under an open
//! dialog) and of `pointer_and_keys.rs` (a click presses once, draws no ring and
//! focuses nothing; a held key presses once). The default style and the ring's colour
//! are unit tests in `src/control.rs`. **This file adds what those cannot say**: where
//! a `width(Fill)` row keeps its content, that data text wraps inside its target, and
//! the target sizes as a table.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must use
//! `--test-threads=1`.

#![cfg(feature = "design")]

mod common;
use common::aaai_cases::{self, SUGGESTION};

use iced::widget::text;
use iced::{Element, Rectangle};
use iced_test::selector::Candidate;

use snora::design::Tokens;

#[derive(Debug, Clone, PartialEq)]
struct Msg(&'static str);

fn leak(tokens: Tokens) -> &'static Tokens {
    Box::leak(Box::new(tokens))
}

fn boxes(element: Element<'static, Msg>, host: f32) -> Vec<Rectangle> {
    let settings = iced::Settings {
        default_text_size: host.into(),
        ..iced::Settings::default()
    };
    let mut found = Vec::new();
    let mut ui = iced_test::Simulator::with_settings(settings, element);
    let _ = ui.find(|c: Candidate<'_>| {
        if let Candidate::Focusable { bounds, .. } = c {
            found.push(bounds);
        }
        None::<()>
    });
    found
}

/// **A `width(Fill)` row keeps its content at the start**, as `control`'s layout does
/// for a menu item: the control is as wide as its container, and its content does not
/// drift to the middle.
#[test]
fn a_fill_row_keeps_its_content_at_the_start() {
    for (name, build) in [
        (
            "engine",
            Box::new(|| aaai_cases::tree_row_engine(Msg("tree")))
                as Box<dyn Fn() -> Element<'static, Msg>>,
        ),
        (
            "design",
            Box::new(|| aaai_cases::tree_row_design(leak(Tokens::light()), Msg("tree"))),
        ),
    ] {
        let element = || iced::widget::container(build()).padding(24).into();
        let target = boxes(element(), 16.0)[0];
        // The control fills its 320 px container.
        assert!(
            (target.width - 320.0).abs() < 1.0,
            "{name}: the row is {:.1} wide, not the container's 320",
            target.width
        );
        // The icon `▸` is the second thing in the row (after a 24 px indent), and the
        // padding is 12: it sits near the start, not the middle.
        let mut ui = iced_test::Simulator::new(element());
        let glyph = ui
            .find("▸")
            .expect("the row draws its icon")
            .visible_bounds()
            .expect("it is visible");
        let from_start = glyph.x - target.x;
        assert!(
            from_start < 60.0,
            "{name}: the icon is {from_start:.1} px from the row's start, so the content is not at the start"
        );
    }
}

/// **Data text wraps inside its target** (RFC-105): a long string with a run that has no
/// space in it stays within the 260 px it was given, on more than one line, and the
/// target grows with it.
#[test]
fn a_suggestion_of_data_text_wraps_inside_its_target() {
    let tokens = leak(Tokens::light());
    let element = || -> Element<'static, Msg> {
        iced::widget::container(aaai_cases::suggestion_design(tokens, Msg("suggestion")))
            .padding(24)
            .into()
    };
    let target = boxes(element(), 16.0)[0];
    assert!(
        target.width <= 260.5,
        "the target is {:.1} wide: the text ran past its container ({} chars)",
        target.width,
        SUGGESTION.len()
    );
    // One line of the label is about 20 px; this is several.
    assert!(
        target.height > 60.0,
        "the target is only {:.1} tall: the text did not wrap",
        target.height
    );
}

/// **The sizes of each case's target**, printed (`--nocapture`) at the default minimum
/// (24) and at `targets.min = 44`, with the host text at 16, and asserted against the
/// floor: `max(24, 1.5 x text, token)` in both dimensions.
#[test]
fn the_floor_is_24_by_default_and_44_with_the_token() {
    println!("\n## target sizes (width x height)\n");
    println!("| case | min token | box |");
    println!("|---|---|---|");
    let mut off = Vec::new();
    for min in [24.0_f32, 44.0] {
        let mut tokens = Tokens::light();
        tokens.targets.min = min;
        let tokens = leak(tokens);
        let cases: [(&str, Element<'static, Msg>); 4] = [
            (
                "glyph beside a label",
                aaai_cases::glyph_label_design(tokens, Msg("o")),
            ),
            (
                "tree row (Fill)",
                aaai_cases::tree_row_design(tokens, Msg("t")),
            ),
            (
                "suggestion",
                aaai_cases::suggestion_design(tokens, Msg("s")),
            ),
            (
                "a lone glyph",
                snora::design::control(tokens, text("★"))
                    .on_press(Msg("g"))
                    .into(),
            ),
        ];
        for (name, element) in cases {
            let b = boxes(element, 16.0)[0];
            println!("| {name} | {min} | {:.1} x {:.1} |", b.width, b.height);
            let floor = min.max(24.0);
            if b.width < floor - 0.01 || b.height < floor - 0.01 {
                off.push(format!(
                    "{name} at min {min}: {:.1} x {:.1}",
                    b.width, b.height
                ));
            }
        }
    }
    assert!(off.is_empty(), "under the floor: {off:?}");
}
