//! RFC-141: **`chip::tag`, a read-only tag: information, not a control.** Five claims, each read from the layout or the
//! frame, in every preset (the renderer follows `ICED_TEST_BACKEND`; the rendered jobs run both):
//!
//! 1. its label holds the preset's own level against its fill, **as drawn** (the recipe's read-only chip was 1.52 to
//!    1.97:1: a disabled chip, which the text floor exempts);
//! 2. it is **not a Tab stop**;
//! 3. a click on it **publishes nothing**;
//! 4. it is **the same height as a filter chip** at the presets' tokens, and **does not grow to `targets.min`** where the
//!    chip does (RFC-141's Unit 0 ruling, (a): a tag is not a pointer target, so it does not look like one);
//! 5. it is **not drawn as selected**: its fill is an unselected chip's, and not the selected chip's.
//!
//! The boundary at 3:1 and the label in the sweeps' own words are `nontext_sweep.rs` and `text_sweep.rs`'s (scene `"tag"`);
//! the style's derivation from `unselected` is `snora-style`'s. Each test has a breakage row in the review request.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use common::sweep::Msg;
use iced::widget::{container, row};
use iced::{Background, Color, Element, Length, Point, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::{Tokens, chip};
use snora::{AppLayout, LayoutDirection::Ltr};

const SCALE: usize = 2;

fn lum(c: [u8; 3]) -> f32 {
    let f = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
}
fn ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}
fn col(c: snora::design::Color) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

/// The element on a page of colour `page`, 24 px in, drawn at 2x.
fn shoot(t: &Tokens, page: Color, el: Element<'static, Msg>) -> (Vec<u8>, Rig<Msg>) {
    let on = container(el)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(page)),
            ..container::Style::default()
        });
    let view = snora::design::render(AppLayout::new(on.into()).direction(Ltr), t);
    let mut rig = Rig::new(view);
    rig.point_at(None);
    rig.redraw();
    let frame = rig.frame_scaled(&snora::design::theme(t), SCALE as f32);
    (frame, rig)
}

fn px(frame: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * 800 * SCALE + x) * 4;
    [frame[i], frame[i + 1], frame[i + 2]]
}

