//! RFC-127: `.id()` on `snora::control` and `snora::design::control`, so that iced's own
//! `focus(id)` and `is_focused(id)` reach a snora control.
//!
//! Through the real engine and the runtime's own interface; a task is run the way the window
//! loop runs it (`Rig::task`). A view that opens a dialog is built **before** its task runs:
//! `iced_winit` collects an update's actions, rebuilds the interfaces from the new `view`, and only
//! then runs the actions (`lib.rs`, the update branch and `run_action`), before the next
//! `RedrawRequested`. That is `Rig::rebuilt_quiet`, then `Rig::task`, then `Rig::redraw`.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use common::removal::{Rig, ring_on};
use iced::Element;
use iced::widget::operation::{focus, is_focused};
use iced::widget::{column, text};
use snora::{AppLayout, Dialog, Sheet};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    A,
    B,
    C,
    Ok,
    Cancel,
    Page,
    Side,
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

/// A view, built again for each rig.
type Build = Box<dyn Fn() -> Element<'static, Msg>>;

/// The same three controls on each path (the second has the id `"b"`).
fn paths() -> Vec<(&'static str, Build)> {
    #[cfg_attr(not(feature = "design"), allow(unused_mut))]
    let mut paths: Vec<(&str, Build)> = vec![("engine", Box::new(|| render_page(false)))];
    #[cfg(feature = "design")]
    {
        let t: &'static snora::design::Tokens = Box::leak(Box::new(snora::design::Tokens::light()));
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
    paths
}

fn dialog_card() -> Element<'static, Msg> {
    column![
        Element::from(ctl("OK", Msg::Ok).id("ok")),
        Element::from(ctl("Cancel", Msg::Cancel).id("cancel")),
    ]
    .into()
}

fn page_control() -> Element<'static, Msg> {
    snora::control(text("Page"))
        .on_press(Msg::Page)
        .id("page-b")
        .into()
}

/// The page (one control, `"page-b"`), and a dialog if `open`.
fn with_dialog(open: bool) -> Element<'static, Msg> {
    let mut layout = AppLayout::new(page_control());
    if open {
        layout = layout.dialog(Dialog::new(dialog_card()));
    }
    snora::render(layout)
}

/// The page (one control, `"page-b"`) under an open sheet.
fn with_sheet() -> Element<'static, Msg> {
    let side = snora::control(text("Side")).on_press(Msg::Side).id("side");
    snora::render(AppLayout::new(page_control()).sheet(Sheet::new(side.into())))
}

fn focused(rig: &mut Rig<Msg>) -> Vec<usize> {
    rig.focused()
}

/// `focus(id)` focuses the control with that id and only it. Both paths.
#[test]
fn focus_by_id_focuses_the_control_with_that_id_and_only_it() {
    for (path, build) in paths() {
        let mut rig = Rig::new(build());
        assert_eq!(focused(&mut rig), Vec::<usize>::new(), "{path}: before");
        let _ = rig.task::<()>(focus("b"));
        assert_eq!(focused(&mut rig), vec![1], "{path}: only the second");
        // And a second call moves it, as iced's does.
        let _ = rig.task::<()>(focus("b"));
        assert_eq!(focused(&mut rig), vec![1], "{path}: again");
    }
}

/// The ring is drawn after `focus(id)`, read from pixels: this frame against the same view never
/// focused, the changed region's centre matched to the nearest target. Both paths.
#[test]
fn the_ring_is_drawn_after_focus_by_id() {
    for (path, build) in paths() {
        let mut rig = Rig::new(build());
        let mut clean = Rig::new(build());
        let _ = rig.task::<()>(focus("b"));
        let ring = ring_on(&mut rig, &mut clean);
        println!("{path}: focus(\"b\"): ring drawn on target {ring:?}");
        assert_eq!(ring, Some(1), "{path}: the ring is not on the control");
    }
}

