//! RFC-127 Unit 0: `.id()` on `snora::control` and `snora::design::control`, measured on a
//! PROTOTYPE. Prints each row of the handoff's table (`--nocapture`) and asserts it.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
//! A task is run the way the window loop runs it (`Rig::task`), and a view that opens a
//! dialog is built **before** the task runs: `iced_winit` collects the update's actions,
//! rebuilds the interfaces from the new `view`, and only then runs the actions
//! (`iced_winit` `lib.rs` 1228-1260 and `run_action`), before the next `RedrawRequested`.
//! That is `Rig::rebuilt_quiet` and then `Rig::task`.
#![cfg(feature = "widgets")]
mod common;

use common::removal::{Rig, ring_on};
use iced::Element;
use iced::widget::operation::{focus, is_focused};
use iced::widget::{column, text};
use iced_runtime::core::event::Status;
use snora::{AppLayout, Dialog};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    A,
    B,
    C,
    Ok,
    Cancel,
    Page,
}

fn ctl<'a>(label: &'static str, message: Msg) -> snora::Control<'a, Msg> {
    snora::control(text(label)).on_press(message)
}

/// Three controls; the second has the id `"b"` (and the third too, if `dup`).
fn page(dup: bool) -> Element<'static, Msg> {
    let c = ctl("C", Msg::C);
    column![
        Element::from(ctl("A", Msg::A)),
        Element::from(ctl("B", Msg::B).id("b")),
        Element::from(if dup { c.id("b") } else { c }),
    ]
    .into()
}

fn render_page(dup: bool) -> Element<'static, Msg> {
    snora::render(AppLayout::new(page(dup)))
}

#[cfg(feature = "design")]
fn tokens() -> &'static snora::design::Tokens {
    Box::leak(Box::new(snora::design::Tokens::light()))
}

fn dialog_card() -> Element<'static, Msg> {
    column![
        Element::from(ctl("OK", Msg::Ok).id("ok")),
        Element::from(ctl("Cancel", Msg::Cancel).id("cancel")),
    ]
    .into()
}

/// The page (one control, `"page-b"`), and a dialog if `open`.
fn with_dialog(open: bool) -> Element<'static, Msg> {
    let body: Element<'static, Msg> = snora::control(text("Page"))
        .on_press(Msg::Page)
        .id("page-b")
        .into();
    let mut layout = AppLayout::new(body);
    if open {
        layout = layout.dialog(Dialog::new(dialog_card()));
    }
    snora::render(layout)
}

/// A view, built again for each rig.
type Build = Box<dyn Fn() -> Element<'static, Msg>>;

fn focused(rig: &mut Rig<Msg>) -> Vec<usize> {
    rig.focused()
}

/// R-1, R-2: only the control with that id is focused, its ring is drawn, Enter presses it once,
/// the next Tab goes to the control after it. Both paths.
#[test]
fn focus_by_id_reaches_one_control_with_its_ring_and_tab_carries_on() {
    #[cfg_attr(not(feature = "design"), allow(unused_mut))]
    let mut paths: Vec<(&str, Build)> = vec![("engine", Box::new(|| render_page(false)))];
    #[cfg(feature = "design")]
    {
        let t = tokens();
        paths.push((
            "design",
            Box::new(move || {
                snora::design::render(
                    AppLayout::new(Element::from(column![
                        Element::from(snora::design::control(t, text("A")).on_press(Msg::A)),
                        Element::from(
                            snora::design::control(t, text("B"))
                                .on_press(Msg::B)
                                .id("b")
                        ),
                        Element::from(snora::design::control(t, text("C")).on_press(Msg::C)),
                    ])),
                    t,
                )
            }),
        ));
    }
    for (path, build) in paths {
        let mut rig = Rig::new(build());
        let mut clean = Rig::new(build());
        assert_eq!(rig.task(is_focused("b")), vec![false], "{path}: before");
        assert_eq!(focused(&mut rig), Vec::<usize>::new(), "{path}");

        let _ = rig.task::<()>(focus("b"));

        assert_eq!(focused(&mut rig), vec![1], "{path}: only the second");
        assert_eq!(rig.task(is_focused("b")), vec![true], "{path}: after");
        let ring = ring_on(&mut rig, &mut clean);
        println!(
            "{path}: focus(\"b\") -> focused {:?}, ring drawn on target {ring:?}",
            focused(&mut rig)
        );
        assert_eq!(
            ring,
            Some(1),
            "{path}: the ring is not drawn on the control"
        );

        rig.messages.clear();
        rig.enter();
        assert_eq!(rig.messages, vec![Msg::B], "{path}: Enter published");
        rig.messages.clear();
        rig.tab(false);
        assert_eq!(
            focused(&mut rig),
            vec![2],
            "{path}: Tab goes to the one after"
        );
    }
}

