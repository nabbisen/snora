//! `snora::design::control` as a `snora::design::button` variant (the 0.62 audit: B-3, O-027).
//!
//! `overlays.md` said that `design::control` with a button variant's style "looks identical" to the
//! button. It does not, as written: `design::button::*` draw their label at the **label size**
//! (`Tokens::typography.label.size`, 14 px in the stock presets) and `text("Save")` is drawn at
//! iced's default (16 px), so the control is 4.3 × 2.6 px larger. **The padding is not the
//! difference**: both are `[5, 10]`, `iced::widget::button`'s own default, which is what the
//! button helpers set and what `design::control` starts from. With the label at that size the two
//! draw the same frames, in every variant and every state; this file asserts both halves.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use iced::Element;
use iced::widget::text;
use snora::design::Tokens;
use snora::design::style::button as variant;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press,
}

type Make = fn(&'static Tokens, Option<f32>) -> Element<'static, Msg>;

/// `design::control` carrying `text("Save")`, at `size` if given, styled as a button variant.
fn control_as(
    t: &'static Tokens,
    size: Option<f32>,
    style: fn(&Tokens, iced::widget::button::Status) -> iced::widget::button::Style,
) -> Element<'static, Msg> {
    let label = match size {
        Some(size) => text("Save").size(size),
        None => text("Save"),
    };
    snora::design::control(t, label)
        .on_press(Msg::Press)
        .style(move |_theme, status| style(t, status))
        .into()
}

fn variants() -> [(&'static str, Make, Make); 4] {
    [
        (
            "primary",
            |t, _| snora::design::button::primary(t, "Save", Msg::Press),
            |t, size| control_as(t, size, variant::primary),
        ),
        (
            "secondary",
            |t, _| snora::design::button::secondary(t, "Save", Msg::Press),
            |t, size| control_as(t, size, variant::secondary),
        ),
        (
            "ghost",
            |t, _| snora::design::button::ghost(t, "Save", Msg::Press),
            |t, size| control_as(t, size, variant::ghost),
        ),
        (
            "danger",
            |t, _| snora::design::button::danger(t, "Save", Msg::Press),
            |t, size| control_as(t, size, variant::danger),
        ),
    ]
}

/// The frames of one element: idle, hovered, pressed (button down, not released), focused. Each
/// state is its own interface, so one never leaks into the next.
fn frames(make: Make, t: &'static Tokens, size: Option<f32>) -> [(&'static str, Vec<u8>); 4] {
    let mut idle = Rig::new(make(t, size));
    let centre = idle.targets()[0].bounds.center();
    let a = idle.frame();

    let mut hover = Rig::new(make(t, size));
    hover.point_at(Some(centre));
    hover.redraw();
    let b = hover.frame();

    let mut press = Rig::new(make(t, size));
    press.point_at(Some(centre));
    press.left_button(true);
    press.redraw();
    let c = press.frame();

    let mut focus = Rig::new(make(t, size));
    focus.focus_at(0);
    focus.redraw();
    let d = focus.frame();

    [("idle", a), ("hovered", b), ("pressed", c), ("focused", d)]
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    a.len().abs_diff(b.len()) + a.iter().zip(b).filter(|(x, y)| x != y).count()
}

fn presets() -> [(&'static str, &'static Tokens); 2] {
    [
        ("light", Box::leak(Box::new(Tokens::light()))),
        ("dark", Box::leak(Box::new(Tokens::dark()))),
    ]
}

#[test]
fn a_control_whose_label_is_at_the_label_size_draws_the_same_frames_as_the_button_in_every_variant()
{
    let mut unequal = Vec::new();
    for (preset, t) in presets() {
        let label = t.typography.label.size;
        for (name, button, control) in variants() {
            let want = frames(button, t, None);
            let got = frames(control, t, Some(label));
            // Premise: the states are told apart, so equality is not four copies of one frame.
            assert!(
                differing(&want[0].1, &want[1].1) > 0,
                "{preset} {name}: hovering changes nothing, so the comparison could not tell it"
            );
            assert!(
                differing(&want[1].1, &want[2].1) > 0,
                "{preset} {name}: pressing draws nothing beyond hovering"
            );
            assert!(
                differing(&want[0].1, &want[3].1) > 0,
                "{preset} {name}: focusing draws nothing"
            );
            for ((state, w), (_, g)) in want.iter().zip(&got) {
                let n = differing(w, g);
                if n != 0 {
                    unequal.push(format!("{preset} {name} {state}: {n} bytes differ"));
                }
            }
        }
    }
    assert!(unequal.is_empty(), "{}", unequal.join("\n"));
}

#[test]
fn a_control_left_at_the_default_text_size_is_not_the_button_which_is_what_the_book_said_it_was() {
    // 14 px against 16: the button helpers set the label size, `text("Save")` does not.
    for (preset, t) in presets() {
        for (name, button, control) in variants() {
            let button_box = Rig::new(button(t, None)).targets()[0].bounds;
            let control_box = Rig::new(control(t, None)).targets()[0].bounds;
            assert!(
                control_box.width > button_box.width && control_box.height > button_box.height,
                "{preset} {name}: {control_box:?} vs {button_box:?}"
            );
        }
    }
}

#[test]
fn the_padding_is_the_same_so_it_is_not_what_differs() {
    // With the label at the same size the boxes are the same size: the padding cannot differ.
    for (preset, t) in presets() {
        let label = t.typography.label.size;
        for (name, button, control) in variants() {
            let a = Rig::new(button(t, None)).targets()[0].bounds;
            let b = Rig::new(control(t, Some(label))).targets()[0].bounds;
            assert_eq!((a.width, a.height), (b.width, b.height), "{preset} {name}");
        }
    }
}