/// The box of what differs from the page, in device pixels, in the first 280 x 200 logical px: `(x0, y0, x1, y1)`.
fn boxed(frame: &[u8]) -> (usize, usize, usize, usize) {
    let bg = px(frame, 2, 300 * SCALE);
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..200 * SCALE {
        for x in 0..280 * SCALE {
            let p = px(frame, x, y);
            if (0..3)
                .map(|c| (i32::from(p[c]) - i32::from(bg[c])).abs())
                .sum::<i32>()
                > 6
            {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    assert!(x1 >= x0, "nothing was drawn");
    (x0, y0, x1, y1)
}

/// `(fill, ink, ink/fill)`: the fill is the commonest colour of the box's central region, the ink the pixel furthest from it.
fn label(frame: &[u8], b: (usize, usize, usize, usize)) -> ([u8; 3], [u8; 3], f32) {
    let (x0, y0, x1, y1) = b;
    let (cx0, cx1, cy0, cy1) = (
        x0 + (x1 - x0) / 6,
        x1 - (x1 - x0) / 6,
        y0 + (y1 - y0) / 4,
        y1 - (y1 - y0) / 4,
    );
    let mut count = std::collections::HashMap::new();
    for y in cy0..cy1 {
        for x in cx0..cx1 {
            *count.entry(px(frame, x, y)).or_insert(0usize) += 1;
        }
    }
    let fill = *count.iter().max_by_key(|(_, n)| **n).unwrap().0;
    let (mut ink, mut best) = (fill, 1.0f32);
    for y in cy0..cy1 {
        for x in cx0..cx1 {
            let p = px(frame, x, y);
            let r = ratio(p, fill);
            if r > best {
                (best, ink) = (r, p);
            }
        }
    }
    (fill, ink, best)
}

/// Every container's bounds, in tree order.
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

// ---- 1. the label's level, as drawn -----------------------------------------------------------------------------------------------

/// **The label holds the preset's own level against the tag's own fill**, read from the pixels, in all six presets: 4.5:1 on
/// `light` and `dark`, 7:1 on the AAA and high-contrast presets. (Unit 0 measured 8.12 to 17.40; the recipe's chip, 1.52 to 1.97.)
#[test]
fn a_tags_label_holds_the_presets_level() {
    let mut failures = Vec::new();
    for (name, t) in Tokens::presets() {
        let level = t.palette.text_contrast.ratio();
        let (frame, _) = shoot(&t, col(t.palette.background), chip::tag::<Msg>(&t, "Rust"));
        let (_, _, got) = label(&frame, boxed(&frame));
        if got < level {
            failures.push(format!(
                "{name}: the label is {got:.2}:1 against its fill, under {level}:1"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failure(s):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

// ---- 2. not a Tab stop ------------------------------------------------------------------------------------------------------------

/// **A tag registers no focus target**, alone and among chips, and Tab never stops on it: a row of `tag, filter, tag` has one
/// target, the chip, and Tab from the start lands on it and from there on nothing else.
#[test]
fn a_tag_is_not_a_tab_stop() {
    for (name, t) in Tokens::presets() {
        let mut alone = Rig::new(chip::tag::<Msg>(&t, "Rust"));
        assert_eq!(
            alone.targets().len(),
            0,
            "{name}: a tag alone has a focus target"
        );
        let mixed: Element<'static, Msg> = row![
            chip::tag::<Msg>(&t, "PDF"),
            chip::filter::<Msg>(&t, "Draft", false, Some(Msg::Go)),
            chip::tag::<Msg>(&t, "Shared"),
        ]
        .into();
        let mut rig = Rig::new(mixed);
        let targets = rig.targets();
        assert_eq!(
            targets.len(),
            1,
            "{name}: only the filter chip is a Tab stop in `tag, filter, tag`"
        );
        // The one target is the chip, which sits between the two tags: the tags are not what Tab reached.
        let tags: Vec<Rectangle> = containers(&mut rig);
        let chip_x = targets[0].bounds.x;
        assert!(
            tags.iter().any(|r| r.x < chip_x) && tags.iter().any(|r| r.x > chip_x),
            "{name}: the row is not `tag, chip, tag` (the probe would be reading something else)"
        );
        rig.tab(false);
        assert_eq!(rig.focused(), vec![0], "{name}: Tab lands on the chip");
        rig.tab(false);
        assert!(
            rig.focused().len() <= 1,
            "{name}: a second Tab left more than one control focused"
        );
    }
}

// ---- 3. no message on a click -------------------------------------------------------------------------------------------------------

/// **A click on a tag publishes nothing**, at its middle and at its padding, while the same click on a filter chip publishes
/// (so the click is a real one: the probe can fail).
#[test]
fn a_click_on_a_tag_publishes_nothing() {
    for (name, t) in Tokens::presets() {
        let mut rig = Rig::new(chip::tag::<Msg>(&t, "Rust"));
        let b = containers(&mut rig)[0];
        rig.click_at(Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0));
        rig.click_at(Point::new(b.x + 2.0, b.y + b.height / 2.0));
        assert!(
            rig.messages.is_empty(),
            "{name}: a click on a tag published {} message(s)",
            rig.messages.len()
        );
        // The control: the same clicks on a filter chip.
        let mut chip_rig = Rig::new(chip::filter::<Msg>(&t, "Rust", false, Some(Msg::Go)));
        let c = chip_rig.targets()[0].bounds;
        chip_rig.click_at(Point::new(c.x + c.width / 2.0, c.y + c.height / 2.0));
        assert_eq!(
            chip_rig.messages.len(),
            1,
            "{name}: the control click did not publish, so this test could not fail"
        );
    }
}

// ---- 4. the height ------------------------------------------------------------------------------------------------------------------

/// A tag's and a filter chip's heights, from the layout: the tag's own container, and the chip's control.
fn heights(t: &Tokens) -> (f32, f32) {
    let mut tag = Rig::new(chip::tag::<Msg>(t, "Rust"));
    let mut chip_rig = Rig::new(chip::filter::<Msg>(t, "Rust", false, Some(Msg::Go)));
    (
        containers(&mut tag)[0].height,
        chip_rig.targets()[0].bounds.height,
    )
}

/// **A tag is as tall as a filter chip with the same label, at the presets' tokens** (`targets.min` 24, below both), so a row
/// of tags and chips lines up.
#[test]
fn a_tag_is_the_same_height_as_a_filter_chip_at_the_presets_tokens() {
    for (name, t) in Tokens::presets() {
        let (tag, chip) = heights(&t);
        assert!(
            (tag - chip).abs() < 0.01,
            "{name}: a tag is {tag} px tall, a filter chip {chip} (targets.min {})",
            t.targets.min
        );
    }
}

/// **And only there**: RFC-141's ruling (a). A design that raises `targets.min` to a touch design's 44 makes the filter chip
/// 44 tall, because the chip is a pointer target, and **the tag stays as tall as its label and padding**, because it is not.
/// A tag the height of a touch target would look pressable, which is the misreading the RFC removes. The rule is guarded in
/// both directions: the chip does grow (so the test is reading a real difference), and the tag does not.
#[test]
fn a_tag_does_not_grow_to_the_minimum_target_where_a_chip_does() {
    let stock = Tokens::light();
    let (stock_tag, stock_chip) = heights(&stock);
    for min in [32.0f32, 44.0] {
        let mut t = Tokens::light();
        t.targets.min = min;
        let (tag, chip) = heights(&t);
        assert!(
            (chip - min).abs() < 0.01 && chip > stock_chip,
            "targets.min {min}: the filter chip is {chip} px tall (stock {stock_chip}); it should take the minimum target"
        );
        assert!(
            (tag - stock_tag).abs() < 0.01,
            "targets.min {min}: the tag is {tag} px tall, stock {stock_tag}: it must not grow to the minimum target"
        );
    }
}

// ---- 5. not selected ---------------------------------------------------------------------------------------------------------------

/// **A tag is not drawn as selected**: its fill is the unselected chip's, and not the selected chip's, as drawn, in every preset.
#[test]
fn a_tag_is_not_drawn_as_selected() {
    for (name, t) in Tokens::presets() {
        let page = col(t.palette.background);
        let fill_of = |el: Element<'static, Msg>| {
            let (frame, _) = shoot(&t, page, el);
            label(&frame, boxed(&frame)).0
        };
        let tag = fill_of(chip::tag::<Msg>(&t, "Rust"));
        let unselected = fill_of(chip::filter::<Msg>(&t, "Rust", false, Some(Msg::Go)));
        let selected = fill_of(chip::filter::<Msg>(&t, "Rust", true, Some(Msg::Go)));
        assert_eq!(
            tag, unselected,
            "{name}: a tag's fill is an unselected chip's"
        );
        assert_ne!(
            tag, selected,
            "{name}: a tag is drawn with the selected chip's fill"
        );
    }
}