/// Enter presses a control focused by `focus(id)`, once.
#[test]
fn enter_presses_a_control_focused_by_id_once() {
    for (path, build) in paths() {
        let mut rig = Rig::new(build());
        rig.enter();
        assert!(rig.messages.is_empty(), "{path}: Enter before any focus");
        let _ = rig.task::<()>(focus("b"));
        rig.enter();
        assert_eq!(rig.messages, vec![Msg::B], "{path}: Enter");
    }
}

/// Tab carries on from a control focused by `focus(id)`: to the one after it.
#[test]
fn tab_carries_on_from_a_control_focused_by_id() {
    for (path, build) in paths() {
        let mut rig = Rig::new(build());
        let _ = rig.task::<()>(focus("b"));
        rig.tab(false);
        assert_eq!(focused(&mut rig), vec![2], "{path}: Tab");
        let _ = rig.task::<()>(focus("b"));
        rig.tab(true);
        assert_eq!(focused(&mut rig), vec![0], "{path}: Shift+Tab");
    }
}

/// `is_focused(id)` reports it: `false`, then `true`.
#[test]
fn is_focused_reports_a_control_focused_by_id() {
    for (path, build) in paths() {
        let mut rig = Rig::new(build());
        assert_eq!(rig.task(is_focused("b")), vec![false], "{path}: before");
        let _ = rig.task::<()>(focus("b"));
        assert_eq!(rig.task(is_focused("b")), vec![true], "{path}: after");
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

/// Rebuilt by the next `view` with the same id: `focus(id)` still finds it and the focus is kept.
#[test]
fn a_rebuilt_view_with_the_same_id_is_found_again() {
    let mut rig = Rig::new(render_page(false));
    let _ = rig.task::<()>(focus("b"));
    let mut rig = rig.rebuilt(render_page(false));
    assert_eq!(focused(&mut rig), vec![1], "the rebuild kept the focus");
    let _ = rig.task::<()>(focus("a-control-that-does-not-exist"));
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
    let _ = rig.task::<()>(focus("b"));
    assert_eq!(focused(&mut rig), vec![1], "found again after the rebuild");
}

/// R-3: `focus(id)` for a page control under an open dialog or sheet focuses nothing beneath
/// **and leaves nothing focused**, even if a control above held focus; `is_focused` gives no
/// answer.
#[test]
fn focus_by_id_beneath_a_cover_focuses_nothing_and_leaves_nothing_focused() {
    for (name, build, above) in [
        ("dialog", Box::new(|| with_dialog(true)) as Build, "ok"),
        ("sheet", Box::new(with_sheet) as Build, "side"),
    ] {
        // From nothing focused.
        let mut rig = Rig::new(build());
        let _ = rig.task::<()>(focus("page-b"));
        assert_eq!(
            focused(&mut rig),
            Vec::<usize>::new(),
            "{name}: from nothing"
        );
        assert_eq!(
            rig.task(is_focused("page-b")),
            Vec::<bool>::new(),
            "{name}: is_focused answered for a covered control"
        );
        // From the cover's own control focused.
        let mut rig = Rig::new(build());
        let _ = rig.task::<()>(focus(above));
        assert_eq!(focused(&mut rig), vec![0], "{name}: {above} is focused");
        let _ = rig.task::<()>(focus("page-b"));
        assert_eq!(
            focused(&mut rig),
            Vec::<usize>::new(),
            "{name}: focus did not stay on {above}, and nothing beneath took it"
        );
        // Tab starts again inside the cover.
        rig.tab(false);
        assert_eq!(focused(&mut rig), vec![0], "{name}: Tab starts again");
    }
}

/// An id that **no control has at all** does exactly what a covered one does, so the rustdoc says
/// it once.
#[test]
fn an_id_that_no_control_has_does_what_a_covered_one_does() {
    let mut rig = Rig::new(render_page(false));
    let _ = rig.task::<()>(focus("nothing-has-this"));
    assert_eq!(focused(&mut rig), Vec::<usize>::new(), "from nothing");
    let _ = rig.task::<()>(focus("b"));
    assert_eq!(focused(&mut rig), vec![1]);
    let _ = rig.task::<()>(focus("nothing-has-this"));
    assert_eq!(
        focused(&mut rig),
        Vec::<usize>::new(),
        "it did not leave the focus where it was"
    );
    assert_eq!(rig.task(is_focused("nothing-has-this")), Vec::<bool>::new());
}

/// Q-1, the pattern the rustdoc shows: the update that opens a dialog also returns `focus(id)` for
/// a control in it. The loop's order is: rebuild from the new view, run the task, redraw.
#[test]
fn the_update_that_opens_a_dialog_can_focus_a_control_in_it() {
    let rig = Rig::new(with_dialog(false));
    let mut rig = rig.rebuilt_quiet(with_dialog(true));
    let _ = rig.task::<()>(focus("cancel"));
    rig.redraw();
    let mut clean = Rig::new(with_dialog(false)).rebuilt(with_dialog(true));
    assert_eq!(focused(&mut rig), vec![1], "Cancel is focused");
    // And its ring is on the first frame, by pixels.
    assert_eq!(
        ring_on(&mut rig, &mut clean),
        Some(1),
        "no ring at the first draw"
    );
    // Enter then presses it, once.
    rig.enter();
    assert_eq!(rig.messages, vec![Msg::Cancel]);
}

/// With the page's own control focused when the dialog opens (the opener): the task wins inside
/// the dialog, and when the dialog closes the opener's focus returns (RFC-119's memory).
#[test]
fn the_task_wins_inside_the_dialog_and_the_opener_returns_when_it_closes() {
    let mut rig = Rig::new(with_dialog(false));
    let _ = rig.task::<()>(focus("page-b"));
    assert_eq!(focused(&mut rig), vec![0]);
    let mut rig = rig.rebuilt_quiet(with_dialog(true));
    let _ = rig.task::<()>(focus("cancel"));
    rig.redraw();
    assert_eq!(focused(&mut rig), vec![1], "only Cancel");
    let mut rig = rig.rebuilt(with_dialog(false));
    assert_eq!(focused(&mut rig), vec![0], "the opener's focus returned");
}

/// The limit the rustdoc states: a task that runs **before** the dialog is in the view finds no
/// such control and focuses nothing. (Not the window loop's order for the update that opens it.)
#[test]
fn a_task_that_runs_before_the_view_has_the_dialog_finds_nothing() {
    let mut rig = Rig::new(with_dialog(false));
    let _ = rig.task::<()>(focus("cancel"));
    let mut rig = rig.rebuilt(with_dialog(true));
    assert_eq!(focused(&mut rig), Vec::<usize>::new());
}

/// Two controls given the same id: `focus(id)` focuses **both**, and `is_focused(id)` answers for
/// the **last** one in tree order, whichever of them holds focus.
#[test]
fn two_controls_with_one_id_are_both_focused_and_is_focused_reports_the_last() {
    let mut rig = Rig::new(render_page(true));
    assert_eq!(rig.task(is_focused("b")), vec![false], "before");
    let _ = rig.task::<()>(focus("b"));
    assert_eq!(
        focused(&mut rig),
        vec![1, 2],
        "focus(id) focuses every match"
    );
    assert_eq!(rig.task(is_focused("b")), vec![true]);
    // Only the second holds focus: the answer is the last control's, which does not.
    let mut rig = Rig::new(render_page(true));
    rig.tab(false);
    rig.tab(false);
    assert_eq!(focused(&mut rig), vec![1]);
    assert_eq!(
        rig.task(is_focused("b")),
        vec![false],
        "the last match answers"
    );
    // Only the last holds focus.
    rig.tab(false);
    assert_eq!(focused(&mut rig), vec![2]);
    assert_eq!(rig.task(is_focused("b")), vec![true]);
}
