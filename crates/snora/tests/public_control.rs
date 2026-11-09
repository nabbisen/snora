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
use iced::{Element, Rectangle, Theme};
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

// ---- the builder: every method reaches the control ------------------------------------

/// **Each builder method does what `button`'s does, through the wrapper.** The wrapper
/// stores its settings and hands them to the internal control only when it becomes an
/// `Element`, so a method that stored its argument and never passed it on would compile
/// and do nothing; each of these fails if it is not passed on.
mod builder {
    use super::*;
    use iced::Length;
    use iced::widget::{row, text};
    use std::cell::Cell;
    use std::rc::Rc;

    /// Click the label and return what the control published.
    fn clicked(control: snora::Control<'static, Msg>) -> Vec<Msg> {
        let element: Element<'static, Msg> = iced::widget::container(control).padding(24).into();
        let mut ui = iced_test::Simulator::new(element);
        let _ = ui.click("go");
        ui.into_messages().collect()
    }

    fn engine() -> snora::Control<'static, Msg> {
        snora::control(text("go"))
    }

    #[test]
    fn on_press_publishes_its_message_on_a_click() {
        assert_eq!(clicked(engine().on_press(Msg("a"))), vec![Msg("a")]);
    }

    #[test]
    fn on_press_with_makes_its_message_when_the_press_happens() {
        let made = Rc::new(Cell::new(0));
        let counter = Rc::clone(&made);
        let control = snora::control(text("go")).on_press_with(move || {
            counter.set(counter.get() + 1);
            Msg("made")
        });
        assert_eq!(made.get(), 0, "the message was made before any press");
        assert_eq!(clicked(control), vec![Msg("made")]);
        assert_eq!(made.get(), 1, "the message is made once per press");
    }

    #[test]
    fn on_press_maybe_some_presses_and_none_is_disabled_and_not_a_focus_target() {
        assert_eq!(
            clicked(engine().on_press_maybe(Some(Msg("a")))),
            vec![Msg("a")]
        );
        assert!(
            clicked(engine().on_press_maybe(None)).is_empty(),
            "a disabled control published"
        );
        let none: Element<'static, Msg> = engine().on_press_maybe(None).into();
        assert!(
            boxes(none, 16.0).is_empty(),
            "a disabled control is a focus target"
        );
        let some: Element<'static, Msg> = engine().on_press_maybe(Some(Msg("a"))).into();
        assert_eq!(boxes(some, 16.0).len(), 1);
    }

    fn size_of(control: snora::Control<'static, Msg>) -> (f32, f32) {
        let b = boxes(control.on_press(Msg("a")).into(), 16.0)[0];
        (b.width, b.height)
    }

    /// The default is `button`'s `[5, 10]`; `padding` changes the target by exactly the
    /// difference, both ways.
    #[test]
    fn padding_changes_the_target_by_the_difference() {
        let (w0, h0) = size_of(engine());
        let (w1, h1) = size_of(engine().padding([20, 40]));
        assert!(
            (w1 - w0 - 60.0).abs() < 0.5,
            "width went {w0:.1} -> {w1:.1}, not +60"
        );
        assert!(
            (h1 - h0 - 30.0).abs() < 0.5,
            "height went {h0:.1} -> {h1:.1}, not +30"
        );
    }

    #[test]
    fn width_and_height_fix_the_target() {
        let (w, h) = size_of(
            engine()
                .width(Length::Fixed(130.0))
                .height(Length::Fixed(77.0)),
        );
        assert!((w - 130.0).abs() < 0.5, "width {w:.1}, not 130");
        assert!((h - 77.0).abs() < 0.5, "height {h:.1}, not 77");
    }

    /// `clip` changes what is drawn when the content overflows, and only then: the same
    /// frame with it set is the same as without it when nothing overflows.
    #[test]
    fn clip_cuts_overflowing_content_at_the_edge() {
        let dir = std::env::temp_dir().join(format!("snora-public-control-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let frame = |clip: bool| {
            let long = text("WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW")
                .wrapping(iced::widget::text::Wrapping::None)
                .size(14);
            let page: Element<'static, Msg> = iced::widget::container(row![
                snora::control(long)
                    .on_press(Msg("a"))
                    .width(Length::Fixed(60.0))
                    .height(Length::Fixed(30.0))
                    .clip(clip)
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
            iced_test::Simulator::with_size(
                iced::Settings::default(),
                iced::Size::new(300.0, 60.0),
                page,
            )
            .snapshot(&Theme::Light)
            .expect("a frame")
        };
        // The first call records the unclipped frame; the rest compare against it.
        let free = dir.join("free");
        assert!(frame(false).matches_hash(&free).expect("hash"));
        assert!(
            frame(false).matches_hash(&free).expect("hash"),
            "the unclipped frame is not repeatable, so the comparison below measures nothing"
        );
        assert!(
            !frame(true).matches_hash(&free).expect("hash"),
            "clip(true) drew the same frame as clip(false) around overflowing text"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
