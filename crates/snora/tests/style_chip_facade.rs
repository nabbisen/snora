//! `snora::design::style::chip` is reachable beside `snora::design::style::button` (RFC-129 Q-5),
//! and is the function the chips draw with.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use iced::Element;
use iced::widget::button::Status;
use snora::design::Tokens;
use snora::design::style::{button, chip};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press,
}

fn leak(t: Tokens) -> &'static Tokens {
    Box::leak(Box::new(t))
}

#[test]
fn the_chip_styles_are_reachable_through_the_facade_and_are_what_a_chip_draws() {
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        // Reachable, and a chip style: a solid fill and the palette's own labels.
        let selected = chip::selected(t, Status::Active);
        let unselected = chip::unselected(t, Status::Active);
        assert_eq!(selected.text_color.r, t.palette.accent_text.r, "{preset}");
        assert_eq!(
            unselected.text_color.r, t.palette.text_secondary.r,
            "{preset}"
        );
        // The sibling that was always there, so the two sit side by side.
        let _ = button::primary(t, Status::Active);

        // And the widget draws with it: the filled chip's pixel is the style's fill.
        let el: Element<'static, Msg> = snora::design::chip::filter(t, "Filter", true, Msg::Press);
        let mut rig = Rig::new(el);
        let b = rig.targets()[0].bounds;
        let frame = rig.frame();
        let i = ((b.y as usize + 3) * 800 + (b.x + b.width / 2.0) as usize) * 4;
        let want = match selected.background {
            Some(iced::Background::Color(c)) => c.into_rgba8(),
            other => panic!("{preset}: not a colour fill: {other:?}"),
        };
        assert_eq!(
            frame[i..i + 3],
            want[..3],
            "{preset}: the chip does not draw the style's fill"
        );
    }
}
