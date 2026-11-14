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

/// **The floor follows the label's size on the design path, not the host's**
/// (`1.5 x the text the control carries`, RFC-115): with a label token of 40 the
/// target is 60, whatever the host's default text is, and with the default label size
/// it stays at 24 even when the host's text is large.
#[test]
fn the_design_floor_follows_the_label_size_not_the_hosts_text() {
    let mut big = Tokens::light();
    big.typography.label.size = 40.0;
    let big = leak(big);
    let b = boxes(
        snora::design::control(big, text("x"))
            .on_press(Msg("a"))
            .into(),
        16.0,
    )[0];
    assert!(
        b.height >= 59.9 && b.width >= 59.9,
        "a 40 px label gave a {:.1} x {:.1} target, under 1.5 x 40",
        b.width,
        b.height
    );

    let normal = leak(Tokens::light());
    let b = boxes(
        snora::design::control(normal, text("x").size(14))
            .on_press(Msg("a"))
            .into(),
        40.0,
    )[0];
    assert!(
        b.height < 40.0,
        "the host's 40 px text raised a design-path target to {:.1}; it follows the \
         label token",
        b.height
    );
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

    /// The statuses a control's style function is asked about when the control is drawn.
    fn statuses(control: snora::Control<'static, Msg>) -> Vec<iced::widget::button::Status> {
        let seen = Rc::new(std::cell::RefCell::new(Vec::new()));
        let record = Rc::clone(&seen);
        let control = control.style(move |theme, status| {
            record.borrow_mut().push(status);
            iced::widget::button::primary(theme, status)
        });
        let element: Element<'static, Msg> = iced::widget::container(control).padding(24).into();
        let mut ui = iced_test::Simulator::new(element);
        let _ = ui.snapshot(&Theme::Light).expect("a frame");
        seen.borrow().clone()
    }

    /// **Without `on_press` a control is drawn as `Status::Disabled`** (the 0.62 audit's H-2, R-099), on
    /// both builders; with one it is not. The recording `.style` is what a reader's own style function
    /// sees. The second half is the premise: a style that was never asked about any status would
    /// satisfy "never Active" for the wrong reason.
    #[test]
    fn a_control_without_on_press_is_styled_as_disabled_on_both_builders() {
        use iced::widget::button::Status;
        let tokens = leak(Tokens::light());
        type Make = fn(&'static Tokens) -> snora::Control<'static, Msg>;
        let builders: [(&str, Make); 2] = [
            ("snora::control", |_| snora::control(text("go"))),
            ("snora::design::control", |t| {
                snora::design::control(t, text("go"))
            }),
        ];
        for (name, make) in builders {
            for none in [make(tokens), make(tokens).on_press_maybe(None)] {
                let got = statuses(none);
                assert!(!got.is_empty(), "{name}: the style was never asked");
                assert!(
                    got.iter().all(|s| *s == Status::Disabled),
                    "{name} without on_press: {got:?}"
                );
            }
            let got = statuses(make(tokens).on_press(Msg("a")));
            assert!(!got.is_empty(), "{name}: the style was never asked");
            assert!(
                got.iter().all(|s| *s != Status::Disabled),
                "{name} with on_press: {got:?}"
            );
        }
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

// ---- the 0.62 audit, Unit 3 ------------------------------------------------------------------

mod audit {
    use super::*;
    use iced::Length;
    use iced::widget::{row, text};
    use std::cell::Cell;
    use std::rc::Rc;

    use common::removal::Rig;

    /// **R-120: the builder methods reach the control on the design path too.** `builder` above runs
    /// them on `snora::control`; "everything else is `snora::control`'s" is a claim about
    /// `snora::design::control`, so the same six run there: `on_press`, `on_press_with` (made once
    /// per press), `on_press_maybe(None)` (disabled, not a target), `padding`, `width` and `height`,
    /// and `clip`.
    #[test]
    fn the_builder_methods_reach_the_design_path_control_too() {
        let tokens = leak(Tokens::light());
        let design = || snora::design::control(tokens, text("go"));
        let clicked = |control: snora::Control<'static, Msg>| -> Vec<Msg> {
            let element: Element<'static, Msg> =
                iced::widget::container(control).padding(24).into();
            let mut ui = iced_test::Simulator::new(element);
            let _ = ui.click("go");
            ui.into_messages().collect()
        };
        assert_eq!(clicked(design().on_press(Msg("a"))), vec![Msg("a")]);

        let made = Rc::new(Cell::new(0));
        let counter = Rc::clone(&made);
        let control = design().on_press_with(move || {
            counter.set(counter.get() + 1);
            Msg("made")
        });
        assert_eq!(made.get(), 0, "the message was made before any press");
        assert_eq!(clicked(control), vec![Msg("made")]);
        assert_eq!(made.get(), 1, "the message is made once per press");

        assert!(clicked(design().on_press_maybe(None)).is_empty());
        let none: Element<'static, Msg> = design().on_press_maybe(None).into();
        assert!(
            boxes(none, 16.0).is_empty(),
            "a disabled control is a target"
        );

        let size = |control: snora::Control<'static, Msg>| {
            let b = boxes(control.on_press(Msg("a")).into(), 16.0)[0];
            (b.width, b.height)
        };
        let (w0, h0) = size(design());
        let (w1, h1) = size(design().padding([20, 40]));
        assert!(
            (w1 - w0 - 60.0).abs() < 0.5 && (h1 - h0 - 30.0).abs() < 0.5,
            "padding"
        );
        let (w, h) = size(
            design()
                .width(Length::Fixed(130.0))
                .height(Length::Fixed(77.0)),
        );
        assert!(
            (w - 130.0).abs() < 0.5 && (h - 77.0).abs() < 0.5,
            "{w} x {h}"
        );

        let dir = std::env::temp_dir().join(format!("snora-audit-clip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let frame = |clip: bool| {
            let long = text("WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW")
                .wrapping(iced::widget::text::Wrapping::None)
                .size(14);
            let page: Element<'static, Msg> = iced::widget::container(row![
                snora::design::control(tokens, long)
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
        let free = dir.join("free");
        assert!(frame(false).matches_hash(&free).expect("hash"));
        assert!(
            frame(false).matches_hash(&free).expect("hash"),
            "not repeatable"
        );
        assert!(
            !frame(true).matches_hash(&free).expect("hash"),
            "clip(true) drew the same frame as clip(false) on the design path"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The ring, read from two frames of the same scene (focused, and not): the pixels that differ on
    /// the row through the control's middle, to its left, up to a little inside it. Returns
    /// `(outer, thickness)` in pixels: **how far out the ring's outer edge is from the control**
    /// (that is what the offset is: the ring is a stroke drawn inward from a quad grown by the
    /// offset, so a ring wider than its offset reaches into the control, as the high-contrast
    /// presets' does), and how wide the stroke is.
    fn ring_left(build: impl Fn() -> Element<'static, Msg>) -> (usize, usize) {
        let mut plain = Rig::new(build());
        let bounds = plain.targets()[0].bounds;
        let plain = plain.frame();
        let mut focused = Rig::new(build());
        focused.focus_at(0);
        focused.redraw();
        let focused = focused.frame();
        let y = bounds.center().y as usize;
        let edge = bounds.x as usize;
        let differs = |x: usize| {
            let i = (y * 800 + x) * 4;
            plain[i..i + 3] != focused[i..i + 3]
        };
        let xs: Vec<usize> = (0..edge + 6).filter(|x| differs(*x)).collect();
        assert!(!xs.is_empty(), "no ring is drawn left of the control");
        let (lo, hi) = (xs[0], *xs.last().unwrap());
        assert_eq!(
            xs.len(),
            hi - lo + 1,
            "the ring is one run of pixels: {xs:?}"
        );
        (edge - lo, hi - lo + 1)
    }

    fn page(control: snora::Control<'static, Msg>) -> Element<'static, Msg> {
        iced::widget::container(control).padding(60).into()
    }

    /// The same, on a page painted with the preset's own `background`, which is what the design
    /// path's ring colour is chosen against (a dark preset's ring would vanish on the rig's white).
    fn page_on(tokens: &Tokens, control: snora::Control<'static, Msg>) -> Element<'static, Msg> {
        let bg = tokens.palette.background;
        let fill = iced::Color::from_rgb(bg.r, bg.g, bg.b);
        iced::widget::container(control)
            .padding(60)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(fill.into()),
                ..Default::default()
            })
            .into()
    }

    /// **R-095, R-096: the ring's width and offset, in a frame, on both paths.** The engine path draws a
    /// 2 px ring offset by 2 px; the design path reads both from `Tokens::focus`: every preset's
    /// (2 px or 3 px wide, 2 px out), and a token changed to 5 px wide at 8 px out.
    #[test]
    fn the_ring_is_as_wide_and_as_far_out_as_the_path_says() {
        assert_eq!(
            ring_left(|| page(snora::control(text("Ring")).on_press(Msg("r")))),
            (2, 2),
            "the engine path: (outer edge, thickness)"
        );
        for (name, tokens) in Tokens::presets() {
            let (width, offset) = (tokens.focus.ring_width, tokens.focus.ring_offset);
            let tokens = leak(tokens);
            assert_eq!(
                ring_left(|| page_on(
                    tokens,
                    snora::design::control(tokens, text("Ring")).on_press(Msg("r"))
                )),
                (offset as usize, width as usize),
                "{name}: (outer edge, thickness) from the tokens"
            );
        }
        let mut wide = Tokens::light();
        wide.focus.ring_width = 5.0;
        wide.focus.ring_offset = 8.0;
        let wide = leak(wide);
        assert_eq!(
            ring_left(|| page_on(
                wide,
                snora::design::control(wide, text("Ring")).on_press(Msg("r"))
            )),
            (8, 5),
            "a token changed to 5 px wide at 8 px out"
        );
    }

    fn loud_style(c: snora::Control<'static, Msg>) -> snora::Control<'static, Msg> {
        use iced::widget::button;
        c.style(|_, _| button::Style {
            background: Some(iced::Color::from_rgb(1.0, 0.0, 0.0).into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.0, 1.0, 0.0),
                width: 3.0,
                radius: 8.0.into(),
            },
            ..button::Style::default()
        })
    }

    /// **R-118: the ring is not part of the style.** A style with a red fill and a 3 px green border:
    /// the ring is still drawn, on all four sides, at 3:1 against the page it sits on (the ring's own
    /// colour is the strongest changed pixel on a side; blended edge pixels are weaker), on both
    /// paths, and at the same distance as under the default style.
    #[test]
    fn a_loud_style_does_not_remove_or_change_the_ring() {
        let tokens = leak(Tokens::light());
        type Make = fn(&'static Tokens, bool) -> snora::Control<'static, Msg>;
        let makes: [(&str, Make); 2] = [
            ("engine", |_, l| {
                let c = snora::control(text("Loud")).on_press(Msg("l"));
                if l { loud_style(c) } else { c }
            }),
            ("design", |t, l| {
                let c = snora::design::control(t, text("Loud")).on_press(Msg("l"));
                if l { loud_style(c) } else { c }
            }),
        ];
        for (name, make) in makes {
            let default = ring_left(|| page(make(tokens, false)));
            let with_style = ring_left(|| page(make(tokens, true)));
            assert_eq!(
                default, with_style,
                "{name}: a loud style moved or resized the ring"
            );

            let mut plain = Rig::new(page(make(tokens, true)));
            let b = plain.targets()[0].bounds;
            let plain = plain.frame();
            let mut focused = Rig::new(page(make(tokens, true)));
            focused.focus_at(0);
            focused.redraw();
            let focused = focused.frame();
            let px = |f: &[u8], x: usize, y: usize| {
                let i = (y * 800 + x) * 4;
                iced::Color::from_rgb8(f[i], f[i + 1], f[i + 2])
            };
            let (cx, cy) = (b.center().x as usize, b.center().y as usize);
            let sides: [(&str, Vec<(usize, usize)>); 4] = [
                ("left", (1..8).map(|d| (b.x as usize - d, cy)).collect()),
                (
                    "right",
                    (0..8).map(|d| ((b.x + b.width) as usize + d, cy)).collect(),
                ),
                ("top", (1..8).map(|d| (cx, b.y as usize - d)).collect()),
                (
                    "bottom",
                    (0..8)
                        .map(|d| (cx, (b.y + b.height) as usize + d))
                        .collect(),
                ),
            ];
            for (side, points) in sides {
                let best = points
                    .into_iter()
                    .filter(|(x, y)| px(&plain, *x, *y) != px(&focused, *x, *y))
                    .map(|(x, y)| common::contrast_ratio(px(&focused, x, y), px(&plain, x, y)))
                    .fold(None, |best: Option<f32>, r| {
                        Some(best.map_or(r, |b| b.max(r)))
                    });
                let Some(best) = best else {
                    panic!("{name}: no ring on the {side} side under a loud style");
                };
                assert!(
                    best >= common::NON_TEXT_MIN,
                    "{name}: the ring on the {side} reads {best:.2}:1 against the page"
                );
            }
        }
    }

    /// **R-094: on the engine path the target follows the host's text size, not the size of whatever
    /// is inside.** Host text 40 with a `size(14)` content: the target is 1.5 x 40 = 60 tall (the
    /// floor follows the host's size, the label's). Host text 16 with a `size(80)` content: the
    /// target is its content's height and does not grow to 1.5 x 80 = 120.
    #[test]
    fn on_the_engine_path_the_floor_follows_the_hosts_text_not_the_content() {
        let b = boxes(
            snora::control(text("x").size(14)).on_press(Msg("a")).into(),
            40.0,
        )[0];
        assert!(
            b.height >= 59.9 && b.width >= 59.9,
            "host 40 with 14 px content: {:.1} x {:.1}, under 1.5 x 40",
            b.width,
            b.height
        );
        let b = boxes(
            snora::control(text("x").size(80)).on_press(Msg("a")).into(),
            16.0,
        )[0];
        assert!(
            b.height > 80.0 && b.height < 118.0,
            "host 16 with 80 px content: {:.1} tall, expected its content's, not 1.5 x 80 = 120",
            b.height
        );
    }

    fn row_of(labels: &[&'static str]) -> Element<'static, Msg> {
        let mut c = iced::widget::column![].spacing(8);
        for l in labels {
            c = c.push(snora::control(text(*l)).on_press(Msg(l)));
        }
        iced::widget::container(c).padding(30).into()
    }

    /// **R-089: when a `snora::control` goes away after being pressed, focus follows snora's rule for
    /// a removal** (RFC-120): to the nearest neighbour in its group, the next or else the previous,
    /// and with none left, nowhere. (The public control declares nothing more than "only this one"
    /// goes, which is the rule's plain case.) The premise is the same removal without the engine,
    /// where iced keeps a widget's state by its position: pressing the **last** control leaves
    /// nothing focused there, while the engine hands focus to the previous one.
    #[test]
    fn a_removed_snora_control_hands_focus_to_its_neighbour() {
        let all = ["one", "two", "three"];
        let focus_then_press = |rig: &mut Rig<Msg>, at: usize, want: &'static str| {
            for _ in 0..=at {
                rig.tab(false);
            }
            assert_eq!(
                rig.focused(),
                vec![at],
                "the premise: control {at} has focus"
            );
            rig.enter();
            assert_eq!(
                rig.messages,
                vec![Msg(want)],
                "the premise: Enter pressed it"
            );
            rig.messages.clear();
        };
        let layout = |labels: &[&'static str]| snora::render(snora::AppLayout::new(row_of(labels)));

        // The last is pressed and removed: raw, nowhere; through the engine, the previous one.
        let mut raw = Rig::new(row_of(&all));
        focus_then_press(&mut raw, 2, "three");
        let mut raw = raw.rebuilt(row_of(&["one", "two"]));
        raw.redraw();
        assert!(
            raw.focused().is_empty(),
            "the premise: without the engine focus goes nowhere, got {:?}",
            raw.focused()
        );
        let mut rig = Rig::new(layout(&all));
        focus_then_press(&mut rig, 2, "three");
        let mut rig = rig.rebuilt(layout(&["one", "two"]));
        rig.redraw();
        assert_eq!(
            rig.focused(),
            vec![1],
            "the last control was removed: focus should be on the previous one, 'two'"
        );

        // A middle one: the next.
        let mut rig = Rig::new(layout(&all));
        focus_then_press(&mut rig, 1, "two");
        let mut rig = rig.rebuilt(layout(&["one", "three"]));
        rig.redraw();
        assert_eq!(
            rig.focused(),
            vec![1],
            "a middle control was removed: focus should be on the next one, 'three'"
        );

        // The only one: nowhere.
        let mut rig = Rig::new(layout(&["only"]));
        focus_then_press(&mut rig, 0, "only");
        let mut rig = rig.rebuilt(layout(&[]));
        rig.redraw();
        assert!(rig.focused().is_empty(), "{:?}", rig.focused());
    }

    /// **R-088: a `snora::control` on the page is out of reach under a sheet from the keyboard**, as
    /// under a dialog: the tree reports only the sheet's control, and Tab and Enter all the way round
    /// publish nothing from the page. (`every_control.rs` runs the dialog; `control_id.rs` the sheet
    /// for `focus(id)`.)
    #[test]
    fn a_page_control_is_out_of_reach_under_a_sheet() {
        let layout = || {
            snora::render(
                snora::AppLayout::new(row_of(&["page"]))
                    .sheet(snora::Sheet::new(
                        snora::control(text("sheet")).on_press(Msg("sheet")).into(),
                    ))
                    .on_close_modals(Msg("close")),
            )
        };
        let mut rig = Rig::new(layout());
        assert_eq!(
            rig.targets().len(),
            1,
            "only the sheet's control is reachable"
        );
        for _ in 0..4 {
            rig.tab(false);
            rig.enter();
        }
        assert!(
            !rig.messages.contains(&Msg("page")),
            "Tab and Enter reached the page under a sheet: {:?}",
            rig.messages
        );
        assert!(
            rig.messages.contains(&Msg("sheet")),
            "the premise: they reached the sheet's own control"
        );
    }
}