/// A click focuses nothing (RFC-121), with an id as without one.
#[test]
fn a_click_on_a_control_with_an_id_focuses_nothing_as_before() {
    let mut rig = Rig::new(render_page(false));
    let at = rig.targets()[1].bounds.center();
    rig.click_at(at);
    assert_eq!(rig.messages, vec![Msg::B]);
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
}

/// Q-1: the update that opens a dialog also returns `focus(id)` for a control in it.
#[test]
fn q1_a_focus_task_returned_with_the_dialogs_opening() {
    // (a) The loop's order: rebuild from the new view, run the task, then the redraw.
    let rig = Rig::new(with_dialog(false));
    let mut rig = rig.rebuilt_quiet(with_dialog(true));
    let _ = rig.task::<()>(focus("cancel"));
    rig.redraw();
    let mut clean = Rig::new(with_dialog(false)).rebuilt(with_dialog(true));
    let n = rig.targets().len();
    let ring = ring_on(&mut rig, &mut clean);
    println!(
        "(a) rebuild, task, redraw: {n} targets, focused {:?}, ring on target {ring:?}",
        focused(&mut rig)
    );
    assert_eq!(focused(&mut rig), vec![1], "(a): Cancel is the second");
    assert_eq!(ring, Some(1), "(a): no ring on Cancel at the first draw");

    // (b) If a task ran **before** the new view existed (not the loop's order).
    let mut rig = Rig::new(with_dialog(false));
    let _ = rig.task::<()>(focus("cancel"));
    let mut rig = rig.rebuilt(with_dialog(true));
    println!("(b) task, rebuild, redraw: focused {:?}", focused(&mut rig));
    assert_eq!(focused(&mut rig), Vec::<usize>::new(), "(b)");

    // (c) With a page control focused when the dialog opens (the opener).
    let mut rig = Rig::new(with_dialog(false));
    let _ = rig.task::<()>(focus("page-b"));
    assert_eq!(focused(&mut rig), vec![0]);
    let mut rig = rig.rebuilt_quiet(with_dialog(true));
    let _ = rig.task::<()>(focus("cancel"));
    rig.redraw();
    println!(
        "(c) opener focused, rebuild, task, redraw: focused {:?}",
        focused(&mut rig)
    );
    assert_eq!(focused(&mut rig), vec![1], "(c): only Cancel");
    // Close the dialog: the opener's focus returns (RFC-119), unless the task chose otherwise.
    let rig = rig.rebuilt(with_dialog(false));
    let mut rig = rig;
    println!(
        "(c) after the dialog closes: focused {:?}",
        focused(&mut rig)
    );
}

/// R-3: `focus(id)` for a page control under an open dialog.
#[test]
fn r3_focus_of_a_covered_control_does_nothing() {
    // From nothing focused.
    let mut rig = Rig::new(with_dialog(true));
    let _ = rig.task::<()>(focus("page-b"));
    println!(
        "from nothing: focused {:?}, is_focused(page-b) {:?}",
        focused(&mut rig),
        rig.task(is_focused("page-b"))
    );
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
    // iced's `is_focused` answers only when it finds the id: for a covered control the
    // task yields **nothing** (not `false`).
    assert_eq!(rig.task(is_focused("page-b")), Vec::<bool>::new());
    // From a dialog control focused: does it stay, or is nothing focused?
    let mut rig = Rig::new(with_dialog(true));
    let _ = rig.task::<()>(focus("ok"));
    assert_eq!(focused(&mut rig), vec![0]);
    let _ = rig.task::<()>(focus("page-b"));
    println!("from OK focused: focused {:?}", focused(&mut rig));
    // Escape has no effect on this; the measurement is what is printed.
    let _ = Status::Ignored;
}

/// Two controls given the same id.
#[test]
fn two_controls_with_one_id() {
    let mut rig = Rig::new(render_page(true));
    let _ = rig.task::<()>(focus("b"));
    println!("same id on two controls: focused {:?}", focused(&mut rig));
    assert_eq!(
        focused(&mut rig).len(),
        2,
        "iced's focus(id) focuses every match"
    );
}

/// A control given an id and rebuilt by the next `view` with the same id.
#[test]
fn a_rebuilt_view_with_the_same_id_is_found_again() {
    let mut rig = Rig::new(render_page(false));
    let _ = rig.task::<()>(focus("b"));
    let mut rig = rig.rebuilt(render_page(false));
    assert_eq!(focused(&mut rig), vec![1], "the rebuild kept the focus");
    // Focus by id after the rebuild (a fresh `view`, the id chosen again).
    let _ = rig.task::<()>(focus("b"));
    assert_eq!(focused(&mut rig), vec![1]);
}
